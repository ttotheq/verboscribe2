use std::path::PathBuf;

pub mod transcript;
pub use transcript::{
    process_transcript, CleanupLevel, DefaultTranscriptProcessor, PersonalDictionary,
    SnippetExpander, SnippetRule, StylePreset, TranscriptProcessingOptions,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationMode {
    PressAndHold,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationState {
    Idle,
    Starting,
    Recording,
    Transcribing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Pressed,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetApp {
    pub name: Option<String>,
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioCapture {
    pub path: PathBuf,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedTranscript {
    pub raw: String,
    pub inserted: String,
}

impl ProcessedTranscript {
    pub fn did_change(&self) -> bool {
        self.raw != self.inserted
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationConfig {
    pub mode: DictationMode,
    pub min_recording_ms: u64,
}

impl Default for DictationConfig {
    fn default() -> Self {
        Self {
            mode: DictationMode::PressAndHold,
            min_recording_ms: 1_000,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum DictationError {
    #[error("microphone permission is required")]
    MicrophonePermissionDenied,
    #[error("recording failed: {0}")]
    Recording(String),
    #[error("recording was too short")]
    RecordingTooShort,
    #[error("transcription failed: {0}")]
    Transcription(String),
    #[error("paste failed: {0}")]
    Paste(String),
    #[error("no transcript is available")]
    NoTranscript,
}

pub trait TargetAppTracker {
    fn capture_target(&mut self) -> Option<TargetApp>;
}

pub trait AudioRecorder {
    fn request_permission(&mut self) -> Result<(), DictationError>;
    fn start(&mut self) -> Result<(), DictationError>;
    fn stop(&mut self) -> Result<AudioCapture, DictationError>;
    fn cancel(&mut self);
}

pub trait TranscriptionProvider {
    fn transcribe(&mut self, audio: &AudioCapture) -> Result<String, DictationError>;
}

pub trait TranscriptProcessor {
    fn process(&self, raw: &str, target: Option<&TargetApp>) -> ProcessedTranscript;
}

pub trait TextInsertionService {
    fn insert(&mut self, text: &str, target: Option<&TargetApp>) -> Result<(), DictationError>;
}

pub struct DictationEngine<Targets, Recorder, Transcriber, Processor, Inserter> {
    config: DictationConfig,
    state: DictationState,
    should_stop_after_start: bool,
    target: Option<TargetApp>,
    last_transcript: Option<String>,
    targets: Targets,
    recorder: Recorder,
    transcriber: Transcriber,
    processor: Processor,
    inserter: Inserter,
}

impl<Targets, Recorder, Transcriber, Processor, Inserter>
    DictationEngine<Targets, Recorder, Transcriber, Processor, Inserter>
where
    Targets: TargetAppTracker,
    Recorder: AudioRecorder,
    Transcriber: TranscriptionProvider,
    Processor: TranscriptProcessor,
    Inserter: TextInsertionService,
{
    pub fn new(
        config: DictationConfig,
        targets: Targets,
        recorder: Recorder,
        transcriber: Transcriber,
        processor: Processor,
        inserter: Inserter,
    ) -> Self {
        Self {
            config,
            state: DictationState::Idle,
            should_stop_after_start: false,
            target: None,
            last_transcript: None,
            targets,
            recorder,
            transcriber,
            processor,
            inserter,
        }
    }

    pub fn state(&self) -> DictationState {
        self.state
    }

    pub fn last_transcript(&self) -> Option<&str> {
        self.last_transcript.as_deref()
    }

    pub fn hotkey(&mut self, event: HotkeyEvent) -> Result<(), DictationError> {
        match (self.config.mode, event) {
            (DictationMode::PressAndHold, HotkeyEvent::Pressed)
                if self.state == DictationState::Idle =>
            {
                self.start_recording()
            }
            (DictationMode::PressAndHold, HotkeyEvent::Released) => match self.state {
                DictationState::Starting => {
                    self.should_stop_after_start = true;
                    Ok(())
                }
                DictationState::Recording => self.stop_transcribe_insert(),
                _ => Ok(()),
            },
            (DictationMode::Toggle, HotkeyEvent::Pressed) => match self.state {
                DictationState::Idle => self.start_recording(),
                DictationState::Starting => {
                    self.should_stop_after_start = true;
                    Ok(())
                }
                DictationState::Recording => self.stop_transcribe_insert(),
                DictationState::Transcribing => Ok(()),
            },
            (DictationMode::Toggle, HotkeyEvent::Released) => Ok(()),
            _ => Ok(()),
        }
    }

    pub fn start_recording(&mut self) -> Result<(), DictationError> {
        if self.state != DictationState::Idle {
            return Ok(());
        }

        self.state = DictationState::Starting;
        self.should_stop_after_start = false;
        self.target = self.targets.capture_target();
        if let Err(error) = self.recorder.request_permission() {
            self.state = DictationState::Idle;
            return Err(error);
        }
        if let Err(error) = self.recorder.start() {
            self.state = DictationState::Idle;
            return Err(error);
        }
        self.state = DictationState::Recording;

        if self.should_stop_after_start {
            self.stop_transcribe_insert()?;
        }

        Ok(())
    }

    pub fn stop_transcribe_insert(&mut self) -> Result<(), DictationError> {
        if self.state != DictationState::Recording {
            return Ok(());
        }

        let audio = match self.recorder.stop() {
            Ok(audio) => audio,
            Err(error) => {
                self.state = DictationState::Idle;
                return Err(error);
            }
        };
        if audio.duration_ms < self.config.min_recording_ms {
            self.state = DictationState::Idle;
            return Err(DictationError::RecordingTooShort);
        }

        self.state = DictationState::Transcribing;
        let raw = match self.transcriber.transcribe(&audio) {
            Ok(raw) => raw,
            Err(error) => {
                self.state = DictationState::Idle;
                return Err(error);
            }
        };
        let processed = self.processor.process(raw.trim(), self.target.as_ref());
        self.last_transcript = Some(processed.inserted.clone());
        let insert_result = self
            .inserter
            .insert(&processed.inserted, self.target.as_ref());
        self.state = DictationState::Idle;
        insert_result
    }

    pub fn cancel(&mut self) {
        if matches!(
            self.state,
            DictationState::Starting | DictationState::Recording
        ) {
            self.recorder.cancel();
        }
        self.should_stop_after_start = false;
        self.state = DictationState::Idle;
    }

    pub fn paste_last(&mut self) -> Result<(), DictationError> {
        let transcript = self
            .last_transcript
            .as_deref()
            .ok_or(DictationError::NoTranscript)?;
        self.inserter.insert(transcript, self.target.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeTargets;

    impl TargetAppTracker for FakeTargets {
        fn capture_target(&mut self) -> Option<TargetApp> {
            Some(TargetApp {
                name: Some("Target".to_string()),
                identifier: Some("local.target".to_string()),
            })
        }
    }

    struct FakeRecorder {
        permission: Result<(), DictationError>,
        start_result: Result<(), DictationError>,
        stop_result: Result<AudioCapture, DictationError>,
        started: bool,
        cancelled: bool,
    }

    impl Default for FakeRecorder {
        fn default() -> Self {
            Self {
                permission: Ok(()),
                start_result: Ok(()),
                stop_result: Ok(AudioCapture {
                    path: PathBuf::from("/tmp/audio.wav"),
                    duration_ms: 1_500,
                }),
                started: false,
                cancelled: false,
            }
        }
    }

    impl AudioRecorder for FakeRecorder {
        fn request_permission(&mut self) -> Result<(), DictationError> {
            self.permission.clone()
        }

        fn start(&mut self) -> Result<(), DictationError> {
            self.started = true;
            self.start_result.clone()
        }

        fn stop(&mut self) -> Result<AudioCapture, DictationError> {
            self.stop_result.clone()
        }

        fn cancel(&mut self) {
            self.cancelled = true;
        }
    }

    struct FakeTranscriber {
        result: Result<String, DictationError>,
    }

    impl Default for FakeTranscriber {
        fn default() -> Self {
            Self {
                result: Ok("hello world".to_string()),
            }
        }
    }

    impl TranscriptionProvider for FakeTranscriber {
        fn transcribe(&mut self, _audio: &AudioCapture) -> Result<String, DictationError> {
            self.result.clone()
        }
    }

    #[derive(Default)]
    struct IdentityProcessor;

    impl TranscriptProcessor for IdentityProcessor {
        fn process(&self, raw: &str, _target: Option<&TargetApp>) -> ProcessedTranscript {
            ProcessedTranscript {
                raw: raw.to_string(),
                inserted: raw.to_string(),
            }
        }
    }

    struct FakeInserter {
        inserted: Vec<String>,
        result: Result<(), DictationError>,
    }

    impl Default for FakeInserter {
        fn default() -> Self {
            Self {
                inserted: Vec::new(),
                result: Ok(()),
            }
        }
    }

    impl TextInsertionService for FakeInserter {
        fn insert(
            &mut self,
            text: &str,
            _target: Option<&TargetApp>,
        ) -> Result<(), DictationError> {
            self.inserted.push(text.to_string());
            self.result.clone()
        }
    }

    fn engine(
        config: DictationConfig,
    ) -> DictationEngine<FakeTargets, FakeRecorder, FakeTranscriber, IdentityProcessor, FakeInserter>
    {
        DictationEngine::new(
            config,
            FakeTargets,
            FakeRecorder::default(),
            FakeTranscriber::default(),
            IdentityProcessor,
            FakeInserter::default(),
        )
    }

    #[test]
    fn press_and_hold_records_transcribes_and_inserts() {
        let mut engine = engine(DictationConfig::default());

        engine.hotkey(HotkeyEvent::Pressed).unwrap();
        assert_eq!(engine.state(), DictationState::Recording);

        engine.hotkey(HotkeyEvent::Released).unwrap();
        assert_eq!(engine.state(), DictationState::Idle);
        assert_eq!(engine.last_transcript(), Some("hello world"));
    }

    #[test]
    fn toggle_records_transcribes_and_inserts() {
        let mut engine = engine(DictationConfig {
            mode: DictationMode::Toggle,
            min_recording_ms: 1_000,
        });

        engine.hotkey(HotkeyEvent::Pressed).unwrap();
        assert_eq!(engine.state(), DictationState::Recording);

        engine.hotkey(HotkeyEvent::Pressed).unwrap();
        assert_eq!(engine.state(), DictationState::Idle);
        assert_eq!(engine.last_transcript(), Some("hello world"));
    }

    #[test]
    fn short_recording_returns_error_and_resets_to_idle() {
        let mut engine = DictationEngine::new(
            DictationConfig::default(),
            FakeTargets,
            FakeRecorder {
                stop_result: Ok(AudioCapture {
                    path: PathBuf::from("/tmp/audio.wav"),
                    duration_ms: 250,
                }),
                ..FakeRecorder::default()
            },
            FakeTranscriber::default(),
            IdentityProcessor,
            FakeInserter::default(),
        );

        engine.hotkey(HotkeyEvent::Pressed).unwrap();
        let error = engine.hotkey(HotkeyEvent::Released).unwrap_err();

        assert_eq!(error, DictationError::RecordingTooShort);
        assert_eq!(engine.state(), DictationState::Idle);
        assert_eq!(engine.last_transcript(), None);
    }

    #[test]
    fn cancel_resets_active_recording() {
        let mut engine = engine(DictationConfig::default());

        engine.hotkey(HotkeyEvent::Pressed).unwrap();
        engine.cancel();

        assert_eq!(engine.state(), DictationState::Idle);
        assert_eq!(engine.last_transcript(), None);
    }

    #[test]
    fn paste_last_requires_a_transcript() {
        let mut engine = engine(DictationConfig::default());

        let error = engine.paste_last().unwrap_err();

        assert_eq!(error, DictationError::NoTranscript);
    }

    #[test]
    fn permission_denial_resets_to_idle() {
        let mut engine = DictationEngine::new(
            DictationConfig::default(),
            FakeTargets,
            FakeRecorder {
                permission: Err(DictationError::MicrophonePermissionDenied),
                ..FakeRecorder::default()
            },
            FakeTranscriber::default(),
            IdentityProcessor,
            FakeInserter::default(),
        );

        let error = engine.hotkey(HotkeyEvent::Pressed).unwrap_err();

        assert_eq!(error, DictationError::MicrophonePermissionDenied);
        assert_eq!(engine.state(), DictationState::Idle);
        assert_eq!(engine.last_transcript(), None);
    }

    #[test]
    fn recorder_start_failure_resets_to_idle() {
        let mut engine = DictationEngine::new(
            DictationConfig::default(),
            FakeTargets,
            FakeRecorder {
                start_result: Err(DictationError::Recording("device unavailable".to_string())),
                ..FakeRecorder::default()
            },
            FakeTranscriber::default(),
            IdentityProcessor,
            FakeInserter::default(),
        );

        let error = engine.hotkey(HotkeyEvent::Pressed).unwrap_err();

        assert_eq!(
            error,
            DictationError::Recording("device unavailable".to_string())
        );
        assert_eq!(engine.state(), DictationState::Idle);
        assert_eq!(engine.last_transcript(), None);
    }

    #[test]
    fn recorder_stop_failure_resets_to_idle() {
        let mut engine = DictationEngine::new(
            DictationConfig::default(),
            FakeTargets,
            FakeRecorder {
                stop_result: Err(DictationError::Recording("no audio captured".to_string())),
                ..FakeRecorder::default()
            },
            FakeTranscriber::default(),
            IdentityProcessor,
            FakeInserter::default(),
        );

        engine.hotkey(HotkeyEvent::Pressed).unwrap();
        let error = engine.hotkey(HotkeyEvent::Released).unwrap_err();

        assert_eq!(
            error,
            DictationError::Recording("no audio captured".to_string())
        );
        assert_eq!(engine.state(), DictationState::Idle);
        assert_eq!(engine.last_transcript(), None);
    }

    #[test]
    fn transcription_failure_resets_to_idle_without_transcript() {
        let mut engine = DictationEngine::new(
            DictationConfig::default(),
            FakeTargets,
            FakeRecorder::default(),
            FakeTranscriber {
                result: Err(DictationError::Transcription("model missing".to_string())),
            },
            IdentityProcessor,
            FakeInserter::default(),
        );

        engine.hotkey(HotkeyEvent::Pressed).unwrap();
        let error = engine.hotkey(HotkeyEvent::Released).unwrap_err();

        assert_eq!(
            error,
            DictationError::Transcription("model missing".to_string())
        );
        assert_eq!(engine.state(), DictationState::Idle);
        assert_eq!(engine.last_transcript(), None);
    }

    #[test]
    fn paste_failure_resets_to_idle_and_preserves_last_transcript() {
        let mut engine = DictationEngine::new(
            DictationConfig::default(),
            FakeTargets,
            FakeRecorder::default(),
            FakeTranscriber::default(),
            IdentityProcessor,
            FakeInserter {
                result: Err(DictationError::Paste("accessibility denied".to_string())),
                ..FakeInserter::default()
            },
        );

        engine.hotkey(HotkeyEvent::Pressed).unwrap();
        let error = engine.hotkey(HotkeyEvent::Released).unwrap_err();

        assert_eq!(
            error,
            DictationError::Paste("accessibility denied".to_string())
        );
        assert_eq!(engine.state(), DictationState::Idle);
        assert_eq!(engine.last_transcript(), Some("hello world"));
    }

    #[test]
    fn duplicate_press_while_recording_is_ignored_in_press_and_hold_mode() {
        let mut engine = engine(DictationConfig::default());

        engine.hotkey(HotkeyEvent::Pressed).unwrap();
        engine.hotkey(HotkeyEvent::Pressed).unwrap();

        assert_eq!(engine.state(), DictationState::Recording);
        assert_eq!(engine.last_transcript(), None);
    }
}
