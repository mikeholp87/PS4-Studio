# Technical feasibility & feature classification

Classes: **SUPPORTED** (works on stock PS4 + implemented), **CONDITIONALLY SUPPORTED** (needs a stated prerequisite, detected at runtime), **EXPERIMENTAL** (works only in simulator/unproven, off by default), **UNSUPPORTED** (disabled UI + explanation).
All entries are "simulator-tested only" until real hardware is used; none has been tested on a physical PS4.

| Feature | Class | Evidence / prerequisite |
|---|---|---|
| Offline simulator mode | SUPPORTED | Local code only |
| Dashboard Studio (design, preview, import/export) | SUPPORTED | Pure desktop app; no console needed |
| Theme profiles, JSON schema validation | SUPPORTED | Local |
| Local backups of settings/profiles/themes | SUPPORTED | Local files + SHA-256 manifest |
| Discovery of PS4 on LAN (UDP 987 SRCH) | CONDITIONALLY SUPPORTED | Wire format verified from chiaki-ng source. Needs console in Ready/Standby on same L2 network; Remote Play/network settings may affect replies **[U]** |
| Manual IP entry + reachability test | SUPPORTED | Uses same SRCH unicast; failure = "no reply", not "offline" |
| Basic status (name, ready/standby, running title) | CONDITIONALLY SUPPORTED | Only fields present in SRCH reply |
| Firmware version | CONDITIONALLY SUPPORTED | Coarse `system-version` hint or user-entered; labelled unverified |
| CPU temp, fan, storage, RAM on stock PS4 | UNSUPPORTED | No documented stock interface. Shown as "unavailable"; manual notes allowed |
| Remote Play launch | CONDITIONALLY SUPPORTED | Needs installed chiaki-ng (AGPL, separate process), Remote Play enabled on PS4, registration done in chiaki-ng. Not installed on dev machine → launcher unit-tested with a fake binary only |
| Remote Play pairing inside PS4 Studio | UNSUPPORTED (v0.1) | Would require handling registration secrets/PSN account id; delegate to chiaki-ng, show guide |
| Remote wake | UNSUPPORTED (v0.1) | Needs registration credential; see research.md |
| Controller (DS4) detection/guidance | CONDITIONALLY SUPPORTED | `gilrs`/evdev; mapping UI only displays detected pads; chiaki-ng does the input |
| GoldHEN FTP read | CONDITIONALLY SUPPORTED | User already runs GoldHEN with FTP on (default port 2121 per README). Verified by real FTP handshake + `LIST`, never by port-open |
| GoldHEN FTP write | CONDITIONALLY SUPPORTED | Above + explicit per-session write unlock + allowlisted dirs + audit log |
| Package install via `/data/pkg` | EXPERIMENTAL | README mentions it; install semantics unverified. Implemented as "upload file to allowlisted dir + instruct user", no auto-trigger |
| BinLoader (9090) payload sending | UNSUPPORTED | Arbitrary payload execution; deliberately excluded |
| Klog (3232) read-only viewer | EXPERIMENTAL | Plain TCP log stream; sanitised before display |
| Plugin management | EXPERIMENTAL | Plugins are files in a GoldHEN dir; layout unverified → file-manager level only |
| PS4 Studio companion (OpenOrbis) | EXPERIMENTAL | Source provided, not built/tested here; needs jailbroken console + toolchain |
| Companion theme transfer | EXPERIMENTAL | Rust protocol + mock only |
| Launching PS4 apps from companion | UNSUPPORTED until verified | Requires verified API; not claimed |
| Replacing official PS4 home screen | UNSUPPORTED | Not possible without system-file modification; explicitly out of scope |
| Android/Google TV (ADB) module | CONDITIONALLY SUPPORTED | Needs `adb` + TV with authorized network debugging; separate from PS4 features |
| HDMI-CEC | EXPERIMENTAL | Needs CEC adapter/driver; deferred |
| Backup of stock PS4 saves/OS | UNSUPPORTED | Sony's own backup only; app shows instructions |
| Linux on PS4 Slim (Approach C) | EXPERIMENTAL / research | Community kernel fork README lists Slim CUH-2215B/2216A as working with kernels 5.4.247/5.15.15 (Baikal/Belize southbridges); needs a kernel exploit on a supported firmware; Sony OS and its games are not available while Linux runs; hardware-risky. Docs only in PS4 Studio |

## Approaches (alt-OS summary; details in P7 docs)
| | A: Debian launcher + Remote Play | B: Native homebrew dashboard | C: Linux on PS4 |
|---|---|---|---|
| Needs jailbreak | No | Yes | Yes (kernel exploit) |
| Risk to console | None | Low–moderate (homebrew only) | High |
| Keeps PS4 games | Yes | Yes | **No** |
| Status here | Build now | Source only | Docs only |
