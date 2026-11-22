# Connection pathways and protocols

| Pathway | Protocol | Status | Capabilities it can unlock |
|---|---|---|---|
| A. Stock console | UDP 987 `SRCH` (verified from chiaki-ng `discovery.c`) | implemented | `console.discovery`, `console.basic_status` |
| B. Remote Play | launches installed chiaki-ng (separate process) | implemented (fake-binary tested) | `console.remote_play` |
| C. Homebrew file service | FTP (GoldHEN README: port 2121, user-enabled), passive mode | implemented (simulator only) | `homebrew.ftp_read`, `homebrew.ftp_write` |
| D. Companion | planned authenticated protocol (P6) | not implemented | `companion.*` |
| E. Simulator | in-process UDP responder + FTP server, 14 scenarios | implemented | all of the above, labelled SIMULATED |

Facts → capabilities is a pure function (`core::capability::evaluate`). A capability is only `available` when a probe produced evidence; an open port alone yields `unavailable`.

Discovery wire format: `SRCH * HTTP/1.1\ndevice-discovery-protocol-version:00020020\n` → reply `HTTP/1.1 200 Ok` (ready) or `620` (standby) with `host-name`, `host-type`, `host-id`, `system-version`, `running-app-*` headers.
