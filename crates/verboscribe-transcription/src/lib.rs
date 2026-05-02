//! Transcription provider implementations.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use verboscribe_core::{AudioCapture, DictationError, TranscriptionProvider};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhisperCppConfig {
    pub binary_path: PathBuf,
    pub model_path: PathBuf,
    pub language_code: String,
    pub prompt_context: String,
    pub timeout: Duration,
}

impl WhisperCppConfig {
    pub fn new(binary_path: impl Into<PathBuf>, model_path: impl Into<PathBuf>) -> Self {
        Self {
            binary_path: binary_path.into(),
            model_path: model_path.into(),
            language_code: "en".to_string(),
            prompt_context: String::new(),
            timeout: Duration::from_secs(120),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhisperCppCommandPlan {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub output_text_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub success: bool,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner: Clone {
    fn run(&self, plan: &WhisperCppCommandPlan) -> Result<CommandOutput, String>;
}

#[derive(Debug, Clone, Default)]
pub struct StdCommandRunner;

impl CommandRunner for StdCommandRunner {
    fn run(&self, plan: &WhisperCppCommandPlan) -> Result<CommandOutput, String> {
        let output = Command::new(&plan.program)
            .args(&plan.args)
            .output()
            .map_err(|error| {
                format!(
                    "could not run whisper.cpp at {}: {error}",
                    plan.program.display()
                )
            })?;

        Ok(CommandOutput {
            success: output.status.success(),
            status: output.status.to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct WhisperCppTranscriber<R = StdCommandRunner> {
    config: WhisperCppConfig,
    runner: R,
}

impl WhisperCppTranscriber<StdCommandRunner> {
    pub fn new(config: WhisperCppConfig) -> Self {
        Self {
            config,
            runner: StdCommandRunner,
        }
    }
}

impl<R: CommandRunner> WhisperCppTranscriber<R> {
    pub fn with_runner(config: WhisperCppConfig, runner: R) -> Self {
        Self { config, runner }
    }

    pub fn config(&self) -> &WhisperCppConfig {
        &self.config
    }

    pub fn validate(&self) -> Result<(), DictationError> {
        validate_binary(&self.config.binary_path)?;
        validate_file(&self.config.model_path, "Whisper model")?;
        Ok(())
    }

    pub fn command_plan(&self, audio_path: &Path, output_base: &Path) -> WhisperCppCommandPlan {
        let mut args = vec![
            "-m".to_string(),
            self.config.model_path.display().to_string(),
            "-f".to_string(),
            audio_path.display().to_string(),
            "-otxt".to_string(),
            "-of".to_string(),
            output_base.display().to_string(),
            "-l".to_string(),
            self.config.language_code.clone(),
            "-nt".to_string(),
        ];

        let prompt = self.config.prompt_context.trim();
        if !prompt.is_empty() {
            args.push("--prompt".to_string());
            args.push(prompt.to_string());
        }

        WhisperCppCommandPlan {
            program: self.config.binary_path.clone(),
            args,
            output_text_path: output_base.with_extension("txt"),
        }
    }

    pub fn transcribe_to_output_base(
        &mut self,
        audio: &AudioCapture,
        output_base: &Path,
    ) -> Result<String, DictationError> {
        self.validate()?;
        validate_file(&audio.path, "Audio file")?;

        let plan = self.command_plan(&audio.path, output_base);
        let output = self
            .runner
            .run(&plan)
            .map_err(DictationError::Transcription)?;

        if !output.success {
            let message = if !output.stderr.is_empty() {
                output.stderr
            } else if !output.stdout.is_empty() {
                output.stdout
            } else {
                format!("whisper.cpp exited with status {}", output.status)
            };
            return Err(DictationError::Transcription(message));
        }

        fs::read_to_string(&plan.output_text_path)
            .map(|text| text.trim().to_string())
            .map_err(|error| {
                DictationError::Transcription(format!(
                    "could not read whisper.cpp transcript {}: {error}",
                    plan.output_text_path.display()
                ))
            })
    }
}

impl<R: CommandRunner> TranscriptionProvider for WhisperCppTranscriber<R> {
    fn transcribe(&mut self, audio: &AudioCapture) -> Result<String, DictationError> {
        let output_base = std::env::temp_dir()
            .join("verboscribe2")
            .join(format!("transcript-{}", unique_suffix()));
        if let Some(parent) = output_base.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                DictationError::Transcription(format!(
                    "could not create transcript temp directory: {error}"
                ))
            })?;
        }

        self.transcribe_to_output_base(audio, &output_base)
    }
}

fn validate_file(path: &Path, label: &str) -> Result<(), DictationError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(DictationError::Transcription(format!(
            "{label} not found at {}",
            path.display()
        )))
    }
}

fn validate_binary(path: &Path) -> Result<(), DictationError> {
    validate_file(path, "whisper.cpp binary")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(path)
            .map_err(|error| DictationError::Transcription(error.to_string()))?
            .permissions()
            .mode();
        if mode & 0o111 == 0 {
            return Err(DictationError::Transcription(format!(
                "whisper.cpp binary is not executable at {}",
                path.display()
            )));
        }
    }

    Ok(())
}

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{}-{nanos}", std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn command_plan_includes_required_whisper_arguments() {
        let mut config = WhisperCppConfig::new("/bin/whisper-cli", "/models/base.bin");
        config.language_code = "auto".to_string();
        config.prompt_context = "Prefer VerboScribe.".to_string();
        let provider = WhisperCppTranscriber::new(config);

        let plan = provider.command_plan(Path::new("/tmp/audio.wav"), Path::new("/tmp/out"));

        assert_eq!(plan.program, PathBuf::from("/bin/whisper-cli"));
        assert_eq!(plan.output_text_path, PathBuf::from("/tmp/out.txt"));
        assert_eq!(
            plan.args,
            vec![
                "-m",
                "/models/base.bin",
                "-f",
                "/tmp/audio.wav",
                "-otxt",
                "-of",
                "/tmp/out",
                "-l",
                "auto",
                "-nt",
                "--prompt",
                "Prefer VerboScribe."
            ]
        );
    }

    #[test]
    fn validate_reports_missing_binary() {
        let provider = WhisperCppTranscriber::new(WhisperCppConfig::new(
            "/missing/whisper-cli",
            "/missing/model.bin",
        ));

        let error = provider.validate().unwrap_err();

        assert!(error.to_string().contains("whisper.cpp binary not found"));
    }

    #[test]
    fn validate_reports_missing_model_after_binary_exists() {
        let directory = tempfile::tempdir().unwrap();
        let binary_path = directory.path().join("whisper-cli");
        write_executable(&binary_path);
        let provider = WhisperCppTranscriber::new(WhisperCppConfig::new(
            &binary_path,
            directory.path().join("missing-model.bin"),
        ));

        let error = provider.validate().unwrap_err();

        assert!(error.to_string().contains("Whisper model not found"));
    }

    #[cfg(unix)]
    #[test]
    fn validate_rejects_non_executable_binary() {
        let directory = tempfile::tempdir().unwrap();
        let binary_path = directory.path().join("whisper-cli");
        File::create(&binary_path).unwrap();
        let provider = WhisperCppTranscriber::new(WhisperCppConfig::new(
            &binary_path,
            directory.path().join("model.bin"),
        ));

        let error = provider.validate().unwrap_err();

        assert!(error.to_string().contains("not executable"));
    }

    #[test]
    fn transcribe_reads_trimmed_output_file_after_success() {
        let directory = tempfile::tempdir().unwrap();
        let binary_path = directory.path().join("whisper-cli");
        let model_path = directory.path().join("model.bin");
        let audio_path = directory.path().join("audio.wav");
        let output_base = directory.path().join("transcript");
        write_executable(&binary_path);
        File::create(&model_path).unwrap();
        File::create(&audio_path).unwrap();
        fs::write(
            output_base.with_extension("txt"),
            "  hello from whisper  \n",
        )
        .unwrap();
        let mut provider = WhisperCppTranscriber::with_runner(
            WhisperCppConfig::new(&binary_path, &model_path),
            FakeRunner {
                output: Ok(CommandOutput {
                    success: true,
                    status: "exit status: 0".to_string(),
                    stdout: String::new(),
                    stderr: String::new(),
                }),
            },
        );

        let transcript = provider
            .transcribe_to_output_base(
                &AudioCapture {
                    path: audio_path,
                    duration_ms: 1_000,
                },
                &output_base,
            )
            .unwrap();

        assert_eq!(transcript, "hello from whisper");
    }

    #[test]
    fn transcribe_maps_non_zero_exit_to_stderr() {
        let directory = tempfile::tempdir().unwrap();
        let binary_path = directory.path().join("whisper-cli");
        let model_path = directory.path().join("model.bin");
        let audio_path = directory.path().join("audio.wav");
        write_executable(&binary_path);
        File::create(&model_path).unwrap();
        File::create(&audio_path).unwrap();
        let mut provider = WhisperCppTranscriber::with_runner(
            WhisperCppConfig::new(&binary_path, &model_path),
            FakeRunner {
                output: Ok(CommandOutput {
                    success: false,
                    status: "exit status: 2".to_string(),
                    stdout: "stdout detail".to_string(),
                    stderr: "model failed".to_string(),
                }),
            },
        );

        let error = provider
            .transcribe_to_output_base(
                &AudioCapture {
                    path: audio_path,
                    duration_ms: 1_000,
                },
                &directory.path().join("missing-output"),
            )
            .unwrap_err();

        assert_eq!(error.to_string(), "transcription failed: model failed");
    }

    #[test]
    fn transcribe_reports_missing_output_after_success() {
        let directory = tempfile::tempdir().unwrap();
        let binary_path = directory.path().join("whisper-cli");
        let model_path = directory.path().join("model.bin");
        let audio_path = directory.path().join("audio.wav");
        write_executable(&binary_path);
        File::create(&model_path).unwrap();
        File::create(&audio_path).unwrap();
        let mut provider = WhisperCppTranscriber::with_runner(
            WhisperCppConfig::new(&binary_path, &model_path),
            FakeRunner {
                output: Ok(CommandOutput {
                    success: true,
                    status: "exit status: 0".to_string(),
                    stdout: String::new(),
                    stderr: String::new(),
                }),
            },
        );

        let error = provider
            .transcribe_to_output_base(
                &AudioCapture {
                    path: audio_path,
                    duration_ms: 1_000,
                },
                &directory.path().join("missing-output"),
            )
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("could not read whisper.cpp transcript"));
    }

    #[derive(Clone)]
    struct FakeRunner {
        output: Result<CommandOutput, String>,
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, _plan: &WhisperCppCommandPlan) -> Result<CommandOutput, String> {
            self.output.clone()
        }
    }

    fn write_executable(path: &Path) {
        let mut file = File::create(path).unwrap();
        file.write_all(b"#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
    }
}
