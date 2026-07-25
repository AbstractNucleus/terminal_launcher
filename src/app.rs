use iced::keyboard::{key::Named, Event as KeyEvent, Key, Modifiers};
use iced::widget::operation::{self, RelativeOffset};
use iced::widget::Id;
use iced::window;
use iced::{Element, Point, Size, Subscription, Task};

use tray_icon::menu::{MenuEvent, MenuId};

use crate::config::Config;
use crate::theme::{self, AppColors};
use crate::ui;

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
    pub(crate) config: Config,
    pub(crate) colors: AppColors,
    pub(crate) search_query: String,
    visible: bool,
    last_shown: std::time::Instant,
    fuzzy_matcher: crate::fuzzy::FuzzyMatcher,
    pub(crate) filtered_indices: Vec<usize>,
    pub(crate) match_positions: Vec<Vec<u32>>,
    pub(crate) selected_index: usize,
    pub(crate) current_view: View,
    pub(crate) editor_name: String,
    pub(crate) editor_entry_type: EntryType,
    pub(crate) editor_path: String,
    pub(crate) editor_host: String,
    pub(crate) editor_port: String,
    pub(crate) editor_selected: Option<usize>,
    pub(crate) editor_confirm_delete: bool,
    pub(crate) first_run: bool,
    config_menu_id: MenuId,
    restart_menu_id: MenuId,
    exit_menu_id: MenuId,
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    Launch,
    LaunchAt(usize),
    HoverAt(usize),
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
    KeyPressed { key: Key, modifiers: Modifiers },
    WindowFocusLost,
    Exit,
    OpenConfig,
    Restart,
}

impl App {
    pub fn new(
        config: Config,
        first_run: bool,
        config_menu_id: MenuId,
        restart_menu_id: MenuId,
        exit_menu_id: MenuId,
    ) -> (Self, Task<Message>) {
        let colors = AppColors::from_settings(&config.settings);
        let filtered_indices: Vec<usize> = (0..config.entry.len()).collect();
        let match_positions = vec![Vec::new(); filtered_indices.len()];
        let current_view = if first_run {
            View::Editor
        } else {
            View::Launcher
        };

        let app = Self {
            config,
            colors,
            search_query: String::new(),
            visible: true,
            last_shown: std::time::Instant::now(),
            fuzzy_matcher: crate::fuzzy::FuzzyMatcher::new(),
            filtered_indices,
            match_positions,
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
        };

        (app, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchChanged(query) => {
                self.search_query = query;
                self.rebuild_filtered_list()
            }
            Message::Launch => {
                let launch_task = self.launch_selected();
                self.search_query.clear();
                let rebuild = self.rebuild_filtered_list();
                self.visible = false;
                Task::batch([launch_task, rebuild, Self::hide_window_task()])
            }
            Message::LaunchAt(view_idx) => {
                if view_idx < self.filtered_indices.len() {
                    self.selected_index = view_idx;
                    self.update(Message::Launch)
                } else {
                    Task::none()
                }
            }
            Message::HoverAt(view_idx) => {
                if view_idx < self.filtered_indices.len() {
                    self.selected_index = view_idx;
                }
                Task::none()
            }
            Message::Hide => {
                self.search_query.clear();
                let rebuild = self.rebuild_filtered_list();
                self.visible = false;
                Task::batch([rebuild, Self::hide_window_task()])
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
                        operation::focus(Id::new("search-input")),
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
                let rebuild = self.rebuild_filtered_list();
                self.first_run = false;
                self.current_view = View::Launcher;
                rebuild
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
                let rebuild = self.rebuild_filtered_list();
                self.first_run = false;
                rebuild
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
            Message::KeyPressed { key, modifiers } => self.handle_key(key, modifiers),
            Message::WindowFocusLost => {
                let since_shown = self.last_shown.elapsed();
                if self.current_view == View::Launcher
                    && self.visible
                    && since_shown > std::time::Duration::from_millis(500)
                {
                    self.search_query.clear();
                    self.selected_index = 0;
                    let rebuild = self.rebuild_filtered_list();
                    self.visible = false;
                    Task::batch([rebuild, Self::hide_window_task()])
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

    fn handle_key(&mut self, key: Key, modifiers: Modifiers) -> Task<Message> {
        match self.current_view {
            View::Launcher => self.handle_launcher_key(key, modifiers),
            View::Editor => self.handle_editor_key(key, modifiers),
        }
    }

    fn handle_launcher_key(&mut self, key: Key, modifiers: Modifiers) -> Task<Message> {
        match key {
            Key::Named(Named::ArrowUp) => self.move_selection(-1),
            Key::Named(Named::ArrowDown) => self.move_selection(1),
            Key::Named(Named::Home) => {
                if !self.filtered_indices.is_empty() {
                    self.selected_index = 0;
                    self.snap_selection()
                } else {
                    Task::none()
                }
            }
            Key::Named(Named::End) => {
                if !self.filtered_indices.is_empty() {
                    self.selected_index = self.filtered_indices.len() - 1;
                    self.snap_selection()
                } else {
                    Task::none()
                }
            }
            Key::Named(Named::Enter) if !modifiers.alt() => self.update(Message::Launch),
            Key::Named(Named::Escape) => self.update(Message::Hide),
            Key::Character(ref c) if modifiers.control() && c.as_str() == "e" => {
                self.update(Message::ToggleEditor)
            }
            Key::Character(ref c) if modifiers.control() && c.as_str() == "n" => {
                self.move_selection(1)
            }
            Key::Character(ref c) if modifiers.control() && c.as_str() == "p" => {
                self.move_selection(-1)
            }
            _ => Task::none(),
        }
    }

    fn handle_editor_key(&mut self, key: Key, modifiers: Modifiers) -> Task<Message> {
        match key {
            Key::Named(Named::Tab) if modifiers.shift() => operation::focus_previous::<Message>(),
            Key::Named(Named::Tab) => operation::focus_next::<Message>(),
            Key::Named(Named::Enter) if !modifiers.alt() => {
                if self.editor_form_valid() {
                    self.update(Message::EditorSave)
                } else {
                    Task::none()
                }
            }
            Key::Named(Named::Escape) => {
                if self.editor_confirm_delete {
                    self.editor_confirm_delete = false;
                    Task::none()
                } else {
                    self.update(Message::EditorCancel)
                }
            }
            Key::Character(ref c) if modifiers.control() && c.as_str() == "e" => {
                self.update(Message::ToggleEditor)
            }
            Key::Character(ref c) if modifiers.control() && c.as_str() == "n" => {
                self.update(Message::EditorNew)
            }
            _ => Task::none(),
        }
    }

    fn editor_form_valid(&self) -> bool {
        if self.editor_name.trim().is_empty() {
            return false;
        }
        match self.editor_entry_type {
            EntryType::Directory => !self.editor_path.trim().is_empty(),
            EntryType::Ssh => !self.editor_host.trim().is_empty(),
        }
    }

    fn move_selection(&mut self, delta: isize) -> Task<Message> {
        let len = self.filtered_indices.len();
        if len == 0 {
            return Task::none();
        }
        let len_i = len as isize;
        let next = (self.selected_index as isize + delta).rem_euclid(len_i) as usize;
        self.selected_index = next;
        self.snap_selection()
    }

    fn snap_selection(&self) -> Task<Message> {
        let len = self.filtered_indices.len();
        if len <= 1 {
            return Task::none();
        }

        let rendered_len = self.rendered_row_count();
        if rendered_len <= 1 {
            return Task::none();
        }

        let rendered_index = self.rendered_index_of_selection();
        let y = rendered_index as f32 / (rendered_len - 1) as f32;
        operation::snap_to(Id::new("launcher-scroll"), RelativeOffset { x: 0.0, y })
    }

    /// Number of rendered rows in the launcher list (entries + headers in browse mode).
    fn rendered_row_count(&self) -> usize {
        let entries = self.filtered_indices.len();
        if !self.search_query.is_empty() {
            return entries;
        }
        let has_dir = self
            .filtered_indices
            .iter()
            .any(|&i| self.config.entry[i].is_directory());
        let has_ssh = self
            .filtered_indices
            .iter()
            .any(|&i| !self.config.entry[i].is_directory());
        entries + usize::from(has_dir) + usize::from(has_ssh)
    }

    fn rendered_index_of_selection(&self) -> usize {
        if !self.search_query.is_empty() {
            return self.selected_index;
        }

        // Browse mode: directories under one header, then SSH under another.
        let mut rendered = 0usize;

        let dirs: Vec<usize> = self
            .filtered_indices
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, entry_idx)| self.config.entry[*entry_idx].is_directory())
            .map(|(view_idx, _)| view_idx)
            .collect();
        let sshs: Vec<usize> = self
            .filtered_indices
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, entry_idx)| !self.config.entry[*entry_idx].is_directory())
            .map(|(view_idx, _)| view_idx)
            .collect();

        if !dirs.is_empty() {
            rendered += 1; // header
            for view_idx in dirs {
                if view_idx == self.selected_index {
                    return rendered;
                }
                rendered += 1;
            }
        }
        if !sshs.is_empty() {
            rendered += 1; // header
            for view_idx in sshs {
                if view_idx == self.selected_index {
                    return rendered;
                }
                rendered += 1;
            }
        }
        rendered.saturating_sub(1)
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

    fn rebuild_filtered_list(&mut self) -> Task<Message> {
        let names: Vec<String> = self
            .config
            .entry
            .iter()
            .map(|e| format!("{} {}", e.name(), e.display_detail()))
            .collect();

        let results = self.fuzzy_matcher.filter(&self.search_query, &names);

        if self.search_query.is_empty() {
            // Browse mode: directories first, then SSH hosts (headers in the view).
            let mut dirs = Vec::new();
            let mut sshs = Vec::new();
            for (idx, _) in &results {
                if self.config.entry[*idx].is_directory() {
                    dirs.push((*idx, Vec::new()));
                } else {
                    sshs.push((*idx, Vec::new()));
                }
            }
            dirs.append(&mut sshs);
            self.filtered_indices = dirs.iter().map(|(i, _)| *i).collect();
            self.match_positions = dirs.into_iter().map(|(_, p)| p).collect();
        } else {
            self.filtered_indices = results.iter().map(|(i, _)| *i).collect();
            self.match_positions = results.into_iter().map(|(_, p)| p).collect();
        }

        self.selected_index = 0;
        operation::snap_to(Id::new("launcher-scroll"), RelativeOffset::START)
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self.current_view {
            View::Launcher => ui::launcher_view(self),
            View::Editor => ui::editor_view(self),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let config_menu_id = self.config_menu_id.clone();
        let restart_menu_id = self.restart_menu_id.clone();
        let exit_menu_id = self.exit_menu_id.clone();
        let hotkey_sub = Subscription::run_with(
            (config_menu_id, restart_menu_id, exit_menu_id),
            |(config_id, restart_id, exit_id)| {
                hotkey_listener(config_id.clone(), restart_id.clone(), exit_id.clone())
            },
        );

        let event_sub = iced::event::listen_with(|event, _status, _id| match event {
            iced::Event::Window(window::Event::Unfocused) => Some(Message::WindowFocusLost),
            iced::Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => {
                Some(Message::KeyPressed { key, modifiers })
            }
            _ => None,
        });

        Subscription::batch([hotkey_sub, event_sub])
    }

    pub fn window_settings() -> window::Settings {
        let width = theme::window_width();
        let height = theme::window_height();

        let mut settings = window::Settings {
            size: Size::new(width, height),
            position: window::Position::SpecificWith(|win, monitor| {
                Point::new(
                    (monitor.width - win.width) / 2.0,
                    monitor.height * 0.30,
                )
            }),
            decorations: false,
            resizable: false,
            transparent: true,
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

fn hotkey_listener(
    config_menu_id: MenuId,
    restart_menu_id: MenuId,
    exit_menu_id: MenuId,
) -> impl iced::futures::Stream<Item = Message> {
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
