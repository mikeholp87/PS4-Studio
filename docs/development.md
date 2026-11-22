# Development

```sh
pnpm install
cd apps/desktop/src-tauri && cargo tauri dev
```
## Tests
| What | Command |
|---|---|
| Rust (unit, integration, simulator, real-ACL Tauri IPC flow) | `cargo test --workspace` |
| Lint/format | `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings` |
| Frontend | `pnpm -r check && pnpm -r lint && pnpm -r test` |
| UI e2e (mocked IPC, fixtures from the real Rust engine) | `cd apps/desktop && PW_CHROMIUM=/path/to/chromium pnpm e2e` (or `pnpm exec playwright install chromium` once) |
| Regenerate e2e fixtures after capability/scenario changes | `./scripts/gen-fixtures.sh` |
| README screenshots | `SCREENSHOTS=1 pnpm e2e e2e/screenshots.spec.ts` |
| C protocol core | `make -C apps/ps4-companion host-test` |

## Simulator
`ps4studio-mock-console` has a binary (`cargo run -p ps4studio-mock-console`) listing scenarios; the app starts the UDP/FTP/companion servers in-process per scenario. Add a scenario in `simulator/mock-console/src/lib.rs` and cover it with a test.

## Adding a Tauri command
1. Implement in `src-tauri/src/*.rs`; 2. list in `build.rs` COMMANDS; 3. register in `lib.rs`; 4. grant in `capabilities/default.json` (`allow-<kebab-name>`); 5. wrap in `src/lib/api.ts`; 6. extend the IPC test and the e2e mock.

## Performance
Targets and measured numbers are in `docs/performance.md`.
