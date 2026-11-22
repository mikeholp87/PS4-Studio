# Firmware compatibility

PS4 Studio **does not assert a supported firmware range for homebrew features.** Reasons:
- GoldHEN releases are published per firmware (2.3 assets: 5.05, 6.72, 9.00); the 2.4b18 notes give no range; the exploit it can be loaded with (PPPwn) states "up to FW 11.00" and lists specific versions. These are the *user's* prerequisite, outside this app, and change over time.
- Stock features (discovery, Remote Play via chiaki-ng) depend on chiaki-ng's own support matrix, which changes per Sony firmware.

## How the app handles firmware
1. Firmware shown in UI is one of: `unknown` (default), `reported` (derived from discovery `system-version`, coarse), or `user-entered`. Never `verified` from these sources alone.
2. Capabilities that need homebrew are gated by **probe results**, not by firmware.
3. Banner on every console profile: "Firmware compatibility has not been established."
4. A fixture-based table of known community-reported firmware ↔ feature results may be added later with source links; empty in v0.1.

## Sources (retrieved 2026-11-22)
- GoldHEN README / releases: https://github.com/GoldHEN/GoldHEN
- PPPwn README (firmware list): https://github.com/SiSTR0/PPPwn
- chiaki-ng `discovery.c` target mapping (system-version → PS4 8.0 / 9.0 / 10.x target): https://github.com/chiaki-ng/chiaki-ng
