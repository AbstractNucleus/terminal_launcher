# terminal-switcher

A fast, keyboard-driven launcher for terminal sessions. Hit a global hotkey, fuzzy-search a list of saved directories and SSH hosts, press Enter, and a new [WezTerm](https://wezterm.org/) window opens at that location.

Built in Rust with [iced](https://iced.rs/). Lives in the system tray. Config is a single TOML file.

## Why

If you regularly hop between a handful of project directories and remote hosts, opening a terminal and `cd`-ing (or running `ssh user@...`) every time gets old. This is a small dedicated launcher just for that — closer to Spotlight or Alfred than to a full terminal multiplexer.

## Features

- Global hotkey to show/hide (default `Alt+Space`)
- Fuzzy search over saved entries
- Two entry types: local **directory** (opens WezTerm with `--cwd`) and **SSH** host (runs `ssh user@host [-p port]` inside WezTerm)
- Built-in editor (`Ctrl+E`) to add, edit, and delete entries — no need to hand-edit the TOML
- System tray icon with Config / Restart / Exit menu
- Customizable color theme via the config file
- Cross-platform (Windows, macOS, Linux)

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable, 2021 edition)
- [WezTerm](https://wezterm.org/install/windows.html) — `wezterm-gui` must be on your `PATH`
- On Linux: a working desktop environment with system tray support (most do)

## Install

### Prebuilt binaries

Grab the latest archive for your platform from the [Releases page](https://github.com/AbstractNucleus/terminal_launcher/releases). Unzip it, drop the binary somewhere on your `PATH`, and run it.

### From source

```sh
git clone https://github.com/AbstractNucleus/terminal_launcher.git
cd terminal_launcher
cargo build --release
```

The binary lands at `target/release/terminal-switcher` (`.exe` on Windows). Copy it somewhere on your `PATH`, or set it up to launch on login.

### Run on startup

- **Windows:** put a shortcut to the binary in `shell:startup` (Win+R, then `shell:startup`).
- **macOS:** add it under System Settings → General → Login Items.
- **Linux:** create a `.desktop` file in `~/.config/autostart/`.

## Usage

1. Launch `terminal-switcher`. On first run it creates a default config and opens the editor.
2. Add your directories and SSH hosts in the editor, or close it and use it from the tray.
3. Press `Alt+Space` anywhere. Type a few characters, use arrow keys to pick, press `Enter` to launch.

### Keybindings

| Key            | Action                                   |
| -------------- | ---------------------------------------- |
| `Alt+Space`    | Show/hide the launcher (configurable)    |
| `↑` / `↓`      | Move selection                           |
| `Enter`        | Launch selected entry                    |
| `Escape`       | Hide the launcher                        |
| `Ctrl+E`       | Toggle the entry editor                  |

### Config file

Located at the platform's standard config directory:

- **Windows:** `%APPDATA%\terminal-switcher\config.toml`
- **macOS:** `~/Library/Application Support/terminal-switcher/config.toml`
- **Linux:** `~/.config/terminal-switcher/config.toml`

You can open it directly from the tray icon's **Config** menu item.

```toml
[settings]
background  = "#0f0f0f"
foreground  = "#e8e8e8"
highlight   = "#c4b5fd"
surface     = "#2e2e2e"
muted       = "#909090"
accent      = "#7a9cc7"
border      = "#2e2e2e"
danger      = "#e06c75"
font_size   = 14

[settings.hotkey]
modifier = "Alt"      # Alt | Ctrl | Shift | Super
key      = "Space"    # Space, Enter, Tab, or a-z

[[entry]]
type = "directory"
name = "My Project"
path = "~/projects/myapp"

[[entry]]
type = "ssh"
name = "Prod Box"
host = "user@prod.example.com"
port = 22              # optional
```

After editing the file directly, restart the app from the tray menu (or hotkey changes won't apply).

## Roadmap / non-goals

This is intentionally minimal. It's a launcher, not a terminal, not a session manager, not a replacement for `tmux`. WezTerm is currently hard-coded as the terminal — making that configurable is on the table if there's interest.

## License

MIT — see [LICENSE](LICENSE).
