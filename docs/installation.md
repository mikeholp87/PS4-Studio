# Installation

## Debian package
```sh
sudo apt install ./ps4-studio_0.1.0_amd64.deb      # pulls libwebkit2gtk-4.1-0 and GTK 3
ps4-studio                                           # or launch "PS4 Studio" from the menu
```
Uninstall: `sudo apt remove ps4-studio` (user data in `~/.local/share/ps4-studio` is kept; delete it manually if you want it gone).

Tested here: the package builds and its contents/metadata are inspected; the extracted binary was launched. A full `apt install` was **not** run on a clean machine by the author. KDE Plasma/Wayland was used for development; GNOME and X11 are **untested**.

## Remote Play prerequisite
Install chiaki-ng: Flathub `io.github.streetpea.Chiaki4deck` (`flatpak install flathub io.github.streetpea.Chiaki4deck`) or the project's GitHub releases. PS4 Studio detects it automatically.

## AppImage / Flatpak
`packaging/flatpak/` and `packaging/appimage/` contain notes and an *untested* Flatpak manifest template. AppImage bundling by Tauri downloads linuxdeploy and needs `patchelf`; it was not built here.

## Build from source
Prerequisites (Debian): `rustup` (stable), Node 22+, pnpm, `libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libsqlite3-dev`, `cargo install tauri-cli --version '^2' --locked`.
```sh
pnpm install && cd apps/desktop/src-tauri && cargo tauri build --bundles deb
```
Output: `target/release/bundle/deb/`.
