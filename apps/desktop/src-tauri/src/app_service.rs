use serde::Serialize;
use verboscribe_core::{
    AudioCapture, AudioRecorder, DictationConfig, DictationEngine, DictationError, DictationMode,
    DictationState, ProcessedTranscript, TargetApp, TargetAppTracker, TextInsertionService,
    TranscriptProcessor, TranscriptionProvider,
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

#[derive(Debug, Default)]
pub struct AppService;

impl AppService {
    pub fn app_status(&self) -> AppStatusDto {
        AppStatusDto {
            app_status: "Desktop shell ready".to_string(),
            engine_state: state_label(DictationState::Idle).to_string(),
            provider: "whisper.cpp".to_string(),
            hotkey: "Control+Option+Space".to_string(),
            recovery: "No recovery needed".to_string(),
            last_transcript: String::new(),
        }
    }

    pub fn dry_run_dictation_state(&self) -> Result<DictationStatusDto, String> {
        let mut engine = DictationEngine::new(
            DictationConfig {
                mode: DictationMode::Toggle,
                min_recording_ms: 1,
            },
            FakeTargets,
            FakeRecorder,
            FakeTranscriber,
            FakeProcessor,
            FakeInserter,
        );

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
}

pub fn state_label(state: DictationState) -> &'static str {
    match state {
        DictationState::Idle => "Idle",
        DictationState::Starting => "Starting",
        DictationState::Recording => "Recording",
        DictationState::Transcribing => "Transcribing",
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
        let status = AppService.app_status();

        assert_eq!(status.engine_state, "Idle");
        assert_eq!(status.provider, "whisper.cpp");
        assert_eq!(status.recovery, "No recovery needed");
    }

    #[test]
    fn dry_run_delegates_through_core_engine() {
        let status = AppService.dry_run_dictation_state().unwrap();

        assert_eq!(status.state, "Idle");
        assert_eq!(
            status.last_transcript,
            Some("dry run transcript".to_string())
        );
    }
}
