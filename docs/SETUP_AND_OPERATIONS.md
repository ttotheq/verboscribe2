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
- Pinned terms.

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
- `whisper.cpp` prompt-context override: empty
- `whisper.cpp` pinned-terms override: empty

Current note: the desktop shell now exposes these fields directly in the
settings surface. Manual edits in `settings.json` still work for advanced or
offline changes under `transcription.whisperCpp.promptContext` and
`transcription.whisperCpp.pinnedTerms`. Older settings files that omit those
keys still load correctly and default both overrides to empty strings. In the
desktop UI, form edits remain drafts until `Save settings to apply` succeeds;
only the saved values affect the live dictation path and hotkey behavior.

## Paste Automation Notes

The current desktop slice uses platform adapters for target capture and
clipboard-first paste insertion.

- macOS: target capture uses `lsappinfo`; target reactivation uses
  `/usr/bin/open -b <bundle-id>`; paste uses a direct `Cmd+V` key event from
  the app process and requires Accessibility permission for `VerboScribe 2`.
- Windows: target capture and activation use a first-pass PowerShell plus
  Win32 interop path; clipboard text is written before paste automation so the
  transcript remains available if paste fails.
- Linux: clipboard and paste automation are not implemented.

For stable macOS Accessibility trust across rebuilds, prefer a stable local
code-signing identity instead of ad-hoc signing. The local run script now
prefers `VERBOSCRIBE_CODESIGN_IDENTITY` when set, or a known local identity
such as `VerboScribe Local Code Signing` or `Whisper Dictation Local Code Signing`
when available. If the app was previously approved while ad-hoc signed, remove
the old Accessibility entry once and add the newly signed app bundle.

## Secrets

Do not store cloud API keys in plain settings files.

- macOS: Keychain.
- Windows: Credential Manager or DPAPI-backed storage.

Settings backup/export must exclude secrets.
