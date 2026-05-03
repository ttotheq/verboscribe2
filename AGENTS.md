# Codex Project Instructions

Start every new Codex session by reading:

1. `HANDOFF.md`
2. `docs/SPRINTS.md`
3. `docs/BACKLOG.md`
4. `docs/ARCHITECTURE.md`
5. `docs/GIT_WORKFLOW.md`

`HANDOFF.md` is the durable continuation brief for closed sessions, laptop
reboots, or context loss. Treat it as the source of truth for current status,
open sprint work, verification commands, and the next recommended actions.
It must be written for a cold-start reader with no prior conversational
context.

## Working Style

- Follow the agile operating model in `docs/AGILE_OPERATING_MODEL.md`.
- Follow the git workflow in `docs/GIT_WORKFLOW.md`.
- Be proactive and autonomous.
- Run the full sprint loop: planning, execution, review, retrospective, and
  retro actions.
- Keep platform-specific code out of `verboscribe-core`.
- Keep verification green, or record blockers explicitly in `HANDOFF.md` and
  `docs/SPRINTS.md`.

## Required Checks

Before declaring work complete, run:

```sh
cargo fmt --all -- --check
./scripts/verify.sh
```

When touching the local `whisper.cpp` provider and the local binary/model/sample
are present, also run:

```sh
./scripts/smoke-whisper-cpp.sh
```

## Handoff Rule

Before pausing, closing a session, or after completing a sprint, update
`HANDOFF.md` with:

- current sprint/status
- completed work
- changed files of interest
- verification results
- blockers/risks
- next recommended actions
- any user/manual QA needed
- branch name or merge status if work happened on a branch

For sprint closeout, the handoff must also be bulletproof for a different AI
model resuming from a clean context window. That means it must explicitly
include:

- current working behavior, stated as an end-to-end flow
- exact next recommended story or sprint candidate
- files to read first for the next slice
- implementation constraints or non-obvious technical decisions that must not be
  relearned
- manual setup or environment requirements needed to exercise the current slice
- a sharp split between implemented, implemented-but-not-manually-verified, and
  not implemented
- the exact verification commands last run and their expected outcomes
