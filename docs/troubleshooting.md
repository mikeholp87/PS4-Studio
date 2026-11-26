# Troubleshooting

| Message / symptom | Meaning and fix |
|---|---|
| "Console not found. Confirm that the Debian computer and PS4 are on the same local network." | Discovery got no reply. **Some consoles ignore the UDP search even when Remote Play is on** (seen on a real PS4 on 2026-11-23): when you press Connect on a saved console, the app also checks the Remote Play service (TCP 9295) and treats a genuine Remote Play reply as "console present". "Search local network" uses only the UDP broadcast, so on such a console add it by IP instead (PS4: Settings → Network → View Connection Status). Check same subnet/VLAN, PS4 on or in Rest Mode, no guest-Wi-Fi isolation, firewall allowing UDP 987 replies. A silent console is shown as "no reply", never as "offline". |
| "chiaki-ng was not found…" | Install chiaki-ng (Flathub or GitHub releases), then reopen Remote Control. |
| Remote Play launches but fails | Enable Remote Play on the PS4 (Settings → Remote Play Connection Settings), register the console inside chiaki-ng, and use the exact nickname. PS4 Studio never passes your registration key or passcode. |
| "File management is unavailable because no supported file service was detected." | No FTP service completed a handshake. This only exists on consoles already running a homebrew environment with FTP enabled (documented default port 2121). Check the port under Settings if yours differs (`ftp_port`). |
| "A port is open but it did not complete an FTP handshake" | Something listens but it is not FTP. Not treated as a file service. |
| "Writes are locked." | Use File Manager → Enable changes… (acknowledge FTP is unencrypted). Changes are only allowed inside `/data`. |
| "That plan has expired or was already used." | Plans last 5 minutes and are single-use. Review the change again. |
| Transfer failed midway | The connection is dropped and reopened; uploads use a `.ps4studio-part` file and overwrite via rename-swap (`.ps4studio-bak`), so the real name is never deleted before the new file is in place. Check the console file list; a local recovery copy may remain under a temp folder named in the error; Backups → Operation history has recovery notes. |
| "the pairing code was rejected" (companion) | 3 wrong attempts lock pairing until the companion restarts. |
| "Companion … uses protocol N; this app needs 1" | Update the companion or the app. |
| Gamepad not detected in the preview | Press a button once so the browser exposes it; check `/dev/input` permissions. |
| Blank window on some Wayland/GPU setups | Try `WEBKIT_DISABLE_COMPOSITING_MODE=1 ps4-studio`. |
| Export a report for a bug | Diagnostics → Export sanitised report (IPs masked, ids and credential-like lines removed). |
