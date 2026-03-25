mod app;
mod config;
mod fuzzy;
mod theme;

use config::Config;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::GlobalHotKeyManager;

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

    iced::application(
        move || app::App::new(config.clone(), first_run),
        app::App::update,
        app::App::view,
    )
    .title("Terminal Switcher")
    .window(app::App::window_settings())
    .subscription(app::App::subscription)
    .run()
}
