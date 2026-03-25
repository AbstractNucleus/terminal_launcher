use iced::keyboard::Event as KeyEvent;
use iced::widget::{button, column, container, radio, row, scrollable, text, text_input, Column};
use iced::window;
use iced::{Element, Size, Subscription, Task};

use tray_icon::menu::MenuEvent;

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
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    MoveUp,
    MoveDown,
    Launch,
    Hide,
    ToggleEditor,
    ToggleVisibility,
    Noop,
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
    WindowFocusLost,
    Exit,
}

impl App {
    pub fn new(config: Config, first_run: bool) -> (Self, Task<Message>) {
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
            Message::Launch => {
                let launch_task = self.launch_selected();
                self.search_query.clear();
                self.selected_index = 0;
                let names: Vec<String> = self
                    .config
                    .entry
                    .iter()
                    .map(|e| format!("{} {}", e.name(), e.display_detail()))
                    .collect();
                self.filtered_indices = self.fuzzy_matcher.filter("", &names);
                self.visible = false;
                let hide_task = window::latest().and_then(|id| {
                    Task::batch([
                        window::set_level(id, window::Level::Normal),
                        window::minimize(id, true),
                    ])
                });
                Task::batch([launch_task, hide_task])
            }
            Message::Hide => {
                self.search_query.clear();
                self.selected_index = 0;
                let names: Vec<String> = self
                    .config
                    .entry
                    .iter()
                    .map(|e| format!("{} {}", e.name(), e.display_detail()))
                    .collect();
                self.filtered_indices = self.fuzzy_matcher.filter("", &names);
                self.visible = false;
                window::latest().and_then(|id| {
                    Task::batch([
                        window::set_level(id, window::Level::Normal),
                        window::minimize(id, true),
                    ])
                })
            }
            Message::ToggleVisibility => {
                if self.visible {
                    self.visible = false;
                    window::latest().and_then(|id| {
                        Task::batch([
                            window::set_level(id, window::Level::Normal),
                            window::minimize(id, true),
                        ])
                    })
                } else {
                    self.visible = true;
                    window::latest().and_then(|id| {
                        Task::batch([
                            window::minimize(id, false),
                            window::set_level(id, window::Level::AlwaysOnTop),
                            window::gain_focus(id),
                        ])
                    })
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
            Message::Noop => Task::none(),
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
                let new_entry = match self.editor_entry_type {
                    EntryType::Directory => crate::config::Entry::Directory {
                        name: self.editor_name.clone(),
                        path: self.editor_path.clone(),
                    },
                    EntryType::Ssh => crate::config::Entry::Ssh {
                        name: self.editor_name.clone(),
                        host: self.editor_host.clone(),
                        port: self.editor_port.parse().ok(),
                    },
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
                Task::none()
            }
            Message::EditorCancel => {
                self.clear_editor_fields();
                self.current_view = View::Launcher;
                Task::none()
            }
            Message::WindowFocusLost => {
                if self.current_view == View::Launcher && self.visible {
                    self.search_query.clear();
                    self.selected_index = 0;
                    self.rebuild_filtered_list();
                    self.visible = false;
                    window::latest().and_then(|id| {
                        Task::batch([
                            window::set_level(id, window::Level::Normal),
                            window::minimize(id, true),
                        ])
                    })
                } else {
                    Task::none()
                }
            }
            Message::Exit => {
                std::process::exit(0);
            }
        }
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

        let title = text("Config Editor").size(20.0).color(fg);

        let name_input = text_input("Entry name", &self.editor_name)
            .on_input(Message::EditorNameChanged)
            .padding(8)
            .size(14.0);

        let type_row = row![
            radio(
                "Directory",
                EntryType::Directory,
                Some(self.editor_entry_type),
                Message::EditorTypeChanged,
            ),
            radio(
                "SSH",
                EntryType::Ssh,
                Some(self.editor_entry_type),
                Message::EditorTypeChanged,
            ),
        ]
        .spacing(20);

        let conditional_fields: Element<'_, Message> = match self.editor_entry_type {
            EntryType::Directory => text_input("Path (e.g. ~/projects)", &self.editor_path)
                .on_input(Message::EditorPathChanged)
                .padding(8)
                .size(14.0)
                .into(),
            EntryType::Ssh => column![
                text_input("Host (e.g. user@host.com)", &self.editor_host)
                    .on_input(Message::EditorHostChanged)
                    .padding(8)
                    .size(14.0),
                text_input("Port (optional)", &self.editor_port)
                    .on_input(Message::EditorPortChanged)
                    .padding(8)
                    .size(14.0),
            ]
            .spacing(8)
            .into(),
        };

        let mut button_row = row![
            button("Save").on_press(Message::EditorSave).padding(8),
            button("Cancel").on_press(Message::EditorCancel).padding(8),
        ]
        .spacing(10);

        if self.editor_selected.is_some() {
            if self.editor_confirm_delete {
                button_row = button_row.push(
                    button("Confirm Delete")
                        .on_press(Message::EditorConfirmDelete)
                        .padding(8),
                );
            } else {
                button_row = button_row.push(
                    button("Delete").on_press(Message::EditorDelete).padding(8),
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
                    .color(fg);

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

        let content = column![
            title,
            name_input,
            type_row,
            conditional_fields,
            button_row,
            text("Entries:").size(16.0).color(fg),
            entry_list,
        ]
        .spacing(10)
        .padding(20);

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(bg)),
                ..Default::default()
            })
            .into()
    }

    fn launcher_view(&self) -> Element<'_, Message> {
        let bg = self.colors.background;
        let highlight = self.colors.highlight;
        let fg = self.colors.foreground;

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
                    .size(self.colors.font_size)
                    .color(fg);

                let row = container(label)
                    .width(iced::Length::Fill)
                    .padding(8)
                    .style(move |_theme: &iced::Theme| {
                        if is_selected {
                            container::Style {
                                background: Some(iced::Background::Color(highlight)),
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
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(bg)),
                ..Default::default()
            })
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard_sub = iced::keyboard::listen().map(|event| match event {
            KeyEvent::KeyPressed {
                key, modifiers, ..
            } => {
                use iced::keyboard::key::Named;
                use iced::keyboard::Key;

                match key {
                    Key::Named(Named::ArrowUp) => Message::MoveUp,
                    Key::Named(Named::ArrowDown) => Message::MoveDown,
                    Key::Named(Named::Enter) => Message::Launch,
                    Key::Named(Named::Escape) => Message::Hide,
                    Key::Character(ref c) if modifiers.control() && c.as_str() == "e" => {
                        Message::ToggleEditor
                    }
                    _ => Message::Noop,
                }
            }
            _ => Message::Noop,
        });

        let hotkey_sub = Subscription::run(hotkey_listener);

        let focus_sub = iced::event::listen_with(|event, _status, _id| match event {
            iced::Event::Window(window::Event::Unfocused) => Some(Message::WindowFocusLost),
            _ => None,
        });

        let hide_taskbar_sub = Subscription::run(hide_from_taskbar);

        Subscription::batch([keyboard_sub, hotkey_sub, focus_sub, hide_taskbar_sub])
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

    fn launch_selected(&self) -> Task<Message> {
        if self.filtered_indices.is_empty() {
            return Task::none();
        }

        let entry_idx = self.filtered_indices[self.selected_index];
        let entry = &self.config.entry[entry_idx];

        let result = match entry {
            crate::config::Entry::Directory { path, .. } => {
                let expanded = shellexpand::tilde(path).to_string();
                std::process::Command::new("alacritty")
                    .arg("--working-directory")
                    .arg(&expanded)
                    .spawn()
            }
            crate::config::Entry::Ssh { host, port, .. } => {
                let mut cmd = std::process::Command::new("alacritty");
                cmd.arg("-e").arg("ssh").arg(host);
                if let Some(p) = port {
                    cmd.arg("-p").arg(p.to_string());
                }
                cmd.spawn()
            }
        };

        if let Err(e) = result {
            eprintln!("Failed to launch alacritty: {}", e);
        }

        Task::none()
    }
}

fn hotkey_listener() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(10, async |mut sender| {
        use iced::futures::SinkExt;
        let hotkey_receiver = global_hotkey::GlobalHotKeyEvent::receiver();
        let menu_receiver = MenuEvent::receiver();
        loop {
            if let Ok(_event) = hotkey_receiver.try_recv() {
                let _ = sender.send(Message::ToggleVisibility).await;
            }
            // Any tray menu event means "Exit" (we only have one menu item)
            if let Ok(_event) = menu_receiver.try_recv() {
                let _ = sender.send(Message::Exit).await;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    })
}

/// One-shot subscription that hides the window from the Windows taskbar
/// by setting the WS_EX_TOOLWINDOW extended style on the HWND.
fn hide_from_taskbar() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(1, async |_sender| {
        // Give the window a moment to be created
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::*;
            // Perform all HWND operations in a block so the non-Send HWND
            // doesn't live across an await point.
            let needs_show = unsafe {
                if let Ok(hwnd) = FindWindowW(None, windows::core::w!("Terminal Switcher")) {
                    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                    SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_TOOLWINDOW.0 as i32);
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    true
                } else {
                    false
                }
            };
            if needs_show {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                unsafe {
                    if let Ok(hwnd) = FindWindowW(None, windows::core::w!("Terminal Switcher")) {
                        let _ = ShowWindow(hwnd, SW_SHOW);
                    }
                }
            }
        }

        // Keep the subscription alive (it won't send any messages, just park)
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    })
}
