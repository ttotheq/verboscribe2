# Roadmap

For the consolidated current feature inventory and the prototype-to-current UI
gap review, read `docs/FEATURE_LIST.md`.

## Milestone 0: Foundation

Status: done.

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

Status: implementation complete, manual QA pending.

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
- Validation work is tracked explicitly in `VS2-016`, `VS2-017`, and
  `VS2-018`.

Acceptance:

- macOS manual QA passes in TextEdit, Terminal, and a browser text field.
- Windows manual QA passes in Notepad, Windows Terminal, and a browser text
  field.
- Local JFK-style `whisper.cpp` smoke test passes on both platforms.
- Paste failures do not lose the transcript.

## Milestone 2: Provider And Settings Hardening

Status: in progress after `VS2-024`.

- Keep extending the desktop settings and operations surface.
- Keep expanding transcript recovery around the shipped paste-last and retry-
  failed actions, including preview-before-insert.
- Add Groq Whisper provider behind the same trait.
- Store Groq API key in platform secret storage.
- Add provider status and actionable errors.
- Add settings import/export excluding secrets.
- Add validation and migrations.

## Milestone 3: Dictation Polish

Status: future.

- Press-and-hold mode if not already enabled in Milestone 1.
- Cancel hotkey.
- Retry last failed transcription.
- Min/max recording durations.
- Low-input warning.
- Optional start/stop sound.

## Milestone 4: Text Processing

Status: future.

- Port raw-first transcript processor behavior.
- Add cleanup levels.
- Add spoken commands.
- Add snippets.
- Add personal dictionary prompt hints.
- Add target-app style presets.

## Milestone 5: Privacy And Memory

Status: future.

- Add transcript history with storage modes:
  - save normally
  - auto-delete after 24 hours
  - never store after insertion
- Add clear history flow.
- Add aggregate usage insights.
- Add reset insights flow.

## Milestone 6: Packaging And Operations

Status: future.

- Add macOS `.app`/DMG build.
- Add Windows installer build.
- Add signing documentation.
- Add launch-at-login.
- Add release notes.
- Add manual QA matrix for both platforms.

## Parallel Track: Mobile Reach

Status: discovery queued after desktop validation and hardening.

Goal: make VerboScribe genuinely usable on phones without assuming desktop
features such as global hotkeys, target reactivation, or clipboard-driven paste
automation exist on mobile.

Product stance:

- Android target: companion app plus systemwide IME.
- iPhone target: companion app first.
- iPhone keyboard support, if pursued later, is insertion-only for
  pre-generated text rather than live microphone dictation.

Scope:

- Run mobile product-shape and provider-strategy spikes before committing to
  implementation stories.
- Reuse `verboscribe-core`, transcript processing, and shared settings/domain
  models where practical.
- Keep mobile platform adapters separate from desktop hotkey, target-tracking,
  and paste adapters.
- Expect native Kotlin and Swift modules for IME, keyboard, and permission
  surfaces even if the main app shell stays Tauri-based.

Acceptance:

- Mobile product constraints and non-goals are documented clearly.
- Android IME foundation and companion flow are backlog-ready.
- iPhone companion flow is backlog-ready with platform limitations explicit.
- The chosen first mobile transcription path is documented with tradeoffs.

## Backlog

- Android IME and companion flow.
- iPhone companion app flow.
- Optional iPhone keyboard insertion surface for pre-generated text.
- Desktop settings and operations surface that exposes the existing backend
  capabilities.
- Model downloader/manager.
- History browser/search.
- Rich snippet manager.
- Rich personal dictionary manager.
- Diagnostics bundle that excludes transcript text by default.
- Updater.
