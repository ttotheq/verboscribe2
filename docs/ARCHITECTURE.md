# Architecture

## Recommendation

Build VerboScribe 2 with **Tauri 2 + Rust core + TypeScript UI**.

The app needs native desktop behavior on both macOS and Windows: global
hotkeys, tray/menu-bar operation, microphone recording, local `whisper.cpp`,
clipboard paste insertion, launch-at-login, credential storage, packaging, and
permission recovery. Tauri gives a small desktop shell and official plugins for
many of these surfaces while keeping privileged logic in Rust.

The UI should be a pragmatic settings/status surface, not a clone of the
macOS-only AppKit prototype. Use Svelte + Vite unless there is a strong reason
to use React.

## Alternatives Considered

### Electron + TypeScript/Node

Pros:

- Mature desktop ecosystem.
- Built-in global shortcuts, tray, clipboard, and packaging workflows.
- Large pool of existing libraries.

Cons:

- Heavier runtime.
- Audio capture, paste automation, whisper.cpp orchestration, and permissions
  still need native modules or sidecars.
- Easier to blur privileged OS behavior into UI code.

### Flutter Desktop + Dart/Rust FFI

Pros:

- Good cross-platform UI toolkit.
- Official desktop targets for macOS and Windows.
- Strong for polished, custom UI.

Cons:

- Global hotkeys, tray behavior, launch-at-login, and paste automation lean more
  on third-party plugins or custom platform plugins.
- Rust FFI adds an extra bridge if the dictation engine is written in Rust.

### Separate Native Apps

Pros:

- Best native integration on each OS.
- AppKit/SwiftUI and WinUI/.NET can each use platform APIs directly.

Cons:

- Duplicates product logic and tests.
- Slower iteration.
- Higher risk that macOS and Windows behavior diverge.

## Target Repository Layout

```text
verboscribe2/
  apps/
    desktop/
      src/                   # Svelte/TypeScript UI
      src-tauri/             # Tauri shell, commands, plugin wiring
  crates/
    verboscribe-core/         # state machine, transcript processing, domain types
    verboscribe-audio/        # recording facade and shared audio types
    verboscribe-transcription/# whisper.cpp and Groq providers
    verboscribe-platform/     # macOS/Windows adapters behind traits
    verboscribe-storage/      # settings, history, insights, backups
  docs/
  scripts/
  tests/
```

## Core Data Flow

```text
HotkeyService / UI command
          |
          v
DictationEngine
          |
          +--> TargetAppTracker
          +--> AudioRecorder
          +--> TranscriptionProvider
          +--> TranscriptProcessor
          +--> TextInsertionService
          +--> HistoryStore
          +--> UsageInsightsStore
```

## Core Interfaces

- `DictationEngine`: platform-neutral state machine. Owns recording,
  transcription, processing, insertion, cancellation, retry, preview, and
  status events.
- `HotkeyService`: registers dictation, cancel, and paste-last shortcuts and
  emits pressed/released events.
- `AudioRecorder`: requests microphone readiness, starts WAV recording, reports
  input levels, and stops to an audio file.
- `TranscriptionProvider`: transcribes an audio file to raw text.
- `TranscriptProcessor`: turns raw transcript text into inserted text using
  cleanup levels, spoken commands, style presets, snippets, and dictionary
  hints.
- `TextInsertionService`: inserts text into the prior target app through
  clipboard plus paste shortcut.
- `ClipboardService`: preserves, writes, and restores clipboard contents where
  possible.
- `TargetAppTracker`: captures the active app before recording and remembers the
  last non-VerboScribe target.
- `SettingsStore`: typed settings with migration/versioning.
- `SecretStore`: stores API keys in Keychain on macOS and Credential
  Manager/DPAPI on Windows.
- `HistoryStore`: optional transcript history with explicit privacy modes.
- `UsageInsightsStore`: local aggregate counters only.
- `LaunchAtLoginService`: macOS login item and Windows startup registration.
- `TrayService`: macOS menu bar and Windows notification area behavior.

## Platform Boundaries

Platform-specific code belongs in `verboscribe-platform` or Tauri shell wiring.
The Rust core should not import macOS frameworks, Win32 APIs, or Tauri types.

Examples:

- macOS text insertion: Accessibility trust, target activation, synthetic
  `Cmd+V`.
- Windows text insertion: foreground window tracking, clipboard write,
  `SendInput` for `Ctrl+V`.
- macOS secrets: Keychain.
- Windows secrets: Credential Manager or DPAPI.
- macOS launch at login: login item.
- Windows launch at login: startup task or registry-backed shortcut.

## Prototype Reference Decisions

Port conceptually:

- State machine behavior from `DictationController`.
- Provider abstraction.
- Raw-first transcript processing.
- Snippet expansion.
- Personal dictionary prompt construction.
- Privacy-first history modes.
- Local aggregate usage insights.
- Settings backup that excludes API keys.
- Manual QA and verification discipline.

Rewrite:

- Audio recording.
- Global hotkeys.
- Text insertion.
- Launch at login.
- UI.
- Packaging.

Discard or defer:

- AppKit `AppDelegate` and `MainViewController`.
- Carbon hotkey implementation.
- AVFoundation-only recording path.
- ApplicationServices-only paste implementation.
- Preview/edit mode, history browser, model installer UI, and usage insights UI
  until the first vertical slice is reliable.
