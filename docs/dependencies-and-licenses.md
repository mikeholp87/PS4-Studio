# Dependencies and licenses

Project license: **GPL-3.0-or-later** (decision 2). Audit run 2026-11-22 from real tool output.

## Rust (`cargo metadata --filter-platform x86_64-unknown-linux-gnu`, 349 packages)
All third-party crates carry permissive or weak-copyleft licenses compatible with GPL-3.0-or-later:
MIT / Apache-2.0 (≈85%), Unicode-3.0 (18), MPL-2.0 (5: `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`, `selectors`; file-level copyleft, unmodified, fine), BSD-2/3-Clause, Zlib, ISC, Unlicense/MIT, CC0/MIT-0, 0BSD. **No GPL-2.0-only, AGPL or proprietary crates.** CI enforces this with `cargo deny` (`deny.toml`).

## JavaScript (`pnpm licenses list`)
Dev+prod tree: MIT (149), Apache-2.0 (20), ISC (9), BSD-2/3-Clause (11), BlueOak-1.0.0 (3), Apache-2.0 OR MIT (3), MPL-2.0 (2). Production (shipped) set: Svelte and helpers (MIT), `@tauri-apps/api` (Apache-2.0 OR MIT), `lucide-svelte` (ISC), `aria-query`/`axobject-query` (Apache-2.0).

## System libraries (dynamically linked by the `.deb`)
`libwebkit2gtk-4.1` (LGPL/BSD mix), GTK 3 (LGPL), standard for Tauri apps.

## Third-party programs (not linked, not redistributed)
| Component | License | Use | Obligation |
|---|---|---|---|
| chiaki-ng | AGPL-3.0 (+OpenSSL exception) | Launched as a separate process | None on our code. Never link/embed/copy; do not bundle its binaries in our package. |
| GoldHEN | none declared | User-installed on the console; we only speak FTP | Not redistributed |
| OpenOrbis PS4 Toolchain | GPL-3.0 | Would build the companion | Companion is GPL-3.0-compatible |
| Itemzflow | GPL-3.0 | Prior art only | No code copied |
| Clean Your TV | no license grant (NOASSERTION) | Prior art only | No code copied |

## Branding
Original icon/UI; no Sony trademarks or artwork. "PlayStation"/"PS4" used nominatively.
