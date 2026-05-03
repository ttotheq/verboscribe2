use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use verboscribe_core::{
    AudioCapture, AudioRecorder, DictationConfig, DictationEngine, DictationError, DictationState,
    ProcessedTranscript, TargetApp, TargetAppTracker, TextInsertionService, TranscriptProcessor,
    TranscriptionProvider,
};
use verboscribe_storage::{
    AppSettings, JsonSettingsStore, SettingsDictationMode, TranscriptionProviderKind,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppStatusDto {
    pub app_status: String,
    pub engine_state: String,
    pub provider: String,
    pub hotkey: String,
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
    pub dictation_mode: String,
    pub min_recording_ms: u64,
    pub hotkey: String,
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
            dictation_mode: dictation_mode_name(settings.dictation.mode).to_string(),
            min_recording_ms: settings.dictation.min_recording_ms,
            hotkey: settings.hotkeys.dictation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppService {
    settings_store: JsonSettingsStore,
}

impl Default for AppService {
    fn default() -> Self {
        Self {
            settings_store: JsonSettingsStore::default_for_app("VerboScribe 2"),
        }
    }
}

impl AppService {
    #[cfg(test)]
    pub fn new(settings_store: JsonSettingsStore) -> Self {
        Self { settings_store }
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

        AppStatusDto {
            app_status: "Desktop shell ready".to_string(),
            engine_state: state_label(DictationState::Idle).to_string(),
            provider: settings.transcription.provider.label().to_string(),
            hotkey: settings.hotkeys.dictation,
            recovery,
            last_transcript: String::new(),
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
        Ok(SettingsDto::from(settings))
    }

    pub fn runtime_status(&self) -> RuntimeEventDto {
        status_event("idle", "Ready for dictation", None)
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

    fn fake_engine(
        &self,
        config: DictationConfig,
    ) -> DictationEngine<FakeTargets, FakeRecorder, FakeTranscriber, FakeProcessor, FakeInserter>
    {
        DictationEngine::new(
            config,
            FakeTargets,
            FakeRecorder,
            FakeTranscriber,
            FakeProcessor,
            FakeInserter,
        )
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
        settings.dictation.mode = dictation_mode;
        settings.dictation.min_recording_ms = dto.min_recording_ms;
        settings.hotkeys.dictation = dto.hotkey;
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
    let recovery = match error {
        DictationError::MicrophonePermissionDenied => RecoveryDto {
            title: "Microphone permission required".to_string(),
            detail: "VerboScribe cannot record until microphone access is allowed.".to_string(),
            next_step: microphone_permission_step(platform),
        },
        DictationError::Recording(message) => RecoveryDto {
            title: "Recording failed".to_string(),
            detail: message,
            next_step: "Check the selected microphone and try recording again.".to_string(),
        },
        DictationError::RecordingTooShort => RecoveryDto {
            title: "Recording was too short".to_string(),
            detail:
                "No transcript was created because the recording ended before the minimum duration."
                    .to_string(),
            next_step: "Hold the dictation hotkey longer before releasing it.".to_string(),
        },
        DictationError::Transcription(message) => RecoveryDto {
            title: "Transcription failed".to_string(),
            detail: message,
            next_step: "Check the provider binary, model path, and audio file, then retry."
                .to_string(),
        },
        DictationError::Paste(message) => RecoveryDto {
            title: "Paste failed".to_string(),
            detail: message,
            next_step: "The transcript remains available so it can be copied or pasted manually."
                .to_string(),
        },
        DictationError::NoTranscript => RecoveryDto {
            title: "No transcript available".to_string(),
            detail: "There is no previous transcript to paste.".to_string(),
            next_step: "Record a new dictation first.".to_string(),
        },
    };

    RuntimeEventDto {
        phase: "failed".to_string(),
        message: recovery.title.clone(),
        recovery: Some(recovery),
        transcript,
    }
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

struct FakeInserter;

impl TextInsertionService for FakeInserter {
    fn insert(&mut self, _text: &str, _target: Option<&TargetApp>) -> Result<(), DictationError> {
        Ok(())
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
    fn settings_load_creates_defaults() {
        let (_temp_dir, service) = temp_service();

        let settings = service.settings().unwrap();

        assert_eq!(settings.provider, "whisperCpp");
        assert_eq!(settings.language, "en");
        assert_eq!(settings.dictation_mode, "pressAndHold");
        assert_eq!(settings.hotkey, "Control+Option+Space");
    }

    #[test]
    fn save_settings_persists_round_trip_values() {
        let (_temp_dir, service) = temp_service();
        let settings = SettingsDto {
            provider: "whisperCpp".to_string(),
            whisper_cpp_binary_path: Some("/bin/whisper-cli".to_string()),
            whisper_cpp_model_path: Some("/models/base.en.bin".to_string()),
            language: "fr".to_string(),
            dictation_mode: "toggle".to_string(),
            min_recording_ms: 400,
            hotkey: "Control+Shift+D".to_string(),
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
    fn paste_failure_event_preserves_transcript_for_manual_recovery() {
        let event = recovery_event(
            DictationError::Paste("target app did not accept paste".to_string()),
            Some("preserved transcript".to_string()),
            RecoveryPlatform::Macos,
        );

        assert_eq!(event.phase, "failed");
        assert_eq!(event.transcript.as_deref(), Some("preserved transcript"));
        assert!(event
            .recovery
            .unwrap()
            .next_step
            .contains("remains available"));
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
}
