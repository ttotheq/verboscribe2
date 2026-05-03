# Manual QA

Run manual QA against packaged builds whenever possible.

## Baseline

- App launches without a terminal.
- Tray/menu-bar icon appears.
- Main window can be shown from tray/menu-bar.
- Settings surface displays status, provider, hotkey, and recovery text.

## Vertical Slice

1. Open a target app with a text field.
2. Put the cursor in the field.
3. Trigger the dictation hotkey.
4. Speak a short phrase.
5. Stop recording.
6. Confirm the transcript inserts into the original target app.
7. Confirm the last transcript is visible in VerboScribe 2.

macOS target apps:

- TextEdit
- Terminal
- Browser text field

Windows target apps:

- Notepad
- Windows Terminal
- Browser text field

## App-Service Integration

- With valid local `whisper.cpp` binary and model paths saved in settings,
  pressing and releasing the dictation hotkey should drive recording,
  transcription, and last-transcript capture in the VerboScribe 2 status UI.
- Missing `whisper.cpp` binary or model paths should report actionable recovery
  text instead of hanging the dictation flow.
- After a successful dictation, the last transcript should remain visible in the
  app status surface even though clipboard insertion is not implemented yet.

## Failure Recovery

- Deny or remove paste automation permission where applicable.
- Dictate a short phrase.
- Expected: transcription succeeds, automated paste fails clearly, and the
  transcript remains on the clipboard for manual paste.

## Provider Checks

Local `whisper.cpp`:

- Missing binary path shows actionable configuration error.
- Missing model path shows actionable configuration error.
- Valid binary/model transcribes a known sample.

Groq Whisper:

- Missing API key shows actionable configuration error.
- Invalid API key reports provider failure without losing local settings.
- Real API key test is required before marking the provider production-ready.

## Recording Checks

- Recording shorter than the minimum threshold is ignored.
- Maximum recording duration stops and transcribes.
- Low microphone input produces a visible warning without stopping recording.
- Default microphone records successfully on macOS and Windows.
- Denied or unavailable microphone input reports recovery text instead of
  hanging the recording flow.
- Recorded audio is accepted by WAV validation before transcription.

## Hotkey Checks

- Default dictation hotkey registers on app launch.
- Hotkey registration failure is visible in the status UI.
- Pressing the global hotkey starts the live dictation flow and updates the
  status surface while recording.
- Releasing the global hotkey stops recording and advances the status surface to
  transcription or recovery.
- Re-registering the configured hotkey does not leave duplicate registrations
  behind.

## Privacy Checks

- Default transcript history mode is `Never store after insertion`.
- History modes are clearly described before enabling storage.
- Settings export excludes API keys.
