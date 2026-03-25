# Terminal Switcher — Design Spec

## Overview

A hotkey-summoned popup application for Windows that launches Alacritty terminal instances with different configurations (starting directories or SSH connections). Built in Rust with the iced GUI framework.

## Goals

- **Fast:** Sub-30ms popup appearance. No runtime overhead.
- **Minimalistic:** Single binary, borderless popup, no unnecessary UI chrome.
- **Keyboard-driven:** Fuzzy search + arrow key navigation. No mouse required.
- **Configurable:** TOML config for entries and theming. In-app editor for convenience.

## Architecture

### Modules

| Module | File | Responsibility |
|--------|------|---------------|
| **app** | `src/app.rs` | iced Application — state, update, view, subscription. Two views: launcher and config editor. |
| **config** | `src/config.rs` | TOML parsing, saving, validation. Serde structs for settings and entries. |
| **fuzzy** | `src/fuzzy.rs` | Fuzzy matching wrapper around the `nucleo` crate. |
| **theme** | `src/theme.rs` | Custom iced `Theme` built from user-configured colors. |
| **main** | `src/main.rs` | Entry point. App setup, global hotkey registration. |

### Data Flow

```
Global Hotkey → Toggle window visibility
                     ↓
              User types in search box
                     ↓
              fuzzy.rs filters entries from config
                     ↓
              Filtered list displayed with selection highlight
                     ↓
              User presses Enter
                     ↓
              App spawns Alacritty with args
                     ↓
              Popup hides, search clears
```

## Config Format

**Location:** `~/.config/terminal-switcher/config.toml`

```toml
[settings]
background = "#1e1e2e"
foreground = "#cdd6f4"
highlight = "#89b4fa"
font_size = 14

[settings.hotkey]
modifier = "Alt"
key = "Space"

[[entry]]
name = "My App"
type = "directory"
path = "~/projects/myapp"

[[entry]]
name = "Prod Server"
type = "ssh"
host = "user@prod.example.com"

[[entry]]
name = "Dev Box"
type = "ssh"
host = "user@dev.example.com"
port = 2222
```

### Entry Types

- **directory:** Launches `alacritty --working-directory <path>`
- **ssh:** Launches `alacritty -e ssh <host> -p <port>` (port defaults to 22)

### Validation

On load, invalid entries are skipped with a warning to stderr. The app does not crash on a malformed config.

## Keyboard Interaction

| Key | Action |
|-----|--------|
| Global hotkey (default: `Alt+Space`) | Toggle popup visibility |
| Any character | Appends to fuzzy search filter |
| `Backspace` | Remove last character from search |
| `Up` / `Down` | Move selection through filtered list |
| `Enter` | Launch selected entry |
| `Escape` | Close popup, clear search |
| `Ctrl+E` | Switch to config editor view |

## Fuzzy Search

Uses the `nucleo` crate (same engine behind Helix editor's picker). Filters and ranks entries in real-time as the user types. Selection resets to the top match on each keystroke.

## Window Behavior

- **Borderless** — no title bar, no system buttons.
- **Always on top** — appears above all other windows.
- **Centered on screen** — spawns at screen center each time.
- **Fixed size** — approximately 500x400px (search box + ~8-10 visible entries). Not resizable.
- **Focus behavior** — grabs keyboard focus on show. Hides automatically if focus is lost.
- **No taskbar presence** — no taskbar entry. System tray icon only (for lifecycle management).

## Theming

Colors (background, foreground, highlight) and font size are read from the config file. These values are mapped to a custom iced `Theme` struct that styles all widgets (text input, list items, selection highlight, etc.).

## Config Editor (In-App)

Accessed via `Ctrl+E`. A structured form view within the same popup window.

### Editor Layout

- **Name** text field
- **Type** radio buttons: Directory / SSH
- **Path** text field (shown for directory type)
- **Host** text field (shown for SSH type)
- **Port** text field (shown for SSH type, optional)
- **Save / Delete / Cancel** buttons
- **Existing entries list** below the form, navigable with arrow keys

### Editor Behavior

- Arrow keys select an existing entry for editing
- Fields update dynamically based on selected type
- Save writes to the TOML config file and returns to launcher view
- Delete removes the selected entry (with confirmation)
- Escape returns to launcher without saving
- Empty form = new entry mode

## Dependencies

| Crate | Purpose |
|-------|---------|
| `iced` 0.14 | GUI framework |
| `nucleo` | Fuzzy matching engine |
| `global-hotkey` | System-wide hotkey registration |
| `serde` + `toml` | Config serialization/deserialization |
| `dirs` | Cross-platform config directory resolution |
| `shellexpand` | Expand `~` in paths |

## Project Structure

```
terminal-switcher/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── config.rs
│   ├── fuzzy.rs
│   └── theme.rs
```

## Distribution

No installer. Single binary distribution:

1. Build with `cargo build --release`
2. Place binary on PATH
3. Optionally add to Windows `shell:startup` folder for auto-start
4. Create config file at `~/.config/terminal-switcher/config.toml`

## Out of Scope (V1)

- Installer / packaged distribution
- Multiple terminal emulator support (Alacritty only)
- Plugin system
- Tab management within Alacritty
- Mouse interaction (keyboard-only by design)
