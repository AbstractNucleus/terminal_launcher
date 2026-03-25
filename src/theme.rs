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
        assert_eq!(colors.background, Color::from_rgb8(0x1e, 0x1e, 0x2e));
    }
}
