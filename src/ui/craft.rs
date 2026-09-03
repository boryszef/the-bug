use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, List, ListItem, Paragraph, Widget},
};

use crate::viewmodel::crafting::CraftOption;

/// The "craft" overlay: a scrollable list of the player's known recipes. Owns
/// only the cursor; which recipes exist and whether they can be crafted comes
/// from [`crate::viewmodel::crafting::options`].
#[derive(Default)]
pub(super) struct Craft {
    cursor: usize,
}

/// What [`App`](super::App) should do after the overlay consumed one key.
#[derive(PartialEq, Eq, Debug)]
pub(super) enum Outcome {
    /// Keep the overlay open.
    Stay,
    /// Close the overlay without crafting.
    Cancel,
    /// Close the overlay and run `game.craft(name)`.
    Craft(&'static str),
}

impl Craft {
    /// Feeds one key to the overlay. `options` is the craft menu in display
    /// order.
    pub(super) fn handle_key(&mut self, key: KeyCode, options: &[CraftOption]) -> Outcome {
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
            KeyCode::Enter | KeyCode::Char('c') => match options.get(self.cursor) {
                Some(option) if option.enabled => Outcome::Craft(option.name),
                _ => Outcome::Stay,
            },
            _ => Outcome::Stay,
        }
    }

    /// Draws the centered popup: the recipe list plus a key hint. Recipes the
    /// player cannot currently afford are dimmed.
    pub(super) fn render(&self, area: Rect, buf: &mut Buffer, options: &[CraftOption]) {
        let inner = super::popup_frame(area, 40, 50, " Craft ", buf);
        let rows = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(inner);

        if options.is_empty() {
            Paragraph::new("You haven't discovered any recipes yet.").render(rows[0], buf);
        } else {
            let items = options.iter().enumerate().map(|(index, option)| {
                let mut style = Style::default();
                if !option.enabled {
                    style = style.add_modifier(Modifier::DIM);
                }
                if index == self.cursor {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                ListItem::new(option.name).style(style)
            });

            let list = List::new(items).block(Block::bordered().title(" Recipes "));
            Widget::render(list, rows[0], buf);
        }

        Paragraph::new("↑↓ move   Enter craft   Esc cancel")
            .alignment(Alignment::Center)
            .render(rows[1], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> Vec<CraftOption> {
        vec![
            CraftOption {
                name: "Cord",
                enabled: true,
            },
            CraftOption {
                name: "Stone Axe",
                enabled: false,
            },
        ]
    }

    #[test]
    fn esc_and_q_cancel() {
        assert_eq!(
            Craft::default().handle_key(KeyCode::Esc, &options()),
            Outcome::Cancel
        );
        assert_eq!(
            Craft::default().handle_key(KeyCode::Char('q'), &options()),
            Outcome::Cancel
        );
    }

    #[test]
    fn cursor_up_saturates_at_zero() {
        let mut craft = Craft::default();
        craft.handle_key(KeyCode::Up, &options());
        assert_eq!(craft.cursor, 0);
    }

    #[test]
    fn cursor_down_stops_at_last_row() {
        let mut craft = Craft::default();
        let opts = options();
        for _ in 0..10 {
            craft.handle_key(KeyCode::Down, &opts);
        }
        assert_eq!(craft.cursor, opts.len() - 1);
    }

    #[test]
    fn enter_on_enabled_recipe_crafts_it() {
        let mut craft = Craft::default();
        assert_eq!(
            craft.handle_key(KeyCode::Enter, &options()),
            Outcome::Craft("Cord")
        );
    }

    #[test]
    fn enter_on_disabled_recipe_stays() {
        let mut craft = Craft::default();
        craft.handle_key(KeyCode::Down, &options()); // -> "Stone Axe", disabled
        assert_eq!(craft.handle_key(KeyCode::Enter, &options()), Outcome::Stay);
    }
}
