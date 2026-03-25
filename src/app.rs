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
