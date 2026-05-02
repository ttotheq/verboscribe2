# VerboScribe 2 Handoff

Last updated: 2026-05-02, after Sprint 2 closeout and handoff documentation.

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
./scripts/smoke-whisper-cpp.sh
```

Expected current result:

- `cargo fmt --all -- --check` passes.
- `./scripts/verify.sh` passes.
- `./scripts/smoke-whisper-cpp.sh` passes on this machine and prints:
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
- Keep `docs/BACKLOG.md`, `docs/SPRINTS.md`, and this handoff current.
- Every sprint must close with full review, retrospective, and retro actions.

Completed:

- Sprint 1: Foundation To Testable Core.
- Sprint 2: Local Audio And Transcription Slice.

Planned next sprint:

- Sprint 3: App-Service Integration And Recovery.

Sprint 3 goal:

Turn the tested provider/audio pieces into an app-service workflow with settings
and status/recovery events, while preparing for real platform adapters.

Sprint 3 candidate items:

- VS2-011: Runtime Status And Recovery Events.
- VS2-010: Minimal Settings Store.
- VS2-012: Platform Smoke Harness.
- VS2-013: Clipboard Safety Contract.

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
- Writes mono 16 kHz 16-bit PCM WAV through Hound.
- Validates transcription-ready WAV files.
- Rejects missing, stereo, wrong sample rate, and float WAVs.
- Converts/clamps f32 samples to i16.

Tauri boundary:

- `apps/desktop/src-tauri/src/app_service.rs`: Tauri-free app service with typed
  DTOs and a dry-run dictation flow through the core engine.
- `apps/desktop/src-tauri/src/commands.rs`: Tauri command adapters.
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

- Live microphone capture is not implemented yet.
- Global hotkeys are not implemented yet.
- Clipboard paste insertion and target app tracking are not implemented yet.
- Settings persistence is not implemented yet.
- Runtime status/recovery event model is not implemented yet.
- Windows-specific paste/target tracking still needs a spike.
- Tauri app has not been launched or manually QA'd; only builds/tests have run.
- `dist/`, packaging, signing, and installer workflows are not built yet.

## Next Recommended Work

Start Sprint 3. Recommended order:

1. Run the resume commands above.
2. Mark Sprint 3 `Status: Active` in `docs/SPRINTS.md`.
3. Implement VS2-010 Minimal Settings Store first, because provider paths and
   language will be needed by app-service workflows.
4. Implement VS2-011 Runtime Status And Recovery Events around the existing
   `AppService`.
5. Implement VS2-012 Platform Smoke Harness by adding scripts/docs for:
   local provider smoke, WAV validation, and later hotkey/paste checks.
6. Keep VS2-014 Live Microphone Capture separate from WAV utilities.
7. Before closing Sprint 3, run the full closeout checklist in
   `docs/AGILE_OPERATING_MODEL.md`.

## User/Manual QA Needed

None immediately.

Manual QA will become necessary when implementing:

- live microphone capture
- global hotkeys
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
