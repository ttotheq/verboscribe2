# Manual QA

Run manual QA against packaged builds whenever possible.

## Baseline

- App launches without a terminal.
- Tray/menu-bar icon appears.
- Main window can be shown from tray/menu-bar.
- Settings surface displays editable provider paths, language, dictation mode,
  dictation hotkeys, prompt context, pinned terms, live status, and recovery
  text.

## Settings Surface

- Treat every edited field in the desktop form as a draft until
  `Save settings to apply` succeeds.
- Saving settings persists the edited values after app relaunch.
- Saving settings re-applies the configured dictation hotkeys without leaving a
  stale registration behind.
- After changing any surfaced setting such as dictation mode, a dictation
  hotkey, binary path, model path, language, prompt context, or pinned terms,
  save first and then verify behavior or persisted value instead of assuming the
  live runtime updated immediately.
- For toggle-mode QA specifically, confirm the second hotkey press stops
  recording only after the mode change has been saved.
- Manual start, stop, and cancel buttons remain visible in packaged builds and
  drive the same runtime status surface as the hotkey path.
- The prototype-gap note in the desktop UI now treats paste-last as shipped and
  still calls out intentionally missing controls such as retry-last,
  preview/edit, cleanup/style/snippets, history, insights, model
  install/refresh, and launch behavior.

## Vertical Slice

1. Open a target app with a text field.
2. Put the cursor in the field.
3. Trigger the dictation hotkey.
4. Speak a short phrase.
5. Stop recording.
6. Confirm the transcript inserts into the original target app.
7. Confirm the last transcript is visible in VerboScribe 2.

Latest macOS note from 2026-05-13:

- The current default shortcut is `Control+Option+Space` in press-and-hold
  mode, so the key must stay held while speaking.
- A real macOS pass inserted transcript text into Notes, confirming the default
  hotkey-to-paste path works.
- The recognized text on a short built-in-mic phrase was only `you`, so audio
  quality and spoken-phrase QA remain open even though insertion worked.

Latest macOS note from 2026-05-15:

- A packaged-app QA pass from the terminal reached recording and transcription,
  then failed at paste with `System Events got an error: osascript is not
  allowed to send keystrokes. (1002)`.
- This confirms the Accessibility-denied paste fallback path is reachable on
  macOS. The transcript should remain available for manual paste.
- Expected operator recovery is now: open `System Settings > Privacy &
  Security > Accessibility`, allow `VerboScribe 2`, then retry.
- The macOS paste path now avoids `System Events` and sends `Cmd+V` directly
  from the app process, so the earlier terminal-specific workaround should no
  longer be needed once the app is rebuilt and rerun.
- After Accessibility was allowed, a live hotkey run that spoke `hello, hello,
  hello` pasted `you`, confirming the end-to-end hotkey-to-paste path works but
  live built-in-mic transcription quality is still weak on this machine.
- Inspection of the latest live-capture artifact on 2026-05-15 showed the
  generated 5-second WAV was valid but contained all-zero samples, so the live
  app is currently feeding silence to `whisper.cpp` on this machine.
- After adding `NSMicrophoneUsageDescription` to the packaged app bundle and
  resetting macOS microphone approval for `local.verboscribe2`, a fresh live
  hotkey run pasted `Testing verbose scribe dictation in text edit 1 2 3`.
- After switching the debug bundle to stable local signing and deleting the
  stale `/Applications/VerboScribe 2.app` copy, a rebuild-and-retest pasted
  `Testing VerboScribe dictation in TextEdit. 1, 2, 3.` without re-adding
  Accessibility permission.
- A packaged-app microphone-denied retest on 2026-05-15 reported the expected
  recovery guidance instead of recording or pasting, closing that denial path.
- A packaged-app Accessibility-denied retest on 2026-05-15 failed automatic
  paste, reported the expected Accessibility recovery guidance, and manual
  `Cmd+V` pasted the exact preserved dictation text, closing that denial path.
- The latest live-capture WAV is now non-zero and `whisper.cpp` is receiving
  real speech again, so the remaining macOS issue is recognition quality and
  formatting, not silent capture or paste failure.
- For quality QA, do not use a clipped one-word utterance as the only check.
  Hold the hotkey, wait about half a second, speak a distinct 3 to 5 second
  phrase such as `Testing VerboScribe dictation in TextEdit one two three`,
  keep holding briefly after speaking, then release.

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
  transcription, target reactivation, paste insertion, and last-transcript
  capture in the VerboScribe 2 status UI.
- Missing `whisper.cpp` binary or model paths should report actionable recovery
  text instead of hanging the dictation flow.
- After a successful dictation, the last transcript should remain visible in the
  app status surface after insertion succeeds.
- If automatic paste fails but a transcript was preserved, the `Paste last
  transcript` button should stay enabled and recovery text should mention that
  the transcript can be retried with that action.
- The dedicated paste-last hotkey should be visible in the settings form and the
  status rail, and after saving a new value it should retry the preserved
  transcript without starting a new recording.
- For QA without the Tauri hotkey layer, use
  `cargo run -p verboscribe2-desktop --example live_dictation_probe -- 6000`
  with a target editor frontmost. On 2026-05-13 this inserted
  `Testing verb ascribe dictation.` into TextEdit and preserved the same text
  on the clipboard.

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
  transcription, insertion, or recovery.
- Re-registering the configured hotkey does not leave duplicate registrations
  behind.

## Privacy Checks

- Default transcript history mode is `Never store after insertion`.
- History modes are clearly described before enabling storage.
- Settings export excludes API keys.
