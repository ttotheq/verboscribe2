# Platform Smoke Harness

Use these checks before claiming platform-facing behavior works. Automated
checks should run from the repo root. Manual checks stay explicit until the
implemented adapters have been exercised on real desktops.

## Automated Local Fixture Checks

Run:

```sh
./scripts/smoke-local-fixtures.sh
```

Coverage:

- WAV validity: runs `verboscribe-audio` tests for writing and validating mono
  16 kHz 16-bit PCM WAV files.
- Local transcription: runs the `whisper.cpp` provider smoke against the local
  binary, model, and JFK sample.

The local transcription smoke uses these defaults and supports environment
overrides:

- `VERBOSCRIBE_WHISPER_CPP_BIN`
- `VERBOSCRIBE_WHISPER_CPP_MODEL`
- `VERBOSCRIBE_WHISPER_CPP_SAMPLE`

## macOS Manual Smoke Checklist

Status: adapter implemented, manual QA pending.

- Hotkey registration: verify the dictation shortcut registers, reports
  conflicts, and receives press/release events outside the app.
- Target tracking: verify the active app before recording is captured and
  remains available while VerboScribe has focus.
- Clipboard write: verify transcript text is placed on the clipboard.
- Paste insertion: verify transcript text pastes back into the previously
  active app after dictation ends.
- Paste fallback: verify a failed paste leaves transcript text available for
  manual paste.
- Microphone permission: verify denial produces recovery text pointing to
  System Settings > Privacy & Security > Microphone.
- Accessibility permission: verify paste automation failure reports recovery
  clearly when System Events access is denied.

## Windows Manual Smoke Checklist

Status: first adapter implemented, manual QA pending.

- Hotkey registration: verify the dictation shortcut registers, reports
  conflicts, and receives press/release events outside the app.
- Target tracking: verify the foreground window before recording is captured and
  remains available while VerboScribe has focus.
- Clipboard write: verify transcript text is placed on the clipboard.
- Paste insertion: verify transcript text pastes back into the previously
  active window after dictation ends.
- Paste fallback: verify a failed paste leaves transcript text available for
  manual paste.
- Microphone permission: verify denial produces recovery text pointing to
  Settings > Privacy & security > Microphone.
- Foreground activation: verify the saved target window is reactivated before
  paste when focus changed during recording.

## Current Gaps

- Packaged app permission prompts have not been manually QA'd.
- Linux clipboard/paste automation is not implemented.
- Windows paste and target tracking behavior still needs manual validation.
