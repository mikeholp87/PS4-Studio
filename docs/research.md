# Research notes

Retrieved 2026-11-22 via GitHub API / web search. Tag legend: **[V]** verified from primary source in this session, **[U]** unverified (from memory/secondary; do not rely on it in code).

## Projects examined
| Project | License [V] | Latest activity [V] | Relevance |
|---|---|---|---|
| [chiaki-ng](https://github.com/chiaki-ng/chiaki-ng) | AGPL-3.0 (source files carry `LicenseRef-AGPL-3.0-only-OpenSSL`) | v1.10.1, 2026-11-03 | Maintained Remote Play client for PS4/PS5 on Linux. We launch it; we do not embed it. |
| [GoldHEN](https://github.com/GoldHEN/GoldHEN) | none declared in repo | 2.4b18, 2024-10-28 (prerelease line; "latest" tag is 2.3) | Homebrew enabler. README lists FTP :2121, BinLoader :9090, Klog :3232, Remote Package Install, `/data/pkg` install, plugins. |
| [OpenOrbis PS4 Toolchain](https://github.com/OpenOrbis/OpenOrbis-PS4-Toolchain) | GPL-3.0 | v0.5.4, 2026-01-04 (beta; clang+lld required) | Only open toolchain for native PS4 homebrew without Sony SDK. |
| [Itemzflow](https://github.com/LightningMods/Itemzflow) | GPL-3.0 | 1.08, 2026-05-31 | Existing "home menu alternative" running as homebrew. Prior art for Approach B. Also bundles game-dumping features; PS4 Studio will not integrate or imitate those. |
| [Clean Your TV](https://github.com/Gathin23/cleanyourtv) | NOASSERTION (no clear license) | v3.6.0, 2026-11-20 | Android/Google TV debloat + launcher over ADB. Prior art for the *TV* module only. Cannot copy code (no license grant). |
| [PPPwn](https://github.com/SiSTR0/PPPwn) | see repo | — | Kernel exploit that GoldHEN 2.4b* can be loaded by. README: PS4 up to FW 11.00. **PS4 Studio never installs or triggers exploits.** Mentioned only to explain why firmware compatibility belongs to the user's existing setup. |
| [fail0verflow/ps4-linux](https://github.com/fail0verflow/ps4-linux) | NOASSERTION | last push 2019-05 | Original Linux-on-PS4 kernel fork. |
| [feeRnt/ps4-linux-12xx](https://github.com/feeRnt/ps4-linux-12xx) | NOASSERTION | 2026-11-16 | Active fork; targets CUH-12xx (original "Phat" revision) with Wi-Fi/BT/HDMI patches. |
| postmarketOS `sony-ps4` ([MR 5194](https://gitlab.postmarketos.org/postmarketOS/pmaports/-/merge_requests/5194/commits)) | — | 2024 snapshot | Added **Phat** only; "Slim and Pro not yet supported"; needs kernel exploit + PS4 Linux Loader; "kernel boots, initramfs needs ps4 amdgpu firmware". |

## Remote Play wire facts [V from chiaki-ng `lib/src/discovery.c`, `lib/include/chiaki/discovery.h`]
- PS4 discovery: UDP **987**, protocol version string `00020020`. (PS5: 9302 / `00030010` — out of scope.)
- Search packet (ASCII): `SRCH * HTTP/1.1\ndevice-discovery-protocol-version:00020020\n`
- Reply is HTTP-like: status `200` = ready, `620` = standby; headers include `host-name`, `host-type`, `host-id`, `system-version`, `host-request-port`, `running-app-titleid`, `running-app-name`, `device-discovery-protocol-version`.
- `system-version` integer: ≥8000000 → PS4 8.0+, ≥7000000 → 7.0+. Only a coarse hint; PS4 Studio shows it as "reported by console", never as verified firmware compatibility.
- Wake: `WAKEUP * HTTP/1.1` with `user-credential:<u64>` — requires a credential obtained during registration. PS4 Studio will not implement wake in v0.1 (credential handling is chiaki-ng's job); capability `console.remote_wake` = UNSUPPORTED until a safe credential source exists.
- Registration (PIN from console "Add Device") and streaming are handled entirely inside chiaki-ng. Port numbers for those were not extracted in this pass **[U]**; not needed since we do not speak them.

## chiaki-ng CLI surface [V from `gui/src/main.cpp`]
Binary target is `chiaki` (CMake `add_executable(chiaki …)`); Flathub id `io.github.streetpea.Chiaki4deck`.
Commands: `stream <nickname> <host>`, `list`, `discover`, `wakeup`. Options include `--profile`, `--fullscreen`, `--zoom`, `--stretch`, `--exit-app-on-stream-exit`, `--opengl`, and secret-bearing `--registkey`, `--morning`, `--passcode`.
**Security consequence:** argv is world-readable via `/proc/<pid>/cmdline`. PS4 Studio must never pass `--registkey/--morning/--passcode`; users register inside chiaki-ng once, and we launch `stream <nickname> <host> [--fullscreen]` using its stored profile.

## GoldHEN [V from README + release notes]
- Features relevant to us: FTP server on 2121; BinLoader 9090 (experimental, can run arbitrary payloads — PS4 Studio will not use it); Klog 3232 (read-only log stream, candidate diagnostic); "Remote Package Install"; `/data/pkg` internal install; plugins.
- Release assets are per-firmware binaries for 2.3 (`505`, `672`, `900`); 2.4b18 notes do not state a firmware range. **No firmware range is asserted by PS4 Studio**; see `firmware-compatibility.md`.
- No documented authentication on the FTP service **[U]** → treat as plaintext, LAN-only, possibly anonymous.

## Remote Play (Sony) [U — page not fetched in this pass]
Sony support page URL given in brief. Known product behaviour from memory: Remote Play must be enabled in PS4 Settings, device added with an 8-digit code, console must be awake or in Rest Mode with network wake enabled. To be re-verified against the live page before README claims it.

## PS4 platform [U/secondary]
Orbis OS is FreeBSD-derived (9.x lineage); hardware is AMD Jaguar x86-64 + GCN "Liverpool" GPU (Slim = CUH-2xxx on the same family). Secure boot chain and hypervisor mean replacing the OS requires a kernel exploit per firmware; no public path preserves native commercial game compatibility. Detailed treatment lives in the alt-OS docs (P7).

## Open items before relevant code is written
1. Fetch Sony Remote Play page (P3).
2. Confirm GoldHEN FTP auth behaviour and directory layout from a primary source (P5) — until then the file manager treats root as read-only and requires a user-configured allowlist.
3. Verify OpenOrbis controller/network APIs when starting P6.
