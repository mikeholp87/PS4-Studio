# Contributing

Thanks for helping. Start with [AGENTS.md](AGENTS.md) (rules, layout, commands) and [docs/current-status.md](docs/current-status.md).

## Ground rules
- Capabilities come from probes, never assumptions. Unknown means unavailable, with a human-readable reason.
- Anything that changes a console goes through INSPECT → PLAN → CONFIRM → APPLY → VERIFY → RECOVER and is audit-logged.
- No exploit installation, firmware changes, DRM/ban evasion, piracy features, or PSN credential handling. Such PRs will be declined.
- Never claim physical-console testing you did not do. Say "simulator-verified" in docs and PRs.
- Test hostile inputs (truncated, oversized, malformed network data) for anything that parses network data.

## Workflow
1. Fork, branch, keep changes small and focused.
2. Run the gates from the README (`cargo fmt/clippy/test`, `pnpm -r check lint test`, Playwright when UI changes).
3. Update `docs/tasks.md` (what changed, tests, limits, next step) and `docs/decisions.md` for non-obvious choices.
4. Use clear commit messages (`feat:`, `fix:`, `docs:`, `test:`, `chore:`).

## Licensing
By contributing you agree your work is licensed GPL-3.0-or-later. Do not paste code from projects without a compatible license (for example Clean Your TV currently has no license grant). chiaki-ng is AGPL-3.0: it is launched as a separate process and must not be linked or copied into this repository.
