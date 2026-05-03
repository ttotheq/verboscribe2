# Product Backlog

## Done

### VS2-001: Establish Buildable Workspace

Type: Story  
Epic: EPIC-01  
User value: Developers can run one command to verify the repo.  
Acceptance criteria:

- Rust workspace includes all crates.
- Desktop frontend builds.
- `scripts/verify.sh` runs core tests and frontend build.
- Missing prerequisites fail with actionable messages.

Status: Done

### VS2-002: Core Dictation State Machine

Type: Story  
Epic: EPIC-02  
User value: The dictation flow is testable before platform adapters exist.  
Acceptance criteria:

- Core engine supports press-and-hold and toggle events.
- Start, stop, cancel, successful insertion, short recording, transcription
  failure, and paste failure have tests.
- Permission denial, recorder start failure, stop failure, rapid events, and
  duplicate start while busy settle predictably.
- Core crate has no Tauri, macOS, or Windows dependencies.

Status: Done

### VS2-003: Transcript Processor Port

Type: Story  
Epic: EPIC-05  
User value: Dictation text behavior matches the useful prototype defaults.  
Acceptance criteria:

- Raw-first default preserves normalized raw transcript.
- Snippets expand standalone triggers.
- Personal dictionary terms produce provider prompt hints.
- Cleanup/spoken command behavior is covered by unit tests.

Status: Done

### VS2-005: Minimal Desktop Status UI

Type: Story  
Epic: EPIC-01  
User value: Users can see that the app launches and what the current milestone is.  
Acceptance criteria:

- Tauri shell shows app status.
- UI has stable status, provider placeholder, and recovery placeholder areas.
- Frontend build passes.

Status: Done

### SPIKE-001: Rust Audio Library Choice

Question: Which Rust audio capture approach is best for macOS and Windows WAV
recording with level metering?

Options evaluated:

- `cpal` plus WAV writer.
- Platform-specific implementation behind `AudioRecorder`.
- Tauri/plugin-based capture if viable.

Output:

- Decision recorded in [SPIKES.md](SPIKES.md) and [DECISIONS.md](DECISIONS.md).

Status: Done

### VS2-004: Local whisper.cpp Provider

Type: Story  
Epic: EPIC-03  
User value: Users can transcribe locally without cloud services.  
Acceptance criteria:

- Provider validates binary and model paths.
- Provider shells out to `whisper.cpp`.
- Provider reads transcript output and returns text.
- Tests cover command construction and missing-path errors.

Status: Done

### VS2-008: WAV Recording Adapter

Type: Story  
Epic: EPIC-04  
User value: User speech is captured in a format supported by providers.  
Acceptance criteria:

- Record mono 16 kHz WAV.
- Stop returns path and duration.
- Permission or device failures are actionable.

Status: Done

### VS2-006: Tauri Command Boundary

Type: Story  
Epic: EPIC-04A  
User value: UI can call backend commands through a typed boundary.  
Acceptance criteria:

- Add commands for app status and dry-run dictation state.
- Commands delegate to backend services rather than embedding product logic.
- Add command-level smoke tests where practical.

Status: Done

### VS2-011: Runtime Status And Recovery Events

Type: Story  
Epic: EPIC-04A  
User value: Users can understand what the app is doing and how to recover from
permission, recording, transcription, or paste failures.  
Acceptance criteria:

- Core or app-service layer emits status and recovery events.
- UI can render idle, recording, transcribing, success, and failure states.
- Paste failure keeps the transcript available.
- Permission failures include platform-specific next-step text at the adapter
  boundary.

Status: Done

### VS2-012: Platform Smoke Harness

Type: Story  
Epic: EPIC-08  
User value: OS-specific behavior can be checked repeatedly without relying on
memory or ad hoc notes.  
Acceptance criteria:

- Add smoke checklist or scripts for WAV validity, local transcription sample,
  hotkey registration, target tracking, clipboard write, and paste fallback.
- macOS and Windows gaps are tracked separately.

Status: Done

### VS2-010: Minimal Settings Store

Type: Story  
Epic: EPIC-06  
User value: Provider paths and hotkey settings survive restarts.  
Acceptance criteria:

- Typed settings model exists.
- Settings persist locally.
- Defaults are documented and test-covered.

Status: Done

## Backlog

### VS2-013: Clipboard Safety Contract

Type: Story  
Epic: EPIC-04  
User value: Dictated text is never lost during paste attempts.  
Acceptance criteria:

- Transcript remains on clipboard if automated paste fails.
- Clipboard preservation/restore policy is documented.
- Platform inserters return typed paste failures.

Status: Backlog

### VS2-014: Live Microphone Capture Adapter

Type: Story  
Epic: EPIC-04  
User value: User speech can be captured from the default microphone for the
vertical dictation flow.  
Acceptance criteria:

- Start captures microphone input through CPAL or a documented fallback.
- Stop writes a transcription-ready WAV or returns an actionable format error.
- Duplicate start and stop-without-start settle predictably.
- Permission/device failures map to typed errors with recovery text.
- Manual QA impact is documented.

Status: Done

### VS2-007: Global Hotkey Adapter

Type: Story  
Epic: EPIC-04  
User value: Dictation can start while another app has focus.  
Acceptance criteria:

- Register/unregister dictation hotkey through Tauri/Rust.
- Capture pressed and released states.
- Show registration failure in UI.

Status: Done

### VS2-009: Clipboard Paste Insertion

Type: Story  
Epic: EPIC-04  
User value: Transcripts appear in the app where the user was typing.  
Acceptance criteria:

- Track target app before recording.
- Copy transcript to clipboard.
- Send platform paste shortcut.
- On failure, leave transcript on clipboard and report recovery guidance.

Status: Backlog

## Spikes

### SPIKE-002: Windows Paste And Target Tracking

Question: Which Windows APIs should be used for foreground window tracking and
paste automation?

Output:

- Short decision note with APIs, permissions, and failure modes.

Status: Backlog
