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
| **fuzzy** | `src/fuzzy.rs` | Fuzzy matching wrapper around the `nucleo-matcher` crate. |
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

**Location:** Resolved via `dirs::config_dir()`. On Windows this is `C:\Users\<user>\AppData\Roaming\terminal-switcher\config.toml`.

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

### Entry Deserialization

Entries use serde's internally tagged enum pattern for type safety:

```rust
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
enum Entry {
    #[serde(rename = "directory")]
    Directory { name: String, path: String },
    #[serde(rename = "ssh")]
    Ssh { name: String, host: String, port: Option<u16> },
}
```

### Validation

On load, invalid entries are skipped with a warning to stderr. The app does not crash on a malformed config.

### First Run

If no config file exists on launch, the app creates a default config with example entries and opens the config editor view immediately.

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

Uses the `nucleo-matcher` crate (the synchronous matching core behind Helix editor's picker). The full `nucleo` crate provides async parallel matching for large datasets — overkill for a small entry list. `nucleo-matcher` provides the `Matcher::fuzzy_match()` API directly, which is sufficient for the expected entry count (under 100 items). Filters and ranks entries in real-time as the user types. Selection resets to the top match on each keystroke.

## Window Behavior

- **Borderless** — no title bar, no system buttons. Set via `decorations: false` in iced `window::Settings`.
- **Always on top** — appears above all other windows. Set via `level: window::Level::AlwaysOnTop`.
- **Centered on screen** — spawns at screen center each time. Set via `position: window::Position::Centered`.
- **Fixed size** — approximately 500x400px (search box + ~8-10 visible entries). Not resizable.
- **Focus behavior** — grabs keyboard focus on show. Hides automatically if focus is lost in **launcher view only**. In **editor view**, focus loss is ignored to prevent losing unsaved changes.
- **Taskbar presence** — V1 accepts a taskbar entry. System tray icon and taskbar hiding are deferred to V2 (requires Win32 interop or `tray-icon` crate, too complex for a first Rust project).

### Window Show/Hide Mechanism

iced 0.14 does not expose a simple `window::show()` / `window::hide()` API. The toggle strategy:

1. On hide: use `window::minimize(id)` combined with `window::change_level(id, Level::Normal)` to remove the window from view.
2. On show: use `window::change_level(id, Level::AlwaysOnTop)` combined with `window::gain_focus(id)` to bring it back.
3. If iced exposes `set_visible` in a future release, migrate to that.

This avoids Win32 FFI while providing the expected behavior.

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
| `nucleo-matcher` | Synchronous fuzzy matching engine |
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
4. On first launch, the app creates a default config at `AppData\Roaming\terminal-switcher\config.toml`

## Technical Integration Notes

### Global Hotkey → iced Event Loop

The `global-hotkey` crate's `GlobalHotKeyManager` must be created on the main thread (same thread as the win32 event loop that iced/winit owns). Integration strategy:

1. Create `GlobalHotKeyManager` and register the hotkey in `main.rs` before launching the iced app.
2. Create a custom `iced::Subscription` using `subscription::unfold` that polls `GlobalHotKeyEvent::receiver()` (a crossbeam channel).
3. When a hotkey event is received, the subscription emits `Message::ToggleVisibility`.
4. The `update` function handles `ToggleVisibility` by toggling the window minimize/restore state.

### Alacritty Launch Errors

If `alacritty` is not on PATH or the command fails, the app logs the error to stderr. V1 does not display error notifications in the popup — the popup has already hidden by the time the error occurs.

## Out of Scope (V1)

- Installer / packaged distribution
- Multiple terminal emulator support (Alacritty only)
- Plugin system
- Tab management within Alacritty
- Mouse interaction (keyboard-only by design)
- System tray icon / taskbar hiding (requires Win32 interop)
