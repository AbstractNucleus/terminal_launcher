# Terminal Switcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a hotkey-summoned popup that launches Alacritty with different configs (directories/SSH), using fuzzy search and arrow-key navigation.

**Architecture:** Rust binary with iced 0.14 GUI. Three pure-logic modules (config, fuzzy, theme) with unit tests, one GUI module (app) verified manually. Global hotkey via `global-hotkey` crate bridged to iced via custom subscription.

**Tech Stack:** Rust, iced 0.14, nucleo-matcher, global-hotkey, serde, toml, dirs, shellexpand

**Spec:** `docs/superpowers/specs/2026-03-25-terminal-switcher-design.md`

---

## File Structure

```
terminal-switcher/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry point, GlobalHotKeyManager setup, iced app launch
│   ├── app.rs           # iced Application: state, update, view, subscription
│   ├── config.rs        # Config structs, TOML load/save, validation, defaults
│   ├── fuzzy.rs         # Fuzzy matching wrapper around nucleo-matcher
│   └── theme.rs         # Custom iced Theme from config colors
```

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Initialize the Cargo project**

Run: `cargo init --name terminal-switcher`

This creates `Cargo.toml` and `src/main.rs` with a hello-world.

- [ ] **Step 2: Add all dependencies to Cargo.toml**

Replace the `[dependencies]` section in `Cargo.toml`:

```toml
[dependencies]
iced = { version = "0.14", features = ["tokio"] }
nucleo-matcher = "0.3"
global-hotkey = "0.6"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
dirs = "6"
shellexpand = "3"

[profile.release]
strip = true
lto = true
codegen-units = 1
```

- [ ] **Step 3: Verify the project compiles**

Run: `cargo check`
Expected: Compiles with no errors (warnings are OK).

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "feat: scaffold project with dependencies"
```

---

### Task 2: Config Module — Structs and Parsing

**Files:**
- Create: `src/config.rs`
- Modify: `src/main.rs` (add `mod config;`)

- [ ] **Step 1: Write tests for config parsing**

Create `src/config.rs` with tests at the bottom:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub settings: Settings,
    #[serde(default)]
    pub entry: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default = "default_foreground")]
    pub foreground: String,
    #[serde(default = "default_highlight")]
    pub highlight: String,
    #[serde(default = "default_font_size")]
    pub font_size: u16,
    #[serde(default)]
    pub hotkey: HotkeyConfig,
}

fn default_background() -> String { "#1e1e2e".to_string() }
fn default_foreground() -> String { "#cdd6f4".to_string() }
fn default_highlight() -> String { "#89b4fa".to_string() }
fn default_font_size() -> u16 { 14 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    #[serde(default = "default_modifier")]
    pub modifier: String,
    #[serde(default = "default_key")]
    pub key: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifier: default_modifier(),
            key: default_key(),
        }
    }
}

fn default_modifier() -> String { "Alt".to_string() }
fn default_key() -> String { "Space".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entry {
    #[serde(rename = "directory")]
    Directory { name: String, path: String },
    #[serde(rename = "ssh")]
    Ssh {
        name: String,
        host: String,
        #[serde(default)]
        port: Option<u16>,
    },
}

impl Entry {
    pub fn name(&self) -> &str {
        match self {
            Entry::Directory { name, .. } => name,
            Entry::Ssh { name, .. } => name,
        }
    }

    pub fn display_detail(&self) -> String {
        match self {
            Entry::Directory { path, .. } => path.clone(),
            Entry::Ssh { host, port, .. } => match port {
                Some(p) => format!("{}:{}", host, p),
                None => host.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_config() {
        let toml_str = r#"
[settings]
background = "#000000"
foreground = "#ffffff"
highlight = "#ff0000"
font_size = 16

[settings.hotkey]
modifier = "Ctrl"
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
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.settings.background, "#000000");
        assert_eq!(config.settings.font_size, 16);
        assert_eq!(config.settings.hotkey.modifier, "Ctrl");
        assert_eq!(config.entry.len(), 3);

        match &config.entry[0] {
            Entry::Directory { name, path } => {
                assert_eq!(name, "My App");
                assert_eq!(path, "~/projects/myapp");
            }
            _ => panic!("Expected Directory entry"),
        }

        match &config.entry[2] {
            Entry::Ssh { name, host, port } => {
                assert_eq!(name, "Dev Box");
                assert_eq!(host, "user@dev.example.com");
                assert_eq!(*port, Some(2222));
            }
            _ => panic!("Expected Ssh entry"),
        }
    }

    #[test]
    fn parse_minimal_config() {
        let toml_str = r#"
[settings]
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.settings.background, "#1e1e2e");
        assert_eq!(config.settings.font_size, 14);
        assert_eq!(config.settings.hotkey.modifier, "Alt");
        assert_eq!(config.settings.hotkey.key, "Space");
        assert!(config.entry.is_empty());
    }

    #[test]
    fn ssh_port_defaults_to_none() {
        let toml_str = r#"
[settings]

[[entry]]
name = "Server"
type = "ssh"
host = "user@host.com"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        match &config.entry[0] {
            Entry::Ssh { port, .. } => assert_eq!(*port, None),
            _ => panic!("Expected Ssh entry"),
        }
    }

    #[test]
    fn entry_name_and_display() {
        let dir = Entry::Directory {
            name: "Foo".to_string(),
            path: "~/foo".to_string(),
        };
        assert_eq!(dir.name(), "Foo");
        assert_eq!(dir.display_detail(), "~/foo");

        let ssh = Entry::Ssh {
            name: "Bar".to_string(),
            host: "user@bar.com".to_string(),
            port: Some(2222),
        };
        assert_eq!(ssh.name(), "Bar");
        assert_eq!(ssh.display_detail(), "user@bar.com:2222");

        let ssh_no_port = Entry::Ssh {
            name: "Baz".to_string(),
            host: "user@baz.com".to_string(),
            port: None,
        };
        assert_eq!(ssh_no_port.display_detail(), "user@baz.com");
    }

    #[test]
    fn roundtrip_serialize() {
        let config = Config {
            settings: Settings {
                background: "#111".to_string(),
                foreground: "#222".to_string(),
                highlight: "#333".to_string(),
                font_size: 12,
                hotkey: HotkeyConfig::default(),
            },
            entry: vec![
                Entry::Directory {
                    name: "Test".to_string(),
                    path: "/tmp/test".to_string(),
                },
            ],
        };
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.settings.background, "#111");
        assert_eq!(deserialized.entry.len(), 1);
    }
}
```

- [ ] **Step 2: Add `mod config;` to main.rs**

Replace `src/main.rs` contents:

```rust
mod config;

fn main() {
    println!("Terminal Switcher");
}
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test config::tests`
Expected: All 5 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/config.rs src/main.rs
git commit -m "feat: add config module with TOML parsing and tests"
```

---

### Task 3: Config Module — Load, Save, and Default Config

**Files:**
- Modify: `src/config.rs`

- [ ] **Step 1: Write tests for load/save/default behavior**

Add these tests to the existing `mod tests` block in `src/config.rs`:

```rust
    #[test]
    fn default_config_has_example_entries() {
        let config = Config::default();
        assert!(!config.entry.is_empty());
        // Should have at least one example of each type
        let has_dir = config.entry.iter().any(|e| matches!(e, Entry::Directory { .. }));
        let has_ssh = config.entry.iter().any(|e| matches!(e, Entry::Ssh { .. }));
        assert!(has_dir, "Default config should have a directory example");
        assert!(has_ssh, "Default config should have an SSH example");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = Config::default();
        config.save_to(&path).unwrap();

        let loaded = Config::load_from(&path).unwrap();
        assert_eq!(loaded.entry.len(), config.entry.len());
        assert_eq!(loaded.settings.background, config.settings.background);
    }

    #[test]
    fn load_nonexistent_returns_error() {
        let result = Config::load_from(std::path::Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
    }
```

- [ ] **Step 2: Add `tempfile` as a dev dependency**

Add to `Cargo.toml` after `[dependencies]`:

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Implement load, save, default, and config_path**

Add these methods and the `Default` impl above the `#[cfg(test)]` block in `src/config.rs`:

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            settings: Settings {
                background: default_background(),
                foreground: default_foreground(),
                highlight: default_highlight(),
                font_size: default_font_size(),
                hotkey: HotkeyConfig::default(),
            },
            entry: vec![
                Entry::Directory {
                    name: "Example Project".to_string(),
                    path: "~/projects".to_string(),
                },
                Entry::Ssh {
                    name: "Example Server".to_string(),
                    host: "user@example.com".to_string(),
                    port: None,
                },
            ],
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("terminal-switcher");
        config_dir.join("config.toml")
    }

    pub fn load_from(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Load config from default path. If file doesn't exist, create default config and save it.
    /// V1 limitation: if the config file is malformed, the entire config fails to load
    /// and a default is created. Per-entry validation (skipping bad entries) is deferred to V2.
    pub fn load_or_create_default() -> (Self, bool) {
        let path = Self::config_path();
        match Self::load_from(&path) {
            Ok(config) => (config, false),
            Err(e) => {
                eprintln!("Config not found or invalid ({}), creating default", e);
                let config = Config::default();
                if let Err(e) = config.save_to(&path) {
                    eprintln!("Warning: could not save default config: {}", e);
                }
                (config, true) // true = first run
            }
        }
    }
}
```

- [ ] **Step 4: Run all config tests**

Run: `cargo test config::tests`
Expected: All 8 tests pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/config.rs
git commit -m "feat: add config load/save/default with first-run support"
```

---

### Task 4: Fuzzy Search Module

**Files:**
- Create: `src/fuzzy.rs`
- Modify: `src/main.rs` (add `mod fuzzy;`)

- [ ] **Step 1: Write tests for fuzzy matching**

Create `src/fuzzy.rs`:

```rust
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// Returns indices of matching items, sorted by match score (best first).
    pub fn filter(&mut self, query: &str, items: &[String]) -> Vec<usize> {
        if query.is_empty() {
            return (0..items.len()).collect();
        }

        let atom = Atom::new(
            query,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut buf = Vec::new();
        let mut scored: Vec<(usize, u16)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                let haystack = Utf32Str::new(item, &mut buf);
                let score = atom.score(haystack, &mut self.matcher)?;
                Some((idx, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(idx, _)| idx).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<String> {
        vec![
            "My App".to_string(),
            "Backend".to_string(),
            "Prod Server".to_string(),
            "Dev Box".to_string(),
        ]
    }

    #[test]
    fn empty_query_returns_all_in_order() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("", &items());
        assert_eq!(result, vec![0, 1, 2, 3]);
    }

    #[test]
    fn exact_match_returns_item() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("Backend", &items());
        assert!(result.contains(&1));
    }

    #[test]
    fn partial_match() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("dev", &items());
        assert!(!result.is_empty());
        // "Dev Box" should be in results
        assert!(result.contains(&3));
    }

    #[test]
    fn no_match_returns_empty() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("zzzzzzz", &items());
        assert!(result.is_empty());
    }

    #[test]
    fn fuzzy_match_skips_characters() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("prd", &items());
        // "Prod Server" should match (p-r-d)
        assert!(result.contains(&2));
    }

    #[test]
    fn case_insensitive() {
        let mut fm = FuzzyMatcher::new();
        let result = fm.filter("MY APP", &items());
        assert!(result.contains(&0));
    }
}
```

- [ ] **Step 2: Add `mod fuzzy;` to main.rs**

Add `mod fuzzy;` after `mod config;` in `src/main.rs`.

- [ ] **Step 3: Run fuzzy tests**

Run: `cargo test fuzzy::tests`
Expected: All 6 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/fuzzy.rs src/main.rs
git commit -m "feat: add fuzzy search module with nucleo-matcher"
```

---

### Task 5: Theme Module

**Files:**
- Create: `src/theme.rs`
- Modify: `src/main.rs` (add `mod theme;`)

- [ ] **Step 1: Write theme module with color parsing and tests**

Create `src/theme.rs`:

```rust
use iced::Color;

use crate::config::Settings;

/// Parse a hex color string (e.g., "#1e1e2e") into an iced Color.
pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::from_rgb8(r, g, b))
}

/// Holds parsed colors from config for use in styling widgets.
#[derive(Debug, Clone)]
pub struct AppColors {
    pub background: Color,
    pub foreground: Color,
    pub highlight: Color,
    pub font_size: f32,
}

impl AppColors {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            background: parse_hex_color(&settings.background)
                .unwrap_or(Color::from_rgb8(0x1e, 0x1e, 0x2e)),
            foreground: parse_hex_color(&settings.foreground)
                .unwrap_or(Color::from_rgb8(0xcd, 0xd6, 0xf4)),
            highlight: parse_hex_color(&settings.highlight)
                .unwrap_or(Color::from_rgb8(0x89, 0xb4, 0xfa)),
            font_size: settings.font_size as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_hex() {
        let color = parse_hex_color("#ff0000").unwrap();
        assert_eq!(color, Color::from_rgb8(255, 0, 0));
    }

    #[test]
    fn parse_hex_lowercase() {
        let color = parse_hex_color("#1e1e2e").unwrap();
        assert_eq!(color, Color::from_rgb8(0x1e, 0x1e, 0x2e));
    }

    #[test]
    fn parse_invalid_hex_returns_none() {
        assert!(parse_hex_color("not-a-color").is_none());
        assert!(parse_hex_color("#fff").is_none());
        assert!(parse_hex_color("").is_none());
    }

    #[test]
    fn app_colors_from_settings() {
        let settings = Settings {
            background: "#000000".to_string(),
            foreground: "#ffffff".to_string(),
            highlight: "#ff0000".to_string(),
            font_size: 16,
            hotkey: Default::default(),
        };
        let colors = AppColors::from_settings(&settings);
        assert_eq!(colors.background, Color::from_rgb8(0, 0, 0));
        assert_eq!(colors.foreground, Color::from_rgb8(255, 255, 255));
        assert_eq!(colors.highlight, Color::from_rgb8(255, 0, 0));
        assert_eq!(colors.font_size, 16.0);
    }

    #[test]
    fn app_colors_fallback_on_invalid_hex() {
        let settings = Settings {
            background: "bad".to_string(),
            foreground: "bad".to_string(),
            highlight: "bad".to_string(),
            font_size: 14,
            hotkey: Default::default(),
        };
        let colors = AppColors::from_settings(&settings);
        // Should use defaults, not panic
        assert_eq!(colors.background, Color::from_rgb8(0x1e, 0x1e, 0x2e));
    }
}
```

- [ ] **Step 2: Add `mod theme;` to main.rs**

Add `mod theme;` after `mod fuzzy;` in `src/main.rs`.

- [ ] **Step 3: Run theme tests**

Run: `cargo test theme::tests`
Expected: All 5 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/theme.rs src/main.rs
git commit -m "feat: add theme module with hex color parsing"
```

---

### Task 6: App Skeleton — Basic iced Window

**Files:**
- Create: `src/app.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create the app module with minimal iced application**

Create `src/app.rs`:

```rust
use iced::widget::{column, container, text, text_input};
use iced::window;
use iced::{Element, Size, Task};

use crate::config::Config;
use crate::theme::AppColors;

pub struct App {
    config: Config,
    colors: AppColors,
    search_query: String,
    visible: bool,
    first_run: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
}

impl App {
    pub fn new(config: Config, first_run: bool) -> (Self, Task<Message>) {
        let colors = AppColors::from_settings(&config.settings);
        (
            Self {
                config,
                colors,
                search_query: String::new(),
                visible: true,
                first_run,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchChanged(query) => {
                self.search_query = query;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let search = text_input("Search...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .size(self.colors.font_size);

        let content = column![search, text("Terminal Switcher").size(self.colors.font_size),]
            .spacing(10)
            .padding(20);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    pub fn window_settings() -> window::Settings {
        window::Settings {
            size: Size::new(500.0, 400.0),
            position: window::Position::Centered,
            decorations: false,
            resizable: false,
            level: window::Level::AlwaysOnTop,
            ..Default::default()
        }
    }
}
```

- [ ] **Step 2: Update main.rs to launch the iced app**

Replace `src/main.rs`:

```rust
mod app;
mod config;
mod fuzzy;
mod theme;

use config::Config;

fn main() -> iced::Result {
    let (config, first_run) = Config::load_or_create_default();

    iced::application("Terminal Switcher", app::App::update, app::App::view)
        .settings(iced::Settings {
            window: app::App::window_settings(),
            ..Default::default()
        })
        .run_with(move || app::App::new(config, first_run))
}
```

- [ ] **Step 3: Verify the app launches**

Run: `cargo run`
Expected: A borderless, always-on-top, centered 500x400 window appears with a search box and "Terminal Switcher" text. Close it with Alt+F4.

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: add basic iced app skeleton with search input"
```

---

### Task 7: Launcher View — Entry List with Fuzzy Filtering

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add state for selection and filtered entries**

Add these fields to the `App` struct in `src/app.rs`:

```rust
    fuzzy_matcher: crate::fuzzy::FuzzyMatcher,
    filtered_indices: Vec<usize>,
    selected_index: usize,
```

Initialize them in `App::new`:

```rust
    let filtered_indices: Vec<usize> = (0..config.entry.len()).collect();
    // ...in the Self block:
    fuzzy_matcher: crate::fuzzy::FuzzyMatcher::new(),
    filtered_indices,
    selected_index: 0,
```

- [ ] **Step 2: Add keyboard messages**

Extend the `Message` enum:

```rust
#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    MoveUp,
    MoveDown,
    Launch,
    Hide,
    ToggleEditor,
}
```

- [ ] **Step 3: Implement search filtering in update**

Update the `SearchChanged` handler:

```rust
Message::SearchChanged(query) => {
    self.search_query = query.clone();
    let names: Vec<String> = self
        .config
        .entry
        .iter()
        .map(|e| format!("{} {}", e.name(), e.display_detail()))
        .collect();
    self.filtered_indices = self.fuzzy_matcher.filter(&query, &names);
    self.selected_index = 0;
    Task::none()
}
```

- [ ] **Step 4: Implement selection navigation**

Add handlers for `MoveUp`, `MoveDown`:

```rust
Message::MoveUp => {
    if self.selected_index > 0 {
        self.selected_index -= 1;
    }
    Task::none()
}
Message::MoveDown => {
    if !self.filtered_indices.is_empty()
        && self.selected_index < self.filtered_indices.len() - 1
    {
        self.selected_index += 1;
    }
    Task::none()
}
```

Add placeholder handlers for `Launch`, `Hide`, `ToggleEditor`:

```rust
Message::Launch => Task::none(),    // implemented in Task 9
Message::Hide => Task::none(),      // implemented in Task 8
Message::ToggleEditor => Task::none(), // implemented in Task 10
```

- [ ] **Step 5: Build the entry list view**

Replace the `view` method:

```rust
    pub fn view(&self) -> Element<'_, Message> {
        let search = text_input("Search...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .size(self.colors.font_size);

        let entry_list: Vec<Element<'_, Message>> = self
            .filtered_indices
            .iter()
            .enumerate()
            .map(|(view_idx, &entry_idx)| {
                let entry = &self.config.entry[entry_idx];
                let is_selected = view_idx == self.selected_index;

                let label = text(format!("{} — {}", entry.name(), entry.display_detail()))
                    .size(self.colors.font_size);

                let row = container(label)
                    .width(iced::Length::Fill)
                    .padding(8)
                    .style(move |_theme: &iced::Theme| {
                        if is_selected {
                            container::Style {
                                background: Some(iced::Background::Color(
                                    iced::Color::from_rgb8(0x89, 0xb4, 0xfa),
                                )),
                                ..Default::default()
                            }
                        } else {
                            container::Style::default()
                        }
                    });

                row.into()
            })
            .collect();

        let entries_column =
            iced::widget::scrollable(iced::widget::Column::with_children(entry_list).spacing(2));

        let content = column![search, entries_column].spacing(10).padding(20);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(
                    iced::Color::from_rgb8(0x1e, 0x1e, 0x2e),
                )),
                ..Default::default()
            })
            .into()
    }
```

- [ ] **Step 6: Verify visually**

Run: `cargo run`
Expected: The popup shows a search box and a list of entries from the default config (or your config file). Typing in the search box should filter entries.

- [ ] **Step 7: Commit**

```bash
git add src/app.rs
git commit -m "feat: add entry list with fuzzy search filtering"
```

---

### Task 8: Keyboard Event Handling and Window Show/Hide

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add keyboard subscription to app**

Add a `subscription` method to `App` in `src/app.rs`:

```rust
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::keyboard::on_key_press(|key, modifiers| {
            use iced::keyboard::key::Named;
            use iced::keyboard::Key;

            match key {
                Key::Named(Named::ArrowUp) => Some(Message::MoveUp),
                Key::Named(Named::ArrowDown) => Some(Message::MoveDown),
                Key::Named(Named::Enter) => Some(Message::Launch),
                Key::Named(Named::Escape) => Some(Message::Hide),
                _ => {
                    if modifiers.control() {
                        if let Key::Character(c) = key {
                            if c.as_str() == "e" {
                                return Some(Message::ToggleEditor);
                            }
                        }
                    }
                    None
                }
            }
        })
    }
```

- [ ] **Step 2: Wire subscription in main.rs**

Update `main` in `src/main.rs` to include subscription:

```rust
fn main() -> iced::Result {
    let (config, first_run) = Config::load_or_create_default();

    iced::application("Terminal Switcher", app::App::update, app::App::view)
        .subscription(app::App::subscription)
        .settings(iced::Settings {
            window: app::App::window_settings(),
            ..Default::default()
        })
        .run_with(move || app::App::new(config, first_run))
}
```

- [ ] **Step 3: Implement Hide message — window minimize**

Add a `window_id` field to `App`:

```rust
    window_id: Option<window::Id>,
```

Initialize it as `None` in `App::new`.

Update the `Hide` handler:

```rust
Message::Hide => {
    self.search_query.clear();
    self.selected_index = 0;
    let names: Vec<String> = self
        .config
        .entry
        .iter()
        .map(|e| format!("{} {}", e.name(), e.display_detail()))
        .collect();
    self.filtered_indices = self.fuzzy_matcher.filter("", &names);
    self.visible = false;
    window::latest()
        .and_then(|id| {
            self.window_id = Some(id);
            Task::batch([
                window::change_level(id, window::Level::Normal),
                window::minimize(id),
            ])
        })
}
```

Note: `window::minimize(id)` minimizes without a boolean toggle. Restoring is done via `window::gain_focus(id)` which un-minimizes and brings the window forward. If `minimize` takes `(Id, bool)` in the actual 0.14 release, adjust accordingly — check compile output.

- [ ] **Step 4: Verify keyboard navigation and hide**

Run: `cargo run`
Expected: Arrow keys move the selection highlight up/down. Escape minimizes the window and clears the search. Enter does nothing yet (implemented in Task 9).

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: add keyboard navigation and window hide"
```

---

### Task 9: Global Hotkey and Alacritty Launching

**Files:**
- Modify: `src/main.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Set up GlobalHotKeyManager in main.rs**

Update `src/main.rs`:

```rust
mod app;
mod config;
mod fuzzy;
mod theme;

use config::Config;
use global_hotkey::{GlobalHotKeyManager, HotKey};
use global_hotkey::hotkey::{Code, Modifiers};

fn parse_modifier(s: &str) -> Modifiers {
    match s.to_lowercase().as_str() {
        "alt" => Modifiers::ALT,
        "ctrl" | "control" => Modifiers::CONTROL,
        "shift" => Modifiers::SHIFT,
        "super" | "win" => Modifiers::SUPER,
        _ => {
            eprintln!("Unknown modifier '{}', defaulting to Alt", s);
            Modifiers::ALT
        }
    }
}

fn parse_key(s: &str) -> Code {
    match s.to_lowercase().as_str() {
        "space" => Code::Space,
        "enter" | "return" => Code::Enter,
        "tab" => Code::Tab,
        s if s.len() == 1 => {
            // Single character keys (a-z, 0-9)
            match s.chars().next().unwrap() {
                'a'..='z' => {
                    // Map a=KeyA, b=KeyB, etc.
                    let codes = [Code::KeyA, Code::KeyB, Code::KeyC, Code::KeyD, Code::KeyE,
                        Code::KeyF, Code::KeyG, Code::KeyH, Code::KeyI, Code::KeyJ,
                        Code::KeyK, Code::KeyL, Code::KeyM, Code::KeyN, Code::KeyO,
                        Code::KeyP, Code::KeyQ, Code::KeyR, Code::KeyS, Code::KeyT,
                        Code::KeyU, Code::KeyV, Code::KeyW, Code::KeyX, Code::KeyY, Code::KeyZ];
                    codes[(s.chars().next().unwrap() as u8 - b'a') as usize]
                }
                _ => {
                    eprintln!("Unknown key '{}', defaulting to Space", s);
                    Code::Space
                }
            }
        }
        _ => {
            eprintln!("Unknown key '{}', defaulting to Space", s);
            Code::Space
        }
    }
}

fn main() -> iced::Result {
    let (config, first_run) = Config::load_or_create_default();

    // Register global hotkey from config before iced takes over the event loop
    let hotkey_manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
    let modifier = parse_modifier(&config.settings.hotkey.modifier);
    let key = parse_key(&config.settings.hotkey.key);
    let hotkey = HotKey::new(Some(modifier), key);
    hotkey_manager
        .register(hotkey)
        .expect("Failed to register hotkey");

    // Keep manager alive — it must not be dropped
    let _hotkey_manager = hotkey_manager;

    iced::application("Terminal Switcher", app::App::update, app::App::view)
        .subscription(app::App::subscription)
        .settings(iced::Settings {
            window: app::App::window_settings(),
            ..Default::default()
        })
        .run_with(move || app::App::new(config, first_run))
}
```

- [ ] **Step 2: Add hotkey subscription to app**

Add a `ToggleVisibility` variant to `Message`:

```rust
    ToggleVisibility,
```

Update the `subscription` method to include hotkey polling:

```rust
    pub fn subscription(&self) -> iced::Subscription<Message> {
        let keyboard_sub = iced::keyboard::on_key_press(|key, modifiers| {
            use iced::keyboard::key::Named;
            use iced::keyboard::Key;

            match key {
                Key::Named(Named::ArrowUp) => Some(Message::MoveUp),
                Key::Named(Named::ArrowDown) => Some(Message::MoveDown),
                Key::Named(Named::Enter) => Some(Message::Launch),
                Key::Named(Named::Escape) => Some(Message::Hide),
                _ => {
                    if modifiers.control() {
                        if let Key::Character(c) = key {
                            if c.as_str() == "e" {
                                return Some(Message::ToggleEditor);
                            }
                        }
                    }
                    None
                }
            }
        });

        // Note: If `subscription::unfold` is not available in iced 0.14,
        // use `iced::subscription::channel` or `Subscription::run` as a fallback.
        // The concept is the same: poll the GlobalHotKeyEvent receiver in an async loop.
        let hotkey_sub = iced::subscription::unfold(
            "global-hotkey",
            (),
            |()| async {
                use global_hotkey::GlobalHotKeyEvent;
                loop {
                    if let Ok(_event) = GlobalHotKeyEvent::receiver().try_recv() {
                        return (Message::ToggleVisibility, ());
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            },
        );

        iced::Subscription::batch([keyboard_sub, hotkey_sub])
    }
```

- [ ] **Step 3: Handle ToggleVisibility in update**

```rust
Message::ToggleVisibility => {
    if self.visible {
        self.visible = false;
        window::latest().and_then(|id| {
            Task::batch([
                window::change_level(id, window::Level::Normal),
                window::minimize(id),
            ])
        })
    } else {
        self.visible = true;
        window::latest().and_then(|id| {
            Task::batch([
                window::change_level(id, window::Level::AlwaysOnTop),
                window::gain_focus(id),
            ])
        })
    }
}
```

- [ ] **Step 4: Implement Launch — spawn Alacritty**

Add a `launch` helper method to `App`:

```rust
    fn launch_selected(&self) -> Task<Message> {
        if self.filtered_indices.is_empty() {
            return Task::none();
        }

        let entry_idx = self.filtered_indices[self.selected_index];
        let entry = &self.config.entry[entry_idx];

        let result = match entry {
            crate::config::Entry::Directory { path, .. } => {
                let expanded = shellexpand::tilde(path).to_string();
                std::process::Command::new("alacritty")
                    .arg("--working-directory")
                    .arg(&expanded)
                    .spawn()
            }
            crate::config::Entry::Ssh { host, port, .. } => {
                let mut cmd = std::process::Command::new("alacritty");
                cmd.arg("-e").arg("ssh").arg(host);
                if let Some(p) = port {
                    cmd.arg("-p").arg(p.to_string());
                }
                cmd.spawn()
            }
        };

        if let Err(e) = result {
            eprintln!("Failed to launch alacritty: {}", e);
        }

        Task::none()
    }
```

Update the `Launch` handler:

```rust
Message::Launch => {
    let launch_task = self.launch_selected();
    self.search_query.clear();
    self.selected_index = 0;
    let names: Vec<String> = self
        .config
        .entry
        .iter()
        .map(|e| format!("{} {}", e.name(), e.display_detail()))
        .collect();
    self.filtered_indices = self.fuzzy_matcher.filter("", &names);
    self.visible = false;

    let hide_task = window::latest().and_then(|id| {
        Task::batch([
            window::change_level(id, window::Level::Normal),
            window::minimize(id),
        ])
    });

    Task::batch([launch_task, hide_task])
}
```

- [ ] **Step 5: Verify hotkey toggle and launching**

Run: `cargo run`
Expected:
1. Window appears on launch
2. Press `Escape` — window hides
3. Press `Alt+Space` — window reappears
4. Select an entry and press `Enter` — Alacritty opens with the correct config
5. The popup hides after launching

- [ ] **Step 6: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: add global hotkey toggle and alacritty launching"
```

---

### Task 10: Config Editor View

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add editor state to App**

Add an enum for the current view and editor fields:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Launcher,
    Editor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    Directory,
    Ssh,
}
```

Add editor state fields to `App`:

```rust
    current_view: View,
    editor_name: String,
    editor_entry_type: EntryType,
    editor_path: String,
    editor_host: String,
    editor_port: String,
    editor_selected: Option<usize>,
    editor_confirm_delete: bool,
```

Initialize all editor fields in `App::new` (empty strings, `View::Launcher`, `EntryType::Directory`, `None`, `false`). If `first_run` is true, set `current_view: View::Editor`.

- [ ] **Step 2: Add editor messages**

Add to the `Message` enum:

```rust
    EditorNameChanged(String),
    EditorTypeChanged(EntryType),
    EditorPathChanged(String),
    EditorHostChanged(String),
    EditorPortChanged(String),
    EditorSelectEntry(usize),
    EditorSave,
    EditorDelete,
    EditorConfirmDelete,
    EditorCancel,
```

- [ ] **Step 3: Implement ToggleEditor and editor update handlers**

```rust
Message::ToggleEditor => {
    self.current_view = match self.current_view {
        View::Launcher => View::Editor,
        View::Editor => View::Launcher,
    };
    self.clear_editor_fields();
    Task::none()
}
```

Add helper method:

```rust
    fn clear_editor_fields(&mut self) {
        self.editor_name.clear();
        self.editor_path.clear();
        self.editor_host.clear();
        self.editor_port.clear();
        self.editor_entry_type = EntryType::Directory;
        self.editor_selected = None;
        self.editor_confirm_delete = false;
    }
```

Implement all `Editor*` message handlers:

```rust
Message::EditorNameChanged(v) => { self.editor_name = v; Task::none() }
Message::EditorTypeChanged(t) => { self.editor_entry_type = t; Task::none() }
Message::EditorPathChanged(v) => { self.editor_path = v; Task::none() }
Message::EditorHostChanged(v) => { self.editor_host = v; Task::none() }
Message::EditorPortChanged(v) => { self.editor_port = v; Task::none() }
Message::EditorSelectEntry(idx) => {
    self.editor_selected = Some(idx);
    self.editor_confirm_delete = false;
    let entry = &self.config.entry[idx];
    self.editor_name = entry.name().to_string();
    match entry {
        crate::config::Entry::Directory { path, .. } => {
            self.editor_entry_type = EntryType::Directory;
            self.editor_path = path.clone();
            self.editor_host.clear();
            self.editor_port.clear();
        }
        crate::config::Entry::Ssh { host, port, .. } => {
            self.editor_entry_type = EntryType::Ssh;
            self.editor_host = host.clone();
            self.editor_port = port.map(|p| p.to_string()).unwrap_or_default();
            self.editor_path.clear();
        }
    }
    Task::none()
}
Message::EditorSave => {
    let new_entry = match self.editor_entry_type {
        EntryType::Directory => crate::config::Entry::Directory {
            name: self.editor_name.clone(),
            path: self.editor_path.clone(),
        },
        EntryType::Ssh => crate::config::Entry::Ssh {
            name: self.editor_name.clone(),
            host: self.editor_host.clone(),
            port: self.editor_port.parse().ok(),
        },
    };
    if let Some(idx) = self.editor_selected {
        self.config.entry[idx] = new_entry;
    } else {
        self.config.entry.push(new_entry);
    }
    if let Err(e) = self.config.save_to(&Config::config_path()) {
        eprintln!("Failed to save config: {}", e);
    }
    self.clear_editor_fields();
    self.rebuild_filtered_list();
    self.current_view = View::Launcher;
    Task::none()
}
Message::EditorDelete => {
    self.editor_confirm_delete = true;
    Task::none()
}
Message::EditorConfirmDelete => {
    if let Some(idx) = self.editor_selected {
        self.config.entry.remove(idx);
        if let Err(e) = self.config.save_to(&Config::config_path()) {
            eprintln!("Failed to save config: {}", e);
        }
    }
    self.clear_editor_fields();
    self.rebuild_filtered_list();
    Task::none()
}
Message::EditorCancel => {
    self.clear_editor_fields();
    self.current_view = View::Launcher;
    Task::none()
}
```

Add helper:

```rust
    fn rebuild_filtered_list(&mut self) {
        let names: Vec<String> = self
            .config
            .entry
            .iter()
            .map(|e| format!("{} {}", e.name(), e.display_detail()))
            .collect();
        self.filtered_indices = self.fuzzy_matcher.filter(&self.search_query, &names);
        self.selected_index = 0;
    }
```

- [ ] **Step 4: Build the editor view**

Add an `editor_view` method:

```rust
    fn editor_view(&self) -> Element<'_, Message> {
        use iced::widget::{button, column, container, radio, row, scrollable, text, text_input, Column};

        let title = text("Config Editor").size(self.colors.font_size + 4.0);

        let name_field = column![
            text("Name:").size(self.colors.font_size),
            text_input("Entry name", &self.editor_name)
                .on_input(Message::EditorNameChanged)
                .padding(8)
                .size(self.colors.font_size),
        ].spacing(4);

        let type_selector = row![
            radio("Directory", EntryType::Directory, Some(self.editor_entry_type.clone()), Message::EditorTypeChanged),
            radio("SSH", EntryType::Ssh, Some(self.editor_entry_type.clone()), Message::EditorTypeChanged),
        ].spacing(20);

        let type_fields: Element<'_, Message> = match self.editor_entry_type {
            EntryType::Directory => {
                column![
                    text("Path:").size(self.colors.font_size),
                    text_input("~/path/to/dir", &self.editor_path)
                        .on_input(Message::EditorPathChanged)
                        .padding(8)
                        .size(self.colors.font_size),
                ].spacing(4).into()
            }
            EntryType::Ssh => {
                column![
                    text("Host:").size(self.colors.font_size),
                    text_input("user@hostname", &self.editor_host)
                        .on_input(Message::EditorHostChanged)
                        .padding(8)
                        .size(self.colors.font_size),
                    text("Port (optional):").size(self.colors.font_size),
                    text_input("22", &self.editor_port)
                        .on_input(Message::EditorPortChanged)
                        .padding(8)
                        .size(self.colors.font_size),
                ].spacing(4).into()
            }
        };

        let mut buttons = row![
            button("Save").on_press(Message::EditorSave).padding(8),
        ].spacing(10);

        if self.editor_selected.is_some() {
            if self.editor_confirm_delete {
                buttons = buttons.push(
                    button("Confirm Delete").on_press(Message::EditorConfirmDelete).padding(8)
                );
            } else {
                buttons = buttons.push(
                    button("Delete").on_press(Message::EditorDelete).padding(8)
                );
            }
        }
        buttons = buttons.push(button("Cancel").on_press(Message::EditorCancel).padding(8));

        let existing_entries: Vec<Element<'_, Message>> = self
            .config
            .entry
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let is_selected = self.editor_selected == Some(idx);
                let label = text(format!("{} ({})", entry.name(), entry.display_detail()))
                    .size(self.colors.font_size);

                button(label)
                    .on_press(Message::EditorSelectEntry(idx))
                    .padding(6)
                    .width(iced::Length::Fill)
                    .into()
            })
            .collect();

        let entries_list = scrollable(Column::with_children(existing_entries).spacing(2));

        let content = column![
            title,
            name_field,
            type_selector,
            type_fields,
            buttons,
            text("--- Existing Entries ---").size(self.colors.font_size),
            entries_list,
        ]
        .spacing(10)
        .padding(20);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(
                    iced::Color::from_rgb8(0x1e, 0x1e, 0x2e),
                )),
                ..Default::default()
            })
            .into()
    }
```

- [ ] **Step 5: Switch view in the main view method**

Update the `view` method to dispatch based on `current_view`:

```rust
    pub fn view(&self) -> Element<'_, Message> {
        match self.current_view {
            View::Launcher => self.launcher_view(),
            View::Editor => self.editor_view(),
        }
    }
```

Rename the existing `view` method body to `launcher_view`:

```rust
    fn launcher_view(&self) -> Element<'_, Message> {
        // ... existing launcher view code ...
    }
```

- [ ] **Step 6: Verify editor view**

Run: `cargo run`
Expected:
1. Press `Ctrl+E` — editor view appears with form fields and existing entries list
2. Click an existing entry — form populates with its data
3. Edit fields and click Save — returns to launcher, entry is updated
4. Add a new entry — fill in fields with no entry selected, click Save
5. Delete an entry — click Delete, then Confirm Delete
6. Press Escape — returns to launcher without changes

- [ ] **Step 7: Commit**

```bash
git add src/app.rs
git commit -m "feat: add config editor view with add/edit/delete"
```

---

### Task 11: Apply Theme Colors from Config

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Use AppColors throughout the views**

Replace all hardcoded `Color::from_rgb8(0x1e, 0x1e, 0x2e)` references in `launcher_view` and `editor_view` with `self.colors.background`, `self.colors.foreground`, `self.colors.highlight`.

For the launcher container background:
```rust
    let bg = self.colors.background;
    container(content)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            ..Default::default()
        })
```

For selected entry highlight:
```rust
    let highlight = self.colors.highlight;
    let row = container(label)
        .style(move |_theme: &iced::Theme| {
            if is_selected {
                container::Style {
                    background: Some(iced::Background::Color(highlight)),
                    ..Default::default()
                }
            } else {
                container::Style::default()
            }
        });
```

For text color, apply `self.colors.foreground` via `.color()` on `text` widgets:
```rust
    let label = text(format!("{} — {}", entry.name(), entry.display_detail()))
        .size(self.colors.font_size)
        .color(self.colors.foreground);
```

Apply the same pattern in `editor_view`.

- [ ] **Step 2: Reload colors when config is saved in editor**

In the `EditorSave` handler, after saving config, update colors:

```rust
    self.colors = AppColors::from_settings(&self.config.settings);
```

- [ ] **Step 3: Verify theming**

Edit the config file to change `background`, `foreground`, `highlight` values. Restart the app.
Expected: The popup reflects the new colors.

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: apply user-configured theme colors throughout UI"
```

---

### Task 12: Focus-Loss Auto-Hide (Launcher Only)

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Subscribe to window focus events**

Add a `WindowFocusLost` message variant:

```rust
    WindowFocusLost,
```

Add a window event subscription in the `subscription` method:

```rust
    let focus_sub = iced::event::listen_with(|event, _status, _id| {
        match event {
            iced::Event::Window(window::Event::Unfocused) => Some(Message::WindowFocusLost),
            _ => None,
        }
    });
```

Add `focus_sub` to the `Subscription::batch`.

- [ ] **Step 2: Handle focus loss — hide only in launcher view**

```rust
Message::WindowFocusLost => {
    if self.current_view == View::Launcher && self.visible {
        self.search_query.clear();
        self.selected_index = 0;
        self.rebuild_filtered_list();
        self.visible = false;
        window::latest().and_then(|id| {
            Task::batch([
                window::change_level(id, window::Level::Normal),
                window::minimize(id),
            ])
        })
    } else {
        Task::none()
    }
}
```

- [ ] **Step 3: Verify focus behavior**

Run: `cargo run`
Expected:
1. Click outside the popup in launcher view — it hides
2. Switch to editor view (`Ctrl+E`), click outside — popup stays visible
3. `Alt+Space` brings it back after focus-loss hide

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: auto-hide on focus loss in launcher view only"
```

---

### Task 13: Final Polish and All-Tests Pass

**Files:**
- All source files

- [ ] **Step 1: Run all unit tests**

Run: `cargo test`
Expected: All tests pass (config, fuzzy, theme modules).

- [ ] **Step 2: Run clippy for linting**

Run: `cargo clippy -- -D warnings`
Expected: No warnings or errors. Fix any that come up.

- [ ] **Step 3: Build release binary**

Run: `cargo build --release`
Expected: Binary at `target/release/terminal-switcher.exe`. Check its size (should be ~10-15MB).

- [ ] **Step 4: Verify full flow end-to-end**

1. Run the release binary
2. Search, navigate, and launch a directory entry
3. `Alt+Space` to bring it back
4. Launch an SSH entry
5. `Ctrl+E` to open editor, add a new entry, save
6. Verify the new entry appears in the launcher
7. Focus-loss hides in launcher, not in editor

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "chore: final polish — clippy clean, release build verified"
```
