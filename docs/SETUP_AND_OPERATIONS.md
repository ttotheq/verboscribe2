# Setup And Operations

## Requirements

- macOS or Windows.
- Node.js and npm.
- Rust toolchain with `cargo`.
- Tauri prerequisites for the target platform.
- `whisper.cpp` for local transcription.

Current local note: Rust was installed with Homebrew during Sprint 1. The
workspace verified with `rustc 1.95.0` and `cargo 1.95.0`.

## Expected Commands

After installing Rust:

```sh
cargo test --workspace
```

After the Tauri shell is wired:

```sh
npm install
npm run dev
npm run build
```

Repository verification:

```sh
./scripts/verify.sh
```

Local `whisper.cpp` provider smoke test, when the local binary/model/sample are
installed:

```sh
./scripts/smoke-whisper-cpp.sh
```

## Local Whisper

The first local provider will shell out to `whisper.cpp`. Expected settings:

- Binary path.
- Model path.
- Language code.
- Prompt/context.

The provider should support both macOS and Windows paths and return clear errors
for missing or non-executable binaries and missing models.

## Secrets

Do not store cloud API keys in plain settings files.

- macOS: Keychain.
- Windows: Credential Manager or DPAPI-backed storage.

Settings backup/export must exclude secrets.
