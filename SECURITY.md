# Security policy

## Supported versions
Only the latest release receives fixes.

## Reporting a vulnerability
Please **do not open a public issue**. Use GitHub's "Report a vulnerability" (private security advisory) on this repository. If that is unavailable, contact the maintainers through the address listed on the repository profile. Include steps to reproduce, affected version, and impact. We aim to acknowledge within 7 days.

## Scope notes
In scope: the desktop app, simulator, protocol implementations (Rust and C), packaging. Out of scope: vulnerabilities in chiaki-ng, GoldHEN, homebrew environments, or Sony software (report those upstream), and issues requiring a malicious local administrator.

Design and known limitations (FTP and the companion link are not encrypted; the companion code is short) are documented in [docs/security.md](docs/security.md); please read it before reporting those as new findings.
