use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Background, Border, Color, Element, Length};

use crate::app::{App, EntryType, Message};
use crate::theme::{AppColors, Metrics};
use crate::ui::{
    field_style, hairline, hint_bar, overlay_scrollbar, panel, row_item, section_header,
    INTER_SEMIBOLD,
};

const BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_WIDTH: f32 = 80.0;

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

    let header = container(
        text("Config Editor")
            .size(metrics.input_font_size)
            .font(INTER_SEMIBOLD)
            .color(fg),
    )
    .width(Length::Fill)
    .height(metrics.input_row_height)
    .padding(iced::Padding {
        top: 0.0,
        right: 16.0,
        bottom: 0.0,
        left: 16.0,
    })
    .align_y(iced::Alignment::Center);

    let name_input = text_input("Entry name", &app.editor_name)
        .on_input(Message::EditorNameChanged)
        .padding(8)
        .size(metrics.name_font_size)
        .id("editor-name")
        .style(input_style);

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

    let label = move |content: &'static str| {
        container(text(content).size(metrics.detail_font_size))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center)
    };

    let secondary_style = move |_theme: &iced::Theme, status: button::Status| {
        let bg_color = match status {
            button::Status::Hovered | button::Status::Pressed => highlight,
            _ => surface,
        };
        button::Style {
            background: Some(Background::Color(bg_color)),
            text_color: fg,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    };

    let primary_style = move |_theme: &iced::Theme, status: button::Status| match status {
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(surface)),
            text_color: muted,
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        },
        button::Status::Hovered | button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color {
                r: (accent.r + 0.06).min(1.0),
                g: (accent.g + 0.06).min(1.0),
                b: (accent.b + 0.06).min(1.0),
                a: 1.0,
            })),
            text_color: bg,
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(accent)),
            text_color: bg,
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    };

    // Armed state swaps label and fill only — bounds stay fixed so the row
    // never shifts.
    let armed = app.editor_confirm_delete;
    let danger_style = move |_theme: &iced::Theme, status: button::Status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        if armed || hovered {
            button::Style {
                background: Some(Background::Color(danger)),
                text_color: bg,
                border: Border {
                    color: danger,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        } else {
            button::Style {
                background: Some(Background::Color(surface)),
                text_color: danger,
                border: Border {
                    color: danger,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        }
    };

    let detail_filled = match app.editor_entry_type {
        EntryType::Directory => !app.editor_path.trim().is_empty(),
        EntryType::Ssh => !app.editor_host.trim().is_empty(),
    };
    let can_save = !app.editor_name.trim().is_empty() && detail_filled;

    let mut button_row = row![
        button(label("Save"))
            .on_press_maybe(can_save.then_some(Message::EditorSave))
            .width(BUTTON_WIDTH)
            .height(BUTTON_HEIGHT)
            .padding(0)
            .style(primary_style),
        button(label("Cancel"))
            .on_press(Message::EditorCancel)
            .width(BUTTON_WIDTH)
            .height(BUTTON_HEIGHT)
            .padding(0)
            .style(secondary_style),
    ]
    .spacing(8);

    if app.editor_selected.is_some() {
        button_row = button_row.push(
            button(label("New"))
                .on_press(Message::EditorNew)
                .width(BUTTON_WIDTH)
                .height(BUTTON_HEIGHT)
                .padding(0)
                .style(secondary_style),
        );
        button_row = button_row.push(
            button(label(if armed { "Sure?" } else { "Delete" }))
                .on_press(if armed {
                    Message::EditorConfirmDelete
                } else {
                    Message::EditorDelete
                })
                .width(BUTTON_WIDTH)
                .height(BUTTON_HEIGHT)
                .padding(0)
                .style(danger_style),
        );
    }

    let mut form_items: Vec<Element<'_, Message>> = Vec::new();
    if app.first_run {
        form_items.push(
            container(
                text("Welcome — these are example entries. Click one to edit, delete them, or add your own.")
                    .size(metrics.detail_font_size)
                    .color(fg),
            )
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(Background::Color(bg)),
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
    form_items.extend([
        name_input.into(),
        type_toggle(app.editor_entry_type, colors, metrics),
        conditional_fields,
        button_row.into(),
    ]);

    let form = container(Column::with_children(form_items).spacing(10))
        .width(Length::Fill)
        .padding(16);

    let entries: Vec<Element<'_, Message>> = app
        .config
        .entry
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            row_item(
                entry.icon(),
                entry.name(),
                &entry.display_detail(),
                &[],
                &[],
                app.editor_selected == Some(idx),
                Message::EditorSelectEntry(idx),
                colors,
                &metrics,
            )
        })
        .collect();

    let entries_header = container(section_header("Entries", metrics, colors)).padding(
        iced::Padding {
            top: 0.0,
            right: metrics.row_inset,
            bottom: 0.0,
            left: metrics.row_inset,
        },
    );

    let entry_list = container(
        scrollable(Column::with_children(entries).spacing(0))
            .height(Length::Fill)
            .style(overlay_scrollbar(colors)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(iced::Padding {
        top: 2.0,
        right: metrics.row_inset,
        bottom: metrics.list_padding,
        left: metrics.row_inset,
    });

    let hints = hint_bar(
        colors,
        &metrics,
        &[
            ("↵", "Save"),
            ("Tab", "Next Field"),
            ("Ctrl+N", "New"),
            ("Esc", "Back"),
        ],
    );

    let content = column![
        header,
        hairline(colors),
        form,
        entries_header,
        entry_list,
        hints
    ]
    .spacing(0);

    panel(content, colors, &metrics, Length::Fill, None)
}

/// Segmented Directory | SSH toggle.
fn type_toggle<'a>(
    selected: EntryType,
    colors: &AppColors,
    metrics: Metrics,
) -> Element<'a, Message> {
    let bg = colors.background;
    let fg = colors.foreground;
    let muted = colors.muted;
    let border_color = colors.border;
    let highlight = colors.highlight;

    let segment = move |label: &'static str, value: EntryType| {
        let is_selected = value == selected;
        button(
            container(text(label).size(metrics.detail_font_size))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::Alignment::Center)
                .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .on_press(Message::EditorTypeChanged(value))
        .style(move |_theme: &iced::Theme, _status: button::Status| button::Style {
            background: Some(Background::Color(if is_selected { highlight } else { bg })),
            text_color: if is_selected { fg } else { muted },
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
    };

    container(
        row![
            segment("Directory", EntryType::Directory),
            segment("SSH", EntryType::Ssh),
        ]
        .spacing(2),
    )
    .width(220.0)
    .height(BUTTON_HEIGHT)
    .padding(2)
    .style(move |_theme: &iced::Theme| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}
