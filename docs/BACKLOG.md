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

### VS2-013: Clipboard Safety Contract

Type: Story  
Epic: EPIC-04  
User value: Dictated text is never lost during paste attempts.  
Acceptance criteria:

- Transcript remains on clipboard if automated paste fails.
- Clipboard preservation/restore policy is documented.
- Platform inserters return typed paste failures.

Status: Done

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

Status: Done

### VS2-015: App-Service Live Dictation Flow

Type: Story  
Epic: EPIC-04A  
User value: The desktop shell can run a real dictation cycle from hotkey event
to local transcript without waiting for clipboard automation.  
Acceptance criteria:

- App service can start and stop a real dictation run with the live recorder.
- Hotkey pressed and released events drive the app-service dictation flow.
- Missing provider configuration surfaces actionable recovery text.
- Last transcript and active dictation state are visible through existing status
  commands.
- Manual QA impact is documented.

Status: Done

### VS2-016: App-Service Dictation Smoke Path

Type: Story  
Epic: EPIC-08  
User value: Platform-facing dictation behavior can be checked repeatedly
without depending only on manual desktop testing.  
Acceptance criteria:

- Add a focused smoke path for the app-service dictation cycle using injectable
  adapters instead of the live microphone.
- Cover both successful insertion and paste-failure transcript preservation.
- The smoke path runs locally without OS permissions or global hotkey
  registration.
- The smoke path is documented in the platform smoke notes.

Status: Done

### VS2-017: macOS End-To-End Dictation QA

Type: Story  
Epic: EPIC-08  
User value: The live dictation flow is validated on a real macOS desktop
before we rely on it for broader platform work.  
Acceptance criteria:

- Run the full hotkey-to-paste flow on macOS with valid local `whisper.cpp`
  paths.
- Verify microphone permission prompts and denied-permission recovery.
- Verify clipboard fallback when paste automation fails.
- Confirm the tested apps and remaining gaps in the handoff and manual QA
  notes.

Status: Done

### VS2-024: Desktop Settings Surface Foundation

Type: Story
Epic: EPIC-06
User value: Users can actually configure the working dictation stack from the
desktop app instead of editing JSON or relying on a status-only shell.
Acceptance criteria:

- Replace the status-only desktop shell with a real settings surface.
- Expose the existing backend settings for `whisper.cpp` binary path, model
  path, language, dictation mode, dictation hotkey, prompt context, and pinned
  terms.
- Keep current status, recovery, and last-transcript visibility in the same
  desktop surface.
- Saving from the desktop UI persists through the existing settings store.
- Manual QA notes call out which prototype controls are still intentionally
  missing after this foundation slice.

Status: Done

### VS2-025: Dedicated Paste-Last Hotkey

Type: Story
Epic: EPIC-04A
User value: Users can retry inserting the preserved last transcript without
recording again or manually clicking back into the app.
Acceptance criteria:

- Persist a dedicated paste-last hotkey with a documented default.
- Route the dedicated hotkey to `paste_last_transcript` without double-firing on
  key release.
- Surface the paste-last hotkey in desktop settings and status UI.
- Update QA and handoff docs to reflect the shipped shortcut and remaining
  recovery gaps.

Status: Done

### VS2-026: Dedicated Cancel Hotkey

Type: Story
Epic: EPIC-04A
User value: Users can abort an in-progress dictation from the keyboard without
triggering transcription, paste, or a duplicate cancel on key release.
Acceptance criteria:

- Persist a dedicated cancel hotkey with a documented default.
- Route the dedicated hotkey to `cancel_dictation` only on key press.
- Surface the cancel hotkey in desktop settings and status UI.
- Update verification and handoff docs to reflect the shipped shortcut.

Status: Done

### VS2-027: Dedicated Retry-Failed Hotkey

Type: Story
Epic: EPIC-04A
User value: Users can retry the preserved failed audio from the keyboard after a
transcription error without starting a new recording or double-triggering on key
release.
Acceptance criteria:

- Persist a dedicated retry-failed hotkey with a documented default.
- Route the dedicated hotkey to `retry_last_failed_transcript` only on key
  press.
- Surface the retry-failed hotkey in desktop settings and status UI.
- Update QA and handoff docs to reflect the shipped shortcut and remaining
  recovery gaps.

Status: Done

## Ready

### VS2-018: Windows Paste Validation And Hardening

Type: Story  
Epic: EPIC-08  
User value: The Windows target-activation and paste path is either validated or
reworked before we depend on it for the vertical slice.  
Acceptance criteria:

- Run the full hotkey-to-paste flow on Windows with valid local `whisper.cpp`
  paths.
- Verify target activation and paste behavior in the standard smoke targets.
- If the first-pass Windows adapter proves unreliable, replace the weak point
  with a direct Rust Win32 implementation or equivalent documented fix.
- Record the tested path, any replacements, and remaining gaps in the handoff
  and manual QA notes.

Status: Ready

### SPIKE-003: Mobile Product Shape And Platform Constraints

Question: What is the smallest phone product that is genuinely useful on both
Android and iPhone without assuming desktop OS automation features?

Output:

- Decision note with Android and iPhone user flows, explicit non-goals, and
  recommended implementation order.
- Explicit confirmation that Android pursues IME plus companion app while
  iPhone pursues companion-app-first because iOS custom keyboards are not the
  primary live-dictation path.
- Shared-core reuse plan plus the native mobile surfaces that still need
  Kotlin or Swift implementations.

Status: Ready

### SPIKE-004: Mobile Transcription Strategy

Question: Which mobile transcription path best fits VerboScribe: bundled local
model, cloud provider, or hybrid fallback?

Output:

- Decision note covering install size, latency, battery and thermal impact,
  offline behavior, privacy, and secret-storage implications on Android and
  iPhone.
- Recommended first mobile provider path plus a fallback plan.

Status: Ready

## Backlog

### VS2-019: Android IME Foundation

Type: Story
Epic: EPIC-09
User value: Android users can invoke VerboScribe from the system keyboard
across apps.
Acceptance criteria:

- Android IME service target exists and builds.
- The IME can insert provided text into standard text fields.
- The companion-app and IME state-sharing approach is documented and tested at
  a minimal integration level.
- Android enablement and permission onboarding are documented.

Status: Backlog

### VS2-020: Android Dictation Flow

Type: Story
Epic: EPIC-09
User value: Android users can record speech and insert transcript text into the
current field from a VerboScribe keyboard flow.
Acceptance criteria:

- The Android companion and IME flow can start recording, stop, transcribe, and
  commit text into the active editor.
- Permission and failure recovery text are explicit.
- Manual QA covers at least one browser text field, one messages-style field,
  and one notes-style field.

Status: Backlog

### VS2-021: iPhone Companion Dictation Flow

Type: Story
Epic: EPIC-09
User value: iPhone users can record speech, transcribe it, and quickly return
text to the app they were using even though iOS does not support the desktop
dictation model.
Acceptance criteria:

- The iPhone app can record speech and produce transcript text through an
  in-app flow.
- The first supported share, copy, or return-to-caller flow is defined and
  tested.
- Permission and failure recovery are documented.
- The design does not assume desktop-style target reactivation or global
  hotkeys.

Status: Backlog

### VS2-022: iPhone Keyboard Insertion Experiment

Type: Story
Epic: EPIC-09
User value: iPhone users may optionally insert previously generated text from a
keyboard surface where iOS allows custom keyboards.
Acceptance criteria:

- The keyboard extension can read shared prepared text from the containing app
  and insert it where custom keyboards are allowed.
- Secure-field, phone-pad, host-app opt-out, and microphone limitations are
  documented in manual QA notes.
- The extension does not attempt live microphone dictation.

Status: Backlog

### VS2-023: VerboScribe 2 Icon Exploration

Type: Story
Epic: EPIC-07
User value: VerboScribe 2 should ship with a distinctive visual identity that
does not read as a recycled mark from the older VerboScribe app.
Acceptance criteria:

- Produce multiple alternate icon or graphic directions for `VerboScribe 2`,
  not just one concept refinement.
- Each direction is visibly distinct from the older VerboScribe app icon and
  avoids looking like a generic microphone or waveform badge.
- Review the options at app-icon sizes, not only as large artwork.
- Document the preferred direction and any follow-up production work needed for
  final bundle assets.

Status: Backlog

- Model downloader/manager.
- History browser/search.
- Rich snippet manager.
- Rich personal dictionary manager.
- Diagnostics bundle that excludes transcript text by default.
- Updater.

## Spikes

### SPIKE-002: Windows Paste And Target Tracking

Question: Which Windows APIs should be used for foreground window tracking and
paste automation?

Output:

- Short decision note with APIs, permissions, and failure modes.

Status: Done
