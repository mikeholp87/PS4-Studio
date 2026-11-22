<!-- Recovered verbatim from Claude Code session 195c2a5a-cb4a-4b40-b987-eb35bacc41c6 on 2026-11-22. Historical context; claims require verification before integration. -->

# PROJECT: PS4 Studio
## Open-Source PlayStation Management and Custom Dashboard for Debian Linux

You are a senior Rust developer, Linux desktop engineer, systems programmer, UI designer, and open-source console homebrew developer.

Your task is to research, architect, build, test, and package a complete desktop application called PS4 Studio.

PS4 Studio will run natively on Debian Linux and connect to a user's PlayStation 4 console over a local network.

The primary test console is a PlayStation 4 Slim.

The project is inspired by Clean Your TV, but the goal is to create an original application designed specifically for PlayStation hardware.

I want more control over my console, the ability to customize its interface, manage supported homebrew features, and eventually experiment with building my own PlayStation-style operating environment.

This must become a working, polished, publicly distributable open-source application.

Do not stop at writing a project plan or mockup. Implement the software incrementally and verify every component that can be tested in the development environment.

---

# 1. CORE OBJECTIVES

Create a Debian desktop application that can:

1. Discover compatible PlayStation consoles on the local network.
2. Connect to a PS4 through supported network protocols.
3. Help users establish Remote Play sessions.
4. Identify supported management capabilities.
5. Display available console information.
6. Manage supported console files and homebrew content.
7. Create backups before making changes.
8. Customize supported application-level themes.
9. Provide a custom PlayStation-style dashboard.
10. Support a native PS4 homebrew companion application when the console permits it.
11. Offer diagnostic and troubleshooting tools.
12. Provide a safe way to restore previous configurations.
13. Offer an optional experimental operating-system research module.

Design the project so ordinary PS4 owners can use its standard features without jailbreaking their consoles.

Clearly distinguish features available on stock PS4 systems from those requiring an existing compatible homebrew environment.

Never imply that Remote Play gives filesystem access or that connecting over HDMI provides direct operating-system access.

---

# 2. RESEARCH BEFORE IMPLEMENTATION

Examine these projects and their official documentation:

Clean Your TV:
https://github.com/Gathin23/cleanyourtv

GoldHEN:
https://github.com/GoldHEN/GoldHEN

OpenOrbis PS4 Toolchain:
https://github.com/OpenOrbis/OpenOrbis-PS4-Toolchain

Chiaki-ng:
https://github.com/chiaki-ng/chiaki-ng

Itemzflow:
https://github.com/LightningMods/Itemzflow

PlayStation Remote Play:
https://www.playstation.com/en-us/support/games/remote-play-settings/

Research and document:

- PS4 Orbis OS architecture and its FreeBSD foundations.
- Console network discovery and Remote Play pairing.
- Existing Linux-compatible Remote Play implementations.
- Supported PS4 homebrew development toolchains.
- Current compatibility requirements for GoldHEN.
- GoldHEN network services and documented capabilities.
- Native PS4 application development and packaging.
- Constraints on third-party PS4 launchers.
- Linux-on-PS4 experiments and their hardware limitations.
- Secure boot restrictions and the feasibility of replacement operating systems.
- Existing projects that provide custom PS4 home interfaces.
- Licensing obligations when integrating existing open-source code.

Verify the relevant repositories, documentation, licenses, and compatibility information before implementing integrations.

Do not rely on an assumed firmware compatibility range.

Create:

docs/research.md
docs/technical-feasibility.md
docs/firmware-compatibility.md
docs/dependencies-and-licenses.md

Every proposed feature must be classified as:

SUPPORTED
CONDITIONALLY SUPPORTED
EXPERIMENTAL
UNSUPPORTED

Document the actual technical evidence behind each classification.

If a feature cannot be safely implemented, leave a clearly described and disabled interface rather than inventing functionality.

---

# 3. TECHNOLOGY STACK

Primary language:
Rust, using the current stable toolchain.

Desktop framework:
Tauri 2.

Frontend:
Svelte 5 and TypeScript.

Styling:
Modern CSS with reusable design tokens.

Build tooling:
Vite, Cargo, and appropriate Tauri tooling.

Persistence:
SQLite for application metadata and local settings.

Secrets:
Linux Secret Service through a supported, maintained credential-storage library.

Testing:
Rust unit and integration tests, Vitest for frontend tests, and Playwright where practical.

Primary operating system:
Debian Linux, on both KDE Plasma and GNOME.

Display support:
X11 and Wayland.

Architectural targets:
x86_64 first, with aarch64 support where practical.

Output packages:
- Debian .deb
- AppImage where supported
- Source build
- Optional Flatpak

Do not use Electron.

The finished application should run as an ordinary user. No root permissions should be required for normal operation.

If a feature requires elevated permissions, isolate that requirement and explain it before requesting authorization.

Select maintained dependencies and pin reproducible versions.

---

# 4. APPLICATION ARCHITECTURE

Design the application as a modular monorepo.

Suggested structure:

ps4-studio/
  apps/
    desktop/
      src/
      src-tauri/
      public/
    ps4-companion/
      src/
      include/
      assets/
  crates/
    core/
    discovery/
    remote-play/
    console-adapters/
    file-transfer/
    backups/
    diagnostics/
    capability-engine/
    security/
  packages/
    ui/
    theme-engine/
    shared-types/
  simulator/
    mock-console/
    fixtures/
  assets/
    icons/
    themes/
  packaging/
    debian/
    appimage/
    flatpak/
  docs/
    architecture.md
    research.md
    technical-feasibility.md
    firmware-compatibility.md
    connection-protocols.md
    security.md
    installation.md
    troubleshooting.md
    development.md
    licenses.md
  tests/
  scripts/
  .github/
    workflows/
  README.md
  CONTRIBUTING.md
  LICENSE
  AGENTS.md
  CLAUDE.md

Use a clean separation between the UI, console transport implementations, capability detection, and device operations.

The desktop application must not depend on successful console access to start.

It must provide a fully functional offline demonstration mode.

---

# 5. CONNECTION SYSTEM

Create a connection wizard.

Supported connection pathways:

A. Stock PS4 network connection
B. Remote Play connection
C. GoldHEN-enabled console services
D. Optional PS4 Studio native companion
E. Offline simulator

Each pathway must have distinct capabilities.

## Network discovery

Implement documented PlayStation discovery methods where practical.

Support:

- Local network discovery.
- Manual console IP entry.
- Connection status.
- Connection testing.
- Stored console profiles.
- Multiple console profiles.
- Connection timeouts and retries.
- Explicit user authorization.

Use existing documented discovery protocols and libraries where appropriate.

Do not assume that an ordinary PS4 exposes SSH, ADB, FTP, or HTTP management endpoints.

Only probe the specified local network using narrowly scoped, noninvasive discovery.

## Remote Play

Research Chiaki-ng and its underlying libraries.

Determine the most practical integration strategy:

Option A: Launch a locally installed Chiaki-ng executable with safe, documented arguments.

Option B: Integrate its libraries directly if technically justified and compatible with the project's licensing.

Start with the simpler, more reliable approach.

Support guided pairing using the PS4's normal Remote Play registration interface.

Where supported:

- Discover consoles.
- Register a device.
- Launch a Remote Play session.
- Display connection status.
- Support controller input.
- Support keyboard navigation when compatible.
- Disconnect cleanly.

Remote Play credentials must never appear in ordinary logs.

Do not silently collect PlayStation account credentials.

## Modified console support

For users who already have a supported homebrew setup, implement optional adapters for available services.

Research documented GoldHEN functionality, including the optional FTP service commonly configured on port 2121.

Do not assume this service is enabled.

Detect and verify individual capabilities before enabling them.

Never silently install exploits or modify console firmware.

---

# 6. CAPABILITY ENGINE

Implement a capability-based system instead of assuming every PS4 supports the same operations.

Represent each console using:

- User-selected console model.
- Network address.
- Reported or manually entered firmware version.
- Available connection protocols.
- Paired Remote Play status.
- Available homebrew services.
- Installed companion version, if present.
- Last successful connection.
- Supported operations.

Use a typed capability model.

Example capabilities:

console.discovery
console.remote_play
console.remote_wake
console.basic_status
homebrew.ftp_read
homebrew.ftp_write
homebrew.package_install
homebrew.plugin_management
homebrew.app_launch
companion.connected
companion.dashboard
companion.theme_management

Each capability must be independently validated.

The UI must show why unavailable features are disabled.

For example:

"Custom console dashboard requires a compatible homebrew environment."

"File management is unavailable because no supported file service was detected."

"Firmware version has not been verified."

Never use a fake successful response or infer a feature solely from an open network port.

---

# 7. USER INTERFACE AND DESIGN SYSTEM

Build an original, professional desktop application.

The interface should feel similar in quality to modern KDE system utilities, with the polish of a premium desktop application.

Do not copy Sony's proprietary interface assets or Clean Your TV's visual identity.

Create your own distinctive design.

Working visual direction:

Application name: PS4 Studio

Primary theme:
Deep charcoal with subtle blue and violet accents.

Support:
- Dark theme.
- Light theme.
- System theme.
- Custom accent colors.

The interface should have:

- A narrow navigation sidebar.
- A clear header.
- A prominent connection-status indicator.
- Consistent cards and tables.
- Appropriate empty states.
- Well-designed dialogs.
- Smooth, restrained transitions.
- Responsive layouts.
- Clear text hierarchy.
- Accessible focus states.
- Keyboard navigation.
- High-DPI display support.

Use Lucide icons or an equivalent appropriately licensed icon set.

Do not add unnecessary animation.

The application must remain responsive during large file transfers and network operations.

## Main application navigation

Create these sections:

1. Overview
2. Connections
3. Remote Control
4. Dashboard Studio
5. Homebrew
6. File Manager
7. Backups
8. Diagnostics
9. Settings

Each section must be functional within its supported capability tier.

---

# 8. OVERVIEW DASHBOARD

The main dashboard should display:

Console profile:
- Console name
- Console model
- Connection type
- Network address
- Online or offline status
- Firmware version, if verified
- Homebrew status, if verified
- Available capabilities

Quick actions:
- Connect
- Disconnect
- Start Remote Play
- Open Dashboard Studio
- Browse Files
- Create Backup
- Open Diagnostics

Display supported console information using real data.

For stock consoles, avoid implying access to system-level measurements unless a documented interface provides them.

If CPU temperature, storage capacity, fan status, or similar values cannot be obtained, show a clear unavailable state.

Allow users to manually record certain information.

---

# 9. REMOTE CONTROL CENTER

Build a dedicated Remote Control section.

The main goal is to provide convenient PS4 control from Debian Linux using supported Remote Play technology.

Implement:

- Pairing instructions.
- Console selection.
- Session launch.
- Connection management.
- Resolution and quality preferences, where supported.
- Controller guidance.
- Full-screen Remote Play.
- Graceful disconnection.
- Connection error reporting.

Do not recreate the entire Remote Play protocol if a maintained library already provides a suitable implementation.

Avoid fragile simulated button presses or undocumented network commands.

Where practical, integrate a controller-mapping interface.

Support the DualShock 4 controller through Linux input facilities using appropriate libraries.

---

# 10. CUSTOM DASHBOARD STUDIO

This is a major feature.

I want to create a custom PlayStation-style user interface.

Create a visual dashboard designer that works on my Debian computer.

The designer must be usable offline.

The user should be able to build and preview dashboard themes without having a connected PS4.

Features:

- Theme creation.
- Theme editing.
- Background customization.
- Layout selection.
- Application tiles.
- Custom shortcuts.
- Icon selection.
- Tile sorting.
- Font customization.
- Color customization.
- Clock widgets.
- Status widgets.
- Navigation settings.
- Controller focus preview.
- Theme export and import.
- Theme reset.

Provide several original dashboard templates:

MINIMAL
A compact interface with simple horizontal navigation.

AURORA
A dark interface with soft gradients and translucent elements.

GRID
A modern app grid with adjustable columns.

CLASSIC
A straightforward controller-friendly layout inspired by traditional console navigation, without copying protected assets.

Allow users to save multiple profiles.

## Dashboard preview

Build an interactive preview that simulates a television display.

Provide:

- 1080p design preview.
- Controller navigation simulation.
- Keyboard arrow navigation.
- Focus animations.
- Theme switching.
- Live changes.
- Import and export of custom themes.

Persist themes as versioned configuration files.

Use validated JSON schemas.

No imported theme should be allowed to execute arbitrary code.

## Deployment

If a compatible PS4 companion is available, allow transferring supported themes to it.

If the console cannot run custom homebrew, keep the dashboard available as a Debian application.

Do not claim that changing the theme overrides the official PS4 home screen.

---

# 11. NATIVE PS4 COMPANION APPLICATION

Investigate whether a small native PS4 homebrew companion can be built using OpenOrbis.

Implement this as a separate project within the repository.

Use C or C++ where required by the supported PS4 toolchain.

The companion should:

- Run as a normal homebrew application.
- Display an original dashboard.
- Load supported theme packages.
- Navigate using the PS4 controller.
- Manage its own configuration files.
- Show available applications through supported APIs.
- Launch applications only if verified APIs permit it.
- Communicate with the Debian desktop app through an authenticated local-network protocol.

Build a lightweight interface suitable for the PS4's hardware.

Avoid unnecessary CPU, GPU, and memory consumption.

Use documented native APIs wherever possible.

The companion must not modify the official PS4 shell or replace protected system files.

Do not install services that automatically run with privileged access.

If the OpenOrbis environment is not available, provide working source code, build scripts, and clear prerequisites, while ensuring that the desktop application still builds successfully.

## Companion communication

Use an explicitly enabled local-network service.

Implement pairing through a displayed code or another secure authorization method.

Require authentication for management operations.

Prefer authenticated, encrypted communication when supported by the runtime and environment.

Use an allowlisted command set.

Prevent remote arbitrary command execution.

Do not expose unrestricted filesystem operations.

Design a versioned protocol for:

- Connection status.
- Companion version.
- Theme list.
- Theme transfer.
- Supported application metadata.
- Diagnostic data.
- Companion configuration.

---

# 12. FILE MANAGER

Create a dual-pane file manager for consoles that expose an authorized, supported file service.

Left pane:
Local Debian filesystem.

Right pane:
Remote console files.

Features:

- Browse directories.
- Transfer files.
- Create permitted directories.
- Rename supported files.
- Delete permitted files after confirmation.
- Transfer progress.
- Pause and cancel where supported.
- Transfer queue.
- Retries and timeouts.
- Checksums where supported.
- Error reporting.

Never assume that FTP traffic is encrypted.

Warn users when connecting through an unencrypted service.

Default to read-only access when initially connecting.

Require explicit authorization before enabling writes.

Use an allowlist for managed application directories.

Do not expose protected firmware partitions as ordinary editable folders.

Prevent path traversal and directory escapes.

Never execute downloaded files automatically.

---

# 13. HOMEBREW MANAGER

Build a homebrew management interface for compatible consoles.

Include:

- Installed homebrew list, when retrievable.
- Locally downloaded packages.
- Package metadata.
- Package compatibility warnings.
- Installation history.
- Installation status.
- Optional plugin management.
- Backup of supported configurations.

Use documented services and interfaces.

Do not guess package-installation endpoints.

The application must never download pirated games or unauthorized copyrighted content.

Use trusted distribution sources.

Validate package metadata and integrity before installation.

Show explicit warnings for unfamiliar packages and untrusted publishers.

Never automatically execute arbitrary payloads.

If a console does not expose an appropriate installation interface, disable installation and explain the requirements.

---

# 14. BACKUP AND RESTORE

Implement a backup manager with a strong focus on recoverability.

Supported backup categories:

- PS4 Studio settings.
- Connection profiles without exported secrets.
- Custom themes.
- Companion settings.
- User-authorized files.
- Supported homebrew configuration files.

For stock PS4 consoles, provide instructions for Sony's built-in backup and restore process.

Do not suggest that ordinary Remote Play can back up internal save data or protected operating-system files.

Features:

- Create backup.
- Browse backups.
- Restore supported backups.
- Backup verification.
- Checksums.
- Date and time.
- File manifest.
- Operation history.

Use versioned backup manifests.

Support restoring individual configuration items.

Before any remote modification, show what will change.

Create an operation record with its expected recovery procedure.

Do not promise automatic rollback of operations for which reliable rollback is impossible.

---

# 15. DIAGNOSTICS

Create a technical diagnostics panel.

Include:

- Network discovery results.
- Connection latency.
- Connection errors.
- Available service checks.
- Remote Play pairing status.
- File-service connectivity.
- Companion compatibility.
- Application logs.
- Local dependency checks.
- Software version checks.

Where supported by a companion, display:

- Available memory.
- Storage information.
- Application version.
- Other verified runtime measurements.

Do not invent hardware metrics.

Provide human-readable error explanations.

Examples:

"Console not found. Confirm that the Debian computer and PS4 are on the same local network."

"Remote Play is disabled in PS4 settings."

"FTP is unavailable. This feature requires an enabled supported service."

"Console firmware compatibility has not been established."

Provide a button to export a sanitized diagnostic report.

Remove credentials, tokens, and sensitive identifiers from exported logs.

---

# 16. OPTIONAL TV INTEGRATION

The user may have an Android TV or Google TV connected to the PS4 through HDMI.

Explore an optional TV integration module.

Treat this as a completely separate device adapter.

If the television supports authorized Android Debug Bridge access, allow:

- Detecting the television.
- Pairing through the TV's supported authorization process.
- Reading basic supported device information.
- Launching installed TV applications.
- Managing compatible launcher settings.
- Switching TV inputs only when an actual supported interface exists.

Research Clean Your TV's approach.

Do not assume ADB works on the PlayStation.

Do not assume all TCL or other smart televisions run Android TV.

Do not treat control over the TV as access to the PlayStation operating system.

Do not automatically disable system applications.

Keep all TV-specific features optional and separate from PS4 management.

Research HDMI-CEC integration only as an optional capability requiring compatible hardware and software.

---

# 17. EXPERIMENTAL OPERATING-SYSTEM PROJECT

Create a separate research track for a possible alternative PlayStation operating environment.

The long-term idea is to investigate whether the PS4 Slim could run a more open, customizable interface or Linux-based environment.

Do not assume Sony's proprietary system can simply be rebuilt.

Start by researching:

- PlayStation 4 hardware architecture.
- Secure boot and signing requirements.
- Existing Linux-on-PS4 projects.
- Hardware compatibility.
- GPU driver availability.
- Controller support.
- Network support.
- Audio support.
- Storage compatibility.
- Boot and recovery limitations.
- Existing legal, independently developed operating-system components.

Compare three distinct approaches.

APPROACH A:
A custom desktop launcher running on Debian and controlling the PS4 through Remote Play.

APPROACH B:
A native PS4 homebrew dashboard running alongside the stock Sony interface.

APPROACH C:
An experimental Linux-based operating environment running on compatible PS4 hardware.

Create a feasibility matrix for each approach.

Do not attempt a firmware flash, boot-chain replacement, kernel exploit, system partition overwrite, or similar high-risk operation during normal application development.

Do not treat a Linux boot experiment as equivalent to replacing the proprietary OS while preserving native commercial PS4 game compatibility.

For the experimental track, first build a fully functional Linux desktop simulation of the proposed console UI.

The simulator should demonstrate:

- Controller navigation.
- Application launching.
- Settings management.
- Theme management.
- System information panels.
- Media library interface.
- Game library interface.
- File browsing.
- Network settings interface.

Use independently developed and legally distributable components.

Create:

docs/alternative-os-research.md
docs/hardware-compatibility.md
docs/boot-and-recovery-risks.md
docs/alternative-os-roadmap.md

Do not perform experimental boot or firmware operations on a real console without a separately reviewed and explicitly approved procedure.

---

# 18. SECURITY REQUIREMENTS

Treat security as a core architectural requirement.

Implement:

- Least-privilege operation.
- Secure credential storage.
- Explicit connection approval.
- Strict Tauri command permissions.
- Network input validation.
- Timeouts for all network operations.
- File path validation.
- Download integrity checks.
- Secure temporary files.
- Sanitized logging.
- Strong separation between read and write capabilities.
- Confirmation for destructive operations.

Do not implement arbitrary remote shell execution.

Do not provide unprotected network management endpoints.

Do not bypass PlayStation Network authentication.

Do not implement DRM circumvention, pirated game distribution, or mechanisms to evade console bans.

Do not disable console security services automatically.

Do not modify official system firmware.

Develop a safe operation model:

INSPECT
Show current state.

PLAN
Display proposed changes.

CONFIRM
Obtain authorization.

APPLY
Perform only validated operations.

VERIFY
Confirm the result.

RECOVER
Offer supported rollback or recovery instructions.

All destructive actions must be recorded in a local audit log.

---

# 19. PERFORMANCE REQUIREMENTS

The application should be lightweight.

Goals:

- Fast startup.
- Responsive UI.
- Low idle CPU usage.
- Moderate memory consumption.
- No unnecessary background daemons.
- Efficient network discovery.
- Asynchronous file operations.
- Minimal dependencies.
- Graceful offline operation.

Set measurable performance targets during implementation.

Use profiling rather than invented performance claims.

Avoid polling when event-driven methods are available.

Ensure large file transfers do not freeze the interface.

Support cancellation of long-running operations.

---

# 20. MOCK PS4 SIMULATOR

Build a realistic console simulator for development.

This is essential because physical PS4 access may not be available to the coding agent.

The simulator should reproduce documented integration behavior through test fixtures.

Simulate:

- Console offline.
- Stock console discovered.
- Remote Play available.
- Pairing required.
- Homebrew service available.
- FTP authentication or connection failure.
- File transfer success.
- File transfer interruption.
- Unsupported firmware.
- Companion connected.
- Companion version mismatch.
- Storage limits.
- Unauthorized operation.
- Network timeout.

The simulator must clearly identify itself as simulated.

Do not generate fake hardware readings and present them as real console results.

Use the simulator for automated testing of the complete user interface.

---

# 21. TESTING AND QUALITY

Implement proper automated tests.

Rust:

cargo fmt --check
cargo clippy --all-targets
cargo test

Frontend:

TypeScript type checking
ESLint
Vitest

Integration:

- Console discovery fixtures.
- Capability negotiation.
- Connection timeouts.
- Remote Play launch validation.
- File transfer handling.
- Path traversal prevention.
- Backup verification.
- Theme imports.
- Companion protocol compatibility.

End-to-end:

- Fresh installation.
- First-run experience.
- Offline operation.
- Console discovery.
- Manual connection.
- Simulator connection.
- Dashboard editing.
- Theme export.
- Recovery workflows.

Include negative tests for malformed or hostile network responses.

Test on Debian with KDE Plasma and GNOME when test environments are available.

Do not claim physical console testing unless actual hardware was used.

---

# 22. DEBIAN PACKAGING

Create a complete Debian package.

Include:

- Desktop launcher.
- Application icon.
- Correct application metadata.
- Dependency declarations.
- Uninstallation support.
- User configuration storage.
- Appropriate filesystem permissions.

Follow the XDG Base Directory specification.

Store:

Configuration in XDG_CONFIG_HOME.

Application data in XDG_DATA_HOME.

Cache in XDG_CACHE_HOME.

Provide build commands and instructions.

Build a .deb artifact when the environment supports it.

Also investigate AppImage packaging.

Do not require users to install the development toolchain to run the packaged application.

---

# 23. OPEN-SOURCE RELEASE

Prepare the repository for GitHub.

Create:

README.md
LICENSE
CONTRIBUTING.md
CODE_OF_CONDUCT.md
SECURITY.md
CHANGELOG.md

The README should contain:

- What PS4 Studio does.
- Screenshots.
- Supported consoles.
- Firmware limitations.
- Debian installation.
- Connection setup.
- Remote Play setup.
- Homebrew requirements.
- Dashboard customization.
- Development instructions.
- Security warnings.
- Troubleshooting.
- Project roadmap.

Choose a project license after reviewing all direct dependencies and their obligations.

In particular, investigate AGPL requirements before embedding Chiaki-ng code.

Keep trademarked console branding separate from original application assets.

State that the project is independent and not endorsed by Sony.

Set up GitHub Actions for:

- Formatting.
- Rust linting.
- Unit tests.
- Frontend tests.
- Security checks.
- Dependency checks.
- Debian package builds.

Ensure the build pipeline reports actual results.

---

# 24. DEVELOPMENT PHASES

Build this project in a practical order.

PHASE 1: Research and architecture

- Inspect existing repositories.
- Verify technical capabilities.
- Identify library licenses.
- Build compatibility matrix.
- Create architecture documents.
- Establish development environment.

PHASE 2: Desktop foundation

- Initialize Rust and Tauri.
- Build Svelte frontend.
- Create design system.
- Implement navigation.
- Add persistent settings.
- Implement mock console.
- Build the initial overview dashboard.

PHASE 3: Connection system

- Implement console discovery.
- Add manual IP connection.
- Implement connection profiles.
- Add capability detection.
- Add supported Remote Play integration.
- Build connection diagnostics.

PHASE 4: Customization

- Implement Dashboard Studio.
- Build the theme engine.
- Add interactive dashboard preview.
- Implement theme import and export.
- Add original theme templates.

PHASE 5: Homebrew integration

- Add optional GoldHEN service adapters.
- Implement safe FTP file management.
- Add validated package management interfaces.
- Add backups and recovery records.

PHASE 6: Native PS4 companion

- Configure OpenOrbis toolchain.
- Build a simple homebrew application.
- Add controller navigation.
- Add theme loading.
- Implement authenticated companion communication.
- Test with compatible hardware when available.

PHASE 7: Experimental operating environment

- Research Linux-based PS4 operating environments.
- Create a standalone desktop simulation.
- Document hardware requirements.
- Evaluate compatibility and recovery risks.
- Keep hardware experiments separate from the production application.

PHASE 8: Release engineering

- Complete documentation.
- Fix known defects.
- Run security checks.
- Verify installation.
- Build Debian packages.
- Prepare GitHub release artifacts.

---

# 25. CODEX AND CLAUDE CODE COLLABORATION

The repository must support development with both OpenAI Codex and Claude Code.

Create an AGENTS.md file with:

- Repository architecture.
- Coding standards.
- Testing requirements.
- Security boundaries.
- Module ownership.
- Build commands.
- Project goals.

Create a CLAUDE.md file with equivalent project context and agent-specific working instructions.

Both files should refer to shared documentation instead of maintaining conflicting instructions.

Use a shared task system.

Create:

docs/tasks.md
docs/current-status.md
docs/decisions.md

For every completed development task, record:

- What was implemented.
- Files changed.
- Tests executed.
- Test results.
- Known limitations.
- Recommended next task.

Use clear Git commits.

Avoid making unrelated large changes.

When handing work between Codex and Claude Code, ensure the next agent can understand the current state directly from the repository.

Do not require the next agent to reconstruct undocumented decisions.

---

# 26. ACCEPTANCE CRITERIA

Consider the first production release complete when:

1. PS4 Studio installs on Debian Linux.
2. The application starts without a connected console.
3. The interface looks polished and works on KDE and GNOME.
4. Offline simulator mode works correctly.
5. The application can discover compatible consoles using documented methods.
6. Manual console connection is supported.
7. Remote Play setup is available and functional when prerequisites are met.
8. The capability system correctly restricts unsupported features.
9. Dashboard Studio supports creating and previewing original themes.
10. Themes can be saved, imported, and exported.
11. Homebrew-enabled consoles can use supported file services.
12. File operations are protected by appropriate safeguards.
13. Backups and operation logs function correctly.
14. Diagnostics provide useful information.
15. The Debian package builds successfully.
16. Automated tests pass.
17. Installation and development documentation is complete.
18. The repository is ready for a public open-source release.

Treat native PS4 dashboard deployment as an additional milestone requiring compatible hardware.

Treat replacement operating-system research as a separate experimental workstream.

Do not block the usable desktop release on either of those milestones.

---

# 27. IMPLEMENTATION INSTRUCTIONS

Begin immediately.

First, inspect the development environment.

Identify:

- Debian version.
- Rust version.
- Node.js version.
- Available package managers.
- Installed development dependencies.
- Current project directory.
- Available build tools.

Then research the referenced projects and document your findings.

Create the repository structure.

Implement the desktop application and offline simulator first.

Build a real, interactive user interface.

Do not create an empty prototype containing only placeholder buttons.

Every visible action must either work or show a clear explanation of the missing prerequisite.

Use real backend commands and typed interfaces.

Prefer smaller, well-tested modules over a monolithic implementation.

At the end of each development phase:

1. Run formatting.
2. Run linting.
3. Run relevant tests.
4. Run the application where possible.
5. Fix discovered failures.
6. Update project documentation.
7. Record remaining limitations.

Continue through the implementation phases without repeatedly asking for approval on ordinary design choices.

Do not run privileged commands, destructive console operations, or risky hardware experiments without explicit permission.

If real PS4 hardware is unavailable, continue building and testing against the simulator.

Make sensible engineering decisions independently.

The final result should be a genuine Debian desktop application that I can install, use, improve, and publish on GitHub.

START NOW.
