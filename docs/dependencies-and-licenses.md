# Dependencies and licenses

Project license: **GPL-3.0-or-later** (decision 2). Machine verification (`cargo deny`, `cargo license`, `pnpm licenses`) is a P8 gate; entries marked [U] come from memory and must be confirmed then.

## Third-party programs (not linked)
| Component | License [V] | Use | Obligation |
|---|---|---|---|
| chiaki-ng | AGPL-3.0 (+OpenSSL exception) | Launched as external process | None on our code (no linking/embedding). If ever embedded, whole combined work must be AGPL-compatible and network-use source offer applies. Do not redistribute its binaries in our package without honouring AGPL. |
| GoldHEN | none declared | User-installed on console; we only talk FTP | Not redistributed. |
| OpenOrbis toolchain | GPL-3.0 | Builds companion | Companion is GPL-3.0 — consistent. |
| Itemzflow | GPL-3.0 | Prior art only | No code copied. |
| Clean Your TV | NOASSERTION | Prior art only | No code copied (no license grant). |

## Rust crates (planned direct deps) [U until cargo-deny runs]
tauri 2 (Apache-2.0/MIT), tokio (MIT), serde (MIT/Apache-2.0), thiserror, rusqlite (MIT), keyring (MIT/Apache-2.0), suppaftp (MIT), gilrs (Apache-2.0/MIT), sha2/hmac (MIT/Apache-2.0).
Runtime system libs: webkit2gtk-4.1 (LGPL/BSD mix), GTK3 (LGPL) — dynamically linked, standard for Tauri .deb.

## JS packages (direct)
svelte (MIT), vite (MIT), lucide-svelte (ISC), @tauri-apps/api (Apache-2.0/MIT) [U].

## Branding
Original icon/UI. No Sony trademarks/artwork in assets. README states: independent project, not endorsed by Sony Interactive Entertainment. "PlayStation"/"PS4" used nominatively only.
