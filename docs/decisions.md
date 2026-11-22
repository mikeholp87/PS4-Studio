# Decisions

| # | Date | Decision | Why |
|---|---|---|---|
| 1 | 2026-11-22 | Remote Play = launch installed chiaki-ng as separate process | chiaki-ng is AGPL-3.0; process boundary avoids license coupling and reuses maintained protocol code |
| 2 | 2026-11-22 | Project license GPL-3.0-or-later | companion uses GPL OpenOrbis; single license; revisit after dependency audit |
| 3 | 2026-11-22 | Secrets via `keyring` crate (Secret Service, pure-Rust backend) | avoids libsecret C dev package (not installed) |
| 4 | 2026-11-22 | Fewer crates than brief's tree: `security`→core, `file-transfer`→console-adapters, `capability-engine`→core | split only when a boundary earns it |
| 5 | 2026-11-22 | Companion C source not built in CI | OpenOrbis toolchain absent; desktop build must not depend on it |
| 6 | 2026-11-22 | Own minimal FTP client instead of `suppaftp` | need hard caps on lines/replies/listings, PASV bounce protection, per-op timeouts and a poison-on-failure rule; ~300 lines |
| 7 | 2026-11-22 | Remote write allowlist = `/data` only; system trees always protected | GoldHEN README documents `/data/pkg`; everything else is read-only until verified |
| 8 | 2026-11-22 | FTP credentials are session-only, never persisted | no feature needs persisted secrets yet; avoids shipping weak storage; Secret Service integration deferred |
| 9 | 2026-11-22 | Sessions start with FTP "not probed"; simulator FTP fact only scripts the fake server | capabilities must come from a probe, even in the simulator |
| 10 | 2026-11-22 | One JSON Schema (`theme.schema.json`) shared by TS (ajv) and Rust (jsonschema) + shared fixture corpus | prevents validator drift |
| 11 | 2026-11-22 | Backups = directory + checksummed manifest, restore only settings/profiles/themes | console files are re-uploaded through the confirmed flow, never silently |
