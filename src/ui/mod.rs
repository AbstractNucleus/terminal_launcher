mod editor;
mod launcher;

pub use editor::editor_view;
pub use launcher::launcher_view;

use iced::font::{self, Font};
use iced::widget::scrollable::{self, AutoScroll, Rail, Scroller};
use iced::widget::text::Span;
use iced::widget::{button, column, container, rich_text, row, rule, span, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length, Shadow, Vector};

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
pub fn panel<'a>(
    content: impl Into<Element<'a, Message>>,
    colors: &AppColors,
    metrics: &Metrics,
) -> Element<'a, Message> {
    let surface = colors.surface;
    let border_color = colors.border;
    let radius = metrics.panel_radius;

    let inner = container(content.into())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &iced::Theme| container::Style {
            background: Some(Background::Color(surface)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: radius.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 32.0,
            },
            ..Default::default()
        });

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
                width: if focused { 1.0 } else { 0.0 },
                radius: 6.0.into(),
            },
            icon: muted,
            placeholder: muted,
            value: fg,
            selection: highlight,
        }
    }
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
    let border = colors.border;
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

    let label = row![icon_el, Space::new().width(8.0), name_el, detail_el]
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
                        color: border,
                        width: 1.0,
                        radius: radius.into(),
                    },
                    ..Default::default()
                }
            } else {
                button::Style {
                    background: None,
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

/// Footer hint bar with a top hairline.
pub fn hint_bar<'a>(colors: &AppColors, metrics: &Metrics) -> Element<'a, Message> {
    let muted = colors.muted;
    let border = colors.border;

    let hairline = rule::horizontal(1).style(move |_theme: &iced::Theme| rule::Style {
        color: border,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    });

    let hints = text("⇅ Select    ↵ Open    ^E Edit    esc Close")
        .size(metrics.hint_font_size)
        .color(muted);

    let bar = container(hints)
        .width(Length::Fill)
        .height(metrics.hint_bar_height - 1.0)
        .padding(iced::Padding {
            top: 0.0,
            right: metrics.row_inset,
            bottom: 0.0,
            left: metrics.row_inset,
        })
        .align_y(iced::Alignment::Center);

    column![hairline, bar].into()
}

pub fn overlay_scrollbar(
    colors: &AppColors,
) -> impl Fn(&iced::Theme, scrollable::Status) -> scrollable::Style + Copy {
    let muted = colors.muted;
    move |_theme: &iced::Theme, _status: scrollable::Status| {
        let rail = Rail {
            background: None,
            border: Border::default(),
            scroller: Scroller {
                background: Background::Color(muted),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        };
        scrollable::Style {
            container: container::Style::default(),
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
