use serde::{Deserialize, Serialize};

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
