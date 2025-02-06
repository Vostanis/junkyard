use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Bar, BarChart, BarGroup},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

use crate::app::App;
use crate::pages::Page;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    // frame.render_widget(
    //     Paragraph::new(format!(
    //         "This is a tui template.\n\
    //             Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
    //             Press left and right to increment and decrement the counter respectively.\n",
    //     ))
    //     .block(
    //         Block::bordered()
    //             .title("Template")
    //             .title_alignment(Alignment::Center)
    //             .border_type(BorderType::Rounded),
    //     )
    //     .style(Style::default().fg(Color::White).bg(Color::Black))
    //     .centered(),
    //     frame.area(),
    // )

    match app.current_page {
        Page::Home => home_page(app, frame),

        _ => {}
    }
}

fn home_page(_app: &mut App, frame: &mut Frame) {
    let vertical = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(3),
    ]);
    let [info, diagram, tab] = vertical.areas(frame.area());

    let text = Text::from(Line::from("NVDA"));
    frame.render_widget(Paragraph::new(text).block(Block::bordered()), info);

    frame.render_widget(
        BarChart::default()
            .block(Block::bordered().title("BarChart"))
            .bar_width(3)
            .bar_gap(1)
            .group_gap(3)
            .bar_style(Style::new().yellow().on_red())
            .value_style(Style::new().red().bold())
            .label_style(Style::new().white())
            .data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
            .data(BarGroup::default().bars(&[Bar::default().value(10), Bar::default().value(20)]))
            .max(4),
        diagram,
    );

    frame.render_widget(Paragraph::new("hello").block(Block::bordered()), tab);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn searchbar(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
