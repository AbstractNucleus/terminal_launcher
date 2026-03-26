# Tray Config Menu Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a "Config" option to the tray icon's right-click menu that opens the config directory in Windows Explorer.

**Architecture:** Two files change. `main.rs` builds the tray menu with Config + separator + Exit, and passes both menu item IDs into the Iced app. `app.rs` distinguishes menu events by ID and either opens Explorer or exits.

**Tech Stack:** `tray-icon` 0.19 (`MenuItem`, `PredefinedMenuItem`, `MenuId`, `MenuEvent`), `std::process::Command` for `explorer.exe`

---

### Task 1: Add Config menu item and separator to tray menu

**Files:**
- Modify: `src/main.rs:11-12` (imports)
- Modify: `src/main.rs:88-102` (tray setup)
- Modify: `src/main.rs:104-105` (app initialization)

- [ ] **Step 1: Update imports in `main.rs`**

Change line 11 from:
```rust
use tray_icon::menu::{Menu, MenuItem};
```
to:
```rust
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
```

- [ ] **Step 2: Build tray menu with Config, separator, Exit**

Replace lines 91-93:
```rust
let tray_menu = Menu::new();
let exit_item = MenuItem::new("Exit", true, None);
tray_menu.append(&exit_item).unwrap();
```
with:
```rust
let tray_menu = Menu::new();
let config_item = MenuItem::new("Config", true, None);
let exit_item = MenuItem::new("Exit", true, None);
tray_menu.append(&config_item).unwrap();
tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
tray_menu.append(&exit_item).unwrap();
```

- [ ] **Step 3: Pass menu item IDs into the Iced app**

Replace lines 104-105:
```rust
iced::application(
    move || app::App::new(config.clone(), first_run),
```
with:
```rust
let config_menu_id = config_item.id().clone();
let exit_menu_id = exit_item.id().clone();
iced::application(
    move || app::App::new(config.clone(), first_run, config_menu_id.clone(), exit_menu_id.clone()),
```

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(tray): add Config menu item with separator to tray menu"
```

---

### Task 2: Handle Config menu event in app

**Files:**
- Modify: `src/app.rs:6` (imports)
- Modify: `src/app.rs:23-40` (App struct)
- Modify: `src/app.rs:42-64` (Message enum)
- Modify: `src/app.rs:67-95` (App::new)
- Modify: `src/app.rs:98-308` (App::update)
- Modify: `src/app.rs:523-556` (subscription)
- Modify: `src/app.rs:608-626` (hotkey_listener)

- [ ] **Step 1: Add `MenuId` import**

Change line 6 from:
```rust
use tray_icon::menu::MenuEvent;
```
to:
```rust
use tray_icon::menu::{MenuEvent, MenuId};
```

- [ ] **Step 2: Add menu ID fields to App struct**

Add two fields to the `App` struct after `editor_confirm_delete: bool,` (line 39):
```rust
    config_menu_id: MenuId,
    exit_menu_id: MenuId,
```

- [ ] **Step 3: Add `OpenConfig` message variant**

Add after `Exit,` (line 63):
```rust
    OpenConfig,
```

- [ ] **Step 4: Update `App::new` to accept menu IDs**

Change the signature on line 67 from:
```rust
    pub fn new(config: Config, first_run: bool) -> (Self, Task<Message>) {
```
to:
```rust
    pub fn new(config: Config, first_run: bool, config_menu_id: MenuId, exit_menu_id: MenuId) -> (Self, Task<Message>) {
```

Add the two new fields to the `Self` initializer, after `editor_confirm_delete: false,` (line 92):
```rust
                config_menu_id,
                exit_menu_id,
```

- [ ] **Step 5: Handle `OpenConfig` in `update`**

Add a new match arm before `Message::Exit` (line 305):
```rust
            Message::OpenConfig => {
                let config_dir = Config::config_path()
                    .parent()
                    .expect("config path has parent")
                    .to_path_buf();
                if let Err(e) = std::process::Command::new("explorer")
                    .arg(&config_dir)
                    .spawn()
                {
                    eprintln!("Failed to open config directory: {}", e);
                }
                Task::none()
            }
```

- [ ] **Step 6: Update subscription to pass menu IDs to listener**

In `subscription` (line 545), change:
```rust
        let hotkey_sub = Subscription::run(hotkey_listener);
```
to:
```rust
        let config_menu_id = self.config_menu_id.clone();
        let exit_menu_id = self.exit_menu_id.clone();
        let hotkey_sub = Subscription::run(move || hotkey_listener(config_menu_id.clone(), exit_menu_id.clone()));
```

- [ ] **Step 7: Update `hotkey_listener` to distinguish menu events by ID**

Replace the entire `hotkey_listener` function (lines 608-626) with:
```rust
fn hotkey_listener(config_menu_id: MenuId, exit_menu_id: MenuId) -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(10, async move |mut sender| {
        use iced::futures::SinkExt;
        let hotkey_receiver = global_hotkey::GlobalHotKeyEvent::receiver();
        let menu_receiver = MenuEvent::receiver();
        loop {
            if let Ok(event) = hotkey_receiver.try_recv() {
                if event.state == global_hotkey::HotKeyState::Pressed {
                    let _ = sender.send(Message::ToggleVisibility).await;
                }
            }
            if let Ok(event) = menu_receiver.try_recv() {
                if event.id == config_menu_id {
                    let _ = sender.send(Message::OpenConfig).await;
                } else if event.id == exit_menu_id {
                    let _ = sender.send(Message::Exit).await;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
}
```

- [ ] **Step 8: Build and verify**

Run: `cargo build`
Expected: Compiles without errors.

- [ ] **Step 9: Manual test**

Run the app, right-click the tray icon. Verify:
1. Menu shows "Config", then a separator, then "Exit"
2. Clicking "Config" opens the `~/.config/terminal-switcher/` folder in Explorer
3. Clicking "Exit" still exits the app
4. The app window is unaffected when Config is clicked

- [ ] **Step 10: Commit**

```bash
git add src/app.rs
git commit -m "feat(tray): handle Config menu click to open config directory in Explorer"
```
