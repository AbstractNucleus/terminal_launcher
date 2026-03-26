# Tray Context Menu — "Config" Option

## Summary

Add a "Config" option to the system tray icon's right-click context menu that opens the config directory in Windows Explorer.

## Menu Layout

```
┌─────────┐
│ Config   │
│─────────│
│ Exit     │
└─────────┘
```

- **Config** — opens `~/.config/terminal-switcher/` in Windows Explorer
- **Separator** — visual divider
- **Exit** — exits the application (existing behavior)

## Implementation

### Files Changed

1. **`src/main.rs`**
   - Add `MenuItem::new("Config", true, None)` before the existing Exit item
   - Add `PredefinedMenuItem::separator()` between Config and Exit
   - Pass both menu item IDs into the Iced app (so events can be distinguished)

2. **`src/app.rs`**
   - Store the Config and Exit menu item IDs in app state (or pass via flags)
   - In the tray menu event subscription, match on item ID:
     - Config ID → `std::process::Command::new("explorer").arg(config_dir).spawn()`
     - Exit ID → `std::process::exit(0)` (existing)
   - Config directory path: parent of `Config::config_path()`

### Dependencies

No new dependencies. Uses `std::process::Command` with `explorer.exe` (Windows-only, consistent with existing platform-specific choices in the app).

### Behavior Details

- Clicking "Config" opens the folder containing `config.toml`, not the file itself
- If the folder doesn't exist yet (shouldn't happen since config is loaded at startup), Explorer will show an error — no special handling needed
- The app window is unaffected; Explorer opens independently
