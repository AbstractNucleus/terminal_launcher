use iced::widget::{column, container, mouse_area, scrollable, text, text_input, Column};
use iced::{Element, Length};

use crate::app::{App, Message};
use crate::fuzzy::split_match_positions;
use crate::theme::Metrics;
use crate::ui::{
    bare_input_style, hairline, hint_bar, overlay_scrollbar, panel, row_item, section_header,
};

pub fn launcher_view(app: &App) -> Element<'_, Message> {
    let colors = &app.colors;
    let metrics = Metrics::from_font_size(app.config.settings.font_size);
    let show_headers = app.search_query.is_empty();

    let search = text_input("Search...", &app.search_query)
        .on_input(Message::SearchChanged)
        .padding(iced::Padding {
            top: 0.0,
            right: 16.0,
            bottom: 0.0,
            left: 16.0,
        })
        .size(metrics.input_font_size)
        .id("search-input")
        .style(bare_input_style(colors));

    let search = container(search)
        .width(Length::Fill)
        .height(metrics.input_row_height)
        .align_y(iced::Alignment::Center);

    let body: Element<'_, Message> = if app.config.entry.is_empty() {
        empty_state("No entries yet. Press Ctrl+E to add some.", app, metrics)
    } else if app.filtered_indices.is_empty() {
        empty_state("No matches.", app, metrics)
    } else {
        // The window never resizes; size the list viewport to its content,
        // capped at max_visible_rows.
        let header_count = if show_headers {
            let dirs = app
                .filtered_indices
                .iter()
                .filter(|&&i| app.config.entry[i].is_directory())
                .count();
            let has_dirs = dirs > 0;
            let has_sshs = dirs < app.filtered_indices.len();
            has_dirs as usize + has_sshs as usize
        } else {
            0
        };
        let rows = app.filtered_indices.len().min(metrics.max_visible_rows);
        let list_height = rows as f32 * metrics.entry_row_height
            + header_count as f32 * metrics.section_header_height;

        let list = if show_headers {
            browse_list(app, metrics)
        } else {
            flat_list(app, metrics)
        };
        container(list)
            .width(Length::Fill)
            .height(list_height + 2.0 * metrics.list_padding)
            .padding(iced::Padding {
                top: metrics.list_padding,
                right: metrics.row_inset,
                bottom: metrics.list_padding,
                left: metrics.row_inset,
            })
            .into()
    };

    let hints = hint_bar(
        colors,
        &metrics,
        &[
            ("⇅", "Select"),
            ("↵", "Open"),
            ("Ctrl+E", "Edit"),
            ("Esc", "Close"),
        ],
    );
    let content = column![search, hairline(colors), body, hints].spacing(0);

    panel(
        content,
        colors,
        &metrics,
        Length::Shrink,
        Some(Message::WindowFocusLost),
    )
}

fn empty_state<'a>(message: &'a str, app: &'a App, metrics: Metrics) -> Element<'a, Message> {
    let surface = app.colors.surface;
    container(
        text(message)
            .size(metrics.name_font_size)
            .color(app.colors.muted),
    )
    .width(Length::Fill)
    .height(3.0 * metrics.entry_row_height)
    .padding(16)
    .style(move |_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(surface)),
        ..Default::default()
    })
    .into()
}

fn browse_list(app: &App, metrics: Metrics) -> Element<'_, Message> {
    let colors = &app.colors;
    let mut children: Vec<Element<'_, Message>> = Vec::new();

    let dirs: Vec<(usize, usize)> = app
        .filtered_indices
        .iter()
        .enumerate()
        .filter(|(_, &entry_idx)| app.config.entry[entry_idx].is_directory())
        .map(|(view_idx, _)| (view_idx, app.filtered_indices[view_idx]))
        .collect();

    let sshs: Vec<(usize, usize)> = app
        .filtered_indices
        .iter()
        .enumerate()
        .filter(|(_, &entry_idx)| !app.config.entry[entry_idx].is_directory())
        .map(|(view_idx, _)| (view_idx, app.filtered_indices[view_idx]))
        .collect();

    if !dirs.is_empty() {
        children.push(section_header("Directories", metrics, colors));
        for (view_idx, entry_idx) in dirs {
            children.push(entry_row(app, metrics, view_idx, entry_idx));
        }
    }

    if !sshs.is_empty() {
        children.push(section_header("SSH Hosts", metrics, colors));
        for (view_idx, entry_idx) in sshs {
            children.push(entry_row(app, metrics, view_idx, entry_idx));
        }
    }

    scrollable(Column::with_children(children).spacing(0))
        .id("launcher-scroll")
        .height(Length::Fill)
        .style(overlay_scrollbar(colors))
        .into()
}

fn flat_list(app: &App, metrics: Metrics) -> Element<'_, Message> {
    let colors = &app.colors;
    let entries: Vec<Element<'_, Message>> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(view_idx, &entry_idx)| entry_row(app, metrics, view_idx, entry_idx))
        .collect();

    scrollable(Column::with_children(entries).spacing(0))
        .id("launcher-scroll")
        .height(Length::Fill)
        .style(overlay_scrollbar(colors))
        .into()
}

fn entry_row(
    app: &App,
    metrics: Metrics,
    view_idx: usize,
    entry_idx: usize,
) -> Element<'static, Message> {
    let entry = &app.config.entry[entry_idx];
    let is_selected = view_idx == app.selected_index;
    let positions = app
        .match_positions
        .get(view_idx)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let name = entry.name().to_string();
    let detail = entry.display_detail();
    let (name_pos, detail_pos) = split_match_positions(positions, name.chars().count());

    let row = row_item(
        entry.icon(),
        &name,
        &detail,
        &name_pos,
        &detail_pos,
        is_selected,
        Message::LaunchAt(view_idx),
        &app.colors,
        &metrics,
    );

    mouse_area(row)
        .on_enter(Message::HoverAt(view_idx))
        .into()
}
