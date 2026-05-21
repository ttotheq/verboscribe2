//! Settings, history, usage insights, and backup storage.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use verboscribe_core::{DictationConfig, DictationMode};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_PROVIDER: TranscriptionProviderKind = TranscriptionProviderKind::WhisperCpp;
pub const DEFAULT_LANGUAGE: &str = "en";
pub const DEFAULT_DICTATION_HOTKEY: &str = "Control+Option+Space";
pub const DEFAULT_MIN_RECORDING_MS: u64 = 1_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub schema_version: u32,
    pub transcription: TranscriptionSettings,
    pub dictation: DictationSettings,
    pub hotkeys: HotkeySettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            transcription: TranscriptionSettings::default(),
            dictation: DictationSettings::default(),
            hotkeys: HotkeySettings::default(),
        }
    }
}

impl AppSettings {
    pub fn dictation_config(&self) -> DictationConfig {
        DictationConfig {
            mode: self.dictation.mode.into(),
            min_recording_ms: self.dictation.min_recording_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionSettings {
    pub provider: TranscriptionProviderKind,
    pub whisper_cpp: WhisperCppSettings,
}

impl Default for TranscriptionSettings {
    fn default() -> Self {
        Self {
            provider: DEFAULT_PROVIDER,
            whisper_cpp: WhisperCppSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TranscriptionProviderKind {
    WhisperCpp,
}

impl TranscriptionProviderKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::WhisperCpp => "whisper.cpp",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WhisperCppSettings {
    pub binary_path: Option<PathBuf>,
    pub model_path: Option<PathBuf>,
    pub language: String,
    #[serde(default)]
    pub prompt_context: String,
    #[serde(default)]
    pub pinned_terms: String,
}

impl Default for WhisperCppSettings {
    fn default() -> Self {
        Self {
            binary_path: None,
            model_path: None,
            language: DEFAULT_LANGUAGE.to_string(),
            prompt_context: String::new(),
            pinned_terms: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DictationSettings {
    pub mode: SettingsDictationMode,
    pub min_recording_ms: u64,
}

impl Default for DictationSettings {
    fn default() -> Self {
        Self {
            mode: SettingsDictationMode::PressAndHold,
            min_recording_ms: DEFAULT_MIN_RECORDING_MS,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SettingsDictationMode {
    PressAndHold,
    Toggle,
}

impl From<SettingsDictationMode> for DictationMode {
    fn from(mode: SettingsDictationMode) -> Self {
        match mode {
            SettingsDictationMode::PressAndHold => Self::PressAndHold,
            SettingsDictationMode::Toggle => Self::Toggle,
        }
    }
}

impl From<DictationMode> for SettingsDictationMode {
    fn from(mode: DictationMode) -> Self {
        match mode {
            DictationMode::PressAndHold => Self::PressAndHold,
            DictationMode::Toggle => Self::Toggle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HotkeySettings {
    pub dictation: String,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            dictation: DEFAULT_DICTATION_HOTKEY.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsStoreError {
    #[error("settings I/O failed for {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("settings JSON is invalid in {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonSettingsStore {
    path: PathBuf,
}

impl JsonSettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn default_for_app(app_name: &str) -> Self {
        Self::new(default_settings_path(app_name))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<AppSettings, SettingsStoreError> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => {
                serde_json::from_str(&contents).map_err(|source| SettingsStoreError::Json {
                    path: self.path.clone(),
                    source,
                })
            }
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(AppSettings::default()),
            Err(source) => Err(SettingsStoreError::Io {
                path: self.path.clone(),
                source,
            }),
        }
    }

    pub fn load_or_create(&self) -> Result<AppSettings, SettingsStoreError> {
        if self.path.exists() {
            self.load()
        } else {
            let settings = AppSettings::default();
            self.save(&settings)?;
            Ok(settings)
        }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), SettingsStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| SettingsStoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let mut contents =
            serde_json::to_string_pretty(settings).map_err(|source| SettingsStoreError::Json {
                path: self.path.clone(),
                source,
            })?;
        contents.push('\n');

        fs::write(&self.path, contents).map_err(|source| SettingsStoreError::Io {
            path: self.path.clone(),
            source,
        })
    }
}

pub fn default_settings_path(app_name: &str) -> PathBuf {
    default_config_dir().join(app_name).join("settings.json")
}

fn default_config_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Some(app_data) = std::env::var_os("APPDATA") {
            return PathBuf::from(app_data);
        }
    }

    if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support");
        }
    }

    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config");
    }

    std::env::temp_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_document_provider_language_mode_and_hotkey() {
        let settings = AppSettings::default();

        assert_eq!(
            settings.transcription.provider,
            TranscriptionProviderKind::WhisperCpp
        );
        assert_eq!(settings.transcription.provider.label(), "whisper.cpp");
        assert_eq!(settings.transcription.whisper_cpp.language, "en");
        assert_eq!(settings.transcription.whisper_cpp.prompt_context, "");
        assert_eq!(settings.transcription.whisper_cpp.pinned_terms, "");
        assert_eq!(settings.dictation.mode, SettingsDictationMode::PressAndHold);
        assert_eq!(settings.dictation.min_recording_ms, 1_000);
        assert_eq!(settings.hotkeys.dictation, "Control+Option+Space");
    }

    #[test]
    fn settings_convert_to_core_dictation_config() {
        let mut settings = AppSettings::default();
        settings.dictation.mode = SettingsDictationMode::Toggle;
        settings.dictation.min_recording_ms = 250;

        assert_eq!(
            settings.dictation_config(),
            DictationConfig {
                mode: DictationMode::Toggle,
                min_recording_ms: 250,
            }
        );
    }

    #[test]
    fn missing_settings_load_as_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = JsonSettingsStore::new(temp_dir.path().join("settings.json"));

        assert_eq!(store.load().unwrap(), AppSettings::default());
        assert!(!store.path().exists());
    }

    #[test]
    fn load_or_create_persists_defaults_when_file_is_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = JsonSettingsStore::new(temp_dir.path().join("nested/settings.json"));

        assert_eq!(store.load_or_create().unwrap(), AppSettings::default());
        assert!(store.path().exists());
    }

    #[test]
    fn save_and_load_round_trip_settings() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = JsonSettingsStore::new(temp_dir.path().join("settings.json"));
        let mut settings = AppSettings::default();
        settings.transcription.whisper_cpp.binary_path =
            Some(PathBuf::from("/usr/bin/whisper-cli"));
        settings.transcription.whisper_cpp.model_path = Some(PathBuf::from("/models/base.en.bin"));
        settings.transcription.whisper_cpp.language = "es".to_string();
        settings.transcription.whisper_cpp.prompt_context =
            "Prefer medical vocabulary when heard clearly.".to_string();
        settings.transcription.whisper_cpp.pinned_terms = "OpenAI, GPT-5".to_string();
        settings.dictation.mode = SettingsDictationMode::Toggle;
        settings.hotkeys.dictation = "Control+Shift+D".to_string();

        store.save(&settings).unwrap();

        assert_eq!(store.load().unwrap(), settings);
    }

    #[test]
    fn load_legacy_settings_without_prompt_fields_uses_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("settings.json");
        fs::write(
            &path,
            r#"{
  "schemaVersion": 1,
  "transcription": {
    "provider": "whisperCpp",
    "whisperCpp": {
      "binaryPath": "/usr/bin/whisper-cli",
      "modelPath": "/models/base.en.bin",
      "language": "en"
    }
  },
  "dictation": {
    "mode": "pressAndHold",
    "minRecordingMs": 1000
  },
  "hotkeys": {
    "dictation": "Control+Option+Space"
  }
}"#,
        )
        .unwrap();

        let settings = JsonSettingsStore::new(&path).load().unwrap();

        assert_eq!(settings.transcription.whisper_cpp.prompt_context, "");
        assert_eq!(settings.transcription.whisper_cpp.pinned_terms, "");
    }

    #[test]
    fn invalid_json_returns_a_settings_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("settings.json");
        fs::write(&path, "{").unwrap();

        let error = JsonSettingsStore::new(&path).load().unwrap_err();

        assert!(matches!(error, SettingsStoreError::Json { .. }));
    }
}
