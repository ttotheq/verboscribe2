# Git Workflow

## Default Model

Use a trunk-based workflow with short-lived branches.

- `main` stays releasable.
- Branches are small and purpose-built.
- Merge back frequently.
- Keep diffs reviewable.

## Branch Types

- `feature/<ticket>`: sprint stories and product work.
- `spike/<topic>`: time-boxed experiments or research.
- `fix/<issue>`: narrow bug fixes.
- `release/<version>`: only when freezing a release or packaging cut.

## Branch Rules

- Start from `main`.
- Rebase or merge `main` often to keep branches current.
- Keep one story or tightly related slice per branch.
- Prefer one branch per AI worker when parallel work is needed.
- Avoid long-lived feature branches unless the work is intentionally isolated.
- Use `git worktree` when parallel branch work would otherwise collide.

## Commit Rules

- Commit small, logical units of work.
- Commit messages should describe the result, not the process.
- Use WIP commits only when necessary and keep them short-lived.
- Run verification before merging or tagging.

## Merge Rules

- Merge after tests/verification pass.
- Prefer squash merges for short-lived story branches if the history is noisy.
- Preserve meaningful commits if the branch contains a small set of coherent
  changes.
- Tag sprint completion or release checkpoints.

## AI-Specific Rules

- Do not let two agents edit the same files in the same branch.
- Keep the main branch of work focused on the sprint goal.
- Treat branch boundaries as coordination boundaries for parallel work.
- Update `HANDOFF.md` when a branch is merged, abandoned, or handed off.

## Suggested Cadence

1. Create `feature/*` branch for the sprint story.
2. Implement and verify locally.
3. Update docs and handoff notes.
4. Merge back to `main`.
5. Tag sprint completion if useful.
