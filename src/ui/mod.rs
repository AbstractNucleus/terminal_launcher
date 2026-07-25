//! The tiny-skia backend repaints only the changed primitives inside a damage
//! region, clearing it to the (transparent) window background first. Any
//! widget that can redraw on its own — rows, the search input, scrollbars,
//! headers — must therefore carry an explicit opaque background, or its
//! redraw punches a transparent hole in the panel.

mod editor;
mod launcher;

pub use editor::editor_view;
pub use launcher::launcher_view;

use iced::font::{self, Font};
use iced::widget::scrollable::{self, AutoScroll, Rail, Scroller};
use iced::widget::text::Span;
use iced::widget::{
    button, column, container, mouse_area, rich_text, row, rule, span, text, text_input, Space,
};
use iced::{Background, Border, Color, Element, Length, Shadow};

use crate::app::Message;
use crate::theme::{AppColors, Metrics};

pub const INTER: Font = Font::with_name("Inter");
pub const INTER_SEMIBOLD: Font = Font {
    family: font::Family::Name("Inter"),
    weight: font::Weight::Semibold,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};
pub const CODICON: Font = Font::with_name("codicon");

/// Transparent outer margin + rounded shadowed panel.
///
/// The window has a fixed size; `height` sizes the framed panel within it
/// (`Shrink` for content-sized, `Fill` to cover the window). With `on_dismiss`,
/// clicks on the transparent area below the panel emit that message.
pub fn panel<'a>(
    content: impl Into<Element<'a, Message>>,
    colors: &AppColors,
    metrics: &Metrics,
    height: Length,
    on_dismiss: Option<Message>,
) -> Element<'a, Message> {
    let surface = colors.surface;
    let border_color = colors.border;
    let radius = metrics.panel_radius;

    let framed = container(content.into())
        .width(Length::Fill)
        .height(height)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(Background::Color(surface)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius.into(),
            },
            // No renderer-drawn shadow: tiny-skia re-blends translucent
            // primitives on partial redraws, visibly darkening the panel
            // with every interaction.
            ..Default::default()
        });

    let inner: Element<'a, Message> = match on_dismiss {
        Some(message) => column![
            framed,
            mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
                .on_press(message),
        ]
        .height(Length::Fill)
        .into(),
        None => framed.into(),
    };

    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(metrics.shadow_inset)
        .style(|_theme: &iced::Theme| container::Style {
            background: None,
            ..Default::default()
        })
        .into()
}

/// Shared text-input style. `well` is the field fill.
pub fn field_style(
    colors: &AppColors,
    well: Color,
) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style + Copy {
    let fg = colors.foreground;
    let muted = colors.muted;
    let accent = colors.accent;
    let border_color = colors.border;
    let highlight = colors.highlight;

    move |_theme: &iced::Theme, status: text_input::Status| {
        let focused = matches!(status, text_input::Status::Focused { .. });
        text_input::Style {
            background: Background::Color(well),
            border: Border {
                color: if focused { accent } else { border_color },
                width: 1.0,
                radius: 6.0.into(),
            },
            icon: muted,
            placeholder: muted,
            value: fg,
            selection: highlight,
        }
    }
}

/// Muted section label with a hairline running to the right.
pub fn section_header<'a>(
    label: &'a str,
    metrics: Metrics,
    colors: &AppColors,
) -> Element<'a, Message> {
    let label_text = text(label)
        .size(metrics.header_font_size)
        .color(colors.muted);
    let surface = colors.surface;

    container(
        row![label_text, hairline(colors)]
            .spacing(8.0)
            .align_y(iced::Alignment::Center),
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
    .style(move |_theme: &iced::Theme| container::Style {
        background: Some(Background::Color(surface)),
        ..Default::default()
    })
    .into()
}

/// Borderless launcher search style: bare text on the panel surface, no box.
pub fn bare_input_style(
    colors: &AppColors,
) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style + Copy {
    let fg = colors.foreground;
    let muted = colors.muted;
    let accent = colors.accent;
    let surface = colors.surface;

    move |_theme: &iced::Theme, _status: text_input::Status| text_input::Style {
        background: Background::Color(surface),
        border: Border::default(),
        icon: muted,
        placeholder: muted,
        value: fg,
        selection: Color { a: 0.35, ..accent },
    }
}

/// 1px separator in the border color.
pub fn hairline<'a>(colors: &AppColors) -> Element<'a, Message> {
    let border = colors.border;
    rule::horizontal(1)
        .style(move |_theme: &iced::Theme| rule::Style {
            color: border,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        })
        .into()
}

fn make_span(
    content: String,
    matched: bool,
    base: Color,
    accent: Color,
) -> Span<'static, (), Font> {
    if matched {
        span(content).color(accent).font(INTER_SEMIBOLD)
    } else {
        span(content).color(base).font(INTER)
    }
}

/// Build rich-text with matched characters in accent + semibold.
pub fn highlighted_text(
    content: &str,
    match_positions: &[u32],
    base: Color,
    accent: Color,
    size: f32,
) -> Element<'static, Message> {
    let chars: Vec<char> = content.chars().collect();
    if chars.is_empty() {
        return text("").size(size).color(base).into();
    }

    let matched: std::collections::HashSet<u32> = match_positions.iter().copied().collect();
    let mut spans = Vec::new();
    let mut run = String::new();
    let mut run_matched = matched.contains(&0);

    for (i, ch) in chars.iter().enumerate() {
        let is_match = matched.contains(&(i as u32));
        if is_match != run_matched && !run.is_empty() {
            spans.push(make_span(std::mem::take(&mut run), run_matched, base, accent));
        }
        run_matched = is_match;
        run.push(*ch);
    }
    if !run.is_empty() {
        spans.push(make_span(run, run_matched, base, accent));
    }

    rich_text(spans).size(size).into()
}

/// Entry row: icon, highlighted name, muted inline detail.
pub fn row_item(
    icon: char,
    name: &str,
    detail: &str,
    name_positions: &[u32],
    detail_positions: &[u32],
    is_selected: bool,
    on_press: Message,
    colors: &AppColors,
    metrics: &Metrics,
) -> Element<'static, Message> {
    let fg = colors.foreground;
    let muted = colors.muted;
    let accent = colors.accent;
    let highlight = colors.highlight;
    let surface = colors.surface;
    let radius = metrics.row_radius;
    let row_height = metrics.entry_row_height;
    let row_inset = metrics.row_inset;

    let icon_el = text(icon.to_string())
        .font(CODICON)
        .size(metrics.name_font_size)
        .color(muted);

    let name_el = highlighted_text(name, name_positions, fg, accent, metrics.name_font_size);

    let spaced_detail = format!("  {detail}");
    let shifted: Vec<u32> = detail_positions.iter().map(|p| p + 2).collect();
    let detail_el = highlighted_text(
        &spaced_detail,
        &shifted,
        muted,
        accent,
        metrics.detail_font_size,
    );

    // Wrap in a Fill container: button lays its content out top-aligned,
    // so the row must center itself within the fixed-height button.
    let label = container(
        row![icon_el, Space::new().width(8.0), name_el, detail_el]
            .align_y(iced::Alignment::Center),
    )
    .height(Length::Fill)
    .align_y(iced::Alignment::Center);

    button(label)
        .on_press(on_press)
        .width(Length::Fill)
        .height(row_height)
        .padding(iced::Padding {
            top: 0.0,
            right: row_inset,
            bottom: 0.0,
            left: row_inset,
        })
        .style(move |_theme: &iced::Theme, _status: button::Status| {
            if is_selected {
                button::Style {
                    background: Some(Background::Color(highlight)),
                    text_color: fg,
                    border: Border {
                        radius: radius.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            } else {
                button::Style {
                    background: Some(Background::Color(surface)),
                    text_color: fg,
                    border: Border {
                        radius: radius.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
        })
        .into()
}

/// Footer hint bar with a top hairline and keycap-styled hints.
/// All hints sit left except the last, which is right-aligned.
pub fn hint_bar<'a>(
    colors: &AppColors,
    metrics: &Metrics,
    hints: &[(&'a str, &'a str)],
) -> Element<'a, Message> {
    let muted = colors.muted;
    let chip_bg = colors.highlight;
    let chip_border = colors.border;
    let size = metrics.hint_font_size;

    let keycap = move |key: &'a str| {
        container(text(key).size(size).color(muted))
            .padding(iced::Padding {
                top: 1.0,
                right: 5.0,
                bottom: 2.0,
                left: 5.0,
            })
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(Background::Color(chip_bg)),
                border: Border {
                    color: chip_border,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
    };
    let hint = move |key: &'a str, label: &'a str| {
        row![keycap(key), text(label).size(size).color(muted)]
            .spacing(6.0)
            .align_y(iced::Alignment::Center)
    };

    let mut bar = row![].spacing(14.0).align_y(iced::Alignment::Center);
    if let Some(((last_key, last_label), rest)) = hints.split_last() {
        for (key, label) in rest {
            bar = bar.push(hint(key, label));
        }
        bar = bar.push(Space::new().width(Length::Fill));
        bar = bar.push(hint(last_key, last_label));
    }

    let surface = colors.surface;
    let bar = container(bar)
        .width(Length::Fill)
        .height(metrics.hint_bar_height - 1.0)
        .padding(iced::Padding {
            top: 0.0,
            right: 16.0,
            bottom: 0.0,
            left: 16.0,
        })
        .align_y(iced::Alignment::Center)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(Background::Color(surface)),
            ..Default::default()
        });

    column![hairline(colors), bar].into()
}

pub fn overlay_scrollbar(
    colors: &AppColors,
) -> impl Fn(&iced::Theme, scrollable::Status) -> scrollable::Style + Copy {
    let muted = colors.muted;
    let surface = colors.surface;
    move |_theme: &iced::Theme, status: scrollable::Status| {
        // Overlay behavior: only show the scroller while the list is hovered or dragged.
        let show = match status {
            scrollable::Status::Hovered {
                is_vertical_scrollbar_disabled,
                ..
            } => !is_vertical_scrollbar_disabled,
            scrollable::Status::Dragged { .. } => true,
            scrollable::Status::Active { .. } => false,
        };
        let scroller_color = if show {
            Color { a: 0.4, ..muted }
        } else {
            Color::TRANSPARENT
        };
        let rail = Rail {
            background: Some(Background::Color(surface)),
            border: Border::default(),
            scroller: Scroller {
                background: Background::Color(scroller_color),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        };
        scrollable::Style {
            container: container::Style {
                background: Some(Background::Color(surface)),
                ..Default::default()
            },
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: AutoScroll {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                shadow: Shadow::default(),
                icon: muted,
            },
        }
    }
}
