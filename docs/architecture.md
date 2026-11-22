# Architecture

```
Svelte UI (apps/desktop/src)  ──invoke──▶  Tauri commands (src-tauri/src/*.rs, thin, ACL-gated)
                                                │
        ┌───────────────┬──────────────┬────────┴────────┬───────────────┐
   crates/core     crates/discovery  crates/remote-play  crates/console-adapters  crates/backups / diagnostics
 capability engine   UDP 987 client   chiaki-ng launcher   path policy, FTP client,    manifests, sanitiser
 SQLite, themes,                                            safe ops, companion client
 companion_proto
        ▲
 simulator/mock-console: real UDP + FTP + companion servers, 14 scenarios, always labelled simulated
 packages/theme-engine: schema, templates, validators (shared with Rust via theme.schema.json + corpus)
 apps/ps4-companion: portable C protocol core (tested) + untested OpenOrbis skeleton
```

## Principles
1. **Capabilities are facts → pure function.** `core::capability::evaluate(Facts)`; probes produce facts; the UI renders reasons.
2. **UI never depends on a console.** Startup touches only the local SQLite DB; the simulator provides a full demo.
3. **Thin commands.** `src-tauri` validates input, holds session state, shapes JSON. Logic lives in crates with tests.
4. **Explicit permissions.** Every command is listed in `build.rs` and `capabilities/default.json`; the real-ACL IPC test fails if a command is not granted.
5. **Safe-op model** for remote changes (see `security.md`).
6. **Honesty.** Simulated data is labelled; hardware values are never invented.

## Differences from the original brief's tree
`security` → `core`; `file-transfer` → `console-adapters`; `capability-engine` → `core`; `ui` package → `apps/desktop/src/lib/components`. Reason: split only when a boundary earns it (docs/decisions.md #4).

## Data
SQLite (`~/.local/share/ps4-studio/ps4-studio.db`): settings (allowlisted keys), console profiles, audit log. Files: `themes/*.json`, `backups/<id>/{manifest.json,files/*}`, `reports/*.txt`. No secrets are stored.
