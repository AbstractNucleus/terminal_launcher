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
    fuzzy_matcher: crate::fuzzy::FuzzyMatcher,
    filtered_indices: Vec<usize>,
    selected_index: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    MoveUp,
    MoveDown,
    Launch,
    Hide,
    ToggleEditor,
}

impl App {
    pub fn new(config: Config, first_run: bool) -> (Self, Task<Message>) {
        let colors = AppColors::from_settings(&config.settings);
        let filtered_indices: Vec<usize> = (0..config.entry.len()).collect();
        (
            Self {
                config,
                colors,
                search_query: String::new(),
                visible: true,
                first_run,
                fuzzy_matcher: crate::fuzzy::FuzzyMatcher::new(),
                filtered_indices,
                selected_index: 0,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
            Message::Launch => Task::none(),
            Message::Hide => Task::none(),
            Message::ToggleEditor => Task::none(),
        }
    }

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
