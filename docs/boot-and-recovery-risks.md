# Boot and recovery risks

**No step in this document is a procedure. PS4 Studio will not perform, script or assist with any of it.** This lists why the experimental track is high-risk so a future, separately reviewed procedure can be judged against it.

| Risk | Why it matters | Mitigation required before any hardware test |
|---|---|---|
| Console cannot be recovered | The PS4's boot chain is signed and hardware-bound [U]; a damaged system area may not be restorable without Sony service tools | Use a dedicated, expendable console only |
| Data loss | Internal HDD/SSD holds saves and games; partition mistakes are unrecoverable | Back up with Sony's own tools first; disconnect or replace the drive for experiments |
| Online bans / account action | Running exploits or modified firmware can lead to PSN restrictions [U] | Keep the console offline; never use it for the user's main account |
| Kernel exploit instability | Exploit success is probabilistic and firmware-specific [U]; failed attempts can crash/freeze the console | Procedure review, known-good firmware only, power-loss precautions |
| Firmware extraction for drivers | Some Wi-Fi/GPU firmware may have to be sourced from Sony's OS [V: README] | Legal review; PS4 Studio will never redistribute it |
| Loss of Sony's OS and games while Linux runs | Approach C is not a way to run PS4 games natively | Set expectations in the UI and docs |
| Heat/fan control | Fan curves and thermal sensors under Linux are unverified [U] | Monitor temperatures externally |

## Rules this project follows
1. No flashing, boot-chain replacement, kernel exploit, system-partition overwrite or similar in normal development.
2. Any hardware experiment needs its own reviewed written procedure, explicit approval, a dedicated console, and a recovery plan.
3. The simulator and the Console UI simulation are the only things the app ships for this track.
