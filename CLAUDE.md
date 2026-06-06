# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

VerboScribe 2 is a cross-platform (macOS + Windows) desktop dictation app: capture a global
hotkey, record mic audio to WAV, transcribe locally with `whisper.cpp` (optional Groq Whisper via
the same provider abstraction), then paste the result into the previously active app. It is a fresh
Tauri 2 + Rust rewrite of the macOS-only Swift prototype at `~/projects/whisper` — port behavior
conceptually, do not copy AppKit/Carbon/AVFoundation code.

## Read first (cold-start context lives in docs, not code)

`HANDOFF.md` is the source of truth for current status, the active sprint, what is implemented vs.
not-yet-verified vs. not implemented, and the next recommended actions. Then `docs/SPRINTS.md`,
`docs/BACKLOG.md`, `docs/ARCHITECTURE.md`, `docs/GIT_WORKFLOW.md`. The handoff must stay bulletproof
for a different model resuming from a clean context — update it before pausing, closing, or after a
sprint (see the Handoff Rule in `AGENTS.md`).

## Commands

```sh
./scripts/verify.sh              # primary gate: cargo test --workspace, then desktop frontend build
cargo fmt --all -- --check       # required before declaring work complete
cargo test -p <crate>            # test a single crate, e.g. -p verboscribe-core
cargo test -p verboscribe2-desktop smoke_dictation_cycle_   # filter to one test by name substring

./scripts/smoke-whisper-cpp.sh   # runs the whisper_cpp_smoke example; needs local binary/model/sample
./scripts/smoke-local-fixtures.sh  # WAV validation + whisper.cpp provider smoke
./scripts/smoke-app-service.sh   # app-service dictation cycle smoke

npm run dev                      # vite dev server for the desktop UI
./script/build_and_run.sh [run|--debug|--logs|--verify]   # macOS: tauri --debug build, codesign, launch the .app bundle
```

Run `./scripts/smoke-whisper-cpp.sh` whenever you touch the local `whisper.cpp` provider and the
local binary/model/sample are present.

## Architecture

Rust workspace (`Cargo.toml`) + an npm workspace for the desktop UI (`package.json`).

**The hard rule:** platform-neutral domain logic lives in `verboscribe-core` and must never import
macOS frameworks, Win32 APIs, or Tauri types. Platform-specific code goes in `verboscribe-platform`
or the Tauri shell. Core talks to the outside world only through traits.

Crates:
- `verboscribe-core` — the `DictationEngine` state machine (Idle → Starting → Recording →
  Transcribing), domain types, and transcript processing (`transcript.rs`: cleanup levels, spoken
  commands, style presets, snippet expansion, personal dictionary). Defines the trait boundaries:
  `AudioRecorder`, `TranscriptionProvider`, `TranscriptProcessor`, `TextInsertionService`,
  `TargetAppTracker`, etc.
- `verboscribe-audio` — `cpal`/`hound` WAV recording, implements the audio trait.
- `verboscribe-transcription` — `whisper.cpp` (and planned Groq) providers behind `TranscriptionProvider`.
- `verboscribe-platform` — macOS/Windows adapters (`DesktopTargetTracker`, `DesktopTextInserter`).
- `verboscribe-storage` — typed settings store (`JsonSettingsStore`) with migration/versioning.

Desktop app (`apps/desktop`):
- `src/main.ts` — TypeScript UI (a settings/status surface, not a clone of the AppKit prototype).
- `src-tauri/src/app_service.rs` — `AppService` wires the core engine to concrete platform
  implementations and exposes `*Dto` serde types to the frontend. This is the integration seam.
- `src-tauri/src/commands.rs` — `#[tauri::command]` handlers (thin wrappers over `AppService`).
- `src-tauri/src/hotkeys.rs` — global shortcut registration via `tauri-plugin-global-shortcut`.
- `src-tauri/src/lib.rs` — Tauri builder, tray/menu-bar setup, command registration.

Data flow: HotkeyService/UI command → `DictationEngine` → TargetAppTracker, AudioRecorder,
TranscriptionProvider, TranscriptProcessor, TextInsertionService, history/insights stores.

## Conventions

- Trunk-based: branch from `main` (`feature/*`, `fix/*`, `spike/*`), keep branches short-lived,
  one story per branch, verify before merge. Don't let two agents edit the same files on one branch.
- macOS-only Tauri APIs must be gated behind `#[cfg(target_os = "macos")]` (see commit bc2a06b).
- Follow the agile operating model in `docs/AGILE_OPERATING_MODEL.md`; record blockers explicitly in
  `HANDOFF.md` / `docs/SPRINTS.md` rather than leaving verification red silently.
