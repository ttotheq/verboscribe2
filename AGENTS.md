# Codex Project Instructions

Start every new Codex session by reading:

1. `HANDOFF.md`
2. `docs/SPRINTS.md`
3. `docs/BACKLOG.md`
4. `docs/ARCHITECTURE.md`

`HANDOFF.md` is the durable continuation brief for closed sessions, laptop
reboots, or context loss. Treat it as the source of truth for current status,
open sprint work, verification commands, and the next recommended actions.

## Working Style

- Follow the agile operating model in `docs/AGILE_OPERATING_MODEL.md`.
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
