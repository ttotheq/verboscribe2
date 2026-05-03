# Platform Smoke Harness

Use these checks before claiming platform-facing behavior works. Automated
checks should run from the repo root. Manual checks stay explicit until the
hotkey, target tracking, clipboard, and paste adapters exist.

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

Status: pending adapter implementation.

- Hotkey registration: verify the dictation shortcut registers, reports
  conflicts, and receives press/release events outside the app.
- Target tracking: verify the active app before recording is captured and
  remains available while VerboScribe has focus.
- Clipboard write: verify transcript text is placed on the clipboard.
- Paste fallback: verify a failed paste leaves transcript text available for
  manual paste.
- Microphone permission: verify denial produces recovery text pointing to
  System Settings > Privacy & Security > Microphone.

## Windows Manual Smoke Checklist

Status: pending adapter implementation.

- Hotkey registration: verify the dictation shortcut registers, reports
  conflicts, and receives press/release events outside the app.
- Target tracking: verify the foreground window before recording is captured and
  remains available while VerboScribe has focus.
- Clipboard write: verify transcript text is placed on the clipboard.
- Paste fallback: verify a failed paste leaves transcript text available for
  manual paste.
- Microphone permission: verify denial produces recovery text pointing to
  Settings > Privacy & security > Microphone.

## Current Gaps

- Live microphone capture is not implemented.
- Global hotkey registration is not implemented.
- Target app/window tracking is not implemented.
- Clipboard paste insertion is not implemented.
- Packaged app permission prompts have not been manually QA'd.
