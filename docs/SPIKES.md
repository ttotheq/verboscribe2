# Spikes

## SPIKE-001: Rust Audio Library Choice

Status: Done  
Date: 2026-05-02

Question:

Which Rust audio capture approach is best for macOS and Windows WAV recording
with level metering?

Decision:

Use **CPAL** for microphone input and **Hound** for WAV writing.

Rationale:

- CPAL is a low-level cross-platform audio I/O library. Its current docs list
  support for default input/output devices, stream formats, and PCM streams, with
  native hosts including macOS CoreAudio and Windows WASAPI.
- Hound directly writes WAVE files, exposes `WavWriter`, and updates/finalizes
  headers so produced recordings can be consumed by `whisper.cpp`.
- This keeps audio capture in Rust and behind the existing `AudioRecorder` core
  trait rather than tying the product flow to a Tauri UI plugin.

Sources:

- https://docs.rs/crate/cpal/latest
- https://docs.rs/hound/latest/hound/struct.WavWriter.html

Implementation shape:

- `verboscribe-audio` owns the cross-platform recorder.
- Use CPAL default input device initially.
- Convert incoming samples to mono 16-bit PCM.
- Write 16 kHz WAV files through Hound.
- Track duration from sample count.
- Track input level from recent sample RMS/peak.
- Return a typed error for missing device, unsupported format, stream build
  failure, stream runtime failure, and file write/finalize failure.

Open implementation detail:

- CPAL may deliver device-native sample rates. The vertical slice can first
  request a 16 kHz mono stream if available, then add resampling if the default
  device does not support it. If device support is inconsistent, add a small
  resampler rather than pushing format complexity into transcription providers.

Rejected for now:

- Tauri microphone plugin: avoid adding a UI/plugin dependency before validating
  the core recording needs.
- Platform-specific AVFoundation/WASAPI first: better reserved for fallback
  adapters if CPAL behavior is insufficient.

## SPIKE-002: Windows Paste And Target Tracking

Status: Done  
Date: 2026-05-02

Question:

Which Windows APIs should be used for foreground window tracking and paste
automation?

Decision:

Use a first-pass **PowerShell + Win32 interop** path:

- capture the foreground window handle with `GetForegroundWindow`
- map it to a process with `GetWindowThreadProcessId`
- reactivate it with `SetForegroundWindow`
- write clipboard text before automation
- trigger paste with `System.Windows.Forms.SendKeys`

Rationale:

- This keeps the first Windows adapter small and isolated behind the existing
  Rust platform boundary.
- Window handle capture gives the insertion path a durable target identity
  without dragging Win32 details into `verboscribe-core`.
- Clipboard-first ordering satisfies the safety requirement that dictated text
  remains available when automation fails.
- Manual QA is still required because foreground activation and `SendKeys`
  behavior can vary across desktop sessions and app types.

Implementation shape:

- `verboscribe-platform` owns the target capture and insertion command planning.
- Target capture returns a `TargetApp` identifier encoded from the saved window
  handle.
- Clipboard write runs before activation or paste.
- Activation and paste failures return typed paste errors with manual recovery
  guidance.

Known limitations:

- This first adapter is intentionally pragmatic rather than final.
- Richer Windows automation may later move to direct Rust Win32 calls if
  PowerShell or `SendKeys` prove unreliable in QA.
