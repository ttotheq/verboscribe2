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

## Privacy Checks

- Default transcript history mode is `Never store after insertion`.
- History modes are clearly described before enabling storage.
- Settings export excludes API keys.
