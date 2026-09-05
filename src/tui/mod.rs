mod app;
mod craft;
mod disassemble;
mod experiment;
mod quests;

pub use app::App;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, BorderType, Widget},
};

/// Draws a rounded bordered block titled `title` filling `area`, and returns
/// the inner rect for the panel's content.
fn panel_frame(area: Rect, title: &str, buf: &mut Buffer) -> Rect {
    let block = Block::bordered()
        .title(title)
        .border_type(BorderType::Rounded);
    let inner = block.inner(area);
    block.render(area, buf);
    inner
}
