# PS4 Studio companion (EXPERIMENTAL, untested on hardware)

A small homebrew app meant to run **alongside** the stock PS4 interface on a console that already runs a homebrew environment. It never modifies or replaces the official shell or any system file, installs no service, and has no privileged component. Nothing here changes the official PS4 home screen.

## What exists
| Piece | State |
|---|---|
| Protocol v1 (framing, pairing proofs, per-frame MAC, command allowlist) | **Implemented and tested** — Rust (`crates/core/src/companion_proto.rs`, client in `crates/console-adapters`, simulator server in `simulator/mock-console`) and portable C (`src/proto.c`) |
| Cross-language correctness | Rust, C and Python agree on `tests/vectors.json` (HMAC proofs, session key, frame MAC) |
| Console entry point `src/ps4_main.c` | **Untested skeleton**: not compiled with OpenOrbis, not run on a console; networking, secure code generation, rendering and controller navigation are TODO |
| Dashboard rendering / controller UI | Not implemented (the Debian app's Dashboard Studio preview is the reference design) |

## Verify what can be verified here
```sh
cd apps/ps4-companion && make host-test      # needs only a C compiler
```

## Building for a console (not verified)
Prerequisites: [OpenOrbis PS4 Toolchain](https://github.com/OpenOrbis/OpenOrbis-PS4-Toolchain) (GPL-3.0, tested against its v0.5.4 layout only on paper), `clang` + `lld`, and a console that already runs a homebrew environment (see docs/firmware-compatibility.md; PS4 Studio does not install or trigger any exploit). Then:
```sh
OO_PS4_TOOLCHAIN=/path/to/OpenOrbis make ps4
```
Packaging into a `.pkg` follows the toolchain's own documentation (create-fself, create-gp4, pkg tool). Check every flag against your toolchain version before relying on it.

## Security properties (and limits)
- Pairing: a one-time 8-digit code shown on the TV; mutual HMAC-SHA256 proofs over fresh nonces; 3 wrong attempts lock pairing.
- Every later frame is MAC'd with a per-connection key, direction-labelled, with a strictly increasing sequence number (replay/reflection resistant).
- Only seven commands exist: `status, list_themes, put_theme, delete_theme, get_config, set_config, diagnostics`. No shell, no file read/write, no launch. `diagnostics` reports only values the app really has (the simulator reports none).
- **Authenticated but NOT encrypted**, and the code is short: a passive observer on your LAN who records a pairing could brute-force the code offline. Pair only on a network you trust. A PAKE (e.g. SPAKE2) would remove this weakness; it is not implemented.

## License
GPL-3.0-or-later (links against GPL OpenOrbis components).
