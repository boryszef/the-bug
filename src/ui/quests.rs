use ratatui::{buffer::Buffer, layout::Rect, widgets::Paragraph, widgets::Widget};

/// Draws the quests panel. Placeholder: not wired to
/// `crate::game::Game::available_quests`/`accept_quest` yet.
pub(super) fn render(area: Rect, buf: &mut Buffer) {
    let inner = super::panel_frame(area, " Quests ", buf);
    Paragraph::new("Quests — coming soon.").render(inner, buf);
}
