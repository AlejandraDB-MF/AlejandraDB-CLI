# AlejandraDB

Terminal tool for managing Android devices through ADB. Lists devices, manages apps, reads device info, captures logcat, and runs backup scripts. Single Rust binary with a full-screen TUI and scriptable CLI subcommands.

## Requirements

- Rust toolchain (stable, edition 2024)
- `adb` binary on PATH, or set `ADB_PATH`
- On Fedora: `sudo dnf install android-tools`

## Install (prebuilt)

Linux (x86_64/aarch64, static musl):

```sh
curl -fsSL https://raw.githubusercontent.com/AlejandraDB-MF/AlejandraDB-CLI/main/install.sh | sh
```

Windows: download `alejandradb.exe` from the [latest release](https://github.com/AlejandraDB-MF/AlejandraDB-CLI/releases/latest) and place it in your PATH.

## Install (from source)

```bash
cargo build --release
./target/release/alejandradb
```

> The Rust project lives in the `adbcli/` directory. The commands above assume you are inside that directory.

## Usage

Run without arguments to open the TUI. Pass a subcommand for CLI mode.

```bash
alejandradb                          # open TUI
alejandradb devices                  # list connected devices
alejandradb apps list                # list installed packages
alejandradb apps uninstall com.foo   # uninstall a package
alejandradb apps install app.apk     # install an APK
alejandradb processes list           # list device processes
alejandradb logcat --lines 100       # capture logcat
alejandradb scripts run --all        # run backup script
alejandradb shell                    # open ADB shell
```

Global flags: `--serial <serial>` selects a device. `--adb <path>` sets the ADB binary.

## TUI Navigation

| Key | Action |
|-----|--------|
| Arrow keys | Move selection |
| Enter | Confirm |
| Esc | Go back |
| q | Quit |
| / | Filter list |
| Space | Toggle selection |
| r | Refresh |
| Tab | Cycle sort mode |
| F | Toggle force mode (unlocks critical apps) |

## Features

- **Device management**: list USB and Wi-Fi devices, select by serial or interactive prompt
- **App management**: list, filter, sort, uninstall, install APKs, enable, disable, reinstall system apps
- **Safety checks**: critical apps (systemui, settings, phone, GMS) need force mode to modify
- **Device info**: manufacturer, model, Android version, battery, storage, SD cards, network
- **Device control**: reboot, recovery, bootloader, power off
- **Logcat**: live capture with pause, scroll, clear, export to file
- **Processes**: list and kill (PID 1 protected)
- **Wi-Fi ADB**: discover devices on local network, connect, disconnect, enable and disable TCP mode
- **Backup scripts**: copy DCIM, Downloads, Documents, Movies, Pictures to a timestamped directory
- **Settings**: language (English/Spanish), output path
- **i18n**: full English and Spanish localization

## Configuration

Config lives at `~/.config/alejandradb/config.toml`:

```toml
language = "en"
output_path = "~/alejandradb"
wifi_subnet = "192.168.1"
```

Leave `wifi_subnet` empty for auto-detection from the local network interface.

The project uses Conventional Commits. See [CONTRIBUTING.md](./llm_files/CONTRIBUTING.md) for commit and release conventions.

## License

GPL-3.0. See [LICENSE](./LICENSE).