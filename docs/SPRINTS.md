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

Status: Planned  
Goal: Turn the tested provider/audio pieces into an app-service workflow with
settings and status/recovery events, while preparing for real platform adapters.

### Candidate Items

- VS2-011: Runtime Status And Recovery Events
- VS2-010: Minimal Settings Store
- VS2-012: Platform Smoke Harness
- VS2-013: Clipboard Safety Contract

### Definition Of Done

- App service exposes status and recovery state for provider/audio failures.
- Minimal settings persist provider paths, language, mode, and hotkey defaults.
- Smoke harness documents/runs local provider and WAV validation checks.
- Verification remains green.
