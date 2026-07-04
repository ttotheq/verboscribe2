# VerboScribe 2 Handoff

Last updated: 2026-07-03, after surfacing `Paste last transcript` recovery in
the desktop UI and making paste-failure recovery explicitly point users toward
that action, on branch `feature/paste-last-recovery`.
Prior update: 2026-06-06, after adding a second, dedicated toggle dictation
hotkey (`Control+Option+D` by default) alongside the existing press-and-hold
`Control+Option+Space` hotkey, on branch `feature/toggle-dictation-hotkey`.
Earlier update: 2026-05-24, after merging PR #1 — the Sprint 9 + Sprint 10
integration branch plus the macOS menu-bar icon fix and notch-placement
diagnosis — into `main` via the first real GitHub PR for this repo, and
catching `origin/main` up with 13 previously-unpushed sprint commits.

## Resume First

Run these commands from the repo root:

```sh
git status --short
cargo fmt --all -- --check
./scripts/verify.sh
./scripts/smoke-app-service.sh
```

If working on the local `whisper.cpp` provider and the local binary/model/sample
are still installed, also run:

```sh
./scripts/smoke-local-fixtures.sh
```

If continuing the live macOS QA slice, also run:

```sh
./script/build_and_run.sh --verify
cargo run -p verboscribe2-desktop --example live_dictation_probe -- 6000
```

Expected current result:

- `git status --short` prints empty after the local paste-last recovery commit.
- `git branch --show-current` prints `feature/paste-last-recovery`.
- `cargo fmt --all -- --check` passes.
- `./scripts/verify.sh` passes.
- `./scripts/smoke-app-service.sh` passes.
- `./scripts/smoke-local-fixtures.sh` passes on this machine and includes:
  `And so my fellow Americans, ask not what your country can do for you, ask what you can do for your country.`
- `./script/build_and_run.sh --verify` passes and rebuilds the bundled macOS
  app at `target/debug/bundle/macos/VerboScribe 2.app`.

## Current Project State

VerboScribe 2 is currently a desktop-first cross-platform dictation app. The
implemented vertical slice targets macOS and Windows. Mobile expansion is now
accepted into planning with this stance:

- Android: companion app plus systemwide IME.
- iPhone: companion app first.
- Any future iPhone keyboard extension is insertion-only follow-up, not the
  primary live-dictation path.

The selected stack is:

- Tauri 2 desktop shell.
- Rust workspace for core, audio, transcription, platform, and storage crates.
- TypeScript frontend through Vite.
- Local-first transcription through `whisper.cpp`.
- Optional Groq Whisper later.

Mobile-specific architecture and provider choices are not yet decided. They are
queued as discovery work in `docs/BACKLOG.md` and `docs/ROADMAP.md`.

The existing macOS-only prototype at `~/projects/whisper` was inspected
read-only. Do not modify it. Product/behavior notes are captured in
`docs/PROTOTYPE_NOTES.md`.

The authoritative current feature inventory and prototype-gap summary now lives
in `docs/FEATURE_LIST.md`. Read it early if the next session is about product
scope, missing UI, or parity with the older macOS prototype.

## Agile Status

Current agile process:

- Follow `docs/AGILE_OPERATING_MODEL.md`.
- Follow `docs/GIT_WORKFLOW.md`.
- Keep `docs/BACKLOG.md`, `docs/SPRINTS.md`, and this handoff current.
- Every sprint must close with full review, retrospective, and retro actions.

Git workflow status:

- `main` is the releasable trunk.
- Short-lived branches are the default for nontrivial work.
- Use `feature/*`, `fix/*`, and `spike/*` branches for focused slices.
- Use `git worktree` when parallel AI work would otherwise collide.
- Tag sprint completions or release checkpoints when useful.

Current branch:

- `main` — clean, in sync with `origin/main`. The Sprint 9 + Sprint 10
  integration branch (`feature/sprint-9-quality-hardening`) was rebase-merged
  via PR #1 and deleted both locally and on the remote. Open a new
  `feature/*` branch for the next slice of work.

GitHub workflow status:

- Repo: https://github.com/ttotheq/verboscribe2
- Until 2026-05-24 this repo had no PR history — all sprint work landed on
  local `main` and was never pushed. PR #1 was the first to exercise the
  CI gate against `origin/main`. Going forward, new sprints should follow
  push-branch → `gh pr create --base main` → CI gate → `gh pr merge
  --rebase --delete-branch`. The CI workflow (`.github/workflows/ci.yml`)
  runs `Verify (ubuntu-latest)` and `Verify (macos-latest)` on PRs and on
  pushes to `main`; both must be green before merge.
- macOS-only Tauri APIs (`ActivationPolicy`, `AppHandle::show`) must be
  cfg-gated; Linux CI caught a regression mid-PR-#1 because they were
  imported unconditionally, and a local Mac build can't catch this class
  of bug. Treat the Linux CI run as the authoritative cross-platform check.

Recent merge status:

- PR #1 `feature/sprint-9-quality-hardening` rebase-merged into `main` on
  2026-05-24, carrying:
  - `bd4f62e chore: ignore Codex workspace state`
  - `4d97682 feat(desktop-macos): add tray icon and stabilize local app packaging`
  - `b297155 fix(dictation): harden macOS runtime recovery and QA seams`
  - `74000df feat(desktop): replace the status shell with a settings surface`
  - `a48d5f0 docs: refresh sprint status and product planning`
  - `9fee901 fix(desktop-macos): restore visible menu-bar icon via Tauri tray + diagnose notch placement`
  - `bc2a06b fix(desktop): gate macOS-only Tauri APIs behind cfg`
- Earlier sprint branches that had been merged to local `main` long ago
  but never pushed are now on `origin/main` as part of the same 2026-05-24
  catch-up push:
  - `feature/sprint-7-clipboard-insertion`
  - `feature/sprint-6-live-dictation`
  - `feature/sprint-5-hotkeys`
  - `feature/ci-baseline`
  - `feature/sprint-4-live-capture`

Completed:

- Sprint 1: Foundation To Testable Core.
- Sprint 2: Local Audio And Transcription Slice.
- Sprint 3: App-Service Integration And Recovery.
- Sprint 4: Live Capture Adapter.
- Sprint 5: Global Hotkey Adapter.
- Sprint 6: Live Dictation Service Integration.
- Sprint 7: Clipboard Insertion And Target Tracking.
- Sprint 8: App-Service Dictation Smoke Path.
- Sprint 9: macOS End-To-End Dictation QA.
- Sprint 10: Desktop Settings Surface Foundation.

Recent implementation and planning updates:

- Generated the standard Tauri app icon set from the current
  `VerboScribe 2` concept art, including `icon.icns` for macOS and `icon.ico`
  for Windows.
- Set `bundle.icon` explicitly in `apps/desktop/src-tauri/tauri.conf.json` so
  locally built bundles no longer depend on implicit default icon discovery.
- Verified that rebuilt macOS app bundles now contain
  `Contents/Resources/icon.icns` and `CFBundleIconFile = icon.icns`, fixing
  the blank Dock icon on the locally built app bundle.
- Added a deterministic app-service smoke path that covers both successful
  insertion and paste-failure transcript preservation without live OS
  dependencies.
- Fixed `script/build_and_run.sh` so `--verify` and `--debug` resolve the real
  bundled executable from `CFBundleExecutable` instead of assuming the display
  name is also the process name.
- Added `apps/desktop/src-tauri/examples/live_dictation_probe.rs` so macOS QA
  can drive the real `AppService` start/stop path without depending on global
  shortcut delivery.
- Added opt-in hotkey debug logging behind `VERBOSCRIBE_DEBUG_HOTKEYS=1` to
  observe registration and pressed/released shortcut events from the live Tauri
  app.
- Updated the desktop status UI so it now shows the active dictation mode plus
  a mode-specific usage hint such as “hold while speaking” or “press again to
  stop,” instead of leaving the hotkey behavior implicit.
- Replaced the macOS paste path so target reactivation now uses
  `/usr/bin/open -b <bundle-id>` and the final `Cmd+V` is sent directly from
  the app process instead of through `System Events`.
- Hardened macOS paste-failure recovery so Accessibility-denied paste failures
  now map to actionable recovery text pointing to `System Settings > Privacy &
  Security > Accessibility` for `VerboScribe 2`.
- Added `NSMicrophoneUsageDescription` to the packaged macOS app bundle so the
  rebuilt app can participate in the correct microphone permission flow.
- Added default `whisper.cpp` prompt context for `VerboScribe`, `VerboScribe 2`,
  `whisper.cpp`, and `TextEdit` so the local model gets a lightweight bias
  toward product and app names.
- Reverted the macOS menu-bar code back to Tauri's built-in tray-icon API,
  unified with the Windows/Linux path; the prior custom AppKit
  `NSStatusItem`/`NSMenu` bridge and its `objc2` + `objc2-app-kit` +
  `objc2-foundation` dependencies were removed.
- Kept `ActivationPolicy::Regular` on macOS so the Dock icon stays present
  alongside the menu-bar extra.
- The tray icon now loads from
  `apps/desktop/src-tauri/icons/concepts/verboscribe2-mark-concept-v1-32.png`
  with `icon_as_template(false)` so the full-color VS2 mark renders instead
  of the alpha-only template silhouette (template mode produced a solid white
  block because the source PNG is fully opaque).
- The menu still exposes `Show VerboScribe 2` and `Quit VerboScribe 2`.
- Root-cause for the prior "icon doesn't appear" report on this MacBook Air
  M3 was diagnosed as **macOS Tahoe per-app extras placement around the
  notch**, not a rendering bug:
  - `NSScreen.auxiliaryTopLeftArea` and `auxiliaryTopRightArea` report the
    notch occupying `x = 646..825` on this 1470-wide logical screen.
  - macOS places per-app extras left-to-right in registration order; when
    other apps (WeatherMenu, RoboForm, Ollama) already occupy slots right
    of the notch, newcomers land in the leftmost slot at roughly
    `x = 805..841`, mostly behind the notch with only ~14 px visible past
    the notch's right edge before the next extra starts.
  - Both the prior custom AppKit `NSStatusItem` and Tauri's tray-icon
    produced the same invisible-behind-notch result; `Accessory` vs
    `Regular` activation policy did not change the placement.
  - The accessible workaround is to Cmd-drag the VS2 extra to a position
    right of the existing per-app extras, or to quit one of the other
    menu-bar apps so the leftmost slot moves past the notch on next launch.
    macOS remembers the new ordering across relaunches.
  - System-wide capacity for menu-bar extras is governed by Control Center
    module visibility (System Settings → Control Center) and any third-party
    menu-bar manager (Ice, Bartender, etc.); it is not something the app
    can change.
- Persisted optional `whisper.cpp` prompt overrides in `settings.json` through
  `transcription.whisperCpp.promptContext` and
  `transcription.whisperCpp.pinnedTerms`, with backward-compatible defaults so
  older settings files still load cleanly.
- Added light audio preprocessing in `crates/verboscribe-audio/src/lib.rs` so
  the recorded WAV sent to `whisper.cpp` now trims obvious dead air around
  detected speech and boosts low-volume captures before writing the mono
  16 kHz file; automated verification passed, but live macOS QA still needs to
  confirm whether this materially improves short-phrase recognition.
- Added a distinct `VerboScribe 2` icon concept as a vector-first master under
  `design/` plus rendered PNG variants under
  `apps/desktop/src-tauri/icons/concepts/`. The concept uses a warm editorial
  palette and a speech-to-text ribbon motif instead of a microphone or generic
  waveform mark so it can diverge visually from the older VerboScribe app.
- Added `VS2-023: VerboScribe 2 Icon Exploration` to `docs/BACKLOG.md` because
  the current concept is only acceptable as a temporary placeholder and should
  return later with multiple stronger options.
- Updated `script/build_and_run.sh` to re-sign the bundled macOS app with a
  stable local code-signing identity when available, instead of leaving rebuilds
  ad-hoc signed.
- Added `docs/FEATURE_LIST.md` as the consolidated feature inventory and
  prototype-to-current UI gap review so product scope is no longer split only
  across epics, backlog, roadmap, and prototype notes.
- Added `VS2-024: Desktop Settings Surface Foundation` to the backlog because
  the current Tauri UI is still a thin status shell while the prototype had a
  much broader settings and operations surface.
- Completed `VS2-024` by replacing the status-only Tauri shell with a real
  settings-and-status surface that edits saved `whisper.cpp` paths, language,
  dictation mode, hotkey, prompt context, and pinned terms directly in the app.
- Added save and reload flows in the desktop UI, re-applied the dictation
  hotkey after saves, and kept live provider, recovery, usage, and
  last-transcript visibility beside the form.
- Added a `Paste last transcript` desktop action backed by the app-service so a
  transcript preserved after paste failure can be retried without recording
  again; empty-state recovery now reports that no previous transcript is
  available.
- Added manual start, stop, cancel, and paste-last buttons to the desktop shell
  for focused runtime QA, plus a browser-preview fallback mode so the Vite
  build still renders outside the Tauri shell when commands are unavailable.
- Follow-up UI hardening: changing dictation mode in the desktop form is still
  a draft until saved, so the UI now shows an explicit unsaved-settings warning,
  changes the save button label to `Save settings to apply`, and no longer
  implies that toggle mode is live immediately after a click.
- Manual QA after the Sprint 10 UI slice confirmed the core toggle mode still
  works; the confusing behavior was that the saved settings file remained on
  `pressAndHold` until the user explicitly saved the form.
- macOS QA findings now show:
  - the default `Control+Option+Space` press-and-hold shortcut does reach the
    live app on this machine,
  - a real default-hotkey run pasted transcript text into Notes,
  - a direct app-service probe pasted into TextEdit and preserved clipboard
    text,
  - a packaged-app terminal QA run reached recording and transcription, then
    failed at paste with `osascript is not allowed to send keystrokes. (1002)`,
  - after Accessibility was allowed, a live run that spoke `hello, hello,
    hello` pasted `you`, confirming the end-to-end path works while live
    built-in-mic transcription quality remained suspect,
  - inspection of the latest 5-second live-capture WAV showed every sample was
    zero, so the app is currently feeding silence to `whisper.cpp` on this
    machine,
  - the recorder now treats an all-zero capture as a microphone-signal failure
    instead of transcribing silence as `you`,
  - after adding `NSMicrophoneUsageDescription` and resetting microphone
    approval for `local.verboscribe2`, a fresh live run pasted `Testing verbose
    scribe dictation in text edit 1 2 3`,
  - the latest live-capture WAV now has non-zero samples, confirming the
    silent-capture issue was resolved on this machine,
  - the macOS end-to-end path now works again, with remaining issues limited to
    recognition quality and formatting on short phrases and product names,
  - the debug bundle is now signed as `local.verboscribe2` with authority
    `Whisper Dictation Local Code Signing`, which should reduce Accessibility
    permission resets across rebuilds,
  - after removing the stale `/Applications/VerboScribe 2.app` copy, a
    rebuild-and-retest pasted `Testing VerboScribe dictation in TextEdit. 1, 2,
    3.` without re-adding Accessibility permission,
  - a packaged-app microphone-denied retest reported the expected recovery
    guidance instead of recording or pasting,
  - after resetting Accessibility approval for `local.verboscribe2`, a
    packaged-app Accessibility-denied retest failed automatic paste, reported
    the expected Accessibility recovery guidance, and manual `Cmd+V` pasted the
    exact preserved dictation text,
  - the next quality experiment now ships a default `whisper.cpp --prompt`
    context for product and app names, but it still needs live manual QA.
- Mobile expansion was added to the planning docs without changing the desktop
  validation priority.
- Changed files of interest:
  - `script/build_and_run.sh`
  - `apps/desktop/src-tauri/src/app_service.rs`
  - `apps/desktop/src-tauri/tauri.conf.json`
  - `apps/desktop/src-tauri/Info.plist`
  - `apps/desktop/src-tauri/icons/32x32.png`
  - `apps/desktop/src-tauri/icons/64x64.png`
  - `apps/desktop/src-tauri/icons/128x128.png`
  - `apps/desktop/src-tauri/icons/128x128@2x.png`
  - `apps/desktop/src-tauri/icons/icon.icns`
  - `apps/desktop/src-tauri/icons/icon.ico`
  - `apps/desktop/src-tauri/icons/icon.png`
  - `apps/desktop/src-tauri/src/hotkeys.rs`
  - `apps/desktop/src-tauri/src/lib.rs`
  - `apps/desktop/src-tauri/Cargo.toml` (objc2 deps removed this session)
  - `apps/desktop/src-tauri/examples/live_dictation_probe.rs`
  - `crates/verboscribe-audio/src/lib.rs`
  - `crates/verboscribe-storage/src/lib.rs`
  - `apps/desktop/src/main.ts`
  - `apps/desktop/src/styles.css`
  - `scripts/smoke-app-service.sh`
  - `docs/MANUAL_QA.md`
  - `docs/BACKLOG.md`
  - `docs/PLATFORM_SMOKE.md`
  - `docs/SETUP_AND_OPERATIONS.md`
  - `docs/SPRINTS.md`
  - `docs/ROADMAP.md`
  - `docs/EPICS.md`
  - `design/verboscribe2-mark-concept-v1.svg`
  - `apps/desktop/src-tauri/icons/concepts/verboscribe2-mark-concept-v1-1024.png`
  - `apps/desktop/src-tauri/icons/concepts/verboscribe2-mark-concept-v1-256.png`
  - `HANDOFF.md`

## Completed Implementation

Foundation:

- Root Rust workspace in `Cargo.toml`.
- Tauri desktop shell under `apps/desktop`.
- Desktop settings and status UI in `apps/desktop/src/main.ts` and
  `apps/desktop/src/styles.css`.
- Placeholder Tauri icon in `apps/desktop/src-tauri/icons/icon.png`.
- Verification script in `scripts/verify.sh`.

Core:

- Platform-neutral `DictationEngine` and traits in
  `crates/verboscribe-core/src/lib.rs`.
- Core state tests for press-and-hold, toggle, cancel, short recording,
  permission denial, recorder failures, transcription failure, paste failure,
  duplicate press, and paste-last empty state.
- Transcript processing in `crates/verboscribe-core/src/transcript.rs`:
  raw-first defaults, snippets, personal dictionary prompt hints, spoken
  commands, cleanup levels, and style presets.

Storage:

- `crates/verboscribe-storage/src/lib.rs`: typed `AppSettings`, local JSON
  `JsonSettingsStore`, defaults for provider/language/mode/hotkey, and
  conversion to core `DictationConfig`.

Transcription:

- `WhisperCppTranscriber` in `crates/verboscribe-transcription/src/lib.rs`.
- Validates binary/model/audio paths and Unix executable bit.
- Builds `whisper-cli` argument vectors without shell strings.
- Uses injectable `CommandRunner` for tests.
- Reads trimmed `<output>.txt` transcript.
- Handles non-zero exit and missing output file.
- Local provider smoke example:
  `crates/verboscribe-transcription/examples/whisper_cpp_smoke.rs`.

Audio:

- WAV utilities in `crates/verboscribe-audio/src/lib.rs`.
- `CpalAudioRecorder` in `crates/verboscribe-audio/src/lib.rs`.
- Thread-backed recording controller keeps the live CPAL stream out of shared
  app-service state while recording is active.
- Writes mono 16 kHz 16-bit PCM WAV through Hound.
- Captures live microphone input through CPAL.
- Validates transcription-ready WAV files.
- Rejects missing, stereo, wrong sample rate, and float WAVs.
- Converts/clamps f32 samples to i16.
- Downmixes multi-channel input and linearly resamples into the provider WAV
  contract.
- Trims obvious dead air around detected speech and boosts low-volume captures
  before writing the WAV fed to `whisper.cpp`.

Platform:

- `crates/verboscribe-platform/src/lib.rs`: platform boundary for target
  capture and clipboard-first paste insertion.
- `DesktopTargetTracker` captures the active app before recording and remembers
  the last non-VerboScribe target.
- `DesktopTextInserter` writes transcript text to the clipboard before
  activation or paste automation.
- Platform command execution is timeout-guarded so permission prompts or hung OS
  commands do not stall the dictation flow.
- First-pass Windows adapter design is documented in `docs/SPIKES.md` and
  implemented through PowerShell plus Win32 interop command planning.

Tauri boundary:

- `apps/desktop/src-tauri/src/app_service.rs`: Tauri-free app service with typed
  DTOs, settings load/save, runtime status/recovery events, hotkey status, a
  dry-run dictation flow, and a real live-capture dictation runtime that now
  attempts target reactivation and paste insertion.
- `apps/desktop/src-tauri/src/commands.rs`: Tauri command adapters, including
  explicit start, stop, and cancel dictation commands.
- `apps/desktop/src-tauri/src/hotkeys.rs`: Tauri global-shortcut plugin setup,
  registration, unregister, settings-shortcut normalization, and forwarding of
  pressed or released events into the real app-service dictation flow.
- `apps/desktop/src-tauri/src/lib.rs`: wires managed `AppService` and command
  handlers.

Docs/process:

- `AGENTS.md`: durable Codex project instructions.
- `docs/AGILE_OPERATING_MODEL.md`: agile operating model.
- `docs/EPICS.md`: epics.
- `docs/BACKLOG.md`: product backlog.
- `docs/SPRINTS.md`: sprint plans/reviews/retros.
- `docs/DECISIONS.md`: architecture decisions.
- `docs/SPIKES.md`: spike decisions.
- `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/MANUAL_QA.md`,
  `docs/SETUP_AND_OPERATIONS.md`, `docs/FEATURE_LIST.md`.
- `.github/workflows/ci.yml`: GitHub Actions baseline CI on macOS and Linux for
  format, tests, frontend build, and repo verification.
- `docs/PLATFORM_SMOKE.md`: automated local fixture smoke and pending
  macOS/Windows manual smoke checklist.
- `scripts/smoke-app-service.sh`: focused app-service smoke path using injected
  adapters instead of live desktop dependencies.
- `scripts/smoke-local-fixtures.sh`: local WAV validation plus `whisper.cpp`
  provider fixture smoke.

## Latest Verification

2026-07-03 (paste-last recovery branch):

- RED: `cargo test -p verboscribe2-desktop
  paste_failure_event_preserves_transcript_for_manual_recovery -- --nocapture`
  failed as expected because paste-failure recovery text did not yet mention
  `Paste last transcript`.
- GREEN: the same targeted test passed after updating paste-failure recovery to
  mention `Paste last transcript`.
- Regression checks passed:
  - `cargo test -p verboscribe2-desktop
    paste_last_transcript_retries_a_preserved_transcript -- --nocapture`
  - `cargo fmt --all -- --check`
  - `./scripts/verify.sh` (79 Rust tests across the workspace: core 21,
    storage 7, audio 16, platform 8, transcription 7, desktop 28; plus the
    npm desktop build)
  - `./scripts/smoke-app-service.sh`
- Docs were updated to treat the desktop paste-last action as shipped while
  keeping retry-last-failed-transcript and paste-last hotkey work in the queue.

2026-06-06 (toggle-hotkey branch):

- `cargo fmt --all -- --check` passed.
- `./scripts/verify.sh` passed: 65 Rust tests across the workspace (core 21,
  storage 7, audio 16, platform 8, transcription 7, desktop 26) plus the npm
  desktop build. New tests: `verboscribe-core` forced-mode toggle on a
  press-and-hold engine; `verboscribe-storage` toggle-hotkey default + legacy
  migration; `verboscribe2-desktop` toggle-hotkey tap cycle and independent
  registration/recovery reporting.
- `apps/desktop` `tsc --noEmit` passed.
- `./script/build_and_run.sh run` rebuilt and launched the bundled app; the
  user confirmed it runs with the changes. The earlier "did not work" report
  was a stale pre-change bundle — source edits + tests do NOT update the
  installed `.app`; a `tauri build` / `build_and_run.sh` is required.
- Pre-existing clippy note in `hotkeys.rs install` (`&app.handle()`) is
  unrelated to this change and left as-is.

Prior verification (2026-05-24):

- `cargo fmt --all -- --check` passed.
- `./scripts/verify.sh` passed (including the npm desktop build).
- `cargo check -p verboscribe2-desktop` passed after removing the orphan
  `macos_status_item.rs` module.
- `./script/build_and_run.sh --verify` was exercised multiple times during
  the notch diagnosis and final Tauri-tray revert; the resulting bundle
  launches cleanly, Accessibility confirms a `AXMenuBarItem` /
  `AXMenuExtra` slot, and the menu opens with the expected actions on click.
- Earlier Sprint 9 QA also passed `./script/build_and_run.sh --verify` and
  produced a macOS app bundle with `Contents/Resources/icon.icns`.

## Files To Read First

For the next development slice, read these first:

1. `HANDOFF.md`
2. `docs/SPRINTS.md`
3. `docs/BACKLOG.md`
4. `docs/FEATURE_LIST.md`
5. `docs/ROADMAP.md`
6. `docs/EPICS.md`
7. `apps/desktop/src/main.ts`
8. `apps/desktop/src/styles.css`
9. `apps/desktop/src-tauri/src/app_service.rs`
10. `crates/verboscribe-storage/src/lib.rs`
11. `crates/verboscribe-platform/src/lib.rs`
12. `apps/desktop/src-tauri/src/hotkeys.rs`
13. `crates/verboscribe-core/src/lib.rs`
14. `docs/PLATFORM_SMOKE.md`
15. `docs/MANUAL_QA.md`
16. `docs/SPIKES.md`

## Current Working Behavior

The current vertical slice works like this:

1. The desktop shell starts and loads persisted settings.
2. The Tauri global shortcut plugin registers two hotkeys: the press-and-hold
   dictation hotkey (`Control+Option+Space`, behavior follows the configured
   `DictationSettings::mode`) and a dedicated toggle hotkey
   (`Control+Option+D`, always toggles regardless of mode). `hotkeys.rs`
   resolves which one fired by matching the event against each role's active
   accelerator (`HotkeyRole::{Dictation, Toggle}`) and routes accordingly.
3. Hotkey `Pressed` is forwarded into `AppService` —
   `handle_hotkey_event` for the press-and-hold key, or
   `handle_toggle_hotkey_event` (forces `DictationMode::Toggle` via the engine's
   `hotkey_with_mode`) for the toggle key.
4. `AppService` lazily builds a real desktop `DictationEngine` from saved
   settings.
5. The platform target tracker captures the active app and remembers the last
   non-VerboScribe target.
6. The live CPAL recorder starts capturing microphone input into a mono
   16 kHz WAV-compatible path.
7. Hotkey `Released` stops recording.
8. The local `whisper.cpp` provider transcribes the captured audio using the
   built-in prompt bias plus any persisted `promptContext` or `pinnedTerms`
   overrides from `settings.json`.
9. The platform inserter writes the transcript to the clipboard before any
   automation attempt.
10. The target app is reactivated and the platform paste shortcut is attempted.
11. The processed transcript is retained as `last_transcript` in app-service
   state whether insertion succeeds or fails.
12. Status commands report idle, recording, transcribing, success, or recovery
   failure state.
13. The desktop UI renders editable provider paths, language, dictation mode,
   the press-and-hold hotkey, the toggle hotkey, prompt context, and pinned
   terms beside the live status surface. The status panel shows independent
   registration state for both hotkeys; a failure on either surfaces as a
   "Hotkey unavailable" health warning.
14. Form edits remain drafts until `Save settings to apply` succeeds.
15. Saving settings persists through the existing store and re-applies the
   dictation hotkey immediately.
16. Manual start, stop, cancel, and paste-last buttons drive the same
    app-service runtime loop as the hotkey path.
17. On macOS, the desktop app uses Tauri's built-in tray-icon API (the same
    one used on Windows and Linux) with `ActivationPolicy::Regular` so the
    Dock icon stays visible. The tray icon renders the full-color VS2 mark
    (`icon_as_template(false)`) and exposes `Show VerboScribe 2` and
    `Quit VerboScribe 2` menu entries. macOS Tahoe notch placement can hide
    the icon on the leftmost per-app extras slot; see the implementation
    notes above for the user-side workaround.

Current endpoint of the slice:

- First-pass end-to-end paste insertion now exists through the platform crate.
- Clipboard-first fallback keeps transcript text available if automation fails.
- A deterministic app-service smoke path now covers success and paste-failure
  flows without live microphone, hotkey, or desktop automation dependencies.
- On this macOS machine, the packaged app can complete the real default
  hotkey-to-paste path into a text editor when the shortcut is held while
  speaking.
- On this macOS machine (MacBook Air M3, 1470-wide logical screen, notch at
  `x = 646..825`), a freshly rebuilt bundled app shows up in the Dock and
  exposes a working menu-bar extra. The extra is positioned in the leftmost
  per-app slot, which on this hardware can fall behind the notch depending
  on which other menu-bar apps are running. The user-side mitigation is
  Cmd-drag the icon out from behind the notch, or quit a competing menu-bar
  app so the leftmost slot lands past the notch on next launch.
- The direct `live_dictation_probe` path can also record, transcribe, reactivate
  TextEdit, paste recognized text, and preserve the clipboard.
- Manual QA is still required before treating macOS or Windows insertion as
  reliable.

## Current Product Truth

This is the most important reality check for the next session:

- The backend vertical slice is still further along than the desktop UI, but
  the gap is smaller now that core `whisper.cpp` settings are exposed in-app.
- The current Tauri UI is no longer only a thin status shell.
- The prototype in `~/projects/whisper` had a much larger settings and
  operations surface.
- Product-scope truth is now centralized in `docs/FEATURE_LIST.md`; do not try
  to reconstruct it only from backlog fragments.

Current desktop UI actually exposes:

- app title
- high-level state badge
- editable `whisper.cpp` binary and model paths
- editable language, dictation mode, and dictation hotkey
- editable prompt context and pinned terms
- provider, mode, hotkey, and recovery panels
- manual start, stop, and cancel buttons
- usage hint
- last transcript
- explicit draft-vs-saved semantics through the unsaved warning and
  `Save settings to apply` button label

Current desktop UI does not yet expose most prototype controls such as:

- cleanup/style/snippets/dictionary controls
- preview/edit-before-insert
- retry-last-failed-transcript and preview/edit-before-insert actions
- transcript history or usage insights
- import/export
- launch-at-login
- model install/refresh
- cancel and paste-last hotkey controls

## Manual Setup For Live Dictation

To exercise the current live dictation path on this machine:

1. Ensure the local `whisper.cpp` binary and model still exist at the paths
   listed in `Current Environment Notes`, or set the matching environment
   overrides for smoke scripts.
2. Save valid `whisper.cpp` binary and model paths into app settings through the
   existing settings flow or settings JSON before trying a real hotkey-driven
   dictation run.
   Optional quality tuning can also be added in the same file through
   `transcription.whisperCpp.promptContext` and
   `transcription.whisperCpp.pinnedTerms`.
   Important: desktop form edits do not affect the live runtime until
   `Save settings to apply` succeeds.
3. Grant microphone permission to the app when macOS or Windows prompts for it.
4. On macOS, also grant Accessibility permission to `VerboScribe 2` if paste
   automation is blocked.
5. Use the configured global hotkey to start and stop recording while another
   text-capable app has focus.

Expected outcome with valid configuration:

- Press starts recording.
- Release stops recording.
- Status advances through recording and transcribing.
- The previous target app is reactivated and receives the paste attempt.
- The last transcript becomes visible in the app status surface.

Expected outcome with missing provider configuration:

- Dictation does not proceed.
- Runtime status reports actionable recovery text for the missing binary or
  model path.

Expected outcome with paste automation failure:

- Transcription still completes.
- Recovery status reports the paste failure.
- The transcript remains on the clipboard for manual paste.

## Implementation Constraints

- `verboscribe-core` must remain platform-neutral.
- Product logic should stay out of Tauri command handlers.
- The live CPAL stream must not be stored directly in shared app-service state.
  `CpalAudioRecorder` uses a thread-backed recording controller because the
  live stream ownership is not a clean shared-state fit on macOS.
- Platform-specific target tracking and paste logic belongs in
  `verboscribe-platform`, not in `verboscribe-core` or Tauri commands.
- Hotkey normalization for user-facing settings strings is separate from plugin
  accelerator syntax.
- Platform automation commands must remain timeout-guarded so permission or
  accessibility failures surface as typed recovery instead of hanging the
  dictation flow.
- `TargetApp.identifier` now carries platform-specific target references such as
  macOS bundle identifiers or Windows window handles. Preserve that boundary if
  new target-aware features are added.
- Do not assume desktop hotkey, target-reactivation, or clipboard-paste
  patterns transfer directly to Android or iPhone.
- Treat iPhone live dictation as a companion-app flow unless future discovery
  proves a better supported path. Do not plan around a microphone-enabled
  iPhone keyboard extension.

## Implemented vs Remaining

Implemented end-to-end:

- persisted settings load/save path
- desktop settings surface for the current local-first stack
- local `whisper.cpp` transcription provider
- live microphone capture adapter
- global dictation hotkey registration and press/release handling
- app-service runtime status and recovery reporting
- app-service smoke path for success and paste-failure dictation flows
- platform target tracking
- clipboard-first paste insertion
- real hotkey-to-paste app-service flow
- manual start, stop, and cancel controls in the desktop shell
- macOS happy-path validation that the default shortcut can drive a full paste
  into a target editor on this machine

Implemented in backend but only partially surfaced in the desktop UI:

- minimum recording duration persistence
- transcript-processing capability in core for snippets, cleanup, spoken
  commands, and style presets

Implemented but not manually verified on real desktops:

- new audio preprocessing quality tweak on the live macOS path
- global hotkey registration conflict behavior
- full desktop settings persistence sweep across every surfaced field in a
  packaged app build
- Windows target activation and paste behavior
- Windows end-to-end live recording and insertion flow

Not implemented:

- cancel hotkey
- paste-last hotkey
- paste-raw action
- retry-last-failed-transcript action
- preview/edit-before-insert flow
- cleanup/style/snippets/dictionary UI
- Groq provider support and secret storage
- transcript history store and UI
- usage insights store and UI
- settings import/export
- launch-at-login
- open-minimized preference
- model catalog/install/refresh UI
- Linux clipboard and paste automation
- Windows CI
- packaging, signing, installers, and release distribution
- Android companion app and IME
- iPhone companion app
- iPhone keyboard insertion experiment
- mobile transcription/provider strategy

Planned validation/hardening work:

- `VS2-018: Windows Paste Validation And Hardening`
- `SPIKE-003: Mobile Product Shape And Platform Constraints`
- `SPIKE-004: Mobile Transcription Strategy`

## Important Decisions

- Tauri 2 + Rust core + TypeScript UI.
- Keep `verboscribe-core` platform-neutral.
- Use CPAL + Hound for recording path, but Sprint 2 only implemented WAV
  utilities, not live microphone capture.
- Do not put product logic or platform adapters directly in Tauri command
  handlers.
- Use injectable runners/adapters for process and platform boundaries so tests
  do not require OS permissions or local binaries.
- Mobile product stance: Android pursues IME plus companion app; iPhone pursues
  companion-app-first. Do not assume desktop dictation mechanics transfer to
  phones.

## Current Environment Notes

This machine currently has:

- `rustc 1.95.0` and `cargo 1.95.0` installed through Homebrew.
- Node `v25.9.0`, npm `11.12.1`.
- Local `whisper.cpp` binary:
  `~/Developer/whisper.cpp/build/bin/whisper-cli`
- Local model:
  `~/Developer/whisper.cpp/models/ggml-base.en.bin`
- Local sample:
  `~/Developer/whisper.cpp/samples/jfk.wav`
- Default input device:
  `MacBook Air Microphone`

The smoke script uses environment overrides if paths differ:

- `VERBOSCRIBE_WHISPER_CPP_BIN`
- `VERBOSCRIBE_WHISPER_CPP_MODEL`
- `VERBOSCRIBE_WHISPER_CPP_SAMPLE`

## Known Gaps And Risks

- `dist/`, packaging, signing, and installer workflows are not built yet.
- Physical live dictation quality on the built-in MacBook Air microphone is
  currently weak on short phrases even though the end-to-end path works.
- The desktop UI is still far behind the prototype even after the settings
  foundation story; transcript actions, text-processing controls, model
  operations, history, insights, and launch behavior are still missing.
- The current `VerboScribe 2` icon concept is usable as a placeholder but still
  needs multiple alternate directions before it should be treated as final.
- Windows live recording, hotkeys, and paste insertion still need manual OS QA.
- macOS paste automation may require Accessibility permission and should be
  expected to fail clearly when it is denied.
- The first Windows adapter is implemented but unverified and may need to move
  to direct Rust Win32 calls if PowerShell or `SendKeys` prove unreliable.
- Linux clipboard and paste automation are not implemented.
- Windows CI is still deferred.
- Mobile architecture is only planned, not spiked or implemented.
- The current `whisper.cpp` shell-out path may not be the right mobile provider
  contract because install size, battery, and background limits differ on
  phones.
- iPhone custom keyboard constraints make it unsafe to promise systemwide live
  dictation parity with Android or desktop.

## Next Recommended Work

Recommended focus after the current macOS QA slice:

1. Run packaged-app manual QA on the new settings surface and confirm that
   saved values persist after relaunch and that changed hotkeys re-register
   cleanly.
   Double-check every surfaced setting, not only toggle mode.
2. Rerun one live macOS QA pass against the new audio preprocessing tweak and
   record whether dead-air trimming plus low-volume normalization meaningfully
   improves short-phrase recognition on the built-in microphone.
3. Run `VS2-018: Windows Paste Validation And Hardening` and either validate
   the current adapter or replace the weak point with a direct Rust Win32
   implementation.
4. If product-surface work stays ahead of OS QA, carve the next desktop story
   around transcript recovery actions such as paste-last, retry, or
   preview-before-insert instead of broadening the settings form further.
5. Add a Windows CI job once the desktop/audio/platform path is stable enough
   that the extra platform gate improves signal more than it adds maintenance
   cost.
6. After the desktop smoke and QA path is stable enough, run
   `SPIKE-003: Mobile Product Shape And Platform Constraints`.
7. Then run `SPIKE-004: Mobile Transcription Strategy` before creating mobile
   implementation branches.
8. Return to `VS2-023: VerboScribe 2 Icon Exploration` when branding polish is
   back in scope.

Exact next story candidate:

- `VS2-018: Windows Paste Validation And Hardening`
- if staying on desktop product surface instead of OS QA:
- carve the next story around transcript recovery actions after a quick Sprint
  10 manual QA pass

Exact next mobile discovery candidate once desktop QA is stable:

- `SPIKE-003: Mobile Product Shape And Platform Constraints`

Recommended follow-up if Windows QA is weak:

- `RA-012`: evaluate direct Rust Win32 input and activation instead of the
  PowerShell path.

## User/Manual QA Needed

Manual recording QA is now needed for:

- live microphone capture
- global hotkey registration
- the toggle hotkey (`Control+Option+D`): tap once to start, tap again to
  stop, and confirm it works independently of the press-and-hold
  `Control+Option+Space` key and the configured dictation mode (covered by
  unit tests; not yet exercised end-to-end with a real keypress)
- desktop settings save semantics, persistence, and hotkey re-registration
- manual start, stop, and cancel controls in the packaged app
- text insertion and clipboard fallback
- final user-facing judgment on the current full-color tray icon (the
  `verboscribe2-mark-concept-v1-32.png` rounded-square mark) versus a
  tighter, transparent-background variant designed specifically for the
  menu bar; the rounded square has built-in padding that makes the visible
  ribbon mark small inside its slot
- launch-at-login

Planned QA and hardening stories:

- `VS2-018: Windows Paste Validation And Hardening`

## Handoff Maintenance Checklist

Before ending any future session:

- Update current sprint status in `docs/SPRINTS.md`.
- Update backlog item statuses in `docs/BACKLOG.md`.
- Record verification results here.
- Record any blockers, risks, and next actions here.
- If a sprint ended, include review, retro, and retro actions.
- Record the branch name and merge status if the work happened off `main`.
- Make the handoff usable by a different AI model with no chat history.
- Include current behavior, next story, key files, constraints, setup steps,
  and implemented-vs-missing status explicitly.
