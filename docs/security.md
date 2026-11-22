# Security model

PS4 Studio runs as an ordinary user, needs no root, has no background daemon, and only talks to hosts the user selects.

## Trust boundaries
| Boundary | Rule | Where enforced | Test |
|---|---|---|---|
| Webview → Rust | Each command is listed in `build.rs` AND granted in `capabilities/default.json`; CSP `default-src 'self'` | Tauri ACL | `ipc_tests` runs the real context (ACL included) |
| Console replies → app | Untrusted. UDP reply ≤2 KB strict parse + sanitise; FTP lines ≤1 KB, replies ≤64 lines, listings ≤4 MB/10 000 entries; names must be single path segments | `discovery`, `console-adapters::ftp` | hostile-server tests |
| FTP data connection | Always to the control peer IP, never to an address the server advertises (no FTP bounce) | `ftp::pasv` | simulator tests |
| Remote changes | Locked by default → explicit unlock + "FTP is unencrypted" acknowledgement → PLAN (shows changes, warnings, recovery) → confirm → APPLY (plan single-use, re-validated, 5 min expiry) → VERIFY → audit row | `files.rs`, `ops.rs` | `ipc_tests`, `ops_sim` |
| Remote paths | Lexical normalisation, `..`/NUL/control/backslash rejected; changes only under `/data`; `/system`, `/system_ex`, `/preinst`, `/update`, `/dev`, `/boot` always protected; the allowlist root itself is unchangeable | `paths::Policy` | unit + sim tests |
| Overwrite/delete | Existing file (≤64 MB) copied into a checksummed local backup first; uploads go to `*.ps4studio-part` then rename | `ops::apply` | `ops_sim` |
| Downloads | Never overwrite (suffix), temp file then rename, mode 0644 (no exec bit), never executed | `ops::download_to` | `ops_sim` |
| Themes / imports | Data only, validated against one JSON Schema in both TS and Rust, ≤256 KB, slug ids, file dialogs opened from Rust (webview never supplies a path) | `core::themes` | shared corpus |
| Remote Play | Separate chiaki-ng process; argv never contains `--registkey/--morning/--passcode`; stdio discarded; executable only from our own detection; nickname validated | `remote-play` | unit tests |
| Logs / reports | Diagnostic export masks IPv4 host octets, MAC/hex ids, credential-like lines, `$HOME` | `diagnostics::sanitize` | unit tests |
| Audit | Every connect, unlock/lock, remote change, download, backup/restore/delete, theme change is recorded locally (SQLite) | `core::db::audit` | `ipc_tests` |

| Companion link | Mutual HMAC pairing proofs, per-frame MAC + direction label + sequence, 3-attempt lockout, 7-command allowlist, theme validation on both ends | `core::companion_proto`, `adapters::companion`, simulator | `companion_sim`, C `make host-test`, cross-language vectors |

## Deliberately not implemented
Remote shell, BinLoader/payload sending, exploit installation, firmware/boot changes, PSN credential handling, DRM/ban evasion, package auto-install or auto-execution, automatic rollback promises on the console.

## Known limitations (be honest)
- FTP is plaintext. Login (if any) is session-only memory, never persisted; Secret Service storage (`keyring`) is **not** wired yet because no feature needs a persisted secret.
- Symlinks on the console cannot be detected through FTP `LIST` reliably; they are shown as links and never descended automatically, but a symlink inside `/data` pointing elsewhere could be written through by the service. Mitigation: only `/data` is writable by policy.
- Backup manifests are checksummed per item but the manifest itself is not signed.
- The `/data` write root and `/data/pkg` target are policy choices based on the GoldHEN README, not verified against a console.
- Nothing has been tested on real hardware.
