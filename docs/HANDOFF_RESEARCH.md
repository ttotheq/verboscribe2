# Handoff Documentation Research

Date: 2026-05-02

## Findings

Good Codex/agent handoff documentation should be durable, concise, and
actionable. It should let a fresh session resume from files on disk without
assuming access to prior chat context.

Best-practice themes:

- Externalize state into a checked-in artifact, not only chat history.
- Keep durable project instructions separate from volatile current-session
  status.
- Include current goal, completed work, changed files, commands to run,
  verification state, blockers, risks, and next actions.
- Treat plans and handoffs as living documents.
- Maintain progress, discoveries, decisions, and retrospective notes for
  multi-hour work.
- Avoid context overload; point to stable docs and summarize only the current
  operational state.
- Use precise commands instead of vague instructions like "run tests."
- Update the handoff at pause/closeout points and after major decisions.

## Sources

- OpenAI Cookbook, "Using PLANS.md for multi-hour problem solving":
  https://cookbook.openai.com/articles/codex_exec_plans/
- VS Code context engineering guide:
  https://code.visualstudio.com/docs/copilot/guides/context-engineering-guide
- AI Pattern Book, "Handoff":
  https://aipatternbook.com/handoff
- OpenAI Agents SDK handoffs guide:
  https://openai.github.io/openai-agents-js/guides/handoffs/

## Applied To This Repo

This repo now uses:

- `AGENTS.md` for durable Codex project instructions.
- `HANDOFF.md` for session-to-session continuation state.
- `docs/SPRINTS.md` for agile sprint state, review, retro, and actions.
- `docs/BACKLOG.md` for current work item status.
- `docs/DECISIONS.md` and `docs/SPIKES.md` for durable decisions.

The handoff is intentionally operational: a future Codex session should start by
reading it, running the listed commands, and continuing with the next sprint
work.
