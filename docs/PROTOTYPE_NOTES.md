# Prototype Notes

The macOS-only prototype in `~/projects/whisper` was inspected read-only on
2026-05-02.

## Product Behavior

- Records microphone audio.
- Transcribes with local `whisper.cpp` or Groq Whisper.
- Supports press-and-hold and toggle recording.
- Uses configurable global shortcuts for dictation, cancel, and paste-last.
- Inserts text by copying to the clipboard and simulating paste.
- Remembers the previous non-VerboScribe foreground app to avoid pasting into
  itself after UI clicks.
- Supports preview/edit-before-insert.
- Defaults new settings to raw-first output with cleanup disabled.
- Supports deterministic cleanup, spoken commands, style presets, snippets, and
  personal dictionary prompt hints.
- Supports optional transcript history, aggregate usage insights, settings
  import/export, launch-at-login, release notes, and manual QA.

## Current Prototype Layout

- `Sources/VerboScribeCore/DictationController.swift`: state machine.
- `AudioRecorder.swift`: AVFoundation WAV recording.
- `HotkeyMonitor.swift`: Carbon global hotkeys.
- `TextInserter.swift`: AppKit/ApplicationServices paste insertion.
- `Transcriber.swift`, `WhisperCPPTranscriber.swift`, `GroqTranscriber.swift`:
  provider abstraction and implementations.
- `TranscriptProcessor.swift`, `SnippetExpander.swift`,
  `PersonalDictionary.swift`: portable product logic.
- `TranscriptHistoryStore.swift`, `UsageInsightsStore.swift`,
  `SettingsBackup.swift`: local persistence behavior.
- `LaunchAtLoginManager.swift`: macOS ServiceManagement wrapper.
- `scripts/verify.sh`, `scripts/package-app.sh`,
  `scripts/install-whisper-cpp.sh`: build/package/smoke-test scripts.

## Porting Guidance

The prototype is a behavior reference, not a codebase to copy. macOS-specific
AppKit, Carbon, AVFoundation, ApplicationServices, and ServiceManagement code
should be rewritten behind cross-platform interfaces.
