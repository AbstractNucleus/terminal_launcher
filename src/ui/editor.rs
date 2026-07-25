use iced::widget::{button, column, container, radio, row, scrollable, text, text_input, Column};
use iced::{Background, Border, Element, Length};

use crate::app::{App, EntryType, Message};
use crate::theme::Metrics;
use crate::ui::{field_style, overlay_scrollbar, panel, INTER};

pub fn editor_view(app: &App) -> Element<'_, Message> {
    let colors = &app.colors;
    let metrics = Metrics::from_font_size(app.config.settings.font_size);
    let bg = colors.background;
    let fg = colors.foreground;
    let surface = colors.surface;
    let muted = colors.muted;
    let border_color = colors.border;
    let accent = colors.accent;
    let highlight = colors.highlight;
    let danger = colors.danger;

    let input_style = field_style(colors, bg);

    let title = text("Config Editor")
        .size(20.0)
        .font(INTER)
        .color(fg);

    let name_input = text_input("Entry name", &app.editor_name)
        .on_input(Message::EditorNameChanged)
        .padding(8)
        .size(metrics.name_font_size)
        .id("editor-name")
        .style(input_style);

    let type_row = row![
        radio(
            "Directory",
            EntryType::Directory,
            Some(app.editor_entry_type),
            Message::EditorTypeChanged,
        )
        .style(move |_theme: &iced::Theme, status| {
            radio::Style {
                background: Background::Color(bg),
                dot_color: accent,
                border_width: 1.0,
                border_color: if matches!(status, radio::Status::Active { is_selected: true }) {
                    accent
                } else {
                    muted
                },
                text_color: Some(fg),
            }
        }),
        radio(
            "SSH",
            EntryType::Ssh,
            Some(app.editor_entry_type),
            Message::EditorTypeChanged,
        )
        .style(move |_theme: &iced::Theme, status| {
            radio::Style {
                background: Background::Color(bg),
                dot_color: accent,
                border_width: 1.0,
                border_color: if matches!(status, radio::Status::Active { is_selected: true }) {
                    accent
                } else {
                    muted
                },
                text_color: Some(fg),
            }
        }),
    ]
    .spacing(20);

    let conditional_fields: Element<'_, Message> = match app.editor_entry_type {
        EntryType::Directory => text_input("Path (e.g. ~/projects)", &app.editor_path)
            .on_input(Message::EditorPathChanged)
            .padding(8)
            .size(metrics.name_font_size)
            .id("editor-path")
            .style(input_style)
            .into(),
        EntryType::Ssh => column![
            text_input("Host (e.g. user@host.com)", &app.editor_host)
                .on_input(Message::EditorHostChanged)
                .padding(8)
                .size(metrics.name_font_size)
                .id("editor-host")
                .style(input_style),
            text_input("Port (optional)", &app.editor_port)
                .on_input(Message::EditorPortChanged)
                .padding(8)
                .size(metrics.name_font_size)
                .id("editor-port")
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
        button::Style {
            background: Some(Background::Color(bg_color)),
            text_color: match status {
                button::Status::Disabled => muted,
                _ => fg,
            },
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 6.0.into(),
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
            background: Some(Background::Color(bg_color)),
            text_color: if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                fg
            } else {
                danger
            },
            border: Border {
                color: danger,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    };

    let detail_filled = match app.editor_entry_type {
        EntryType::Directory => !app.editor_path.trim().is_empty(),
        EntryType::Ssh => !app.editor_host.trim().is_empty(),
    };
    let can_save = !app.editor_name.trim().is_empty() && detail_filled;

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

    if app.editor_selected.is_some() {
        button_row = button_row.push(
            button("New")
                .on_press(Message::EditorNew)
                .padding(8)
                .style(btn_style),
        );
        if app.editor_confirm_delete {
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

    let entries: Vec<Element<'_, Message>> = app
        .config
        .entry
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = app.editor_selected == Some(idx);
            let label = text(format!("{} — {}", entry.name(), entry.display_detail()))
                .size(metrics.name_font_size)
                .color(fg);

            button(container(label).width(Length::Fill).padding(4))
                .on_press(Message::EditorSelectEntry(idx))
                .padding(4)
                .style(move |_theme: &iced::Theme, _status| {
                    if is_selected {
                        button::Style {
                            background: Some(Background::Color(highlight)),
                            text_color: fg,
                            border: Border {
                                color: border_color,
                                width: 1.0,
                                radius: 6.0.into(),
                            },
                            ..Default::default()
                        }
                    } else {
                        button::Style {
                            background: Some(Background::Color(bg)),
                            text_color: fg,
                            border: Border {
                                radius: 6.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }
                })
                .width(Length::Fill)
                .into()
        })
        .collect();

    let entry_list = scrollable(Column::with_children(entries).spacing(4))
        .style(overlay_scrollbar(colors));

    let mut items: Vec<Element<'_, Message>> = Vec::new();
    if app.first_run {
        items.push(
            container(
                text("Welcome — these are example entries. Click one to edit, delete them, or add your own.")
                    .size(13.0)
                    .color(fg),
            )
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(Background::Color(surface)),
                border: Border {
                    color: accent,
                    width: 1.0,
                    radius: 6.0.into(),
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

    panel(content, colors, &metrics)
}
