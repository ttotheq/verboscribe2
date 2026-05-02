# VerboScribe 2

VerboScribe 2 is a new cross-platform desktop dictation app for macOS and
Windows. It is inspired by the existing macOS-only VerboScribe prototype in
`~/projects/whisper`, but this repository is a fresh implementation with
platform integrations isolated behind explicit interfaces.

## Product Direction

The first milestone is a vertical slice:

1. Capture a global dictation hotkey.
2. Record microphone audio to a WAV file.
3. Transcribe with local `whisper.cpp`.
4. Paste the result into the previously active app.

The long-term product keeps the prototype's core ideas: local-first dictation,
optional Groq Whisper, clipboard paste insertion, tray/menu-bar operation,
privacy-first transcript history, snippets, personal dictionary, settings
backup, usage insights, and practical manual QA.

## Proposed Stack

- Tauri 2 desktop shell.
- Rust core and native integration layer.
- TypeScript UI, initially Svelte + Vite.
- Local `whisper.cpp` through a provider abstraction.
- Optional Groq Whisper through the same provider abstraction.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and
[docs/ROADMAP.md](docs/ROADMAP.md).

## Current Status

Sprint 1 is complete. The repository has the planned documentation, a Rust
workspace, a Tauri desktop shell, a tested platform-neutral dictation engine,
ported transcript processing behavior, and a minimal status UI.

Verification:

```sh
./scripts/verify.sh
```
