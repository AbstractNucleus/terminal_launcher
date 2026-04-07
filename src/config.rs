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
    #[serde(default = "default_surface")]
    pub surface: String,
    #[serde(default = "default_muted")]
    pub muted: String,
    #[serde(default = "default_accent")]
    pub accent: String,
    #[serde(default = "default_border")]
    pub border: String,
    #[serde(default = "default_danger")]
    pub danger: String,
    #[serde(default = "default_font_size")]
    pub font_size: u16,
    #[serde(default)]
    pub hotkey: HotkeyConfig,
}

fn default_background() -> String { "#0f0f0f".to_string() }
fn default_foreground() -> String { "#e8e8e8".to_string() }
fn default_highlight() -> String { "#c4b5fd".to_string() }
fn default_surface() -> String { "#2e2e2e".to_string() }
fn default_muted() -> String { "#909090".to_string() }
fn default_accent() -> String { "#7a9cc7".to_string() }
fn default_border() -> String { "#2e2e2e".to_string() }
fn default_danger() -> String { "#e06c75".to_string() }
fn default_font_size() -> u16 { 14 }

impl Default for Settings {
    fn default() -> Self {
        Self {
            background: default_background(),
            foreground: default_foreground(),
            highlight: default_highlight(),
            surface: default_surface(),
            muted: default_muted(),
            accent: default_accent(),
            border: default_border(),
            danger: default_danger(),
            font_size: default_font_size(),
            hotkey: HotkeyConfig::default(),
        }
    }
}

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

impl Default for Config {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_config() {
        let toml_str = r##"
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
"##;
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
        assert_eq!(config.settings.background, "#0f0f0f");
        assert_eq!(config.settings.foreground, "#e8e8e8");
        assert_eq!(config.settings.highlight, "#c4b5fd");
        assert_eq!(config.settings.surface, "#2e2e2e");
        assert_eq!(config.settings.muted, "#909090");
        assert_eq!(config.settings.accent, "#7a9cc7");
        assert_eq!(config.settings.border, "#2e2e2e");
        assert_eq!(config.settings.danger, "#e06c75");
        assert_eq!(config.settings.font_size, 14);
        assert_eq!(config.settings.hotkey.modifier, "Alt");
        assert_eq!(config.settings.hotkey.key, "Space");
        assert!(config.entry.is_empty());
    }

    #[test]
    fn parse_old_config_missing_new_fields() {
        let toml_str = r##"
[settings]
background = "#1e1e2e"
foreground = "#cdd6f4"
highlight = "#89b4fa"
font_size = 14
"##;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.settings.background, "#1e1e2e");
        assert_eq!(config.settings.surface, "#2e2e2e");
        assert_eq!(config.settings.muted, "#909090");
        assert_eq!(config.settings.danger, "#e06c75");
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
                surface: "#444".to_string(),
                muted: "#555".to_string(),
                accent: "#666".to_string(),
                border: "#777".to_string(),
                danger: "#888".to_string(),
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
        assert_eq!(deserialized.settings.surface, "#444");
        assert_eq!(deserialized.settings.danger, "#888");
        assert_eq!(deserialized.entry.len(), 1);
    }

    #[test]
    fn default_config_has_example_entries() {
        let config = Config::default();
        assert!(!config.entry.is_empty());
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
}
