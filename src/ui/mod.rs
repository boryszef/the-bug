mod app;
mod craft;
mod disassemble;
mod experiment;
mod help;

pub use app::App;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Clear, Widget},
};

/// A `Rect` covering `percent_x` × `percent_y` of `area`, centered.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

/// Clears a centered popup area, draws a rounded bordered block titled `title`,
/// and returns the inner rect for the popup's content.
fn popup_frame(area: Rect, percent_x: u16, percent_y: u16, title: &str, buf: &mut Buffer) -> Rect {
    let area = centered_rect(percent_x, percent_y, area);
    Clear.render(area, buf);

    let block = Block::bordered()
        .title(title)
        .border_type(BorderType::Rounded);
    let inner = block.inner(area);
    block.render(area, buf);
    inner
}
