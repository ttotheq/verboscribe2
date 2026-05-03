# VerboScribe 2 Handoff

Last updated: 2026-05-02, after Sprint 7 closeout.

## Resume First

Run these commands from the repo root:

```sh
git status --short
cargo fmt --all -- --check
./scripts/verify.sh
```

If working on the local `whisper.cpp` provider and the local binary/model/sample
are still installed, also run:

```sh
./scripts/smoke-local-fixtures.sh
```

Expected current result:

- `cargo fmt --all -- --check` passes.
- `./scripts/verify.sh` passes.
- `./scripts/smoke-local-fixtures.sh` passes on this machine and includes:
  `And so my fellow Americans, ask not what your country can do for you, ask what you can do for your country.`

## Current Project State

VerboScribe 2 is a fresh cross-platform macOS/Windows desktop dictation app.
The selected stack is:

- Tauri 2 desktop shell.
- Rust workspace for core, audio, transcription, platform, and storage crates.
- TypeScript frontend through Vite.
- Local-first transcription through `whisper.cpp`.
- Optional Groq Whisper later.

The existing macOS-only prototype at `~/projects/whisper` was inspected
read-only. Do not modify it. Product/behavior notes are captured in
`docs/PROTOTYPE_NOTES.md`.

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

- `main`

Recent merge status:

- `feature/sprint-7-clipboard-insertion` merged into `main`
- `feature/sprint-6-live-dictation` merged into `main`
- `feature/sprint-5-hotkeys` merged into `main`
- `feature/ci-baseline` merged into `main`
- `feature/sprint-4-live-capture` merged into `main`

Completed:

- Sprint 1: Foundation To Testable Core.
- Sprint 2: Local Audio And Transcription Slice.
- Sprint 3: App-Service Integration And Recovery.
- Sprint 4: Live Capture Adapter.
- Sprint 5: Global Hotkey Adapter.
- Sprint 6: Live Dictation Service Integration.
- Sprint 7: Clipboard Insertion And Target Tracking.

## Completed Implementation

Foundation:

- Root Rust workspace in `Cargo.toml`.
- Tauri desktop shell under `apps/desktop`.
- Minimal status UI in `apps/desktop/src/main.ts` and
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
  `docs/SETUP_AND_OPERATIONS.md`.
- `.github/workflows/ci.yml`: GitHub Actions baseline CI on macOS and Linux for
  format, tests, frontend build, and repo verification.
- `docs/PLATFORM_SMOKE.md`: automated local fixture smoke and pending
  macOS/Windows manual smoke checklist.
- `scripts/smoke-local-fixtures.sh`: local WAV validation plus `whisper.cpp`
  provider fixture smoke.

## Latest Verification

- `cargo fmt --all -- --check` passed.
- `./scripts/verify.sh` passed.
- `./scripts/smoke-local-fixtures.sh` passed.

## Files To Read First

For the next development slice, read these first:

1. `HANDOFF.md`
2. `docs/SPRINTS.md`
3. `docs/BACKLOG.md`
4. `apps/desktop/src-tauri/src/app_service.rs`
5. `crates/verboscribe-platform/src/lib.rs`
6. `apps/desktop/src-tauri/src/hotkeys.rs`
7. `crates/verboscribe-core/src/lib.rs`
8. `docs/PLATFORM_SMOKE.md`
9. `docs/MANUAL_QA.md`
10. `docs/SPIKES.md`

## Current Working Behavior

The current vertical slice works like this:

1. The desktop shell starts and loads persisted settings.
2. The Tauri global shortcut plugin registers the configured dictation hotkey.
3. Hotkey `Pressed` is forwarded into `AppService`.
4. `AppService` lazily builds a real desktop `DictationEngine` from saved
   settings.
5. The platform target tracker captures the active app and remembers the last
   non-VerboScribe target.
6. The live CPAL recorder starts capturing microphone input into a mono
   16 kHz WAV-compatible path.
7. Hotkey `Released` stops recording.
8. The local `whisper.cpp` provider transcribes the captured audio.
9. The platform inserter writes the transcript to the clipboard before any
   automation attempt.
10. The target app is reactivated and the platform paste shortcut is attempted.
11. The processed transcript is retained as `last_transcript` in app-service
   state whether insertion succeeds or fails.
12. Status commands report idle, recording, transcribing, success, or recovery
   failure state.

Current endpoint of the slice:

- First-pass end-to-end paste insertion now exists through the platform crate.
- Clipboard-first fallback keeps transcript text available if automation fails.
- Manual QA is still required before treating macOS or Windows insertion as
  reliable.

## Manual Setup For Live Dictation

To exercise the current live dictation path on this machine:

1. Ensure the local `whisper.cpp` binary and model still exist at the paths
   listed in `Current Environment Notes`, or set the matching environment
   overrides for smoke scripts.
2. Save valid `whisper.cpp` binary and model paths into app settings through the
   existing settings flow or settings JSON before trying a real hotkey-driven
   dictation run.
3. Grant microphone permission to the app when macOS or Windows prompts for it.
4. On macOS, also grant Accessibility permission if paste automation is blocked
   by `System Events`.
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

## Implemented vs Remaining

Implemented:

- persisted settings
- local `whisper.cpp` transcription provider
- live microphone capture adapter
- global hotkey registration and pressed or released event handling
- app-service runtime status and recovery reporting
- platform target tracking
- clipboard-first paste insertion
- real hotkey-to-paste app-service flow

Implemented but not manually verified on real desktops:

- full hotkey-to-paste live dictation flow
- live microphone permissions and device behavior
- global hotkey registration conflict behavior
- macOS Accessibility-denied paste fallback behavior
- Windows target activation and paste behavior

Not implemented:

- Linux clipboard and paste automation
- Windows CI
- packaging, signing, installers, and release distribution

## Important Decisions

- Tauri 2 + Rust core + TypeScript UI.
- Keep `verboscribe-core` platform-neutral.
- Use CPAL + Hound for recording path, but Sprint 2 only implemented WAV
  utilities, not live microphone capture.
- Do not put product logic or platform adapters directly in Tauri command
  handlers.
- Use injectable runners/adapters for process and platform boundaries so tests
  do not require OS permissions or local binaries.

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

The smoke script uses environment overrides if paths differ:

- `VERBOSCRIBE_WHISPER_CPP_BIN`
- `VERBOSCRIBE_WHISPER_CPP_MODEL`
- `VERBOSCRIBE_WHISPER_CPP_SAMPLE`

## Known Gaps And Risks

- Tauri app has not been launched or manually QA'd; only builds/tests have run.
- `dist/`, packaging, signing, and installer workflows are not built yet.
- Live recording still needs manual OS QA on macOS and Windows.
- Global hotkey registration still needs manual OS QA on macOS and Windows.
- Paste insertion still needs manual OS QA on macOS and Windows.
- macOS paste automation may require Accessibility permission and should be
  expected to fail clearly when it is denied.
- The first Windows adapter is implemented but unverified and may need to move
  to direct Rust Win32 calls if PowerShell or `SendKeys` prove unreliable.
- Linux clipboard and paste automation are not implemented.
- Windows CI is still deferred.

## Next Recommended Work

Start Sprint 8. Recommended focus:

1. Implement `VS2-016: App-Service Dictation Smoke Path` so success and
   paste-failure flows can be checked without the live microphone.
2. Run manual QA on macOS for end-to-end recording, target capture, paste
   insertion, clipboard fallback, microphone permission, and Accessibility
   failure behavior.
3. Run manual QA on Windows for target activation and paste behavior before
   relying on the first adapter design.
4. Add a Windows CI job once the desktop/audio/platform path is stable enough
   that the extra platform gate improves signal more than it adds maintenance
   cost.

Exact next story candidate:

- `VS2-016: App-Service Dictation Smoke Path`

Recommended follow-up if Windows QA is weak:

- `RA-012`: evaluate direct Rust Win32 input and activation instead of the
  PowerShell path.

## User/Manual QA Needed

Manual recording QA is now needed for:

- full hotkey-to-paste flow with valid local `whisper.cpp` paths
- live microphone capture
- global hotkey registration
- text insertion and clipboard fallback
- tray/menu-bar behavior
- launch-at-login
- packaged app permissions

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
