#![windows_subsystem = "windows"]

mod app;
mod config;
mod fuzzy;
mod theme;
mod ui;

use config::Config;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::GlobalHotKeyManager;
use iced::Font;
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};

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
        s if s.len() == 1 => match s.chars().next().unwrap() {
            'a'..='z' => {
                let codes = [
                    Code::KeyA,
                    Code::KeyB,
                    Code::KeyC,
                    Code::KeyD,
                    Code::KeyE,
                    Code::KeyF,
                    Code::KeyG,
                    Code::KeyH,
                    Code::KeyI,
                    Code::KeyJ,
                    Code::KeyK,
                    Code::KeyL,
                    Code::KeyM,
                    Code::KeyN,
                    Code::KeyO,
                    Code::KeyP,
                    Code::KeyQ,
                    Code::KeyR,
                    Code::KeyS,
                    Code::KeyT,
                    Code::KeyU,
                    Code::KeyV,
                    Code::KeyW,
                    Code::KeyX,
                    Code::KeyY,
                    Code::KeyZ,
                ];
                codes[(s.chars().next().unwrap() as u8 - b'a') as usize]
            }
            _ => {
                eprintln!("Unknown key '{}', defaulting to Space", s);
                Code::Space
            }
        },
        _ => {
            eprintln!("Unknown key '{}', defaulting to Space", s);
            Code::Space
        }
    }
}

fn main() -> iced::Result {
    // Default to the software renderer: wgpu's DX12 surfaces on Windows only
    // support opaque composition, which turns the transparent shadow margin
    // into a solid rectangle. The tiny-skia path composites per-pixel alpha
    // correctly, and CPU rendering is plenty for a panel this size.
    if std::env::var_os("ICED_BACKEND").is_none() {
        std::env::set_var("ICED_BACKEND", "tiny-skia");
    }

    let (config, first_run) = Config::load_or_create_default();

    let hotkey_manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
    let modifier = parse_modifier(&config.settings.hotkey.modifier);
    let key = parse_key(&config.settings.hotkey.key);
    let hotkey = HotKey::new(Some(modifier), key);
    hotkey_manager
        .register(hotkey)
        .expect("Failed to register hotkey");
    let _hotkey_manager = hotkey_manager; // keep alive

    // --- Tray icon setup ---
    let icon = create_tray_icon_image(&config);
    let tray_menu = Menu::new();
    let config_item = MenuItem::new("Config", true, None);
    let restart_item = MenuItem::new("Restart", true, None);
    let exit_item = MenuItem::new("Exit", true, None);
    tray_menu.append(&config_item).unwrap();
    tray_menu.append(&restart_item).unwrap();
    tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
    tray_menu.append(&exit_item).unwrap();

    let tooltip = format!(
        "Terminal Switcher — {}+{}",
        config.settings.hotkey.modifier, config.settings.hotkey.key
    );
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(tooltip)
        .with_icon(icon)
        .with_menu_on_left_click(false)
        .build()
        .expect("Failed to create tray icon");

    let config_menu_id = config_item.id().clone();
    let restart_menu_id = restart_item.id().clone();
    let exit_menu_id = exit_item.id().clone();

    iced::application(
        move || {
            app::App::new(
                config.clone(),
                first_run,
                config_menu_id.clone(),
                restart_menu_id.clone(),
                exit_menu_id.clone(),
            )
        },
        app::App::update,
        app::App::view,
    )
    .title("Terminal Switcher")
    .font(include_bytes!("../assets/Inter-Regular.ttf").as_slice())
    .font(include_bytes!("../assets/Inter-SemiBold.ttf").as_slice())
    .font(include_bytes!("../assets/codicons.ttf").as_slice())
    .default_font(Font::with_name("Inter"))
    // Transparent window background so the shadow margin shows the desktop;
    // without this, iced fills the window with the theme's opaque base color.
    .style(|app: &app::App, _theme| iced::theme::Style {
        background_color: iced::Color::TRANSPARENT,
        text_color: app.colors.foreground,
    })
    .transparent(true)
    .window(app::App::window_settings())
    .subscription(app::App::subscription)
    .run()
}

/// Draw a 16x16 chevron ">" in the foreground color on transparent background.
fn create_tray_icon_image(config: &Config) -> Icon {
    let (r, g, b) = theme::parse_hex_rgb(&config.settings.foreground).unwrap_or((0xc4, 0xb5, 0xfd));
    let w = 16usize;
    let h = 16usize;
    let mut rgba = vec![0u8; w * h * 4];

    let mut paint = |x: usize, y: usize| {
        let i = (y * w + x) * 4;
        rgba[i] = r;
        rgba[i + 1] = g;
        rgba[i + 2] = b;
        rgba[i + 3] = 255;
    };
    for y in 2..=8usize {
        let x = y + 2;
        paint(x, y);
        paint(x + 1, y);
    }
    for y in 8..=14usize {
        let x = 18 - y;
        paint(x, y);
        paint(x + 1, y);
    }

    Icon::from_rgba(rgba, w as u32, h as u32).expect("Failed to create tray icon image")
}
