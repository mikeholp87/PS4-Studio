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

## Fallback: Remote Play service probe (TCP 9295)
Observed on real hardware (2026-11-23, one PS4 on Wi-Fi): the console ignored every UDP 987 search (broadcast, unicast, several source ports) but answered an unauthenticated `GET /sce/rp/session` on TCP 9295 with `HTTP/1.1 403` and an `RP-Version` header. `discovery::probe_remote_play` sends exactly one such request (no credentials, no registration key), accepts only a reply with an `RP-Version` header of digits/dots, caps the reply at 2 KB and times out every read. It yields `DiscoveryFact { via: RemotePlayPort }`: console present, but name, standby/ready state and firmware are unknown. It is used by Connect on a saved console, not by the broadcast search. Why the UDP search was ignored is not established (a host firewall dropping replies is one untested possibility).
