#![windows_subsystem = "windows"]

mod app;
mod config;
mod fuzzy;
mod theme;

use config::Config;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::GlobalHotKeyManager;
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
    // Create a 16x16 solid icon (highlight color from config, or a default blue)
    let icon = create_tray_icon_image(&config);
    let tray_menu = Menu::new();
    let config_item = MenuItem::new("Config", true, None);
    let restart_item = MenuItem::new("Restart", true, None);
    let exit_item = MenuItem::new("Exit", true, None);
    tray_menu.append(&config_item).unwrap();
    tray_menu.append(&restart_item).unwrap();
    tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
    tray_menu.append(&exit_item).unwrap();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Terminal Switcher")
        .with_icon(icon)
        .with_menu_on_left_click(false)
        .build()
        .expect("Failed to create tray icon");
    // _tray_icon must stay alive for the icon to remain visible

    let config_menu_id = config_item.id().clone();
    let restart_menu_id = restart_item.id().clone();
    let exit_menu_id = exit_item.id().clone();
    iced::application(
        move || app::App::new(config.clone(), first_run, config_menu_id.clone(), restart_menu_id.clone(), exit_menu_id.clone()),
        app::App::update,
        app::App::view,
    )
    .title("Terminal Switcher")
    .window(app::App::window_settings())
    .subscription(app::App::subscription)
    .run()
}

/// Create a simple 16x16 solid-color RGBA icon for the system tray.
fn create_tray_icon_image(config: &Config) -> Icon {
    let (r, g, b) = theme::parse_hex_rgb(&config.settings.highlight).unwrap_or((0xc4, 0xb5, 0xfd));
    let width = 16u32;
    let height = 16u32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..(width * height) {
        rgba.push(r);
        rgba.push(g);
        rgba.push(b);
        rgba.push(255);
    }
    Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon image")
}

