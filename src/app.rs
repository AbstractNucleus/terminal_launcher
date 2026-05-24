use iced::keyboard::Event as KeyEvent;
use iced::widget::{
    button, column, container, radio, row, scrollable, text, text_input, Column,
};
use iced::window;
use iced::{Element, Size, Subscription, Task};

use tray_icon::menu::{MenuEvent, MenuId};

use crate::config::Config;
use crate::theme::AppColors;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Launcher,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Directory,
    Ssh,
}

pub struct App {
    config: Config,
    colors: AppColors,
    search_query: String,
    visible: bool,
    last_shown: std::time::Instant,
    fuzzy_matcher: crate::fuzzy::FuzzyMatcher,
    filtered_indices: Vec<usize>,
    selected_index: usize,
    current_view: View,
    editor_name: String,
    editor_entry_type: EntryType,
    editor_path: String,
    editor_host: String,
    editor_port: String,
    editor_selected: Option<usize>,
    editor_confirm_delete: bool,
    first_run: bool,
    config_menu_id: MenuId,
    restart_menu_id: MenuId,
    exit_menu_id: MenuId,
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    MoveUp,
    MoveDown,
    Launch,
    LaunchAt(usize),
    Hide,
    ToggleEditor,
    ToggleVisibility,
    EditorNameChanged(String),
    EditorTypeChanged(EntryType),
    EditorPathChanged(String),
    EditorHostChanged(String),
    EditorPortChanged(String),
    EditorSelectEntry(usize),
    EditorSave,
    EditorDelete,
    EditorConfirmDelete,
    EditorCancel,
    EditorNew,
    KeyEnter,
    KeyEscape,
    WindowFocusLost,
    Exit,
    OpenConfig,
    Restart,
}

impl App {
    pub fn new(config: Config, first_run: bool, config_menu_id: MenuId, restart_menu_id: MenuId, exit_menu_id: MenuId) -> (Self, Task<Message>) {
        let colors = AppColors::from_settings(&config.settings);
        let filtered_indices: Vec<usize> = (0..config.entry.len()).collect();
        let current_view = if first_run {
            View::Editor
        } else {
            View::Launcher
        };
        (
            Self {
                config,
                colors,
                search_query: String::new(),
                visible: true,
                last_shown: std::time::Instant::now(),
                fuzzy_matcher: crate::fuzzy::FuzzyMatcher::new(),
                filtered_indices,
                selected_index: 0,
                current_view,
                editor_name: String::new(),
                editor_entry_type: EntryType::Directory,
                editor_path: String::new(),
                editor_host: String::new(),
                editor_port: String::new(),
                editor_selected: None,
                editor_confirm_delete: false,
                first_run,
                config_menu_id,
                restart_menu_id,
                exit_menu_id,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchChanged(query) => {
                self.search_query = query;
                self.rebuild_filtered_list();
                Task::none()
            }
            Message::MoveUp => {
                if !self.filtered_indices.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.filtered_indices.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
                Task::none()
            }
            Message::MoveDown => {
                if !self.filtered_indices.is_empty() {
                    self.selected_index =
                        (self.selected_index + 1) % self.filtered_indices.len();
                }
                Task::none()
            }
            Message::Launch => {
                let launch_task = self.launch_selected();
                self.search_query.clear();
                self.rebuild_filtered_list();
                self.visible = false;
                Task::batch([launch_task, Self::hide_window_task()])
            }
            Message::LaunchAt(view_idx) => {
                if view_idx < self.filtered_indices.len() {
                    self.selected_index = view_idx;
                    self.update(Message::Launch)
                } else {
                    Task::none()
                }
            }
            Message::Hide => {
                self.search_query.clear();
                self.rebuild_filtered_list();
                self.visible = false;
                Self::hide_window_task()
            }
            Message::ToggleVisibility => {
                if self.visible {
                    self.visible = false;
                    Self::hide_window_task()
                } else {
                    self.visible = true;
                    self.last_shown = std::time::Instant::now();
                    let window_task = window::latest().and_then(|id| {
                        Task::batch([
                            window::minimize(id, false),
                            window::set_level(id, window::Level::AlwaysOnTop),
                            window::gain_focus(id),
                        ])
                    });
                    Task::batch([
                        window_task,
                        iced::widget::operation::focus("search-input"),
                    ])
                }
            }
            Message::ToggleEditor => {
                self.current_view = match self.current_view {
                    View::Launcher => View::Editor,
                    View::Editor => View::Launcher,
                };
                self.clear_editor_fields();
                Task::none()
            }
            Message::EditorNameChanged(v) => {
                self.editor_name = v;
                Task::none()
            }
            Message::EditorTypeChanged(t) => {
                self.editor_entry_type = t;
                Task::none()
            }
            Message::EditorPathChanged(v) => {
                self.editor_path = v;
                Task::none()
            }
            Message::EditorHostChanged(v) => {
                self.editor_host = v;
                Task::none()
            }
            Message::EditorPortChanged(v) => {
                self.editor_port = v;
                Task::none()
            }
            Message::EditorSelectEntry(idx) => {
                self.editor_selected = Some(idx);
                self.editor_confirm_delete = false;
                let entry = &self.config.entry[idx];
                self.editor_name = entry.name().to_string();
                match entry {
                    crate::config::Entry::Directory { path, .. } => {
                        self.editor_entry_type = EntryType::Directory;
                        self.editor_path = path.clone();
                        self.editor_host.clear();
                        self.editor_port.clear();
                    }
                    crate::config::Entry::Ssh { host, port, .. } => {
                        self.editor_entry_type = EntryType::Ssh;
                        self.editor_host = host.clone();
                        self.editor_port = port.map(|p| p.to_string()).unwrap_or_default();
                        self.editor_path.clear();
                    }
                }
                Task::none()
            }
            Message::EditorSave => {
                let name = self.editor_name.trim();
                let path = self.editor_path.trim();
                let host = self.editor_host.trim();
                if name.is_empty() {
                    return Task::none();
                }
                let new_entry = match self.editor_entry_type {
                    EntryType::Directory => {
                        if path.is_empty() {
                            return Task::none();
                        }
                        crate::config::Entry::Directory {
                            name: name.to_string(),
                            path: path.to_string(),
                        }
                    }
                    EntryType::Ssh => {
                        if host.is_empty() {
                            return Task::none();
                        }
                        crate::config::Entry::Ssh {
                            name: name.to_string(),
                            host: host.to_string(),
                            port: self.editor_port.trim().parse().ok(),
                        }
                    }
                };
                if let Some(idx) = self.editor_selected {
                    self.config.entry[idx] = new_entry;
                } else {
                    self.config.entry.push(new_entry);
                }
                if let Err(e) = self.config.save_to(&Config::config_path()) {
                    eprintln!("Failed to save config: {}", e);
                }
                self.colors = AppColors::from_settings(&self.config.settings);
                self.clear_editor_fields();
                self.rebuild_filtered_list();
                self.first_run = false;
                self.current_view = View::Launcher;
                Task::none()
            }
            Message::EditorDelete => {
                self.editor_confirm_delete = true;
                Task::none()
            }
            Message::EditorConfirmDelete => {
                if let Some(idx) = self.editor_selected {
                    self.config.entry.remove(idx);
                    if let Err(e) = self.config.save_to(&Config::config_path()) {
                        eprintln!("Failed to save config: {}", e);
                    }
                }
                self.clear_editor_fields();
                self.rebuild_filtered_list();
                self.first_run = false;
                Task::none()
            }
            Message::EditorCancel => {
                self.clear_editor_fields();
                self.current_view = View::Launcher;
                Task::none()
            }
            Message::EditorNew => {
                if self.current_view == View::Editor {
                    self.clear_editor_fields();
                }
                Task::none()
            }
            Message::KeyEnter => match self.current_view {
                View::Launcher => self.update(Message::Launch),
                View::Editor => self.update(Message::EditorSave),
            },
            Message::KeyEscape => match self.current_view {
                View::Launcher => self.update(Message::Hide),
                View::Editor => self.update(Message::EditorCancel),
            },
            Message::WindowFocusLost => {
                let since_shown = self.last_shown.elapsed();
                if self.current_view == View::Launcher
                    && self.visible
                    && since_shown > std::time::Duration::from_millis(500)
                {
                    self.search_query.clear();
                    self.selected_index = 0;
                    self.rebuild_filtered_list();
                    self.visible = false;
                    Self::hide_window_task()
                } else {
                    Task::none()
                }
            }
            Message::OpenConfig => {
                let config_dir = Config::config_path()
                    .parent()
                    .expect("config path has parent")
                    .to_path_buf();

                #[cfg(target_os = "windows")]
                let open_cmd = "explorer";
                #[cfg(target_os = "macos")]
                let open_cmd = "open";
                #[cfg(target_os = "linux")]
                let open_cmd = "xdg-open";

                if let Err(e) = std::process::Command::new(open_cmd)
                    .arg(&config_dir)
                    .spawn()
                {
                    eprintln!("Failed to open config directory: {}", e);
                }
                Task::none()
            }
            Message::Restart => {
                if let Ok(exe) = std::env::current_exe() {
                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        const CREATE_NO_WINDOW: u32 = 0x08000000;
                        let exe_path = exe.to_string_lossy().to_string();
                        if let Err(e) = std::process::Command::new("cmd")
                            .args(["/C", "ping", "-n", "2", "127.0.0.1", ">nul", "&", &exe_path])
                            .creation_flags(CREATE_NO_WINDOW)
                            .spawn()
                        {
                            eprintln!("Failed to restart: {}", e);
                        }
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        if let Err(e) = std::process::Command::new(&exe).spawn() {
                            eprintln!("Failed to restart: {}", e);
                        }
                    }
                }
                std::process::exit(0);
            }
            Message::Exit => {
                std::process::exit(0);
            }
        }
    }

    fn hide_window_task() -> Task<Message> {
        window::latest().and_then(|id| {
            Task::batch([
                window::set_level(id, window::Level::Normal),
                window::minimize(id, true),
            ])
        })
    }

    fn clear_editor_fields(&mut self) {
        self.editor_name.clear();
        self.editor_path.clear();
        self.editor_host.clear();
        self.editor_port.clear();
        self.editor_entry_type = EntryType::Directory;
        self.editor_selected = None;
        self.editor_confirm_delete = false;
    }

    fn rebuild_filtered_list(&mut self) {
        let names: Vec<String> = self
            .config
            .entry
            .iter()
            .map(|e| format!("{} {}", e.name(), e.display_detail()))
            .collect();
        self.filtered_indices = self.fuzzy_matcher.filter(&self.search_query, &names);
        self.selected_index = 0;
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self.current_view {
            View::Launcher => self.launcher_view(),
            View::Editor => self.editor_view(),
        }
    }

    fn editor_view(&self) -> Element<'_, Message> {
        let bg = self.colors.background;
        let highlight = self.colors.highlight;
        let fg = self.colors.foreground;
        let surface = self.colors.surface;
        let muted = self.colors.muted;
        let border_color = self.colors.border;
        let danger = self.colors.danger;

        let input_style = move |_theme: &iced::Theme, status: text_input::Status| {
            let border = iced::Border {
                color: if matches!(status, text_input::Status::Focused { .. }) {
                    highlight
                } else {
                    border_color
                },
                width: 1.0,
                radius: 4.0.into(),
            };
            text_input::Style {
                background: iced::Background::Color(surface),
                border,
                icon: muted,
                placeholder: muted,
                value: fg,
                selection: highlight,
            }
        };

        let title = text("Config Editor").size(20.0).color(fg);

        let name_input = text_input("Entry name", &self.editor_name)
            .on_input(Message::EditorNameChanged)
            .padding(8)
            .size(14.0)
            .style(input_style);

        let type_row = row![
            radio(
                "Directory",
                EntryType::Directory,
                Some(self.editor_entry_type),
                Message::EditorTypeChanged,
            )
            .style(move |_theme: &iced::Theme, status| {
                radio::Style {
                    background: iced::Background::Color(bg),
                    dot_color: highlight,
                    border_width: 1.0,
                    border_color: if matches!(status, radio::Status::Active { is_selected: true }) {
                        highlight
                    } else {
                        muted
                    },
                    text_color: Some(fg),
                }
            }),
            radio(
                "SSH",
                EntryType::Ssh,
                Some(self.editor_entry_type),
                Message::EditorTypeChanged,
            )
            .style(move |_theme: &iced::Theme, status| {
                radio::Style {
                    background: iced::Background::Color(bg),
                    dot_color: highlight,
                    border_width: 1.0,
                    border_color: if matches!(status, radio::Status::Active { is_selected: true }) {
                        highlight
                    } else {
                        muted
                    },
                    text_color: Some(fg),
                }
            }),
        ]
        .spacing(20);

        let conditional_fields: Element<'_, Message> = match self.editor_entry_type {
            EntryType::Directory => text_input("Path (e.g. ~/projects)", &self.editor_path)
                .on_input(Message::EditorPathChanged)
                .padding(8)
                .size(14.0)
                .style(input_style)
                .into(),
            EntryType::Ssh => column![
                text_input("Host (e.g. user@host.com)", &self.editor_host)
                    .on_input(Message::EditorHostChanged)
                    .padding(8)
                    .size(14.0)
                    .style(input_style),
                text_input("Port (optional)", &self.editor_port)
                    .on_input(Message::EditorPortChanged)
                    .padding(8)
                    .size(14.0)
                    .style(input_style),
            ]
            .spacing(8)
            .into(),
        };

        let btn_style = move |_theme: &iced::Theme, status: button::Status| {
            let bg_color = match status {
                button::Status::Hovered | button::Status::Pressed => highlight,
                _ => surface,
            };
            let text_color = match status {
                button::Status::Hovered | button::Status::Pressed => bg,
                button::Status::Disabled => muted,
                _ => fg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color,
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        };

        let danger_btn_style = move |_theme: &iced::Theme, status: button::Status| {
            let bg_color = match status {
                button::Status::Hovered | button::Status::Pressed => danger,
                _ => surface,
            };
            button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                    fg
                } else {
                    danger
                },
                border: iced::Border {
                    color: danger,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        };

        let detail_filled = match self.editor_entry_type {
            EntryType::Directory => !self.editor_path.trim().is_empty(),
            EntryType::Ssh => !self.editor_host.trim().is_empty(),
        };
        let can_save = !self.editor_name.trim().is_empty() && detail_filled;

        let mut button_row = row![
            button("Save")
                .on_press_maybe(can_save.then_some(Message::EditorSave))
                .padding(8)
                .style(btn_style),
            button("Cancel")
                .on_press(Message::EditorCancel)
                .padding(8)
                .style(btn_style),
        ]
        .spacing(10);

        if self.editor_selected.is_some() {
            button_row = button_row.push(
                button("New")
                    .on_press(Message::EditorNew)
                    .padding(8)
                    .style(btn_style),
            );
            if self.editor_confirm_delete {
                button_row = button_row.push(
                    button("Confirm Delete")
                        .on_press(Message::EditorConfirmDelete)
                        .padding(8)
                        .style(danger_btn_style),
                );
            } else {
                button_row = button_row.push(
                    button("Delete")
                        .on_press(Message::EditorDelete)
                        .padding(8)
                        .style(danger_btn_style),
                );
            }
        }

        let entries: Vec<Element<'_, Message>> = self
            .config
            .entry
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let is_selected = self.editor_selected == Some(idx);
                let label = text(format!("{} — {}", entry.name(), entry.display_detail()))
                    .size(14.0)
                    .color(if is_selected { bg } else { fg });

                button(
                    container(label).width(iced::Length::Fill).padding(4),
                )
                .on_press(Message::EditorSelectEntry(idx))
                .padding(4)
                .style(move |_theme: &iced::Theme, _status| {
                    if is_selected {
                        button::Style {
                            background: Some(iced::Background::Color(highlight)),
                            text_color: bg,
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    } else {
                        button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: fg,
                            ..Default::default()
                        }
                    }
                })
                .width(iced::Length::Fill)
                .into()
            })
            .collect();

        let entry_list = scrollable(Column::with_children(entries).spacing(4));

        let mut items: Vec<Element<'_, Message>> = Vec::new();
        if self.first_run {
            items.push(
                container(
                    text("Welcome — these are example entries. Click one to edit, delete them, or add your own.")
                        .size(13.0)
                        .color(fg),
                )
                .padding(10)
                .width(iced::Length::Fill)
                .style(move |_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(surface)),
                    border: iced::Border {
                        color: highlight,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into(),
            );
        }
        items.extend([
            title.into(),
            name_input.into(),
            type_row.into(),
            conditional_fields,
            button_row.into(),
            text("Entries:").size(16.0).color(muted).into(),
            entry_list.into(),
        ]);

        let content = Column::with_children(items).spacing(10).padding(20);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn launcher_view(&self) -> Element<'_, Message> {
        let bg = self.colors.background;
        let highlight = self.colors.highlight;
        let fg = self.colors.foreground;
        let surface = self.colors.surface;
        let muted = self.colors.muted;
        let border_color = self.colors.border;

        let search = text_input("Search...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .size(self.colors.font_size)
            .id("search-input")
            .style(move |_theme: &iced::Theme, status| {
                let border = iced::Border {
                    color: if matches!(status, text_input::Status::Focused { .. }) {
                        highlight
                    } else {
                        border_color
                    },
                    width: 1.0,
                    radius: 4.0.into(),
                };
                text_input::Style {
                    background: iced::Background::Color(surface),
                    border,
                    icon: muted,
                    placeholder: muted,
                    value: fg,
                    selection: highlight,
                }
            });

        let entry_list: Vec<Element<'_, Message>> = self
            .filtered_indices
            .iter()
            .enumerate()
            .map(|(view_idx, &entry_idx)| {
                let entry = &self.config.entry[entry_idx];
                let is_selected = view_idx == self.selected_index;

                let name_text = text(entry.name())
                    .size(self.colors.font_size)
                    .color(if is_selected { bg } else { fg });
                let detail_text = text(format!(" — {}", entry.display_detail()))
                    .size(self.colors.font_size)
                    .color(if is_selected { bg } else { muted });

                let label_row = row![name_text, detail_text];

                button(label_row)
                    .on_press(Message::LaunchAt(view_idx))
                    .width(iced::Length::Fill)
                    .padding(8)
                    .style(move |_theme: &iced::Theme, status: button::Status| {
                        let bg_color = if is_selected {
                            Some(highlight)
                        } else if matches!(status, button::Status::Hovered) {
                            Some(surface)
                        } else {
                            None
                        };
                        button::Style {
                            background: bg_color.map(iced::Background::Color),
                            text_color: if is_selected { bg } else { fg },
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .into()
            })
            .collect();

        let body: Element<'_, Message> = if self.config.entry.is_empty() {
            container(
                text("No entries yet. Press Ctrl+E to add some.")
                    .size(self.colors.font_size)
                    .color(muted),
            )
            .padding(20)
            .into()
        } else if self.filtered_indices.is_empty() {
            container(text("No matches.").size(self.colors.font_size).color(muted))
                .padding(20)
                .into()
        } else {
            scrollable(Column::with_children(entry_list).spacing(2)).into()
        };

        let content = column![search, body].spacing(10).padding(20);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let config_menu_id = self.config_menu_id.clone();
        let restart_menu_id = self.restart_menu_id.clone();
        let exit_menu_id = self.exit_menu_id.clone();
        let hotkey_sub = Subscription::run_with(
            (config_menu_id, restart_menu_id, exit_menu_id),
            |(config_id, restart_id, exit_id)| hotkey_listener(config_id.clone(), restart_id.clone(), exit_id.clone()),
        );

        let event_sub = iced::event::listen_with(|event, _status, _id| {
            use iced::keyboard::key::Named;
            use iced::keyboard::Key;

            match event {
                iced::Event::Window(window::Event::Unfocused) => Some(Message::WindowFocusLost),
                iced::Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => match key {
                    Key::Named(Named::ArrowUp) => Some(Message::MoveUp),
                    Key::Named(Named::ArrowDown) => Some(Message::MoveDown),
                    Key::Named(Named::Enter) if !modifiers.alt() => Some(Message::KeyEnter),
                    Key::Named(Named::Escape) => Some(Message::KeyEscape),
                    Key::Character(ref c) if modifiers.control() && c.as_str() == "e" => {
                        Some(Message::ToggleEditor)
                    }
                    Key::Character(ref c) if modifiers.control() && c.as_str() == "n" => {
                        Some(Message::EditorNew)
                    }
                    _ => None,
                },
                _ => None,
            }
        });

        Subscription::batch([hotkey_sub, event_sub])
    }

    pub fn window_settings() -> window::Settings {
        let mut settings = window::Settings {
            size: Size::new(500.0, 400.0),
            position: window::Position::Centered,
            decorations: false,
            resizable: false,
            level: window::Level::AlwaysOnTop,
            ..Default::default()
        };

        #[cfg(target_os = "windows")]
        {
            settings.platform_specific.skip_taskbar = true;
        }

        #[cfg(target_os = "linux")]
        {
            settings.platform_specific.override_redirect = true;
        }

        settings
    }

    fn launch_selected(&self) -> Task<Message> {
        if self.filtered_indices.is_empty() {
            return Task::none();
        }

        let entry_idx = self.filtered_indices[self.selected_index];
        let entry = &self.config.entry[entry_idx];

        let result = match entry {
            crate::config::Entry::Directory { path, .. } => {
                let expanded = shellexpand::tilde(path).to_string();
                std::process::Command::new("wezterm-gui")
                    .arg("start")
                    .arg("--cwd")
                    .arg(&expanded)
                    .spawn()
            }
            crate::config::Entry::Ssh { host, port, .. } => {
                let mut cmd = std::process::Command::new("wezterm-gui");
                cmd.args(["start", "--"]).arg("ssh").arg(host);
                if let Some(p) = port {
                    cmd.arg("-p").arg(p.to_string());
                }
                cmd.spawn()
            }
        };

        if let Err(e) = result {
            eprintln!("Failed to launch wezterm: {}", e);
        }

        Task::none()
    }
}

fn hotkey_listener(config_menu_id: MenuId, restart_menu_id: MenuId, exit_menu_id: MenuId) -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(10, async move |mut sender| {
        use iced::futures::SinkExt;
        let hotkey_receiver = global_hotkey::GlobalHotKeyEvent::receiver();
        let menu_receiver = MenuEvent::receiver();
        loop {
            if let Ok(event) = hotkey_receiver.try_recv() {
                if event.state == global_hotkey::HotKeyState::Pressed {
                    let _ = sender.send(Message::ToggleVisibility).await;
                }
            }
            if let Ok(event) = menu_receiver.try_recv() {
                if event.id == config_menu_id {
                    let _ = sender.send(Message::OpenConfig).await;
                } else if event.id == restart_menu_id {
                    let _ = sender.send(Message::Restart).await;
                } else if event.id == exit_menu_id {
                    let _ = sender.send(Message::Exit).await;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
}

