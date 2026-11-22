# Alternative operating environment — research track

**Status: research and a desktop UI simulation only. PS4 Studio performs no boot, flash, exploit or firmware operation, and nothing in this repository should be read as a procedure for doing so.** Tags: **[V]** verified from a primary source this session (see `research.md`), **[U]** from memory/secondary, unverified.

## What the idea can and cannot mean
The PS4 runs Sony's Orbis OS (FreeBSD-derived [U]) on an AMD x86-64 SoC. "Building my own PlayStation-style OS" splits into three very different projects:

| | A. Debian launcher + Remote Play | B. Native homebrew dashboard beside the stock UI | C. Linux (or other OS) running on the console hardware |
|---|---|---|---|
| What it is | PS4 Studio today: a TV-style UI on Debian, PS4 streamed via chiaki-ng | A PS4 homebrew app (Itemzflow-like, OpenOrbis) shown from the stock UI | Replace the running OS with a Linux kernel on the PS4 |
| Needs a jailbreak/exploit | No | Yes (existing homebrew environment) | Yes, a kernel-level exploit on a supported firmware [V: kernel fork README says "exploitable PlayStation 4 systems"] |
| PS4 games keep working | Yes (streamed) | Yes (stock OS stays) | **No** while Linux runs; no native commercial game compatibility |
| Risk to the console | None | Low–moderate (homebrew crashes, bans if online [U]) | High (bricking risk, data loss, no supported recovery from Sony) |
| Legal/ToS | Clean | Grey: depends on jailbreak; PS4 Studio never installs exploits | Same, plus bootloader-level modifications |
| In this repo | Implemented | Protocol + untested skeleton (`apps/ps4-companion`) | Docs + UI simulation only |

## Evidence summary (C)
- Community kernel forks exist and are actively maintained. The `feeRnt/ps4-linux-12xx` README **[V]** lists working combinations for Phat (CUH-1xxx), **Slim (CUH-2215B Baikal → 5.4.247; CUH-2216A Baikal B1/Belize → 5.4.247 or 5.15.15)** and Pro (CUH-7xxx Baikal → 5.4.247), and names the three southbridge families Aeolia, Belize, Baikal. It says the kernel is "tailored to run on exploitable PlayStation 4 systems".
- postmarketOS has a `sony-ps4` device package (2024 MR) that covered Phat only at the time and noted "kernel boots, initramfs needs ps4 amdgpu firmware" **[V]**; i.e. distro packaging lags the community kernels.
- Original work by fail0verflow (2016–2019 kernel fork, last push 2019) **[V]** proved the hardware can run Linux.
- Wi-Fi/Bluetooth chips differ by revision (Marvell 88w8897 on some Phat, MediaTek 7668 on some Slim/Pro) and have needed per-model patches **[V]**.

## Things that are unknown or unverified for a Slim
GPU acceleration quality and firmware blobs, HDMI audio, power management/fan control, sleep/wake, internal drive controller on every revision, controller (DualShock 4) over USB vs. Bluetooth, and long-term stability **[U]**. These must be established by someone with a *dedicated, expendable* console under a separately reviewed procedure.

## Independent, legally distributable components for a custom UI
A PlayStation-style environment would be built from unrelated parts: Linux kernel (GPL-2.0) + Mesa/amdgpu (MIT/GPL) + a compositor (e.g. wlroots/cage, MIT) + a UI shell. PS4 Studio's Dashboard Studio renderer (GPL-3.0-or-later, original design, no Sony assets) is a candidate shell. Sony firmware, fonts, icons, sounds and the PS4 game ecosystem are **not** usable.

## Decision
Approach A is the product. Approach B is an optional, hardware-gated milestone. Approach C remains a research workstream that never blocks a release (see `alternative-os-roadmap.md`). The in-app **Console UI simulation** (Settings → Experimental) demonstrates what a custom shell could feel like without touching any console.
