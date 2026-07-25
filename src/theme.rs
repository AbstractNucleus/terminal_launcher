use iced::Color;

use crate::config::{self, Settings};

/// Parse "#rrggbb" into (r, g, b) tuple.
pub fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Parse a hex color string (e.g., "#1e1e2e") into an iced Color.
pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let (r, g, b) = parse_hex_rgb(hex)?;
    Some(Color::from_rgb8(r, g, b))
}

/// Holds parsed colors from config for use in styling widgets.
#[derive(Debug, Clone)]
pub struct AppColors {
    pub background: Color,
    pub foreground: Color,
    pub highlight: Color,
    pub surface: Color,
    pub muted: Color,
    pub border: Color,
    pub accent: Color,
    pub danger: Color,
    pub font_size: f32,
}

impl AppColors {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            background: parse_hex_color(&settings.background)
                .unwrap_or_else(|| parse_hex_color(&config::default_background()).unwrap()),
            foreground: parse_hex_color(&settings.foreground)
                .unwrap_or_else(|| parse_hex_color(&config::default_foreground()).unwrap()),
            highlight: parse_hex_color(&settings.highlight)
                .unwrap_or_else(|| parse_hex_color(&config::default_highlight()).unwrap()),
            surface: parse_hex_color(&settings.surface)
                .unwrap_or_else(|| parse_hex_color(&config::default_surface()).unwrap()),
            muted: parse_hex_color(&settings.muted)
                .unwrap_or_else(|| parse_hex_color(&config::default_muted()).unwrap()),
            border: parse_hex_color(&settings.border)
                .unwrap_or_else(|| parse_hex_color(&config::default_border()).unwrap()),
            accent: parse_hex_color(&settings.accent)
                .unwrap_or_else(|| parse_hex_color(&config::default_accent()).unwrap()),
            danger: parse_hex_color(&settings.danger)
                .unwrap_or_else(|| parse_hex_color(&config::default_danger()).unwrap()),
            font_size: settings.font_size as f32,
        }
    }
}

/// Layout metrics for the Cursor-style panel.
#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub panel_width: f32,
    pub shadow_inset: f32,
    pub panel_radius: f32,
    pub list_padding: f32,
    pub input_row_height: f32,
    pub section_header_height: f32,
    pub entry_row_height: f32,
    pub row_radius: f32,
    pub row_inset: f32,
    pub hint_bar_height: f32,
    pub max_visible_rows: usize,
    pub input_font_size: f32,
    pub name_font_size: f32,
    pub detail_font_size: f32,
    pub hint_font_size: f32,
    pub header_font_size: f32,
}

impl Metrics {
    pub fn from_font_size(font_size: u16) -> Self {
        let font_size = font_size as f32;
        Self {
            panel_width: 600.0,
            shadow_inset: 24.0,
            panel_radius: 12.0,
            list_padding: 6.0,
            input_row_height: 44.0,
            section_header_height: 24.0,
            entry_row_height: 30.0,
            row_radius: 6.0,
            row_inset: 8.0,
            hint_bar_height: 30.0,
            max_visible_rows: 9,
            input_font_size: font_size + 1.0,
            name_font_size: font_size,
            detail_font_size: font_size - 1.0,
            hint_font_size: font_size - 3.0,
            header_font_size: font_size - 2.0,
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::from_font_size(14)
    }
}

/// Fixed window height: fits the tallest launcher list and the editor.
/// The window never resizes (resizing thrashes the software renderer's
/// damage tracking); the launcher panel shrinks to content and the rest
/// of the window stays transparent.
pub fn window_height() -> f32 {
    let m = Metrics::default();
    let launcher_max = m.input_row_height
        + 1.0 // hairline under the input
        + m.max_visible_rows as f32 * m.entry_row_height
        + 2.0 * m.section_header_height
        + 2.0 * m.list_padding
        + m.hint_bar_height
        + 2.0 * m.shadow_inset;
    launcher_max.max(editor_height())
}

/// Fixed editor window height (content + shadow margin).
pub fn editor_height() -> f32 {
    560.0
}

/// Window width including the transparent shadow margin on both sides.
pub fn window_width() -> f32 {
    Metrics::default().panel_width + 2.0 * Metrics::default().shadow_inset
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
            surface: "#222222".to_string(),
            muted: "#888888".to_string(),
            accent: "#0000ff".to_string(),
            border: "#333333".to_string(),
            danger: "#ff0000".to_string(),
            font_size: 16,
            hotkey: Default::default(),
        };
        let colors = AppColors::from_settings(&settings);
        assert_eq!(colors.background, Color::from_rgb8(0, 0, 0));
        assert_eq!(colors.foreground, Color::from_rgb8(255, 255, 255));
        assert_eq!(colors.highlight, Color::from_rgb8(255, 0, 0));
        assert_eq!(colors.surface, Color::from_rgb8(0x22, 0x22, 0x22));
        assert_eq!(colors.muted, Color::from_rgb8(0x88, 0x88, 0x88));
        assert_eq!(colors.border, Color::from_rgb8(0x33, 0x33, 0x33));
        assert_eq!(colors.accent, Color::from_rgb8(0, 0, 255));
        assert_eq!(colors.danger, Color::from_rgb8(255, 0, 0));
        assert_eq!(colors.font_size, 16.0);
    }

    #[test]
    fn app_colors_fallback_on_invalid_hex() {
        let settings = Settings {
            background: "bad".to_string(),
            foreground: "bad".to_string(),
            highlight: "bad".to_string(),
            surface: "bad".to_string(),
            muted: "bad".to_string(),
            accent: "bad".to_string(),
            border: "bad".to_string(),
            danger: "bad".to_string(),
            font_size: 14,
            hotkey: Default::default(),
        };
        let colors = AppColors::from_settings(&settings);
        assert_eq!(colors.background, Color::from_rgb8(0x14, 0x14, 0x14));
        assert_eq!(colors.surface, Color::from_rgb8(0x21, 0x21, 0x21));
        assert_eq!(colors.accent, Color::from_rgb8(0xD9, 0xA0, 0x5B));
        assert_eq!(colors.danger, Color::from_rgb8(0xF1, 0x4C, 0x4C));
    }

    #[test]
    fn window_height_fits_max_launcher_and_editor() {
        assert!(window_height() >= editor_height());
        assert_eq!(window_height(), 560.0);
    }

    #[test]
    fn editor_height_is_fixed() {
        assert_eq!(editor_height(), 560.0);
    }

    #[test]
    fn metrics_derive_font_sizes() {
        let m = Metrics::from_font_size(14);
        assert_eq!(m.input_font_size, 15.0);
        assert_eq!(m.name_font_size, 14.0);
        assert_eq!(m.detail_font_size, 13.0);
        assert_eq!(m.hint_font_size, 11.0);
        assert_eq!(m.header_font_size, 12.0);
    }
}
