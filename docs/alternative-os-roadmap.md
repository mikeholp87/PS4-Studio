# Alternative OS roadmap

Separate workstream. **Does not block the desktop release.**

| Stage | Goal | Hardware needed | State |
|---|---|---|---|
| 0 | Research + feasibility (this folder) | none | done (docs) |
| 1 | Console UI simulation on Debian: controller navigation, app launch, settings, themes, system info, media/game libraries, file browser, network settings — all clearly labelled simulation | none | done (Settings → Experimental) |
| 2 | Extract the shell into a standalone Wayland kiosk app on Debian (a "console mode" for a living-room PC) | a PC | not started |
| 3 | Native homebrew dashboard (Approach B) using the verified companion protocol | a homebrew-capable console + OpenOrbis | protocol done, console app untested |
| 4 | Reviewed, written boot experiment on a dedicated console (Approach C) | an expendable console | **not started; requires explicit approval and a separate procedure** |
| 5 | Decide whether a Linux console image is worth maintaining | — | depends on 4 |

Exit criteria for stage 4 to even be considered: signed-off procedure, dedicated console, verified firmware/model match from a primary source, recovery plan, legal review of any firmware blobs.
