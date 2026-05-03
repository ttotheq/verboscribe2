# Epics

## EPIC-01 Foundation And Delivery System

Create a maintainable cross-platform workspace with verification, documentation,
and clean boundaries.

Success outcomes:

- Rust workspace and Tauri desktop shell exist.
- Core crate is platform-neutral.
- Verification script catches build/test regressions.
- Agile planning artifacts are current.

## EPIC-02 Dictation Vertical Slice

Deliver the first useful app loop: global hotkey, microphone recording,
transcription, and paste into the previously active app.

Success outcomes:

- macOS and Windows app launch.
- Hotkey starts/stops recording.
- Local `whisper.cpp` returns a transcript.
- Paste insertion works or leaves text on clipboard with recovery guidance.
- The full loop is validated on macOS and Windows through manual QA and smoke
  coverage.

## EPIC-03 Transcription Providers

Support local-first transcription and optional cloud transcription through one
provider abstraction.

Success outcomes:

- `whisper.cpp` provider is reliable and smoke-tested.
- Groq Whisper provider is available behind explicit settings.
- Provider errors are actionable.
- API keys are stored in platform secret storage.

## EPIC-04 Platform Integration

Implement OS-specific behavior behind clear interfaces.

Success outcomes:

- macOS and Windows hotkey adapters.
- macOS and Windows text insertion adapters.
- Target app tracking.
- Tray/menu-bar behavior.
- Launch-at-login.
- Platform adapters are isolated behind testable boundaries and documented
  recovery paths.

## EPIC-04A Runtime Orchestration And Recovery

Bridge UI, tray/menu-bar, hotkeys, and long-running backend work without leaking
platform details into the core.

Success outcomes:

- Tauri commands delegate to an app service/controller layer.
- Backend emits status/progress/recovery events for the UI.
- Cancellation works during recording and transcription.
- Permission and paste failures are explicit, recoverable, and test-covered.
- The app-service boundary exposes the current dictation state clearly enough to
  support cold-start resumption and manual QA.

## EPIC-05 Dictation Quality

Make dictated text useful in daily writing contexts.

Success outcomes:

- Raw-first transcript behavior.
- Cleanup levels.
- Spoken commands.
- Snippets.
- Personal dictionary prompt hints.
- Target-app style presets.

## EPIC-06 Privacy, Memory, And Settings

Add durable local settings and optional local memory with privacy controls.

Success outcomes:

- Typed settings with migrations.
- Transcript history privacy modes.
- Usage insights.
- Settings backup/restore excluding secrets.

## EPIC-07 Packaging And Release Readiness

Make the app installable, testable, and maintainable on macOS and Windows.

Success outcomes:

- macOS bundle/DMG.
- Windows installer.
- Signing/notarization guidance.
- Manual QA matrix.
- Release notes and diagnostics.

## EPIC-08 Platform Smoke Harness

Build repeatable checks for OS-specific behavior that cannot be fully covered by
unit tests.

Success outcomes:

- WAV format validation.
- Local `whisper.cpp` sample transcription smoke test.
- Clipboard and paste fallback checks.
- Hotkey registration smoke checks.
- Manual QA scripts/checklists stay aligned with implemented behavior.
- Smoke and manual QA checks make it obvious which behaviors are implemented
  versus validated on real desktops.
