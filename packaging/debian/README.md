The `.deb` is produced by Tauri's bundler (`cargo tauri build --bundles deb`); metadata lives in
`apps/desktop/src-tauri/tauri.conf.json` (`bundle`). It installs the binary, a `.desktop` launcher and icons,
and declares GTK/WebKitGTK runtime dependencies. No maintainer scripts run anything as root.
