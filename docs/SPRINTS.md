# Sprints

## Sprint 1: Foundation To Testable Core

Status: Done  
Goal: Make the repo buildable and establish a platform-neutral core that can
drive the dictation vertical slice.

### Committed Items

- VS2-001: Establish Buildable Workspace
- VS2-002: Core Dictation State Machine
- VS2-003: Transcript Processor Port
- VS2-005: Minimal Desktop Status UI
- SPIKE-001: Rust Audio Library Choice

### Stretch Items

- VS2-011: Runtime Status And Recovery Events
- VS2-012: Platform Smoke Harness

### Definition Of Done

- `scripts/verify.sh` passes in an environment with Rust installed.
- Frontend build passes locally.
- Core tests cover state transitions and transcript processing.
- Architecture and backlog docs reflect any changes.

### Current Risks

- `cargo` is not installed or not on `PATH` in the current environment.
- Tauri build cannot be fully validated until Rust is available.
- Cross-platform audio and paste APIs need spike decisions before platform work.

### Execution Notes

- Initial documentation and workspace scaffold are complete.
- Frontend build passes.
- Rust tests are blocked by missing `cargo`.
- AI architecture review recommended hardening all failure transitions before
  adding platform adapters.
- Core state-machine failure cleanup has been implemented for microphone
  permission denial, recorder start failure, recorder stop failure,
  transcription failure, paste failure, and duplicate press while recording.
- Added unit coverage for those paths, pending execution once Rust is available.
- AI backlog review added explicit backlog coverage for runtime orchestration,
  permissions/recovery UX, platform smoke harness, audio validation,
  `whisper.cpp` operations, clipboard safety, and minimal settings.

### Sprint Review

Increment delivered:

- VS2-001 is complete: Rust was installed through Homebrew, the workspace now
  compiles, and `scripts/verify.sh` passes.
- VS2-002 is complete: the core dictation state machine has deterministic
  cleanup on success and failure paths, with unit coverage.
- VS2-003 is complete: transcript processor behavior has been ported for
  raw-first defaults, snippets, personal dictionary prompt hints, spoken
  commands, cleanup, and style presets.
- VS2-005 is complete: the desktop shell has a minimal status UI and frontend
  build passes.
- SPIKE-001 is complete: CPAL + Hound is the accepted initial recording
  approach.
- The next sprint should start VS2-004 local `whisper.cpp` provider and
  VS2-008 WAV recording adapter, then connect them through a Tauri app service.
- Final verification: `./scripts/verify.sh` passed with 20 core tests plus the
  desktop frontend build.

Acceptance criteria passed:

- Workspace verification passes through `scripts/verify.sh`.
- Frontend build passes locally.
- Core tests cover state transitions and transcript processing.
- Architecture, backlog, decisions, spike notes, and sprint logs were updated.

Blocked or deferred:

- Runtime status/recovery events were identified but deferred to a later sprint.
- Platform smoke harness was identified but deferred.
- No OS-level manual QA is required yet because no microphone, hotkey, or paste
  adapter was implemented.

Backlog changes:

- Added EPIC-04A for runtime orchestration and recovery.
- Added EPIC-08 for platform smoke harness.
- Marked VS2-001, VS2-002, VS2-003, VS2-005, and SPIKE-001 done.
- Moved VS2-004, VS2-008, and VS2-006 into Ready for Sprint 2 consideration.

User review:

- Review the product direction and Sprint 2 goal.
- No manual OS testing is requested yet.

### Retrospective

What worked:

- Parallel AI reviews found meaningful backlog and architecture gaps before
  platform implementation began.
- Installing Rust during the sprint removed the main verification blocker.
- Core-first implementation caught state and regex behavior issues while the
  code was still easy to change.
- Keeping platform APIs out of `verboscribe-core` preserved testability.

What slowed us down:

- The initial closeout did not run the full agile retrospective ceremony.
- Missing Rust was discovered only when verification first ran.
- Tauri's icon requirement was not captured in the initial scaffold checklist.
- Swift regex behavior did not map directly to Rust `regex`, requiring a
  boundary-matching rewrite.

What should change next sprint:

- Run the sprint closeout checklist automatically before declaring a sprint
  complete.
- Add prerequisite checks earlier in each sprint.
- Treat framework-required assets/configuration as part of foundation
  acceptance criteria.
- For ports from Swift to Rust, check library semantic differences before
  assuming direct translation.

Improvement actions:

| ID | Action | Owner | Target | Status |
| --- | --- | --- | --- | --- |
| RA-001 | Use the sprint closeout checklist before every final sprint summary. | Lead AI | Sprint 2 closeout | Done |
| RA-002 | Add prerequisite checks at sprint start when new toolchains or OS features are involved. | Lead AI | Sprint 2 planning | Done |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Next scaffold/update | Open |
| RA-004 | Verify language/library behavior differences during ports before broad implementation. | Lead AI | Sprint 2 implementation | Done |

### Historical Notes

- Installing Rust unblocked the repo and should be documented as a hard
  prerequisite for future machines.
- Tauri requires icon assets even for tests; a placeholder icon was added early
  to keep verification green.
- Rust regex does not support look-around, so transcript boundary matching uses
  captured boundary characters instead.

## Sprint 2: Local Audio And Transcription Slice

Status: Done  
Goal: Produce a real local transcript from a recorded or fixture WAV through the
same provider path the app will use in the vertical slice.

### Committed Items

- VS2-004: Local whisper.cpp Provider
- VS2-008: WAV Recording Adapter
- VS2-006: Tauri Command Boundary

### Stretch Items

- VS2-012: Platform Smoke Harness
- VS2-011: Runtime Status And Recovery Events

### Definition Of Done

- `whisper.cpp` provider validates paths and has unit coverage for errors and
  command construction.
- Audio crate can write and validate a mono WAV file.
- Tauri command boundary delegates through an app-service layer.
- Verification remains green.

### Risks And Dependencies

- Local `whisper.cpp` may not be installed on every machine; provider tests
  should avoid requiring the real binary except for explicit smoke tests.
- Live microphone capture may require OS permissions and manual testing.
- Audio format conversion may require resampling if the default input device does
  not support 16 kHz mono directly.
- Tauri command design should not instantiate platform implementations directly
  inside command handlers.

### Retro Actions Carried In

| ID | Action | Owner | Status |
| --- | --- | --- | --- |
| RA-001 | Use the sprint closeout checklist before every final sprint summary. | Lead AI | Open |
| RA-002 | Add prerequisite checks at sprint start when new toolchains or OS features are involved. | Lead AI | Open |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Open |
| RA-004 | Verify language/library behavior differences during ports before broad implementation. | Lead AI | Open |

### Planned Execution Approach

1. Run prerequisite checks for `whisper.cpp`, Rust, Node, and microphone-related
   environment assumptions.
2. Implement `verboscribe-transcription` provider with fake-process-testable
   command construction and path validation.
3. Implement WAV writing/validation utilities in `verboscribe-audio` before live
   microphone capture.
4. Add a Tauri app-service boundary that can be tested without real platform
   adapters.
5. Run `scripts/verify.sh` and update sprint review/retro.

### Execution Notes

- Prerequisite check passed for Rust, Cargo, Node, npm, local `whisper.cpp`
  binary, local base English model, and JFK sample WAV.
- VS2-004 complete: `verboscribe-transcription` now has
  `WhisperCppTranscriber`, path validation, command planning, injectable command
  runner, output-file reading, provider tests, and a local smoke example.
- VS2-008 complete for the Sprint 2 contract: `verboscribe-audio` can write and
  validate mono 16 kHz 16-bit PCM WAV files, reject incompatible WAVs, calculate
  duration from samples, and clamp f32 samples to i16.
- VS2-006 complete: Tauri commands now delegate to a Tauri-free `AppService`
  with typed DTOs and a dry-run dictation flow through the core engine.
- Local provider smoke: `./scripts/smoke-whisper-cpp.sh` produced the JFK sample
  transcript through `WhisperCppTranscriber`.

### Sprint Review

Increment delivered:

- Local `whisper.cpp` transcription provider implemented behind the core
  `TranscriptionProvider` trait.
- WAV writer/validator utilities implemented in `verboscribe-audio`.
- Tauri command boundary split into `AppService` plus command adapters.
- Provider-based smoke script added for local `whisper.cpp`.
- Backlog updated to move VS2-004, VS2-008, and VS2-006 to Done.

Acceptance criteria passed:

- Provider validates binary/model/audio paths and executable bit on Unix.
- Provider command planning is unit-tested.
- Provider success, non-zero exit, and missing-output behavior are unit-tested.
- Audio crate writes valid mono 16 kHz 16-bit WAV files.
- Audio crate rejects missing, stereo, wrong-rate, and float WAV files.
- Tauri app service is tested without real platform adapters.
- Verification remains green.

Blocked or deferred:

- Live microphone capture is deferred. Sprint 2 built the WAV file contract first
  because live CPAL recording introduces OS permission and device-format
  behavior.
- Runtime status/recovery events are still deferred to Sprint 3.
- Full platform smoke harness is not complete; only local `whisper.cpp` provider
  smoke was added.

Backlog changes:

- VS2-004, VS2-008, and VS2-006 moved to Done.
- VS2-011, VS2-012, and VS2-010 moved to Ready for Sprint 3 consideration.

User review:

- No manual OS testing required yet.
- The local `whisper.cpp` provider smoke test can be rerun with
  `./scripts/smoke-whisper-cpp.sh`.

Verification:

- `cargo fmt --all -- --check` passed.
- `./scripts/verify.sh` passed.
- Provider smoke `./scripts/smoke-whisper-cpp.sh` passed.

### Retrospective

What worked:

- Prerequisite checks at sprint start prevented surprises and closed RA-002.
- Read-only AI architecture reviews gave concrete boundaries for the provider,
  audio utilities, and Tauri service layer.
- Injectable command runner made provider behavior testable without depending on
  a real `whisper.cpp` binary.
- Building WAV validation before live recording kept the audio work deterministic
  and fast.

What slowed us down:

- Formatting was checked late, so several files needed a broad `cargo fmt` pass.
- VS2-008's name implies live recording, but the Sprint 2 Definition of Done was
  narrower: write and validate a transcription-ready WAV.
- Provider smoke testing was not originally in the committed stories, but it was
  necessary to prove the sprint goal against a real fixture.

What should change next sprint:

- Run `cargo fmt --all -- --check` before the final verification pass and after
  each substantial Rust patch.
- Rename or split future audio stories so live microphone capture is distinct
  from WAV format utilities.
- Include at least one smoke command for any story that claims an end-to-end
  local fixture path.

Improvement actions:

| ID | Action | Owner | Target | Status |
| --- | --- | --- | --- | --- |
| RA-001 | Use the sprint closeout checklist before every final sprint summary. | Lead AI | Sprint 2 closeout | Done |
| RA-002 | Add prerequisite checks at sprint start when new toolchains or OS features are involved. | Lead AI | Sprint 2 planning | Done |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Next scaffold/update | Open |
| RA-004 | Verify language/library behavior differences during ports before broad implementation. | Lead AI | Sprint 2 implementation | Done |
| RA-005 | Run Rust formatting checks before final verification. | Lead AI | Sprint 3 execution | Open |
| RA-006 | Split live microphone capture from WAV utilities in backlog wording. | Lead AI | Sprint 3 planning | Done |

## Sprint 3: App-Service Integration And Recovery

Status: Done  
Goal: Turn the tested provider/audio pieces into an app-service workflow with
settings and status/recovery events, while preparing for real platform adapters.

### Committed Items

- VS2-010: Minimal Settings Store
- VS2-011: Runtime Status And Recovery Events
- VS2-012: Platform Smoke Harness

### Candidate Items

- VS2-013: Clipboard Safety Contract

### Definition Of Done

- App service exposes status and recovery state for provider/audio failures.
- Minimal settings persist provider paths, language, mode, and hotkey defaults.
- Smoke harness documents/runs local provider and WAV validation checks.
- Verification remains green.

### Sprint Start Notes

- Started on branch `feature/vs2-010-settings-store`.
- Baseline verification passed: `cargo fmt --all -- --check`,
  `./scripts/verify.sh`, and `./scripts/smoke-whisper-cpp.sh`.
- Begin with VS2-010 so provider paths and language settings are available
  before app-service recovery/status wiring.

### Execution Notes

- VS2-010 complete: `verboscribe-storage` now has a typed settings model,
  JSON file-backed store, documented local settings paths/defaults, and tests
  for defaults, persistence, invalid JSON, and conversion to core dictation
  config.
- The Tauri app service now exposes settings load/save commands and reads
  provider/hotkey status from the settings store.
- Verification after VS2-010: `cargo fmt --all -- --check` passed and
  `./scripts/verify.sh` passed.
- VS2-011 complete: the app service now exposes runtime status events, dry-run
  dictation event sequences, and recovery DTOs for permission, recording,
  transcription, paste, and empty-transcript failures.
- Paste recovery events preserve the transcript for manual recovery, and
  microphone permission recovery includes platform-specific next-step text.
- Verification after VS2-011: `cargo fmt --all -- --check` passed and
  `./scripts/verify.sh` passed.
- VS2-012 complete: added `scripts/smoke-local-fixtures.sh` and
  `docs/PLATFORM_SMOKE.md` to run local WAV/provider fixture checks and track
  macOS/Windows manual smoke gaps for hotkeys, target tracking, clipboard, and
  paste fallback.
- Local fixture smoke `./scripts/smoke-local-fixtures.sh` passed.

### Sprint Review

Increment delivered:

- Minimal settings store with typed defaults, JSON persistence, and conversion
  to core dictation config.
- Tauri app-service commands for settings load/save.
- Runtime status and recovery event DTOs around the app service.
- Dry-run dictation event sequence covering idle, recording, transcribing, and
  success states.
- Recovery mapping for permission, recording, transcription, paste, and
  empty-transcript failures.
- Local platform smoke harness and macOS/Windows manual smoke checklist.

Acceptance criteria passed:

- Settings model persists provider paths, language, mode, minimum recording
  length, and hotkey defaults.
- Defaults are documented in setup/operations docs and covered by tests.
- App service exposes status/recovery state for provider/audio/paste-class
  failures.
- Paste failure events preserve transcript text for manual recovery.
- Permission failures include platform-specific next-step text.
- Smoke harness covers local provider and WAV validation checks.
- Verification remains green.

Blocked or deferred:

- Live microphone capture remains deferred to VS2-014.
- Global hotkeys, target tracking, clipboard write, and paste fallback remain
  manual checklist placeholders until platform adapters exist.

Backlog changes:

- VS2-010, VS2-011, and VS2-012 moved to Done.
- VS2-013 remains in Backlog.

User review:

- No manual OS testing is required yet because no live microphone, hotkey,
  target tracking, or paste adapter was implemented.
- The local fixture smoke can be rerun with
  `./scripts/smoke-local-fixtures.sh`.

Verification:

- `cargo fmt --all -- --check` passed.
- `./scripts/verify.sh` passed.
- `./scripts/smoke-local-fixtures.sh` passed.

### Retrospective

What worked:

- Starting with settings gave the app-service layer concrete provider, language,
  mode, and hotkey data to report.
- Keeping settings in `verboscribe-storage` preserved the platform-neutral core
  boundary.
- Runtime recovery mapping was testable without waiting for real OS adapters.
- The smoke harness made the local fixture checks repeatable instead of relying
  on handoff notes.

What slowed us down:

- The feature branch name was scoped to VS2-010 even though the sprint continued
  through VS2-011 and VS2-012.
- Formatting checks caught several long Rust lines after patching instead of
  during the edit.

What should change next sprint:

- Use a sprint-level branch name when continuing through multiple committed
  stories in one work session.
- Run `cargo fmt --all` immediately after larger Rust patches, then
  `cargo fmt --all -- --check` before verification.
- Add platform adapter stories only with explicit manual QA notes attached.

Improvement actions:

| ID | Action | Owner | Target | Status |
| --- | --- | --- | --- | --- |
| RA-005 | Run Rust formatting checks before final verification. | Lead AI | Sprint 3 execution | Done |
| RA-007 | Use sprint-level branch names when implementing multiple stories together. | Lead AI | Sprint 4 planning | Open |
| RA-008 | Attach manual QA notes to every platform adapter story before implementation. | Lead AI | Sprint 4 planning | Open |

## Sprint 4: Live Capture Adapter

Status: Done  
Goal: Capture live microphone audio into a transcription-ready WAV through the
same core recorder boundary the vertical slice will use.

### Committed Items

- VS2-014: Live Microphone Capture Adapter

### Candidate Items

- VS2-013: Clipboard Safety Contract

### Definition Of Done

- `verboscribe-audio` provides a live recorder implementation using CPAL or a
  documented fallback.
- Stop returns a transcription-ready WAV file and duration.
- Duplicate start and stop-without-start settle predictably.
- Permission and device failures map into typed dictation errors.
- Manual QA impact is documented.
- Verification remains green.

### Risks And Dependencies

- Default input devices may expose stereo or non-16 kHz formats, so conversion
  to the provider contract is required.
- macOS and Windows microphone permission behavior is partly OS-driven and
  cannot be fully validated in unit tests.
- Manual OS-level recording checks are required before calling the adapter
  production-ready.

### Retro Actions Carried In

| ID | Action | Owner | Status |
| --- | --- | --- | --- |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Open |
| RA-007 | Use sprint-level branch names when implementing multiple stories together. | Lead AI | Done |
| RA-008 | Attach manual QA notes to every platform adapter story before implementation. | Lead AI | Done |

### Planned Execution Approach

1. Run baseline verification on `feature/sprint-4-live-capture`.
2. Implement a CPAL-backed recorder in `verboscribe-audio` with predictable
   start/stop/cancel state handling.
3. Downmix and resample captured audio into the existing mono 16 kHz WAV
   contract.
4. Add unit coverage for format conversion and recorder state/error paths.
5. Re-run required verification and update sprint notes.

### Sprint Start Notes

- Started on branch `feature/sprint-4-live-capture`.
- Baseline verification passed: `cargo fmt --all -- --check` and
  `./scripts/verify.sh`.
- Manual QA expectations for recording checks were updated in
  `docs/MANUAL_QA.md` before implementation.

### Execution Notes

- VS2-014 complete: `verboscribe-audio` now exposes a CPAL-backed
  `CpalAudioRecorder` that implements the core `AudioRecorder` trait.
- Captured device audio is normalized into the existing mono 16 kHz WAV
  contract through downmixing, linear resampling, and i16 conversion.
- Recorder state now settles predictably for duplicate start, stop-without-start,
  and cancel.
- CPAL permission-like failures map to `DictationError::MicrophonePermissionDenied`;
  other device/stream failures map to `DictationError::Recording`.
- Verification after VS2-014: `cargo fmt --all -- --check`,
  `./scripts/verify.sh`, and `./scripts/smoke-local-fixtures.sh` passed.

### Sprint Review

Increment delivered:

- CPAL-backed live microphone recorder behind the existing core recorder trait.
- Audio normalization from device-native input into transcription-ready mono
  16 kHz WAV output.
- Unit coverage for recorder state, permission/device error mapping, and audio
  normalization helpers.
- Manual QA notes extended for live recording behavior.

Acceptance criteria passed:

- Live recorder implementation exists in `verboscribe-audio`.
- Stop returns a transcription-ready WAV path and duration.
- Duplicate start and stop-without-start settle predictably.
- Permission and device failures map into typed dictation errors.
- Manual QA impact is documented.
- Verification remains green.

Blocked or deferred:

- Real OS-level recording QA is still required on macOS and Windows before the
  adapter should be considered production-ready.
- Hotkey, target tracking, and paste integration remain separate stories.

Backlog changes:

- VS2-014 moved to Done.

User review:

- Manual recording QA is now required on macOS and Windows.

Verification:

- `cargo fmt --all -- --check` passed.
- `./scripts/verify.sh` passed.
- `./scripts/smoke-local-fixtures.sh` passed.

### Retrospective

What worked:

- Keeping the recorder behind the existing core trait avoided any churn in
  `verboscribe-core`.
- Pure helper tests for downmixing and resampling made the device-format work
  deterministic.
- Starting Sprint 4 with manual QA notes satisfied the retro action instead of
  leaving it for closeout.

What slowed us down:

- CPAL stream sendability differs across platforms, so the recorder session had
  to own a non-`Send` stream wrapper.
- Very short fixture captures rounded down to `0 ms`, which required explicit
  duration handling in the adapter.

What should change next sprint:

- Add a small app-service smoke path for starting/stopping the live recorder
  once it is wired into the desktop boundary.
- Keep platform adapter tests split between deterministic unit tests and
  explicit manual QA checklists.

Improvement actions:

| ID | Action | Owner | Target | Status |
| --- | --- | --- | --- | --- |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Next scaffold/update | Open |
| RA-009 | Add an app-service smoke path when a platform adapter first lands. | Lead AI | Sprint 5 planning | Open |

## Sprint 5: Global Hotkey Adapter

Status: Done  
Goal: Register the dictation hotkey through the desktop shell and expose
registration or event state clearly enough to support the next vertical-slice
integration work.

### Committed Items

- VS2-007: Global Hotkey Adapter

### Candidate Items

- VS2-013: Clipboard Safety Contract

### Definition Of Done

- Register and unregister the dictation hotkey through Tauri/Rust.
- Capture pressed and released shortcut states.
- Show registration failure in the UI.
- Verification remains green.
- Manual QA impact is documented.

### Risks And Dependencies

- Global hotkey availability is OS- and environment-dependent; another app can
  claim the same accelerator.
- The current slice tracks hotkey state but does not yet start or stop live
  dictation.
- Manual desktop QA is required because registration behavior depends on the
  active OS session.

### Retro Actions Carried In

| ID | Action | Owner | Status |
| --- | --- | --- | --- |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Open |
| RA-009 | Add an app-service smoke path when a platform adapter first lands. | Lead AI | Open |

### Planned Execution Approach

1. Start from `main` on branch `feature/sprint-5-hotkeys`.
2. Add the official Tauri 2 global-shortcut plugin in the desktop shell.
3. Keep registration state in the app-service layer so the UI can surface
   registration errors and last pressed or released events.
4. Re-run required verification and update manual QA notes for hotkeys.

### Sprint Start Notes

- Started on branch `feature/sprint-5-hotkeys`.
- The next development slice was selected from the existing backlog as VS2-007.
- Hotkey-specific manual QA notes were attached during sprint closeout.

### Execution Notes

- VS2-007 complete: the desktop shell now installs the official Tauri 2
  global-shortcut plugin and registers the configured dictation hotkey on
  startup.
- Thin Rust commands now support hotkey register and unregister operations
  without putting product logic in the command handlers.
- `AppService` now tracks hotkey registration state, active accelerator, and
  last pressed or released event so the UI can expose registration failure.
- The frontend now refreshes status periodically so hotkey event state is
  visible without reloading the app.
- Verification after VS2-007: `cargo fmt --all -- --check`,
  `npm --workspace apps/desktop run build`, and `./scripts/verify.sh` passed.

### Sprint Review

Increment delivered:

- Tauri global shortcut plugin wiring in the desktop shell.
- Rust-side register and unregister flow for the configured dictation hotkey.
- UI-visible hotkey registration status and last pressed or released event.
- Hotkey normalization tests and service-state coverage.
- Manual QA notes for hotkey behavior on real desktops.

Acceptance criteria passed:

- Dictation hotkey registers and unregisters through Tauri/Rust.
- Pressed and released shortcut states are captured in the app-service state.
- Registration failure is shown in the status UI.
- Verification remains green.
- Manual QA impact is documented.

Blocked or deferred:

- The hotkey does not yet trigger live dictation; that integration is a later
  slice.
- Real desktop QA is still required on macOS and Windows to validate conflict
  handling and event delivery.

Backlog changes:

- VS2-007 moved to Done.

User review:

- Manual hotkey QA is now required on macOS and Windows.

Verification:

- `cargo fmt --all -- --check` passed.
- `npm --workspace apps/desktop run build` passed.
- `./scripts/verify.sh` passed.

### Retrospective

What worked:

- Keeping Tauri plugin wiring in the shell and status state in `AppService`
  preserved the existing boundary.
- Using the official plugin kept the implementation small and aligned with the
  supported desktop platforms.
- Status polling was enough to make pressed and released events visible without
  prematurely redesigning the frontend.

What slowed us down:

- The plugin shortcut API accepted borrowed strings but not owned `String`
  values, so registration calls needed one compile-and-correct pass.
- Human-readable settings shortcuts needed a normalization layer before the
  plugin could register them reliably.

What should change next sprint:

- Wire hotkey pressed and released events into the real dictation workflow
  instead of just status tracking.
- Add a narrow smoke path around live capture plus hotkey registration once the
  app-service flow exists.

Improvement actions:

| ID | Action | Owner | Target | Status |
| --- | --- | --- | --- | --- |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Next scaffold/update | Open |
| RA-009 | Add an app-service smoke path when a platform adapter first lands. | Lead AI | Sprint 6 planning | Open |
| RA-010 | Keep user-facing hotkey strings separate from plugin accelerator syntax. | Lead AI | Sprint 6 implementation | Open |

## Sprint 6: Live Dictation Service Integration

Status: Done  
Goal: Connect hotkey events, live capture, and local transcription through the
desktop app service so the shell can complete a real dictation cycle without
clipboard insertion yet.

### Committed Items

- VS2-015: App-Service Live Dictation Flow

### Candidate Items

- VS2-013: Clipboard Safety Contract

### Definition Of Done

- App service can start and stop a real dictation cycle with live capture.
- Hotkey pressed and released events drive the app-service dictation flow.
- Missing provider configuration surfaces actionable recovery text.
- Status commands expose active dictation state and last transcript.
- Verification remains green.
- Manual QA impact is documented.

### Risks And Dependencies

- Live recorder state must remain safe to own across hotkey-triggered start and
  stop operations on macOS.
- The provider path still depends on local `whisper.cpp` configuration.
- Clipboard insertion is not implemented yet, so the integration ends at
  transcript capture.

### Retro Actions Carried In

| ID | Action | Owner | Status |
| --- | --- | --- | --- |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Open |
| RA-009 | Add an app-service smoke path when a platform adapter first lands. | Lead AI | Open |
| RA-010 | Keep user-facing hotkey strings separate from plugin accelerator syntax. | Lead AI | Open |

### Planned Execution Approach

1. Clean the handoff so Sprint 6 starts from a current top-level summary.
2. Make the live recorder safe to retain inside shared app-service state.
3. Add real app-service start and stop dictation methods plus hotkey wiring.
4. Re-run required verification and update sprint notes.

### Sprint Start Notes

- Started from `main` after Sprint 5 merge and handoff review, then moved onto
  branch `feature/sprint-6-live-dictation` for implementation and closeout.
- VS2-015 was added to the backlog and selected as the Sprint 6 committed item.
- The implementation will stop at transcript capture; clipboard insertion
  remains a later slice.

### Execution Notes

- `CpalAudioRecorder` was refactored to retain recording control through a
  thread-backed controller so the desktop app service can own a live recorder
  inside shared state without depending on a non-`Send` CPAL stream handle.
- VS2-015 complete: `AppService` now builds a real desktop dictation engine
  from saved settings, keeps runtime dictation state and last transcript, and
  surfaces actionable recovery when provider configuration is missing.
- Desktop command handlers now expose explicit start, stop, and cancel
  dictation commands in addition to the existing status and settings commands.
- The Tauri global shortcut handler now forwards pressed and released events
  into the real app-service dictation flow instead of updating status only.
- Manual QA notes were updated to reflect the new transcript-capture workflow
  and the remaining clipboard gap.

### Sprint Review

Increment delivered:

- Real app-service dictation runtime using the live CPAL recorder and local
  `whisper.cpp` transcriber.
- Hotkey-driven pressed and released flow through the desktop shell into the
  app-service dictation engine.
- Actionable provider-configuration recovery through the existing runtime and
  app-status DTOs.
- Last-transcript retention in app-service status without waiting for clipboard
  insertion.
- A recorder ownership refactor that keeps CPAL stream handling off the shared
  app-service thread boundary.

Acceptance criteria passed:

- App service can start and stop a real dictation cycle with live capture.
- Hotkey pressed and released events drive the app-service dictation flow.
- Missing provider configuration surfaces actionable recovery text.
- Status commands expose active dictation state and last transcript.
- Verification remains green.
- Manual QA impact is documented.

Blocked or deferred:

- Clipboard insertion and target-app tracking are still separate stories.
- Real desktop QA is still required on macOS and Windows with a configured
  local `whisper.cpp` installation before relying on this flow day to day.
- A dedicated app-service smoke path for the real dictation cycle is still
  worth adding once a stable fixture strategy exists around the recorder path.

Backlog changes:

- VS2-015 moved to Done.

User review:

- Manual QA is now required for the full hotkey-to-transcript desktop flow.

Verification:

- `cargo fmt --all -- --check` passed.
- `./scripts/verify.sh` passed.
- `./scripts/smoke-local-fixtures.sh` passed.

### Retrospective

What worked:

- Refactoring the recorder first removed the main ownership blocker before the
  app-service integration work started.
- Keeping transcript capture as a no-op insertion service let the desktop shell
  prove the real recorder and transcriber path without prematurely folding in
  clipboard automation.
- Reusing the existing status DTOs kept the frontend unchanged while still
  exposing the important runtime state.

What slowed us down:

- The initial Sprint 6 edits were left half-finished in `app_service.rs`, so
  one compile-and-repair pass was needed before the integration could settle.
- Provider configuration defaults are intentionally empty, which means the real
  path needs explicit recovery coverage before it is useful interactively.

What should change next sprint:

- Add clipboard insertion and target tracking as the next vertical slice so the
  captured transcript can leave the app without manual copy steps.
- Add a narrow automated smoke path around the app-service dictation cycle when
  a deterministic recorder test strategy is available.

Improvement actions:

| ID | Action | Owner | Target | Status |
| --- | --- | --- | --- | --- |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Next scaffold/update | Open |
| RA-009 | Add an app-service smoke path when a platform adapter first lands. | Lead AI | Sprint 7 planning | Open |
| RA-010 | Keep user-facing hotkey strings separate from plugin accelerator syntax. | Lead AI | Sprint 6 implementation | Done |

## Sprint 7: Clipboard Insertion And Target Tracking

Status: Done  
Goal: Paste captured transcripts back into the previously active app while
making clipboard fallback explicit when automated insertion fails.

### Committed Items

- VS2-009: Clipboard Paste Insertion
- VS2-013: Clipboard Safety Contract

### Candidate Items

- SPIKE-002: Windows Paste And Target Tracking

### Definition Of Done

- A platform adapter captures the target app before recording and reuses it for
  insertion.
- Dictation flow copies transcript text to the clipboard before attempting
  automation.
- Automated paste failures leave transcript text available on the clipboard and
  surface recovery guidance.
- Platform-specific code stays out of `verboscribe-core`.
- Verification remains green.
- Manual QA impact is documented.

### Risks And Dependencies

- macOS paste automation may depend on Accessibility permission and can fail in
  ways that need clear timeout and recovery handling.
- Windows target activation and paste behavior still need careful platform
  design, even if a first adapter lands this sprint.
- The current desktop UI is status-focused, so validation relies on status DTOs
  and manual QA rather than a rich insertion workflow UI.

### Retro Actions Carried In

| ID | Action | Owner | Status |
| --- | --- | --- | --- |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Open |
| RA-009 | Add an app-service smoke path when a platform adapter first lands. | Lead AI | Open |

### Planned Execution Approach

1. Add `verboscribe-platform` adapters for target capture and clipboard-first
   paste insertion with timeout-guarded command execution.
2. Wire the real platform inserter and tracker into `AppService`.
3. Add focused tests for command planning, clipboard safety, and app-service
   recovery behavior.
4. Re-run required verification, update manual QA notes, and close the sprint.

### Sprint Start Notes

- Started from `main` after Sprint 6 closeout and cold-start handoff hardening.
- Branch: `feature/sprint-7-clipboard-insertion`.
- The sprint is taking the next recommended vertical slice from the handoff:
  clipboard insertion plus target-app tracking.

### Execution Notes

- `verboscribe-platform` now owns a real `DesktopTargetTracker` and
  `DesktopTextInserter` with timeout-guarded command execution instead of
  leaving target tracking and insertion as shell placeholders.
- Target capture now remembers the last non-VerboScribe app so the insertion
  path can avoid pasting into the VerboScribe window after UI interaction.
- Clipboard write now happens before activation or paste automation so the
  dictated transcript remains available even when insertion fails.
- `AppService` now wires the real platform tracker and inserter into the live
  dictation engine instead of the Sprint 6 preview inserter.
- SPIKE-002 was completed as part of the platform adapter design and documented
  in `docs/SPIKES.md`.
- Manual QA, platform smoke, and operations docs were updated to reflect the
  new insertion path and remaining platform risks.

### Sprint Review

Increment delivered:

- Real platform target tracking for the active app before recording.
- Clipboard-first insertion service that writes transcript text before any paste
  automation attempt.
- macOS paste activation plus `Cmd+V` command planning with timeout-guarded
  execution.
- First-pass Windows target activation and paste command planning using
  PowerShell plus Win32 interop.
- Desktop app-service wiring that now inserts into the target app instead of
  stopping at transcript capture.

Acceptance criteria passed:

- A platform adapter captures the target app before recording and reuses it for
  insertion.
- Dictation flow copies transcript text to the clipboard before attempting
  automation.
- Automated paste failures leave transcript text available on the clipboard and
  surface recovery guidance.
- Platform-specific code stays out of `verboscribe-core`.
- Verification remains green.
- Manual QA impact is documented.

Blocked or deferred:

- Manual OS QA is still required on macOS and Windows before claiming the paste
  flow is reliable in real desktops.
- Linux clipboard and paste automation are still not implemented.
- If Windows QA shows `SendKeys` or PowerShell activation is unreliable, the
  adapter may need to move to direct Rust Win32 calls in a later sprint.

Backlog changes:

- VS2-009 moved to Done.
- VS2-013 moved to Done.
- SPIKE-002 moved to Done.
- VS2-016 added as Ready for the next app-service smoke-path slice.

User review:

- Manual QA is now required for full end-to-end paste insertion on macOS and
  Windows, including permission-denied fallback cases.

Verification:

- `cargo fmt --all -- --check` passed.
- `./scripts/verify.sh` passed.
- `./scripts/smoke-local-fixtures.sh` passed.

### Retrospective

What worked:

- Keeping command planning inside `verboscribe-platform` made the new adapter
  testable without requiring live OS automation during unit tests.
- Clipboard-first ordering gave the paste safety contract a concrete
  implementation instead of leaving it as a documentation promise.
- Wiring the platform adapter through the existing `DictationEngine` boundary
  avoided churn in `verboscribe-core`.

What slowed us down:

- OS automation commands can hang or block on permissions, so the adapter
  needed explicit timeout handling before it was safe to integrate.
- Windows automation had to be designed without local execution, which keeps
  manual QA risk higher than the macOS path.

What should change next sprint:

- Add an app-service smoke path around the real dictation cycle now that paste
  insertion exists.
- Consider a more direct Windows implementation if manual QA shows PowerShell
  activation or `SendKeys` is weak.

Improvement actions:

| ID | Action | Owner | Target | Status |
| --- | --- | --- | --- | --- |
| RA-003 | Add framework asset/config requirements to foundation stories when scaffolding new shells. | Lead AI | Next scaffold/update | Open |
| RA-009 | Add an app-service smoke path when a platform adapter first lands. | Lead AI | Sprint 8 planning | Open |
| RA-011 | Keep platform automation commands timeout-guarded so permission prompts do not hang the dictation flow. | Lead AI | Sprint 7 implementation | Done |
| RA-012 | Evaluate direct Rust Win32 input and activation if Windows manual QA shows the PowerShell path is unreliable. | Lead AI | Post-Sprint 7 QA follow-up | Open |
