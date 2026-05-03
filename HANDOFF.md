# VerboScribe 2 Handoff

Last updated: 2026-05-02, after Sprint 5 closeout.

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

- `feature/sprint-5-hotkeys` merged into `main`
- `feature/ci-baseline` merged into `main`
- `feature/sprint-4-live-capture` merged into `main`

Completed:

- Sprint 1: Foundation To Testable Core.
- Sprint 2: Local Audio And Transcription Slice.
- Sprint 3: App-Service Integration And Recovery.
- Sprint 4: Live Capture Adapter.
- Sprint 5: Global Hotkey Adapter.

Sprint 3 goal:

Turn the tested provider/audio pieces into an app-service workflow with settings
and status/recovery events, while preparing for real platform adapters.

Sprint 3 completed items:

- VS2-011: Runtime Status And Recovery Events.
- VS2-010: Minimal Settings Store.
- VS2-012: Platform Smoke Harness.

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
- Writes mono 16 kHz 16-bit PCM WAV through Hound.
- Captures live microphone input through CPAL.
- Validates transcription-ready WAV files.
- Rejects missing, stereo, wrong sample rate, and float WAVs.
- Converts/clamps f32 samples to i16.
- Downmixes multi-channel input and linearly resamples into the provider WAV
  contract.

Tauri boundary:

- `apps/desktop/src-tauri/src/app_service.rs`: Tauri-free app service with typed
  DTOs, settings load/save, runtime status/recovery events, hotkey status, and
  a dry-run dictation flow through the core engine.
- `apps/desktop/src-tauri/src/commands.rs`: Tauri command adapters.
- `apps/desktop/src-tauri/src/hotkeys.rs`: Tauri global-shortcut plugin setup,
  registration, unregister, and settings-shortcut normalization.
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

- Clipboard paste insertion and target app tracking are not implemented yet.
- Windows-specific paste/target tracking still needs a spike.
- Tauri app has not been launched or manually QA'd; only builds/tests have run.
- `dist/`, packaging, signing, and installer workflows are not built yet.
- Live recording still needs manual OS QA on macOS and Windows.
- Global hotkey registration still needs manual OS QA on macOS and Windows.
- Hotkeys currently update status state only; they do not yet start or stop the
  live dictation workflow.

## Next Recommended Work

Start Sprint 6. Recommended focus:

1. Choose the next vertical-slice integration story. Recommended first item:
   app-service wiring that connects the live recorder and hotkey events into a
   real dictation flow.
2. Add an app-service smoke path when the next platform adapter lands.
3. Add a Windows CI job once the desktop/audio path is stable enough that the
   extra platform gate improves signal more than it adds maintenance cost.
4. Run manual recording QA on macOS and Windows before relying on live capture
   for further vertical-slice work.

Sprint 3 verification on `feature/vs2-010-settings-store`:

- `git status --short --branch` showed existing documentation edits carried
  forward from `main`.
- `cargo fmt --all -- --check` passed.
- `./scripts/verify.sh` passed.
- `./scripts/smoke-whisper-cpp.sh` passed and produced the expected JFK sample
  transcript.
- After VS2-010, `cargo fmt --all -- --check` passed and
  `./scripts/verify.sh` passed.
- After VS2-011, `cargo fmt --all -- --check` passed and
  `./scripts/verify.sh` passed.
- `./scripts/smoke-local-fixtures.sh` passed.
- Sprint 4 baseline on `feature/sprint-4-live-capture`: `cargo fmt --all -- --check`
  passed and `./scripts/verify.sh` passed.
- After VS2-014, `cargo fmt --all -- --check`, `./scripts/verify.sh`, and
  `./scripts/smoke-local-fixtures.sh` passed.
- On `feature/ci-baseline`, `cargo fmt --all -- --check`,
  `cargo test --workspace`, `npm --workspace apps/desktop run build`, and
  `./scripts/verify.sh` passed.
- After VS2-007, `cargo fmt --all -- --check`,
  `npm --workspace apps/desktop run build`, and `./scripts/verify.sh` passed.

## User/Manual QA Needed

Manual recording QA is now needed for:

- live microphone capture
- global hotkey registration
- text insertion
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
