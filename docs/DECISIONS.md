# Architecture Decisions

## ADR-001: Use Tauri 2 With A Rust Core

Status: Accepted  
Date: 2026-05-02

Decision:

Use Tauri 2 for the desktop shell, Rust for core/product/platform logic, and a
TypeScript frontend.

Context:

The app needs native macOS and Windows behavior: hotkeys, tray/menu-bar,
microphone recording, local `whisper.cpp`, clipboard paste insertion,
launch-at-login, settings, secrets, and packaging.

Alternatives considered:

- Electron.
- Flutter Desktop.
- Separate native macOS and Windows apps.

Consequences:

- Privileged logic stays in Rust and can be tested independently.
- The frontend remains small and focused on settings/status.
- Some platform integrations still require native Rust/OS adapters.

## ADR-002: Keep Core Crate Platform-Neutral

Status: Accepted  
Date: 2026-05-02

Decision:

`verboscribe-core` must not depend on Tauri, macOS frameworks, Win32 APIs, or UI
libraries.

Consequences:

- State machine behavior can be tested quickly with mocks.
- macOS and Windows adapters can evolve independently.
- Integration code must translate platform events into core traits and domain
types.

## ADR-003: Use CPAL And Hound For Initial Recording

Status: Accepted  
Date: 2026-05-02

Decision:

Use CPAL for microphone input and Hound for WAV writing in
`verboscribe-audio`.

Context:

The vertical slice needs local microphone capture on macOS and Windows, a valid
WAV file for `whisper.cpp`, duration reporting, and input-level warnings.

Consequences:

- Audio capture remains in Rust and behind the core `AudioRecorder` trait.
- The first implementation can be cross-platform before adding fallback
  platform-specific adapters.
- We need to handle device-native formats and add resampling if 16 kHz mono is
  not available directly.
