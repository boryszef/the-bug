use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    text::Line,
    widgets::{Paragraph, Widget},
};

/// Draws the keyboard-shortcuts overlay.
pub(super) fn render(area: Rect, buf: &mut Buffer) {
    let inner = super::popup_frame(area, 60, 50, " Help ", buf);

    let text = vec![
        Line::from("Keyboard shortcuts"),
        Line::from(""),
        Line::from("← → ↑ ↓    Move"),
        Line::from("s          Search"),
        Line::from("c          Craft menu"),
        Line::from("e          Experiment"),
        Line::from("?          Show help"),
        Line::from("q          Quit"),
    ];

    Paragraph::new(text)
        .alignment(Alignment::Left)
        .render(inner, buf);
}
