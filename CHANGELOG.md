# Changelog

## 0.1.0 (unreleased)
First public version. Developed and tested against the built-in simulator; **no physical console testing**.

- Desktop app (Tauri 2, Svelte 5, Rust): Overview, Connections, Remote Control, Dashboard Studio, Homebrew, File Manager, Backups, Diagnostics, Settings.
- Capability engine: typed capabilities verified by probes, each unavailable feature explains why.
- Discovery (UDP 987) and chiaki-ng launcher (separate process, no secret flags).
- Hardened FTP client, remote path policy (writes only in `/data`), plan/confirm/apply/verify flow with pre-change backups and audit log.
- Dashboard Studio with four original templates, JSON-Schema-validated themes, import/export, 1080p preview.
- Backups with checksummed manifests and per-item restore (settings, profiles, themes).
- Companion protocol v1 (Rust + portable C), simulator companion; PS4 console app is an untested skeleton.
- Console UI simulation (experimental) and alternative-OS research docs.
- Debian package (`.deb`).
