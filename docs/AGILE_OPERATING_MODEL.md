# Agile Operating Model

## Working Agreement

VerboScribe 2 will use a lightweight Scrum/Kanban hybrid:

- Epics define product outcomes.
- Sprints are short execution cycles with a clear goal and reviewable increment.
- Stories are small, testable slices that move one sprint goal forward.
- Spikes are time-boxed research tasks with a written decision or prototype.
- Bugs are defects in accepted behavior.
- Chores cover tooling, documentation, CI, packaging, and maintenance.

Because the development team is AI-assisted, most planning and execution should
be autonomous. The user should only need to make product tradeoff decisions,
approve large scope changes, provide credentials/hardware access, and perform
manual OS-level QA that cannot be automated from the repo.

## Cadence

Default sprint length: one focused work session or one calendar week, whichever
comes first.

Each sprint has:

- Sprint goal.
- Committed stories.
- Definition of done.
- Risks and dependencies.
- Execution notes.
- Sprint review.
- Retrospective.
- Improvement actions carried into the next sprint.

## Proactive Autonomy Rules

The lead agent should run the full agile loop without waiting to be prompted:

- Start each sprint by selecting a goal and a small set of ready stories.
- Use subagents for parallel review or implementation when tasks are bounded.
- Keep the backlog and sprint log current during execution.
- Close every sprint with a review and retrospective before starting the next
  sprint.
- Convert retrospective findings into concrete action items with owner and
  status.
- Keep verification green or explicitly record blockers.
- Ask the user only for product tradeoffs, credentials, hardware/manual QA, or
  irreversible actions.

## AI Team Model

The lead agent owns product coherence, architecture, integration, and final
verification. Subagents can be used for parallel work when the task is bounded
and has a clear output.

Recommended AI roles:

- Product planner: epics, story slicing, acceptance criteria.
- Architecture reviewer: boundaries, dependency risks, platform strategy.
- Core worker: Rust domain logic and tests.
- Platform worker: macOS/Windows adapters.
- Desktop worker: Tauri shell and TypeScript UI.
- QA reviewer: verification scripts, test gaps, manual QA updates.
- Documentation editor: roadmap, sprint review, release notes.

Subagent rules:

- Assign a narrow responsibility and clear file ownership for code edits.
- Avoid overlapping write scopes.
- Review and integrate every subagent result before accepting it.
- Keep the main branch of work focused on the sprint goal.

## Backlog Management

Backlog items use this shape:

```text
ID:
Type:
Epic:
Title:
User value:
Acceptance criteria:
Dependencies:
Notes:
Status:
```

Statuses:

- `Backlog`
- `Ready`
- `In Progress`
- `Blocked`
- `Review`
- `Done`

## Definition Of Ready

A story is ready when:

- The user value is clear.
- Acceptance criteria are testable.
- Dependencies are known.
- The expected files/modules are reasonably bounded.
- The story can be completed without unresolved product decisions.

## Definition Of Done

A story is done when:

- Code/docs are updated.
- Automated tests are added or updated when practical.
- Verification commands were run, or blockers are explicitly recorded.
- Manual QA impact is documented when OS behavior changes.
- The change preserves platform isolation.
- The sprint board status is updated.

## Review And Retro

Sprint review answers:

- What increment was delivered?
- What acceptance criteria passed?
- What is blocked or deferred?
- What changed in the backlog?
- What should the user review manually?

Sprint retro answers:

- What worked?
- What slowed us down?
- What should change next sprint?
- What concrete action items were created?
- Who owns each action?
- What is the current status of each action?

Sprint closeout checklist:

- Review notes completed.
- Retrospective completed.
- Retro actions added to the next sprint or backlog.
- Backlog statuses updated.
- Verification command recorded.
- Next sprint goal proposed.

## User Involvement

The user is expected for:

- Prioritizing product decisions when tradeoffs are meaningful.
- Supplying API keys only when needed for manual provider verification.
- Installing prerequisites such as Rust if they are not available.
- Testing microphone, hotkeys, permissions, and paste behavior on real OS
  desktops.

The AI team should otherwise continue autonomously through implementation,
verification, documentation, and sprint review.
