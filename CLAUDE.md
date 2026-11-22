# CLAUDE.md

Read `AGENTS.md` first — it is authoritative for architecture, rules, commands and handoff.
Do not duplicate or contradict it here.

Claude Code specifics:
- Plan file for the original brief lives under `~/.claude/plans/` but the repo docs (`docs/tasks.md`, `docs/current-status.md`) are the real state.
- Prefer `rg`/`fd`; shell output is proxied through `rtk`.
- Use the simulator (`simulator/mock-console`) for any console behaviour; do not probe real hosts without user approval.
