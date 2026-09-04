use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, List, ListItem, Paragraph, Widget},
};

use crate::game::Item;
use crate::i18n::{self, Language};

/// The "disassemble" overlay: a scrollable list of the inventory items that can
/// be taken apart. Owns only the cursor; which items qualify comes from
/// [`crate::viewmodel::disassembly::options`].
#[derive(Default)]
pub(super) struct Disassemble {
    cursor: usize,
}

/// What [`App`](super::App) should do after the overlay consumed one key.
#[derive(PartialEq, Eq, Debug)]
pub(super) enum Outcome {
    /// Keep the overlay open.
    Stay,
    /// Close the overlay without disassembling.
    Cancel,
    /// Close the overlay and run `game.disassemble(item)`.
    Disassemble(Item),
}

impl Disassemble {
    /// Feeds one key to the overlay. `options` is the item list in display
    /// order.
    pub(super) fn handle_key(&mut self, key: KeyCode, options: &[Item]) -> Outcome {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => Outcome::Cancel,
            KeyCode::Up => {
                self.cursor = self.cursor.saturating_sub(1);
                Outcome::Stay
            }
            KeyCode::Down => {
                if self.cursor + 1 < options.len() {
                    self.cursor += 1;
                }
                Outcome::Stay
            }
            KeyCode::Enter | KeyCode::Char('d') => match options.get(self.cursor) {
                Some(&item) => Outcome::Disassemble(item),
                None => Outcome::Stay,
            },
            _ => Outcome::Stay,
        }
    }

    /// Draws the panel: the item list plus a key hint.
    pub(super) fn render(&self, area: Rect, buf: &mut Buffer, options: &[Item], lang: Language) {
        let inner = super::panel_frame(area, &i18n::ui("panel-disassemble-title", lang), buf);
        let rows = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(inner);

        if options.is_empty() {
            Paragraph::new(i18n::ui("disassemble-empty", lang)).render(rows[0], buf);
        } else {
            let items = options.iter().enumerate().map(|(index, &item)| {
                let style = if index == self.cursor {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(i18n::item(item, lang)).style(style)
            });

            let list = List::new(items)
                .block(Block::bordered().title(i18n::ui("panel-items-title", lang)));
            Widget::render(list, rows[0], buf);
        }

        Paragraph::new(i18n::ui("disassemble-hint", lang))
            .alignment(Alignment::Center)
            .render(rows[1], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> Vec<Item> {
        vec![Item::StoneAxe, Item::WoodenBow]
    }

    #[test]
    fn esc_and_q_cancel() {
        assert_eq!(
            Disassemble::default().handle_key(KeyCode::Esc, &options()),
            Outcome::Cancel
        );
        assert_eq!(
            Disassemble::default().handle_key(KeyCode::Char('q'), &options()),
            Outcome::Cancel
        );
    }

    #[test]
    fn cursor_up_saturates_at_zero() {
        let mut menu = Disassemble::default();
        menu.handle_key(KeyCode::Up, &options());
        assert_eq!(menu.cursor, 0);
    }

    #[test]
    fn cursor_down_stops_at_last_row() {
        let mut menu = Disassemble::default();
        let opts = options();
        for _ in 0..10 {
            menu.handle_key(KeyCode::Down, &opts);
        }
        assert_eq!(menu.cursor, opts.len() - 1);
    }

    #[test]
    fn enter_disassembles_the_item_under_the_cursor() {
        let mut menu = Disassemble::default();
        menu.handle_key(KeyCode::Down, &options());
        assert_eq!(
            menu.handle_key(KeyCode::Enter, &options()),
            Outcome::Disassemble(Item::WoodenBow)
        );
    }

    #[test]
    fn enter_on_empty_list_stays() {
        let mut menu = Disassemble::default();
        assert_eq!(menu.handle_key(KeyCode::Enter, &[]), Outcome::Stay);
    }
}
