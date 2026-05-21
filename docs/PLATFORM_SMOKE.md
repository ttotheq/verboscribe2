# Platform Smoke Harness

Use these checks before claiming platform-facing behavior works. Automated
checks should run from the repo root. Manual checks stay explicit until the
implemented adapters have been exercised on real desktops.

## Automated Local Fixture Checks

## Automated App-Service Smoke

Run:

```sh
./scripts/smoke-app-service.sh
```

Coverage:

- App-service dictation cycle: starts and stops dictation through `AppService`
  using injected target, recorder, transcription, and insertion adapters.
- Success path: verifies the transcript is retained and reported after a
  successful insertion.
- Paste-failure path: verifies recovery status is surfaced while the transcript
  remains available for manual paste.
- Isolation: runs without live microphone permission, global hotkey
  registration, desktop automation, or local `whisper.cpp` binaries.

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

Status: adapter implemented, happy path plus denied-permission branches manually
verified on this machine; quality and cross-platform follow-up remain.

- Latest findings from 2026-05-15:
  - A packaged-app QA pass reached recording and transcription, then failed at
    paste with `System Events got an error: osascript is not allowed to send
    keystrokes. (1002)`, confirming the Accessibility-denied path is
    reproducible on macOS.
  - The macOS paste path now avoids `System Events` and reactivates the target
    with `/usr/bin/open -b`, then sends `Cmd+V` directly from the app process.
  - Recovery guidance should point the operator only to `System Settings >
    Privacy & Security > Accessibility` for `VerboScribe 2`.
  - After Accessibility was allowed, a live default-hotkey run that spoke
    `hello, hello, hello` pasted `you`, confirming paste automation can succeed
    while live built-in-mic transcription quality remains weak.
  - Inspection of the latest live-capture artifact showed the generated
    5-second WAV was valid but contained all-zero samples, so the current live
    capture path is feeding silence to `whisper.cpp` on this machine.
  - After adding `NSMicrophoneUsageDescription` to the packaged app bundle and
    resetting macOS microphone approval for `local.verboscribe2`, a fresh live
    hotkey run pasted `Testing verbose scribe dictation in text edit 1 2 3`.
  - The latest live-capture WAV now has non-zero samples, confirming the
    silent-capture bug was resolved on this machine.
  - After switching the debug bundle to stable local signing and removing the
    stale `/Applications/VerboScribe 2.app` copy, a rebuild-and-retest pasted
    `Testing VerboScribe dictation in TextEdit. 1, 2, 3.` without re-adding
    Accessibility permission.
  - A packaged-app microphone-denied retest reported the expected recovery
    guidance, so the macOS microphone-denial branch is now manually verified.
  - After resetting Accessibility approval for `local.verboscribe2`, a
    packaged-app Accessibility-denied retest failed automatic paste, reported
    the expected Accessibility recovery guidance, and manual `Cmd+V` pasted the
    exact preserved dictation text, so that denial branch is now manually
    verified on the current direct-paste path.
  - The next manual quality check should use a distinct 3 to 5 second phrase
    with a short hold before and after speaking so startup or release timing is
    less likely to clip the utterance.

- Previous findings from 2026-05-13:
  - `./script/build_and_run.sh --verify` now launches the packaged debug app
    correctly after resolving the real `CFBundleExecutable` from the bundle.
  - The product-default macOS shortcut is `Control+Option+Space` in
    press-and-hold mode, and a physical run pasted transcript text into Notes.
  - The current default input device on this machine is `MacBook Air
    Microphone`.
  - `cargo run -p verboscribe2-desktop --example live_dictation_probe -- 6000`
    inserted `Testing verb ascribe dictation.` into TextEdit and preserved the
    same text on the clipboard, proving the real app-service capture,
    transcription, reactivation, and paste path works on macOS.
  - A physical built-in-mic run produced a valid mono 16 kHz WAV plus a
    transcript file containing only `you`, so live speech quality still needs
    tuning or clearer operator guidance on this machine.

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
- Silent capture: verify a zero-signal recording reports an actionable
  microphone-signal recovery instead of transcribing silence as `you`.
- Accessibility permission: verify paste automation failure reports recovery
  clearly when `VerboScribe 2` lacks Accessibility access.

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

- Stable-signed rebuilds now preserve Accessibility trust on this machine for
  the happy path.
- Live capture now works again on this machine, but transcription quality is
  still imperfect on short and product-name phrases such as `VerboScribe`.
- Linux clipboard/paste automation is not implemented.
- Windows paste and target tracking behavior still needs manual validation.
