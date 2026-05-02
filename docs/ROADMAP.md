# Roadmap

## Milestone 0: Foundation

Status: started.

- Create Tauri/Rust workspace structure.
- Document architecture and MVP scope.
- Add platform-neutral domain types and interfaces.
- Add a tested `DictationEngine` skeleton with mocked dependencies.
- Add a verification script.

Acceptance:

- `cargo test --workspace` passes once Rust is installed.
- The core crate has no Tauri, macOS, or Windows API dependencies.
- State transitions are covered for start, stop, cancel, success, and failure.

## Milestone 1: Vertical Slice

Goal: global hotkey -> record audio -> transcribe -> paste into active app.

Scope:

- Tauri desktop app launches on macOS and Windows.
- Tray/menu-bar item is present.
- Dictation hotkey can be registered and unregistered.
- Recording creates a valid 16 kHz mono WAV.
- Local `whisper.cpp` provider transcribes that WAV.
- Text insertion pastes into the app active before recording.
- Clipboard fallback is explicit when automated paste fails.
- Minimal settings persist: hotkey, mode, provider, whisper binary path, model
  path, language.

Acceptance:

- macOS manual QA passes in TextEdit, Terminal, and a browser text field.
- Windows manual QA passes in Notepad, Windows Terminal, and a browser text
  field.
- Local JFK-style `whisper.cpp` smoke test passes on both platforms.
- Paste failures do not lose the transcript.

## Milestone 2: Provider And Settings Hardening

- Add Groq Whisper provider behind the same trait.
- Store Groq API key in platform secret storage.
- Add provider status and actionable errors.
- Add settings import/export excluding secrets.
- Add validation and migrations.

## Milestone 3: Dictation Polish

- Press-and-hold mode if not already enabled in Milestone 1.
- Cancel hotkey.
- Paste-last action and hotkey.
- Retry last failed transcription.
- Min/max recording durations.
- Low-input warning.
- Optional start/stop sound.

## Milestone 4: Text Processing

- Port raw-first transcript processor behavior.
- Add cleanup levels.
- Add spoken commands.
- Add snippets.
- Add personal dictionary prompt hints.
- Add target-app style presets.

## Milestone 5: Privacy And Memory

- Add transcript history with storage modes:
  - save normally
  - auto-delete after 24 hours
  - never store after insertion
- Add clear history flow.
- Add aggregate usage insights.
- Add reset insights flow.

## Milestone 6: Packaging And Operations

- Add macOS `.app`/DMG build.
- Add Windows installer build.
- Add signing documentation.
- Add launch-at-login.
- Add release notes.
- Add manual QA matrix for both platforms.

## Backlog

- Model downloader/manager.
- History browser/search.
- Rich snippet manager.
- Rich personal dictionary manager.
- Diagnostics bundle that excludes transcript text by default.
- Updater.
