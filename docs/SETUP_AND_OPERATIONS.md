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

GitHub CI runs these same baseline checks on `macos-latest` and
`ubuntu-latest`:

```sh
cargo fmt --all -- --check
cargo test --workspace
npm --workspace apps/desktop run build
./scripts/verify.sh
```

Windows CI is intentionally deferred for now. Add it when the desktop and audio
adapter path is stable enough that a third platform gate will improve signal
more than it adds maintenance cost.

Local `whisper.cpp` provider smoke test, when the local binary/model/sample are
installed:

```sh
./scripts/smoke-whisper-cpp.sh
```

Local fixture smoke harness:

```sh
./scripts/smoke-local-fixtures.sh
```

See [PLATFORM_SMOKE.md](PLATFORM_SMOKE.md) for automated and manual platform
smoke coverage.

## Local Whisper

The first local provider will shell out to `whisper.cpp`. Expected settings:

- Binary path.
- Model path.
- Language code.
- Prompt/context.

The provider should support both macOS and Windows paths and return clear errors
for missing or non-executable binaries and missing models.

## Local Settings

The app stores non-secret settings in a local JSON file:

- macOS: `~/Library/Application Support/VerboScribe 2/settings.json`
- Windows: `%APPDATA%\VerboScribe 2\settings.json`
- Linux/dev fallback: `$XDG_CONFIG_HOME/VerboScribe 2/settings.json` or
  `~/.config/VerboScribe 2/settings.json`

Current defaults:

- Provider: `whisper.cpp`
- Language: `en`
- Dictation mode: press and hold
- Minimum recording length: `1000` ms
- Dictation hotkey: `Control+Option+Space`
- `whisper.cpp` binary path: unset
- `whisper.cpp` model path: unset

## Secrets

Do not store cloud API keys in plain settings files.

- macOS: Keychain.
- Windows: Credential Manager or DPAPI-backed storage.

Settings backup/export must exclude secrets.
