use iced::widget::{column, container, mouse_area, scrollable, text, text_input, Column};
use iced::{Element, Length};

use crate::app::{App, Message};
use crate::fuzzy::split_match_positions;
use crate::theme::Metrics;
use crate::ui::{field_style, hint_bar, overlay_scrollbar, panel, row_item};

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
        .style(field_style(colors, colors.surface));

    let search = container(search)
        .width(Length::Fill)
        .height(metrics.input_row_height)
        .align_y(iced::Alignment::Center);

    let body: Element<'_, Message> = if app.config.entry.is_empty() {
        container(
            text("No entries yet. Press Ctrl+E to add some.")
                .size(metrics.name_font_size)
                .color(colors.muted),
        )
        .padding(20)
        .into()
    } else if app.filtered_indices.is_empty() {
        container(
            text("No matches.")
                .size(metrics.name_font_size)
                .color(colors.muted),
        )
        .padding(20)
        .into()
    } else if show_headers {
        browse_list(app, metrics)
    } else {
        flat_list(app, metrics)
    };

    let content = column![search, body, hint_bar(colors, &metrics)]
        .spacing(0)
        .padding(iced::Padding {
            top: metrics.panel_side_padding,
            right: metrics.panel_side_padding,
            bottom: 0.0,
            left: metrics.panel_side_padding,
        });

    panel(content, colors, &metrics)
}

fn section_header<'a>(
    label: &'a str,
    metrics: Metrics,
    muted: iced::Color,
) -> Element<'a, Message> {
    container(
        text(label)
            .size(metrics.header_font_size)
            .color(muted),
    )
    .width(Length::Fill)
    .height(metrics.section_header_height)
    .padding(iced::Padding {
        top: 0.0,
        right: metrics.row_inset,
        bottom: 0.0,
        left: metrics.row_inset,
    })
    .align_y(iced::Alignment::Center)
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
        children.push(section_header("Directories", metrics, colors.muted));
        for (view_idx, entry_idx) in dirs {
            children.push(entry_row(app, metrics, view_idx, entry_idx));
        }
    }

    if !sshs.is_empty() {
        children.push(section_header("SSH Hosts", metrics, colors.muted));
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
