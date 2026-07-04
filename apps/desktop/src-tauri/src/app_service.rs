use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use verboscribe_audio::CpalAudioRecorder;
use verboscribe_core::{
    AudioCapture, AudioRecorder, DefaultTranscriptProcessor, DictationConfig, DictationEngine,
    DictationError, DictationMode, DictationState, HotkeyEvent, PersonalDictionary,
    ProcessedTranscript, TargetApp, TargetAppTracker, TextInsertionService,
    TranscriptProcessingOptions, TranscriptProcessor, TranscriptionProvider,
};
use verboscribe_platform::{DesktopTargetTracker, DesktopTextInserter};
use verboscribe_storage::{
    AppSettings, JsonSettingsStore, SettingsDictationMode, TranscriptionProviderKind,
};
use verboscribe_transcription::{WhisperCppConfig, WhisperCppTranscriber};

const DEFAULT_WHISPER_PROMPT_CONTEXT: &str =
    "This is desktop dictation for VerboScribe 2 on macOS and Windows. Preserve exact product names, application names, and technical terms when they are heard clearly.";
const DEFAULT_WHISPER_PINNED_TERMS: &str = "VerboScribe, VerboScribe 2, whisper.cpp, TextEdit";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppStatusDto {
    pub app_status: String,
    pub engine_state: String,
    pub provider: String,
    pub dictation_mode: String,
    pub hotkey: String,
    pub toggle_hotkey: String,
    pub paste_last_hotkey: String,
    pub usage_hint: String,
    pub recovery: String,
    pub last_transcript: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DictationStatusDto {
    pub state: String,
    pub last_transcript: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub provider: String,
    pub whisper_cpp_binary_path: Option<String>,
    pub whisper_cpp_model_path: Option<String>,
    pub language: String,
    pub whisper_cpp_prompt_context: String,
    pub whisper_cpp_pinned_terms: String,
    pub dictation_mode: String,
    pub min_recording_ms: u64,
    pub hotkey: String,
    pub toggle_hotkey: String,
    pub paste_last_hotkey: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEventDto {
    pub phase: String,
    pub message: String,
    pub recovery: Option<RecoveryDto>,
    pub transcript: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryDto {
    pub title: String,
    pub detail: String,
    pub next_step: String,
}

impl From<AppSettings> for SettingsDto {
    fn from(settings: AppSettings) -> Self {
        Self {
            provider: provider_name(settings.transcription.provider).to_string(),
            whisper_cpp_binary_path: settings
                .transcription
                .whisper_cpp
                .binary_path
                .map(path_to_string),
            whisper_cpp_model_path: settings
                .transcription
                .whisper_cpp
                .model_path
                .map(path_to_string),
            language: settings.transcription.whisper_cpp.language,
            whisper_cpp_prompt_context: settings.transcription.whisper_cpp.prompt_context,
            whisper_cpp_pinned_terms: settings.transcription.whisper_cpp.pinned_terms,
            dictation_mode: dictation_mode_name(settings.dictation.mode).to_string(),
            min_recording_ms: settings.dictation.min_recording_ms,
            hotkey: settings.hotkeys.dictation,
            toggle_hotkey: settings.hotkeys.dictation_toggle,
            paste_last_hotkey: settings.hotkeys.paste_last,
        }
    }
}

#[derive(Clone)]
pub struct AppService {
    settings_store: JsonSettingsStore,
    hotkey_state: Arc<Mutex<HotkeyRuntimeState>>,
    toggle_hotkey_state: Arc<Mutex<HotkeyRuntimeState>>,
    paste_last_hotkey_state: Arc<Mutex<HotkeyRuntimeState>>,
    dictation_runtime: Arc<Mutex<DictationRuntimeState>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct HotkeyRuntimeState {
    configured_shortcut: Option<String>,
    active_accelerator: Option<String>,
    last_event: Option<HotkeyEventState>,
    registration_error: Option<String>,
    registered: bool,
}

/// Identifies which of the two dictation hotkeys an action applies to. The
/// dictation hotkey follows the configured `DictationSettings::mode` (default
/// press-and-hold); the toggle hotkey always toggles regardless of that mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyRole {
    Dictation,
    Toggle,
    PasteLast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEventState {
    Pressed,
    Released,
}

impl HotkeyEventState {
    fn label(self) -> &'static str {
        match self {
            Self::Pressed => "Pressed",
            Self::Released => "Released",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HotkeyStatusSnapshot {
    configured_shortcut: String,
    active_accelerator: Option<String>,
    last_event: Option<HotkeyEventState>,
    registration_error: Option<String>,
    registered: bool,
}

trait RuntimeDictationEngine {
    fn state(&self) -> DictationState;
    fn start_recording(&mut self) -> Result<(), DictationError>;
    fn stop_transcribe_insert(&mut self) -> Result<(), DictationError>;
    fn paste_last(&mut self) -> Result<(), DictationError>;
    fn cancel(&mut self);
    fn hotkey(&mut self, event: HotkeyEvent) -> Result<(), DictationError>;
    fn hotkey_with_mode(
        &mut self,
        mode: DictationMode,
        event: HotkeyEvent,
    ) -> Result<(), DictationError>;
    fn last_transcript(&self) -> Option<&str>;
}

impl<Targets, Recorder, Transcriber, Processor, Inserter> RuntimeDictationEngine
    for DictationEngine<Targets, Recorder, Transcriber, Processor, Inserter>
where
    Targets: TargetAppTracker,
    Recorder: AudioRecorder,
    Transcriber: TranscriptionProvider,
    Processor: TranscriptProcessor,
    Inserter: TextInsertionService,
{
    fn state(&self) -> DictationState {
        DictationEngine::state(self)
    }

    fn start_recording(&mut self) -> Result<(), DictationError> {
        DictationEngine::start_recording(self)
    }

    fn stop_transcribe_insert(&mut self) -> Result<(), DictationError> {
        DictationEngine::stop_transcribe_insert(self)
    }

    fn paste_last(&mut self) -> Result<(), DictationError> {
        DictationEngine::paste_last(self)
    }

    fn cancel(&mut self) {
        DictationEngine::cancel(self);
    }

    fn hotkey(&mut self, event: HotkeyEvent) -> Result<(), DictationError> {
        DictationEngine::hotkey(self, event)
    }

    fn hotkey_with_mode(
        &mut self,
        mode: DictationMode,
        event: HotkeyEvent,
    ) -> Result<(), DictationError> {
        DictationEngine::hotkey_with_mode(self, mode, event)
    }

    fn last_transcript(&self) -> Option<&str> {
        DictationEngine::last_transcript(self)
    }
}

type BoxedDictationEngine = Box<dyn RuntimeDictationEngine + Send>;

#[derive(Default)]
struct DictationRuntimeState {
    engine: Option<BoxedDictationEngine>,
    settings: Option<AppSettings>,
    recovery: Option<RecoveryDto>,
    last_transcript: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DictationStatusSnapshot {
    state: DictationState,
    recovery: Option<RecoveryDto>,
    last_transcript: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManualDictationAction {
    Start,
    Stop,
    Cancel,
}

impl Default for AppService {
    fn default() -> Self {
        Self {
            settings_store: JsonSettingsStore::default_for_app("VerboScribe 2"),
            hotkey_state: Arc::new(Mutex::new(HotkeyRuntimeState::default())),
            toggle_hotkey_state: Arc::new(Mutex::new(HotkeyRuntimeState::default())),
            paste_last_hotkey_state: Arc::new(Mutex::new(HotkeyRuntimeState::default())),
            dictation_runtime: Arc::new(Mutex::new(DictationRuntimeState::default())),
        }
    }
}

impl AppService {
    #[cfg(test)]
    pub fn new(settings_store: JsonSettingsStore) -> Self {
        Self {
            settings_store,
            hotkey_state: Arc::new(Mutex::new(HotkeyRuntimeState::default())),
            toggle_hotkey_state: Arc::new(Mutex::new(HotkeyRuntimeState::default())),
            paste_last_hotkey_state: Arc::new(Mutex::new(HotkeyRuntimeState::default())),
            dictation_runtime: Arc::new(Mutex::new(DictationRuntimeState::default())),
        }
    }

    pub fn app_status(&self) -> AppStatusDto {
        let settings = self.settings_store.load();
        let (settings, recovery) = match settings {
            Ok(settings) => (settings, "No recovery needed".to_string()),
            Err(error) => (
                AppSettings::default(),
                format!("Settings unavailable: {error}"),
            ),
        };
        let hotkey_status = self.hotkey_status(HotkeyRole::Dictation, &settings.hotkeys.dictation);
        let toggle_hotkey_status =
            self.hotkey_status(HotkeyRole::Toggle, &settings.hotkeys.dictation_toggle);
        let paste_last_hotkey_status =
            self.hotkey_status(HotkeyRole::PasteLast, &settings.hotkeys.paste_last);
        let dictation_status = self.dictation_status_snapshot();
        let recovery = hotkey_status
            .registration_error
            .as_ref()
            .or(toggle_hotkey_status.registration_error.as_ref())
            .or(paste_last_hotkey_status.registration_error.as_ref())
            .map(|error| format!("Hotkey unavailable: {error}"))
            .or_else(|| {
                dictation_status
                    .recovery
                    .as_ref()
                    .map(format_recovery_summary)
            })
            .unwrap_or(recovery);

        AppStatusDto {
            app_status: app_status_label(dictation_status.state).to_string(),
            engine_state: state_label(dictation_status.state).to_string(),
            provider: settings.transcription.provider.label().to_string(),
            dictation_mode: dictation_mode_name(settings.dictation.mode).to_string(),
            hotkey: format_hotkey_status(&hotkey_status),
            toggle_hotkey: format_hotkey_status(&toggle_hotkey_status),
            paste_last_hotkey: format_hotkey_status(&paste_last_hotkey_status),
            usage_hint: usage_hint(
                settings.dictation.mode,
                dictation_status.state,
                &hotkey_status.configured_shortcut,
            ),
            recovery,
            last_transcript: dictation_status.last_transcript.unwrap_or_default(),
        }
    }

    pub fn settings(&self) -> Result<SettingsDto, String> {
        self.settings_store
            .load_or_create()
            .map(SettingsDto::from)
            .map_err(|error| error.to_string())
    }

    pub fn save_settings(&self, settings: SettingsDto) -> Result<SettingsDto, String> {
        let settings = AppSettings::try_from(settings)?;
        self.settings_store
            .save(&settings)
            .map_err(|error| error.to_string())?;
        self.mutate_dictation_runtime(|runtime| {
            let should_reset_engine = runtime
                .engine
                .as_ref()
                .is_none_or(|engine| engine.state() == DictationState::Idle);
            if should_reset_engine {
                runtime.engine = None;
                runtime.settings = None;
                runtime.recovery = None;
            }
        });
        Ok(SettingsDto::from(settings))
    }

    pub fn runtime_status(&self) -> RuntimeEventDto {
        let status = self.dictation_status_snapshot();
        if let Some(recovery) = status.recovery {
            return RuntimeEventDto {
                phase: "failed".to_string(),
                message: recovery.title.clone(),
                recovery: Some(recovery),
                transcript: status.last_transcript,
            };
        }

        match status.state {
            DictationState::Idle => {
                if status.last_transcript.is_some() {
                    status_event(
                        "succeeded",
                        "Last dictation captured",
                        status.last_transcript,
                    )
                } else {
                    status_event("idle", "Ready for dictation", None)
                }
            }
            DictationState::Starting => status_event("starting", "Preparing dictation", None),
            DictationState::Recording => status_event("recording", "Recording audio", None),
            DictationState::Transcribing => {
                status_event("transcribing", "Transcribing audio", None)
            }
        }
    }

    pub fn dry_run_dictation_events(&self) -> Result<Vec<RuntimeEventDto>, String> {
        let settings = self
            .settings_store
            .load()
            .map_err(|error| error.to_string())?;
        let mut events = vec![self.runtime_status()];
        let mut config = settings.dictation_config();
        config.min_recording_ms = 1;
        let mut engine = self.fake_engine(config);

        if let Err(error) = engine.start_recording() {
            events.push(recovery_event(error, None, current_recovery_platform()));
            return Ok(events);
        }
        events.push(status_event("recording", "Recording audio", None));
        events.push(status_event("transcribing", "Transcribing audio", None));

        if let Err(error) = engine.stop_transcribe_insert() {
            events.push(recovery_event(
                error,
                engine.last_transcript().map(ToString::to_string),
                current_recovery_platform(),
            ));
            return Ok(events);
        }

        events.push(status_event(
            "succeeded",
            "Transcript inserted",
            engine.last_transcript().map(ToString::to_string),
        ));
        Ok(events)
    }

    pub fn dry_run_dictation_state(&self) -> Result<DictationStatusDto, String> {
        let settings = self
            .settings_store
            .load()
            .map_err(|error| error.to_string())?;
        let mut engine = self.fake_engine(DictationConfig {
            mode: settings.dictation.mode.into(),
            min_recording_ms: 1,
        });

        engine
            .start_recording()
            .map_err(|error| error.to_string())?;
        engine
            .stop_transcribe_insert()
            .map_err(|error| error.to_string())?;

        Ok(DictationStatusDto {
            state: state_label(engine.state()).to_string(),
            last_transcript: engine.last_transcript().map(ToString::to_string),
        })
    }

    pub fn start_dictation(&self) -> Result<DictationStatusDto, String> {
        self.drive_manual_dictation(ManualDictationAction::Start)
    }

    pub fn stop_dictation(&self) -> Result<DictationStatusDto, String> {
        self.drive_manual_dictation(ManualDictationAction::Stop)
    }

    pub fn cancel_dictation(&self) -> Result<DictationStatusDto, String> {
        self.drive_manual_dictation(ManualDictationAction::Cancel)
    }

    pub fn paste_last_transcript(&self) -> Result<DictationStatusDto, String> {
        let has_transcript = self.with_dictation_runtime(|runtime| {
            runtime.last_transcript.is_some()
                || runtime
                    .engine
                    .as_ref()
                    .is_some_and(|engine| engine.last_transcript().is_some())
        });
        if !has_transcript {
            self.mutate_dictation_runtime(|runtime| {
                runtime.recovery = Some(recovery_for_error(
                    &DictationError::NoTranscript,
                    current_recovery_platform(),
                ));
            });
            return Err(DictationError::NoTranscript.to_string());
        }

        self.drive_dictation(|engine| engine.paste_last())
    }

    pub fn handle_hotkey_event(
        &self,
        event: HotkeyEventState,
    ) -> Result<DictationStatusDto, String> {
        self.record_hotkey_event(HotkeyRole::Dictation, event);
        let event = hotkey_event(event);
        self.drive_dictation(|engine| engine.hotkey(event))
    }

    /// Drive the dedicated toggle hotkey. Always uses `DictationMode::Toggle`
    /// regardless of the configured dictation mode, so a tap starts dictation
    /// and a second tap stops it.
    pub fn handle_toggle_hotkey_event(
        &self,
        event: HotkeyEventState,
    ) -> Result<DictationStatusDto, String> {
        self.record_hotkey_event(HotkeyRole::Toggle, event);
        let event = hotkey_event(event);
        self.drive_dictation(|engine| engine.hotkey_with_mode(DictationMode::Toggle, event))
    }

    /// Drive the dedicated paste-last hotkey. Only the key press should retry
    /// insertion; release is ignored so a normal tap does not double-trigger.
    pub fn handle_paste_last_hotkey_event(
        &self,
        event: HotkeyEventState,
    ) -> Result<DictationStatusDto, String> {
        self.record_hotkey_event(HotkeyRole::PasteLast, event);
        match event {
            HotkeyEventState::Pressed => self.paste_last_transcript(),
            HotkeyEventState::Released => Ok(self.current_dictation_status()),
        }
    }

    fn dictation_status_snapshot(&self) -> DictationStatusSnapshot {
        self.with_dictation_runtime(|runtime| DictationStatusSnapshot {
            state: runtime
                .engine
                .as_ref()
                .map(|engine| engine.state())
                .unwrap_or(DictationState::Idle),
            recovery: runtime.recovery.clone(),
            last_transcript: runtime.last_transcript.clone(),
        })
    }

    fn drive_manual_dictation(
        &self,
        action: ManualDictationAction,
    ) -> Result<DictationStatusDto, String> {
        match action {
            ManualDictationAction::Start => self.drive_dictation(|engine| engine.start_recording()),
            ManualDictationAction::Stop => {
                self.drive_dictation(|engine| engine.stop_transcribe_insert())
            }
            ManualDictationAction::Cancel => {
                self.mutate_dictation_runtime(|runtime| {
                    if let Some(engine) = runtime.engine.as_mut() {
                        engine.cancel();
                    }
                    runtime.recovery = None;
                });
                Ok(self.current_dictation_status())
            }
        }
    }

    fn drive_dictation(
        &self,
        run: impl FnOnce(&mut dyn RuntimeDictationEngine) -> Result<(), DictationError>,
    ) -> Result<DictationStatusDto, String> {
        let settings = self
            .settings_store
            .load_or_create()
            .map_err(|error| error.to_string())?;

        let result = self.mutate_dictation_runtime_result(|runtime| {
            if let Err(error) = Self::ensure_dictation_engine(runtime, &settings) {
                runtime.recovery = Some(recovery_for_error(&error, current_recovery_platform()));
                return Err(error);
            }
            let engine = runtime
                .engine
                .as_mut()
                .expect("dictation engine should exist after successful setup");

            match run(engine.as_mut()) {
                Ok(()) => {
                    runtime.recovery = None;
                    runtime.last_transcript = engine.last_transcript().map(ToString::to_string);
                    Ok(())
                }
                Err(error) => {
                    runtime.recovery =
                        Some(recovery_for_error(&error, current_recovery_platform()));
                    runtime.last_transcript = engine.last_transcript().map(ToString::to_string);
                    Err(error)
                }
            }
        });

        match result {
            Ok(()) => Ok(self.current_dictation_status()),
            Err(error) => Err(error.to_string()),
        }
    }

    fn ensure_dictation_engine(
        runtime: &mut DictationRuntimeState,
        settings: &AppSettings,
    ) -> Result<(), DictationError> {
        let should_rebuild = runtime.engine.is_none()
            || (runtime.engine.as_ref().map(|engine| engine.state()) == Some(DictationState::Idle)
                && runtime.settings.as_ref() != Some(settings));

        if should_rebuild {
            runtime.engine = Some(Self::build_desktop_engine(settings)?);
            runtime.settings = Some(settings.clone());
            runtime.recovery = None;
        }

        Ok(())
    }

    fn build_desktop_engine(
        settings: &AppSettings,
    ) -> Result<BoxedDictationEngine, DictationError> {
        Ok(Box::new(DictationEngine::new(
            settings.dictation_config(),
            DesktopTargetTracker::default(),
            CpalAudioRecorder::new(std::env::temp_dir().join("verboscribe2").join("recordings")),
            build_transcriber(settings)?,
            DefaultTranscriptProcessor::new(TranscriptProcessingOptions::default()),
            DesktopTextInserter::default(),
        )))
    }

    fn current_dictation_status(&self) -> DictationStatusDto {
        let status = self.dictation_status_snapshot();
        DictationStatusDto {
            state: state_label(status.state).to_string(),
            last_transcript: status.last_transcript,
        }
    }

    fn fake_engine(
        &self,
        config: DictationConfig,
    ) -> DictationEngine<FakeTargets, FakeRecorder, FakeTranscriber, FakeProcessor, FakeInserter>
    {
        self.fake_engine_with_inserter(config, FakeInserter::succeed())
    }

    fn fake_engine_with_inserter(
        &self,
        config: DictationConfig,
        inserter: FakeInserter,
    ) -> DictationEngine<FakeTargets, FakeRecorder, FakeTranscriber, FakeProcessor, FakeInserter>
    {
        DictationEngine::new(
            config,
            FakeTargets,
            FakeRecorder,
            FakeTranscriber,
            FakeProcessor,
            inserter,
        )
    }

    #[cfg(test)]
    fn install_test_dictation_engine(&self, settings: AppSettings, engine: BoxedDictationEngine) {
        self.settings_store
            .save(&settings)
            .expect("test settings should save");
        self.mutate_dictation_runtime(|runtime| {
            runtime.engine = Some(engine);
            runtime.settings = Some(settings);
            runtime.recovery = None;
            runtime.last_transcript = None;
        });
    }

    pub fn set_hotkey_registered(
        &self,
        role: HotkeyRole,
        configured_shortcut: String,
        active_accelerator: String,
    ) {
        self.mutate_hotkey_state(role, |state| {
            state.configured_shortcut = Some(configured_shortcut);
            state.active_accelerator = Some(active_accelerator);
            state.registration_error = None;
            state.registered = true;
        });
    }

    pub fn set_hotkey_registration_failed(
        &self,
        role: HotkeyRole,
        configured_shortcut: String,
        error: String,
    ) {
        self.mutate_hotkey_state(role, |state| {
            state.configured_shortcut = Some(configured_shortcut);
            state.active_accelerator = None;
            state.last_event = None;
            state.registration_error = Some(error);
            state.registered = false;
        });
    }

    pub fn clear_hotkey_registration(&self, role: HotkeyRole, configured_shortcut: String) {
        self.mutate_hotkey_state(role, |state| {
            state.configured_shortcut = Some(configured_shortcut);
            state.active_accelerator = None;
            state.last_event = None;
            state.registration_error = None;
            state.registered = false;
        });
    }

    pub fn active_hotkey_accelerator(&self, role: HotkeyRole) -> Option<String> {
        self.with_hotkey_state(role, |state| state.active_accelerator.clone())
    }

    pub fn record_hotkey_event(&self, role: HotkeyRole, event: HotkeyEventState) {
        self.mutate_hotkey_state(role, |state| {
            state.last_event = Some(event);
        });
    }

    fn hotkey_status(&self, role: HotkeyRole, configured_shortcut: &str) -> HotkeyStatusSnapshot {
        self.with_hotkey_state(role, |state| HotkeyStatusSnapshot {
            configured_shortcut: state
                .configured_shortcut
                .clone()
                .unwrap_or_else(|| configured_shortcut.to_string()),
            active_accelerator: state.active_accelerator.clone(),
            last_event: state.last_event,
            registration_error: state.registration_error.clone(),
            registered: state.registered,
        })
    }

    fn hotkey_state_for(&self, role: HotkeyRole) -> &Arc<Mutex<HotkeyRuntimeState>> {
        match role {
            HotkeyRole::Dictation => &self.hotkey_state,
            HotkeyRole::Toggle => &self.toggle_hotkey_state,
            HotkeyRole::PasteLast => &self.paste_last_hotkey_state,
        }
    }

    fn mutate_hotkey_state(&self, role: HotkeyRole, mutate: impl FnOnce(&mut HotkeyRuntimeState)) {
        if let Ok(mut state) = self.hotkey_state_for(role).lock() {
            mutate(&mut state);
        }
    }

    fn with_hotkey_state<T>(
        &self,
        role: HotkeyRole,
        read: impl FnOnce(&HotkeyRuntimeState) -> T,
    ) -> T {
        let state = self
            .hotkey_state_for(role)
            .lock()
            .expect("hotkey state lock poisoned");
        read(&state)
    }

    fn mutate_dictation_runtime(&self, mutate: impl FnOnce(&mut DictationRuntimeState)) {
        if let Ok(mut runtime) = self.dictation_runtime.lock() {
            mutate(&mut runtime);
        }
    }

    fn mutate_dictation_runtime_result<T>(
        &self,
        mutate: impl FnOnce(&mut DictationRuntimeState) -> Result<T, DictationError>,
    ) -> Result<T, DictationError> {
        let mut runtime = self
            .dictation_runtime
            .lock()
            .expect("dictation runtime lock poisoned");
        mutate(&mut runtime)
    }

    fn with_dictation_runtime<T>(&self, read: impl FnOnce(&DictationRuntimeState) -> T) -> T {
        let runtime = self
            .dictation_runtime
            .lock()
            .expect("dictation runtime lock poisoned");
        read(&runtime)
    }
}

impl TryFrom<SettingsDto> for AppSettings {
    type Error = String;

    fn try_from(dto: SettingsDto) -> Result<Self, Self::Error> {
        let provider = match dto.provider.as_str() {
            "whisperCpp" => TranscriptionProviderKind::WhisperCpp,
            unsupported => {
                return Err(format!("unsupported transcription provider: {unsupported}"))
            }
        };
        let dictation_mode = match dto.dictation_mode.as_str() {
            "pressAndHold" => SettingsDictationMode::PressAndHold,
            "toggle" => SettingsDictationMode::Toggle,
            unsupported => return Err(format!("unsupported dictation mode: {unsupported}")),
        };

        let mut settings = AppSettings::default();
        settings.transcription.provider = provider;
        settings.transcription.whisper_cpp.binary_path =
            dto.whisper_cpp_binary_path.map(PathBuf::from);
        settings.transcription.whisper_cpp.model_path =
            dto.whisper_cpp_model_path.map(PathBuf::from);
        settings.transcription.whisper_cpp.language = dto.language;
        settings.transcription.whisper_cpp.prompt_context = dto.whisper_cpp_prompt_context;
        settings.transcription.whisper_cpp.pinned_terms = dto.whisper_cpp_pinned_terms;
        settings.dictation.mode = dictation_mode;
        settings.dictation.min_recording_ms = dto.min_recording_ms;
        settings.hotkeys.dictation = dto.hotkey;
        settings.hotkeys.dictation_toggle = dto.toggle_hotkey;
        settings.hotkeys.paste_last = dto.paste_last_hotkey;
        Ok(settings)
    }
}

pub fn state_label(state: DictationState) -> &'static str {
    match state {
        DictationState::Idle => "Idle",
        DictationState::Starting => "Starting",
        DictationState::Recording => "Recording",
        DictationState::Transcribing => "Transcribing",
    }
}

fn app_status_label(state: DictationState) -> &'static str {
    match state {
        DictationState::Idle => "Ready for dictation",
        DictationState::Starting => "Preparing dictation",
        DictationState::Recording => "Recording audio",
        DictationState::Transcribing => "Transcribing audio",
    }
}

fn provider_name(provider: TranscriptionProviderKind) -> &'static str {
    match provider {
        TranscriptionProviderKind::WhisperCpp => "whisperCpp",
    }
}

fn dictation_mode_name(mode: SettingsDictationMode) -> &'static str {
    match mode {
        SettingsDictationMode::PressAndHold => "pressAndHold",
        SettingsDictationMode::Toggle => "toggle",
    }
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn format_hotkey_status(status: &HotkeyStatusSnapshot) -> String {
    let mut details = if status.registered {
        vec!["Registered".to_string()]
    } else if status.registration_error.is_some() {
        vec!["Registration failed".to_string()]
    } else {
        vec!["Not registered".to_string()]
    };
    if let Some(event) = status.last_event {
        details.push(format!("last {}", event.label()));
    }
    if status.active_accelerator.is_some() && status.registered {
        details.push("active".to_string());
    }
    format!("{} ({})", status.configured_shortcut, details.join(", "))
}

fn usage_hint(
    mode: SettingsDictationMode,
    state: DictationState,
    configured_shortcut: &str,
) -> String {
    match (mode, state) {
        (SettingsDictationMode::PressAndHold, DictationState::Idle) => {
            format!("Hold {configured_shortcut} while speaking, then release to transcribe.")
        }
        (SettingsDictationMode::PressAndHold, DictationState::Starting) => {
            format!("Keep holding {configured_shortcut} while recording starts.")
        }
        (SettingsDictationMode::PressAndHold, DictationState::Recording) => {
            format!("Keep holding {configured_shortcut} while speaking. Release to transcribe.")
        }
        (SettingsDictationMode::PressAndHold, DictationState::Transcribing) => {
            "Wait while VerboScribe transcribes and pastes the latest recording.".to_string()
        }
        (SettingsDictationMode::Toggle, DictationState::Idle) => {
            format!("Press {configured_shortcut} to start dictation, then press it again to stop.")
        }
        (SettingsDictationMode::Toggle, DictationState::Starting) => {
            format!("Recording is starting. Press {configured_shortcut} again if you need to stop early.")
        }
        (SettingsDictationMode::Toggle, DictationState::Recording) => {
            format!("Speak now. Press {configured_shortcut} again to stop and transcribe.")
        }
        (SettingsDictationMode::Toggle, DictationState::Transcribing) => {
            "Wait while VerboScribe transcribes and pastes the latest recording.".to_string()
        }
    }
}

fn hotkey_event(event: HotkeyEventState) -> HotkeyEvent {
    match event {
        HotkeyEventState::Pressed => HotkeyEvent::Pressed,
        HotkeyEventState::Released => HotkeyEvent::Released,
    }
}

fn status_event(phase: &str, message: &str, transcript: Option<String>) -> RuntimeEventDto {
    RuntimeEventDto {
        phase: phase.to_string(),
        message: message.to_string(),
        recovery: None,
        transcript,
    }
}

fn recovery_event(
    error: DictationError,
    transcript: Option<String>,
    platform: RecoveryPlatform,
) -> RuntimeEventDto {
    let recovery = recovery_for_error(&error, platform);

    RuntimeEventDto {
        phase: "failed".to_string(),
        message: recovery.title.clone(),
        recovery: Some(recovery),
        transcript,
    }
}

fn recovery_for_error(error: &DictationError, platform: RecoveryPlatform) -> RecoveryDto {
    match error {
        DictationError::MicrophonePermissionDenied => RecoveryDto {
            title: "Microphone permission required".to_string(),
            detail: "VerboScribe cannot record until microphone access is allowed.".to_string(),
            next_step: microphone_permission_step(platform),
        },
        DictationError::Recording(message) => {
            if recording_error_is_silent_capture(message) {
                RecoveryDto {
                    title: "No microphone signal detected".to_string(),
                    detail: message.clone(),
                    next_step: silent_capture_step(platform),
                }
            } else {
                RecoveryDto {
                    title: "Recording failed".to_string(),
                    detail: message.clone(),
                    next_step: "Check the selected microphone and try recording again.".to_string(),
                }
            }
        }
        DictationError::RecordingTooShort => RecoveryDto {
            title: "Recording was too short".to_string(),
            detail:
                "No transcript was created because the recording ended before the minimum duration."
                    .to_string(),
            next_step: "Hold the dictation hotkey longer before releasing it.".to_string(),
        },
        DictationError::Transcription(message) => RecoveryDto {
            title: "Transcription failed".to_string(),
            detail: message.clone(),
            next_step: "Check the provider binary, model path, and audio file, then retry."
                .to_string(),
        },
        DictationError::Paste(message) => paste_recovery(message, platform),
        DictationError::NoTranscript => RecoveryDto {
            title: "No transcript available".to_string(),
            detail: "There is no previous transcript to paste.".to_string(),
            next_step: "Record a new dictation first.".to_string(),
        },
    }
}

fn format_recovery_summary(recovery: &RecoveryDto) -> String {
    format!(
        "{}: {} Next: {}",
        recovery.title, recovery.detail, recovery.next_step
    )
}

fn paste_recovery(message: &str, platform: RecoveryPlatform) -> RecoveryDto {
    if platform == RecoveryPlatform::Macos && paste_error_needs_accessibility_permission(message) {
        RecoveryDto {
            title: "Accessibility permission required".to_string(),
            detail:
                "macOS blocked VerboScribe from sending the paste shortcut because Accessibility access is not available."
                    .to_string(),
            next_step: "Open System Settings > Privacy & Security > Accessibility and allow VerboScribe 2, then retry dictation."
                .to_string(),
        }
    } else {
        RecoveryDto {
            title: "Paste failed".to_string(),
            detail: message.to_string(),
            next_step:
                "The transcript remains available so it can be pasted manually or retried with Paste last transcript."
                    .to_string(),
        }
    }
}

fn paste_error_needs_accessibility_permission(message: &str) -> bool {
    let lowercase = message.to_ascii_lowercase();
    lowercase.contains("not allowed to send keystrokes")
        || lowercase.contains("accessibility permission is not granted")
        || (lowercase.contains("system events") && lowercase.contains("1002"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryPlatform {
    Macos,
    Windows,
    Other,
}

fn current_recovery_platform() -> RecoveryPlatform {
    if cfg!(target_os = "macos") {
        RecoveryPlatform::Macos
    } else if cfg!(target_os = "windows") {
        RecoveryPlatform::Windows
    } else {
        RecoveryPlatform::Other
    }
}

fn microphone_permission_step(platform: RecoveryPlatform) -> String {
    match platform {
        RecoveryPlatform::Macos => {
            "Open System Settings > Privacy & Security > Microphone and allow VerboScribe 2."
                .to_string()
        }
        RecoveryPlatform::Windows => {
            "Open Settings > Privacy & security > Microphone and allow desktop app access."
                .to_string()
        }
        RecoveryPlatform::Other => {
            "Allow microphone access in the operating system privacy settings.".to_string()
        }
    }
}

fn silent_capture_step(platform: RecoveryPlatform) -> String {
    match platform {
        RecoveryPlatform::Macos => "Open System Settings > Privacy & Security > Microphone and confirm VerboScribe 2 is allowed, then check System Settings > Sound > Input for the active microphone."
            .to_string(),
        RecoveryPlatform::Windows => {
            "Open Settings > Privacy & security > Microphone, allow desktop app access, then confirm the active input device in Sound settings."
                .to_string()
        }
        RecoveryPlatform::Other => {
            "Check the operating system microphone permission and confirm the active input device."
                .to_string()
        }
    }
}

fn recording_error_is_silent_capture(message: &str) -> bool {
    message.contains("captured audio was silent")
}

fn build_transcriber(settings: &AppSettings) -> Result<WhisperCppTranscriber, DictationError> {
    let binary_path = settings
        .transcription
        .whisper_cpp
        .binary_path
        .clone()
        .ok_or_else(|| {
            DictationError::Transcription("whisper.cpp binary path is not configured".to_string())
        })?;
    let model_path = settings
        .transcription
        .whisper_cpp
        .model_path
        .clone()
        .ok_or_else(|| {
            DictationError::Transcription("whisper.cpp model path is not configured".to_string())
        })?;

    let mut config = WhisperCppConfig::new(binary_path, model_path);
    config.language_code = if settings
        .transcription
        .whisper_cpp
        .language
        .trim()
        .is_empty()
    {
        "en".to_string()
    } else {
        settings
            .transcription
            .whisper_cpp
            .language
            .trim()
            .to_string()
    };
    config.prompt_context = whisper_prompt_context(settings);

    Ok(WhisperCppTranscriber::new(config))
}

fn whisper_prompt_context(settings: &AppSettings) -> String {
    let user_context = settings.transcription.whisper_cpp.prompt_context.trim();
    let user_pinned_terms = settings.transcription.whisper_cpp.pinned_terms.trim();
    if user_context.is_empty() && user_pinned_terms.is_empty() {
        return default_whisper_prompt_context();
    }

    let context = if user_context.is_empty() {
        DEFAULT_WHISPER_PROMPT_CONTEXT.to_string()
    } else {
        format!("{DEFAULT_WHISPER_PROMPT_CONTEXT}\n\n{user_context}")
    };

    let pinned_terms = if user_pinned_terms.is_empty() {
        DEFAULT_WHISPER_PINNED_TERMS.to_string()
    } else {
        format!("{DEFAULT_WHISPER_PINNED_TERMS}\n{user_pinned_terms}")
    };

    PersonalDictionary::combined_prompt(&context, &pinned_terms)
}

fn default_whisper_prompt_context() -> String {
    PersonalDictionary::combined_prompt(
        DEFAULT_WHISPER_PROMPT_CONTEXT,
        DEFAULT_WHISPER_PINNED_TERMS,
    )
}

struct FakeTargets;

impl TargetAppTracker for FakeTargets {
    fn capture_target(&mut self) -> Option<TargetApp> {
        Some(TargetApp {
            name: Some("Dry Run Target".to_string()),
            identifier: Some("local.verboscribe2.dry-run".to_string()),
        })
    }
}

struct FakeRecorder;

impl AudioRecorder for FakeRecorder {
    fn request_permission(&mut self) -> Result<(), DictationError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), DictationError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<AudioCapture, DictationError> {
        Ok(AudioCapture {
            path: std::env::temp_dir().join("verboscribe2-dry-run.wav"),
            duration_ms: 100,
        })
    }

    fn cancel(&mut self) {}
}

struct FakeTranscriber;

impl TranscriptionProvider for FakeTranscriber {
    fn transcribe(&mut self, _audio: &AudioCapture) -> Result<String, DictationError> {
        Ok("dry run transcript".to_string())
    }
}

struct FakeProcessor;

impl TranscriptProcessor for FakeProcessor {
    fn process(&self, raw: &str, _target: Option<&TargetApp>) -> ProcessedTranscript {
        ProcessedTranscript {
            raw: raw.to_string(),
            inserted: raw.to_string(),
        }
    }
}

struct FakeInserter {
    outcomes: Vec<Result<(), DictationError>>,
}

impl FakeInserter {
    fn succeed() -> Self {
        Self {
            outcomes: vec![Ok(())],
        }
    }

    #[cfg(test)]
    fn fail(message: &str) -> Self {
        Self {
            outcomes: vec![Err(DictationError::Paste(message.to_string()))],
        }
    }

    #[cfg(test)]
    fn fail_once_then_succeed(message: &str) -> Self {
        Self {
            outcomes: vec![Err(DictationError::Paste(message.to_string())), Ok(())],
        }
    }
}

impl TextInsertionService for FakeInserter {
    fn insert(&mut self, _text: &str, _target: Option<&TargetApp>) -> Result<(), DictationError> {
        if self.outcomes.len() > 1 {
            return self.outcomes.remove(0);
        }

        self.outcomes.first().cloned().unwrap_or(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_status_returns_shell_defaults() {
        let (_temp_dir, service) = temp_service();
        let status = service.app_status();

        assert_eq!(status.engine_state, "Idle");
        assert_eq!(status.provider, "whisper.cpp");
        assert_eq!(status.dictation_mode, "pressAndHold");
        assert_eq!(status.hotkey, "Control+Option+Space (Not registered)");
        assert_eq!(status.toggle_hotkey, "Control+Option+D (Not registered)");
        assert_eq!(
            status.paste_last_hotkey,
            "Control+Option+V (Not registered)"
        );
        assert_eq!(
            status.usage_hint,
            "Hold Control+Option+Space while speaking, then release to transcribe."
        );
        assert_eq!(status.recovery, "No recovery needed");
    }

    #[test]
    fn dry_run_delegates_through_core_engine() {
        let (_temp_dir, service) = temp_service();
        let status = service.dry_run_dictation_state().unwrap();

        assert_eq!(status.state, "Idle");
        assert_eq!(
            status.last_transcript,
            Some("dry run transcript".to_string())
        );
    }

    #[test]
    fn dry_run_emits_runtime_status_events() {
        let (_temp_dir, service) = temp_service();

        let events = service.dry_run_dictation_events().unwrap();

        assert_eq!(
            events
                .iter()
                .map(|event| event.phase.as_str())
                .collect::<Vec<_>>(),
            vec!["idle", "recording", "transcribing", "succeeded"]
        );
        assert_eq!(
            events.last().and_then(|event| event.transcript.as_deref()),
            Some("dry run transcript")
        );
    }

    #[test]
    fn smoke_dictation_cycle_succeeds_with_injected_adapters() {
        let (_temp_dir, service) = temp_service();
        install_smoke_engine(&service, FakeInserter::succeed());

        let started = service.start_dictation().unwrap();
        let runtime_while_recording = service.runtime_status();
        let stopped = service.stop_dictation().unwrap();
        let runtime_after_stop = service.runtime_status();

        assert_eq!(started.state, "Recording");
        assert_eq!(runtime_while_recording.phase, "recording");
        assert_eq!(stopped.state, "Idle");
        assert_eq!(
            stopped.last_transcript.as_deref(),
            Some("dry run transcript")
        );
        assert_eq!(runtime_after_stop.phase, "succeeded");
        assert_eq!(
            runtime_after_stop.transcript.as_deref(),
            Some("dry run transcript")
        );
        assert_eq!(service.app_status().last_transcript, "dry run transcript");
    }

    #[test]
    fn toggle_hotkey_taps_start_and_stop_on_a_press_and_hold_engine() {
        let (_temp_dir, service) = temp_service();
        // install_smoke_engine configures press-and-hold; the toggle hotkey must
        // still toggle without the user changing the dictation mode.
        install_smoke_engine(&service, FakeInserter::succeed());

        let started = service
            .handle_toggle_hotkey_event(HotkeyEventState::Pressed)
            .unwrap();
        assert_eq!(started.state, "Recording");

        // A tap is press-then-release; release must not stop a toggle recording.
        let after_release = service
            .handle_toggle_hotkey_event(HotkeyEventState::Released)
            .unwrap();
        assert_eq!(after_release.state, "Recording");

        let stopped = service
            .handle_toggle_hotkey_event(HotkeyEventState::Pressed)
            .unwrap();
        assert_eq!(stopped.state, "Idle");
        assert_eq!(
            stopped.last_transcript.as_deref(),
            Some("dry run transcript")
        );
    }

    #[test]
    fn app_status_reports_toggle_hotkey_registration_independently() {
        let (_temp_dir, service) = temp_service();
        service.set_hotkey_registered(
            HotkeyRole::Toggle,
            "Control+Option+D".to_string(),
            "ctrl+alt+d".to_string(),
        );

        let status = service.app_status();

        assert_eq!(status.hotkey, "Control+Option+Space (Not registered)");
        assert_eq!(
            status.toggle_hotkey,
            "Control+Option+D (Registered, active)"
        );
        assert_eq!(
            status.paste_last_hotkey,
            "Control+Option+V (Not registered)"
        );
    }

    #[test]
    fn app_status_surfaces_toggle_hotkey_registration_failure_as_recovery() {
        let (_temp_dir, service) = temp_service();
        service.set_hotkey_registration_failed(
            HotkeyRole::Toggle,
            "Control+Option+D".to_string(),
            "shortcut already registered by another app".to_string(),
        );

        let status = service.app_status();

        assert_eq!(
            status.toggle_hotkey,
            "Control+Option+D (Registration failed)"
        );
        assert_eq!(
            status.paste_last_hotkey,
            "Control+Option+V (Not registered)"
        );
        assert!(status.recovery.contains("Hotkey unavailable"));
    }

    #[test]
    fn smoke_dictation_cycle_preserves_transcript_when_paste_fails() {
        let (_temp_dir, service) = temp_service();
        install_smoke_engine(
            &service,
            FakeInserter::fail("target app did not accept paste"),
        );

        service.start_dictation().unwrap();
        let error = service.stop_dictation().unwrap_err();
        let runtime_after_failure = service.runtime_status();

        assert!(error.contains("target app did not accept paste"));
        assert_eq!(runtime_after_failure.phase, "failed");
        assert_eq!(runtime_after_failure.message, "Paste failed");
        assert_eq!(
            runtime_after_failure.transcript.as_deref(),
            Some("dry run transcript")
        );
        assert!(runtime_after_failure
            .recovery
            .as_ref()
            .is_some_and(|recovery| recovery.next_step.contains("remains available")));
        assert_eq!(service.app_status().last_transcript, "dry run transcript");
    }

    #[test]
    fn paste_last_transcript_retries_a_preserved_transcript() {
        let (_temp_dir, service) = temp_service();
        install_smoke_engine(
            &service,
            FakeInserter::fail_once_then_succeed("target app did not accept paste"),
        );

        service.start_dictation().unwrap();
        service.stop_dictation().unwrap_err();

        let retried = service.paste_last_transcript().unwrap();
        let runtime = service.runtime_status();

        assert_eq!(retried.state, "Idle");
        assert_eq!(
            retried.last_transcript.as_deref(),
            Some("dry run transcript")
        );
        assert_eq!(runtime.phase, "succeeded");
        assert_eq!(runtime.message, "Last dictation captured");
        assert_eq!(runtime.transcript.as_deref(), Some("dry run transcript"));
    }

    #[test]
    fn paste_last_hotkey_retries_a_preserved_transcript() {
        let (_temp_dir, service) = temp_service();
        install_smoke_engine(
            &service,
            FakeInserter::fail_once_then_succeed("target app did not accept paste"),
        );

        service.start_dictation().unwrap();
        service.stop_dictation().unwrap_err();

        let retried = service
            .handle_paste_last_hotkey_event(HotkeyEventState::Pressed)
            .unwrap();
        let after_release = service
            .handle_paste_last_hotkey_event(HotkeyEventState::Released)
            .unwrap();
        let runtime = service.runtime_status();

        assert_eq!(retried.state, "Idle");
        assert_eq!(
            retried.last_transcript.as_deref(),
            Some("dry run transcript")
        );
        assert_eq!(after_release.state, "Idle");
        assert_eq!(runtime.phase, "succeeded");
        assert_eq!(runtime.transcript.as_deref(), Some("dry run transcript"));
    }

    #[test]
    fn paste_last_transcript_requires_a_previous_transcript() {
        let (_temp_dir, service) = temp_service();

        let error = service.paste_last_transcript().unwrap_err();
        let runtime = service.runtime_status();

        assert!(error.contains("no transcript is available"));
        assert_eq!(runtime.phase, "failed");
        assert_eq!(runtime.message, "No transcript available");
        assert!(runtime
            .recovery
            .as_ref()
            .is_some_and(|recovery| recovery.detail.contains("previous transcript")));
    }

    #[test]
    fn settings_load_creates_defaults() {
        let (_temp_dir, service) = temp_service();

        let settings = service.settings().unwrap();

        assert_eq!(settings.provider, "whisperCpp");
        assert_eq!(settings.language, "en");
        assert_eq!(settings.whisper_cpp_prompt_context, "");
        assert_eq!(settings.whisper_cpp_pinned_terms, "");
        assert_eq!(settings.dictation_mode, "pressAndHold");
        assert_eq!(settings.hotkey, "Control+Option+Space");
        assert_eq!(settings.toggle_hotkey, "Control+Option+D");
        assert_eq!(settings.paste_last_hotkey, "Control+Option+V");
    }

    #[test]
    fn save_settings_persists_round_trip_values() {
        let (_temp_dir, service) = temp_service();
        let settings = SettingsDto {
            provider: "whisperCpp".to_string(),
            whisper_cpp_binary_path: Some("/bin/whisper-cli".to_string()),
            whisper_cpp_model_path: Some("/models/base.en.bin".to_string()),
            language: "fr".to_string(),
            whisper_cpp_prompt_context: "Prefer medical vocabulary.".to_string(),
            whisper_cpp_pinned_terms: "OpenAI, GPT-5".to_string(),
            dictation_mode: "toggle".to_string(),
            min_recording_ms: 400,
            hotkey: "Control+Shift+D".to_string(),
            toggle_hotkey: "Control+Option+D".to_string(),
            paste_last_hotkey: "Control+Option+V".to_string(),
        };

        let saved = service.save_settings(settings.clone()).unwrap();
        let loaded = service.settings().unwrap();

        assert_eq!(saved, settings);
        assert_eq!(loaded, settings);
    }

    #[test]
    fn save_settings_rejects_unknown_provider() {
        let (_temp_dir, service) = temp_service();
        let mut settings = service.settings().unwrap();
        settings.provider = "groq".to_string();

        let error = service.save_settings(settings).unwrap_err();

        assert!(error.contains("unsupported transcription provider"));
    }

    #[test]
    fn app_status_surfaces_hotkey_registration_failure() {
        let (_temp_dir, service) = temp_service();
        service.set_hotkey_registration_failed(
            HotkeyRole::Dictation,
            "Control+Option+Space".to_string(),
            "shortcut already registered by another app".to_string(),
        );

        let status = service.app_status();

        assert_eq!(status.hotkey, "Control+Option+Space (Registration failed)");
        assert!(status.recovery.contains("Hotkey unavailable"));
    }

    #[test]
    fn app_status_tracks_last_hotkey_event() {
        let (_temp_dir, service) = temp_service();
        service.set_hotkey_registered(
            HotkeyRole::Dictation,
            "Control+Option+Space".to_string(),
            "ctrl+alt+space".to_string(),
        );
        service.record_hotkey_event(HotkeyRole::Dictation, HotkeyEventState::Pressed);

        let status = service.app_status();

        assert_eq!(
            status.hotkey,
            "Control+Option+Space (Registered, last Pressed, active)"
        );
    }

    #[test]
    fn app_status_describes_toggle_mode_usage() {
        let (_temp_dir, service) = temp_service();
        let mut settings = service.settings().unwrap();
        settings.dictation_mode = "toggle".to_string();
        settings.hotkey = "Control+Shift+D".to_string();
        service.save_settings(settings).unwrap();

        let status = service.app_status();

        assert_eq!(status.dictation_mode, "toggle");
        assert_eq!(
            status.usage_hint,
            "Press Control+Shift+D to start dictation, then press it again to stop."
        );
    }

    #[test]
    fn app_status_includes_recovery_next_step() {
        let (_temp_dir, service) = temp_service();
        install_smoke_engine(
            &service,
            FakeInserter::fail("target app did not accept paste"),
        );

        service.start_dictation().unwrap();
        service.stop_dictation().unwrap_err();
        let status = service.app_status();

        assert!(status.recovery.contains("Paste failed"));
        assert!(status.recovery.contains("Next:"));
        assert!(status.recovery.contains("remains available"));
    }

    #[test]
    fn start_dictation_surfaces_missing_provider_configuration() {
        let (_temp_dir, service) = temp_service();

        let error = service.start_dictation().unwrap_err();
        let status = service.runtime_status();

        assert!(error.contains("whisper.cpp binary path is not configured"));
        assert_eq!(status.phase, "failed");
        assert_eq!(status.message, "Transcription failed");
        assert!(status
            .recovery
            .as_ref()
            .is_some_and(|recovery| recovery.detail.contains("binary path is not configured")));
    }

    #[test]
    fn build_transcriber_includes_default_prompt_context() {
        let mut settings = AppSettings::default();
        settings.transcription.whisper_cpp.binary_path = Some("/bin/whisper-cli".into());
        settings.transcription.whisper_cpp.model_path = Some("/models/base.en.bin".into());

        let transcriber = build_transcriber(&settings).unwrap();
        let prompt = &transcriber.config().prompt_context;

        assert!(prompt.contains("VerboScribe 2"));
        assert!(prompt.contains("whisper.cpp"));
        assert!(prompt.contains("TextEdit"));
    }

    #[test]
    fn build_transcriber_merges_default_and_user_prompt_terms() {
        let mut settings = AppSettings::default();
        settings.transcription.whisper_cpp.binary_path = Some("/bin/whisper-cli".into());
        settings.transcription.whisper_cpp.model_path = Some("/models/base.en.bin".into());
        settings.transcription.whisper_cpp.prompt_context =
            "Prefer engineering and product vocabulary when heard clearly.".to_string();
        settings.transcription.whisper_cpp.pinned_terms =
            "Padres, Petco Park, VerboScribe".to_string();

        let transcriber = build_transcriber(&settings).unwrap();
        let prompt = &transcriber.config().prompt_context;

        assert!(prompt.contains(DEFAULT_WHISPER_PROMPT_CONTEXT));
        assert!(prompt.contains("Prefer engineering and product vocabulary"));
        assert!(prompt.contains("Padres"));
        assert!(prompt.contains("Petco Park"));
        assert!(prompt.contains("VerboScribe 2"));
        assert_eq!(prompt.matches("Padres").count(), 1);
    }

    #[test]
    fn paste_failure_event_preserves_transcript_for_manual_recovery() {
        let event = recovery_event(
            DictationError::Paste("target app did not accept paste".to_string()),
            Some("preserved transcript".to_string()),
            RecoveryPlatform::Macos,
        );

        let recovery = event.recovery.unwrap();

        assert_eq!(event.phase, "failed");
        assert_eq!(event.transcript.as_deref(), Some("preserved transcript"));
        assert!(recovery.next_step.contains("remains available"));
        assert!(recovery.next_step.contains("Paste last transcript"));
    }

    #[test]
    fn paste_failure_event_maps_macos_accessibility_denial_to_permission_recovery() {
        let event = recovery_event(
            DictationError::Paste(
                "paste shortcut failed: Accessibility permission is not granted to VerboScribe 2"
                    .to_string(),
            ),
            Some("preserved transcript".to_string()),
            RecoveryPlatform::Macos,
        );

        let recovery = event.recovery.unwrap();

        assert_eq!(recovery.title, "Accessibility permission required");
        assert!(recovery.detail.contains("Accessibility access"));
        assert!(recovery
            .next_step
            .contains("Accessibility and allow VerboScribe 2"));
    }

    #[test]
    fn paste_failure_event_maps_legacy_system_events_denial_to_permission_recovery() {
        let event = recovery_event(
            DictationError::Paste(
                "paste shortcut failed: 36:38: execution error: System Events got an error: osascript is not allowed to send keystrokes. (1002)"
                    .to_string(),
            ),
            Some("preserved transcript".to_string()),
            RecoveryPlatform::Macos,
        );

        let recovery = event.recovery.unwrap();

        assert_eq!(recovery.title, "Accessibility permission required");
        assert!(recovery
            .next_step
            .contains("Accessibility and allow VerboScribe 2"));
    }

    #[test]
    fn silent_capture_recording_failure_maps_to_microphone_signal_recovery() {
        let event = recovery_event(
            DictationError::Recording(
                "captured audio was silent; check Microphone permission and the selected input device, then try again."
                    .to_string(),
            ),
            None,
            RecoveryPlatform::Macos,
        );

        let recovery = event.recovery.unwrap();

        assert_eq!(recovery.title, "No microphone signal detected");
        assert!(recovery
            .next_step
            .contains("Privacy & Security > Microphone"));
        assert!(recovery.next_step.contains("Sound > Input"));
    }

    #[test]
    fn permission_failure_event_includes_platform_specific_recovery() {
        let event = recovery_event(
            DictationError::MicrophonePermissionDenied,
            None,
            RecoveryPlatform::Macos,
        );

        assert!(event
            .recovery
            .unwrap()
            .next_step
            .contains("System Settings > Privacy & Security > Microphone"));
    }

    fn temp_service() -> (tempfile::TempDir, AppService) {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = JsonSettingsStore::new(temp_dir.path().join("settings.json"));
        (temp_dir, AppService::new(store))
    }

    fn install_smoke_engine(service: &AppService, inserter: FakeInserter) {
        let mut settings = AppSettings::default();
        settings.dictation.min_recording_ms = 1;
        let engine = service.fake_engine_with_inserter(settings.dictation_config(), inserter);
        service.install_test_dictation_engine(settings, Box::new(engine));
    }
}
