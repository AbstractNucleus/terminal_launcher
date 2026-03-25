mod app;
mod config;
mod fuzzy;
mod theme;

use config::Config;

fn main() -> iced::Result {
    let (config, first_run) = Config::load_or_create_default();

    iced::application(
        move || app::App::new(config.clone(), first_run),
        app::App::update,
        app::App::view,
    )
    .title("Terminal Switcher")
    .window(app::App::window_settings())
    .run()
}
