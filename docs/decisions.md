# Decisions

| # | Date | Decision | Why |
|---|---|---|---|
| 1 | 2026-11-22 | Remote Play = launch installed chiaki-ng as separate process | chiaki-ng is AGPL-3.0; process boundary avoids license coupling and reuses maintained protocol code |
| 2 | 2026-11-22 | Project license GPL-3.0-or-later | companion uses GPL OpenOrbis; single license; revisit after dependency audit |
| 3 | 2026-11-22 | Secrets via `keyring` crate (Secret Service, pure-Rust backend) | avoids libsecret C dev package (not installed) |
| 4 | 2026-11-22 | Fewer crates than brief's tree: `security`→core, `file-transfer`→console-adapters, `capability-engine`→core | split only when a boundary earns it |
| 5 | 2026-11-22 | Companion C source not built in CI | OpenOrbis toolchain absent; desktop build must not depend on it |
