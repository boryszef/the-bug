use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, List, ListItem, Paragraph, Widget},
};

use crate::game::Material;

/// The "experiment" overlay: pick materials from the inventory and try to
/// discover a recipe.
#[derive(Default)]
pub(super) struct Experiment {
    selected: Vec<(Material, u32)>,
    cursor: usize,
    focus: Focus,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum Focus {
    #[default]
    Available,
    Selected,
}

/// What [`App`](super::App) should do after the overlay consumed one key.
#[derive(PartialEq, Eq, Debug)]
pub(super) enum Outcome {
    /// Keep the overlay open.
    Stay,
    /// Close the overlay without running anything.
    Cancel,
    /// Close the overlay and run `game.experiment(&materials)`.
    Run(Vec<(Material, u32)>),
}

impl Experiment {
    /// Feeds one key to the overlay. `inventory` is the player's inventory in
    /// display order: `(material, owned quantity)`.
    pub(super) fn handle_key(&mut self, key: KeyCode, inventory: &[(Material, u32)]) -> Outcome {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => return Outcome::Cancel,
            KeyCode::Char('e') => return Outcome::Run(std::mem::take(&mut self.selected)),
            KeyCode::Tab => self.toggle_focus(),
            KeyCode::Up => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Down => {
                if self.cursor + 1 < self.list_len(inventory) {
                    self.cursor += 1;
                }
            }
            KeyCode::Right | KeyCode::Enter => self.add_current(inventory),
            KeyCode::Left | KeyCode::Backspace if self.focus == Focus::Selected => {
                self.remove_current();
            }
            _ => {}
        }
        Outcome::Stay
    }

    /// Draws the centered popup: an "Available" and a "Selected" column plus a
    /// key hint. The "Available" column shows what can still be added
    /// (owned minus already selected).
    pub(super) fn render(&self, area: Rect, buf: &mut Buffer, inventory: &[(Material, u32)]) {
        let inner = super::popup_frame(area, 70, 70, " Experiment ", buf);

        let rows = Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).split(inner);
        let columns = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        material_list(
            columns[0],
            buf,
            &self.available(inventory),
            " Available ",
            self.cursor_for(Focus::Available),
        );
        material_list(
            columns[1],
            buf,
            &self.selected,
            " Selected ",
            self.cursor_for(Focus::Selected),
        );

        Paragraph::new("↑↓ move   ←→ add/remove   Tab switch   e run   Esc cancel")
            .alignment(Alignment::Center)
            .render(rows[1], buf);
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Available => Focus::Selected,
            Focus::Selected => Focus::Available,
        };
        self.cursor = 0;
    }

    /// Length of the list the cursor currently moves through.
    fn list_len(&self, inventory: &[(Material, u32)]) -> usize {
        match self.focus {
            Focus::Available => inventory.len(),
            Focus::Selected => self.selected.len(),
        }
    }

    /// The cursor position, but only for the column that currently has focus.
    fn cursor_for(&self, focus: Focus) -> Option<usize> {
        (self.focus == focus).then_some(self.cursor)
    }

    fn selected_quantity(&self, material: Material) -> u32 {
        self.selected
            .iter()
            .find(|(m, _)| *m == material)
            .map_or(0, |(_, quantity)| *quantity)
    }

    /// Moves one unit of the material under the cursor into the selection,
    /// capped at the amount the player owns.
    fn add_current(&mut self, inventory: &[(Material, u32)]) {
        if self.focus != Focus::Available {
            return;
        }

        let Some(&(material, owned)) = inventory.get(self.cursor) else {
            return;
        };
        if self.selected_quantity(material) >= owned {
            return;
        }

        match self.selected.iter_mut().find(|(m, _)| *m == material) {
            Some((_, quantity)) => *quantity += 1,
            None => self.selected.push((material, 1)),
        }
    }

    /// Returns one unit of the material under the cursor to the inventory,
    /// dropping the row once it reaches zero.
    fn remove_current(&mut self) {
        if self.selected.is_empty() {
            return;
        }

        let index = self.cursor.min(self.selected.len() - 1);
        let (_, quantity) = &mut self.selected[index];
        *quantity -= 1;

        if *quantity == 0 {
            self.selected.remove(index);
            self.cursor = self.cursor.min(self.selected.len().saturating_sub(1));
        }
    }

    /// Inventory with the already-selected amounts removed (saturating).
    fn available(&self, inventory: &[(Material, u32)]) -> Vec<(Material, u32)> {
        inventory
            .iter()
            .map(|&(material, owned)| {
                (
                    material,
                    owned.saturating_sub(self.selected_quantity(material)),
                )
            })
            .collect()
    }
}

fn material_list(
    area: Rect,
    buf: &mut Buffer,
    materials: &[(Material, u32)],
    title: &str,
    cursor: Option<usize>,
) {
    let items = materials
        .iter()
        .enumerate()
        .map(|(index, &(material, quantity))| {
            let style = if cursor == Some(index) {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(format!("{material:?}  {quantity}")).style(style)
        });

    let list = List::new(items).block(Block::bordered().title(title));
    Widget::render(list, area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Material::{Stick, Stone, Vine};

    /// Sorted the same way `sorted_inventory` sorts: by debug name.
    fn inventory() -> Vec<(Material, u32)> {
        vec![(Stick, 1), (Stone, 3), (Vine, 2)]
    }

    #[test]
    fn cursor_up_saturates_at_zero() {
        let mut exp = Experiment::default();
        exp.handle_key(KeyCode::Up, &inventory());
        assert_eq!(exp.cursor, 0);
    }

    #[test]
    fn cursor_down_stops_at_available_end() {
        let mut exp = Experiment::default();
        let inv = inventory();
        for _ in 0..10 {
            exp.handle_key(KeyCode::Down, &inv);
        }
        assert_eq!(exp.cursor, inv.len() - 1);
    }

    #[test]
    fn cursor_down_stops_at_selected_end() {
        let mut exp = Experiment::default();
        let inv = inventory();
        exp.handle_key(KeyCode::Right, &inv); // select Stick
        exp.handle_key(KeyCode::Down, &inv);
        exp.handle_key(KeyCode::Right, &inv); // select Stone
        exp.handle_key(KeyCode::Tab, &inv); // focus Selected, cursor 0
        for _ in 0..10 {
            exp.handle_key(KeyCode::Down, &inv);
        }
        assert_eq!(exp.cursor, exp.selected.len() - 1);
    }

    #[test]
    fn tab_toggles_focus_and_resets_cursor() {
        let mut exp = Experiment::default();
        let inv = inventory();
        exp.handle_key(KeyCode::Down, &inv);
        assert_eq!(exp.cursor, 1);

        exp.handle_key(KeyCode::Tab, &inv);
        assert_eq!(exp.focus, Focus::Selected);
        assert_eq!(exp.cursor, 0);

        exp.handle_key(KeyCode::Tab, &inv);
        assert_eq!(exp.focus, Focus::Available);
    }

    #[test]
    fn right_adds_material_under_cursor() {
        let mut exp = Experiment::default();
        let inv = inventory();
        exp.handle_key(KeyCode::Down, &inv); // cursor -> Stone
        exp.handle_key(KeyCode::Right, &inv);
        assert_eq!(
            exp.handle_key(KeyCode::Char('e'), &inv),
            Outcome::Run(vec![(Stone, 1)])
        );
    }

    #[test]
    fn right_twice_increments_quantity() {
        let mut exp = Experiment::default();
        let inv = inventory();
        exp.handle_key(KeyCode::Down, &inv); // Stone, owned 3
        exp.handle_key(KeyCode::Right, &inv);
        exp.handle_key(KeyCode::Right, &inv);
        assert_eq!(
            exp.handle_key(KeyCode::Char('e'), &inv),
            Outcome::Run(vec![(Stone, 2)])
        );
    }

    #[test]
    fn add_is_capped_at_owned_quantity() {
        let mut exp = Experiment::default();
        let inv = inventory();
        for _ in 0..5 {
            exp.handle_key(KeyCode::Right, &inv); // Stick, owned 1
        }
        assert_eq!(
            exp.handle_key(KeyCode::Char('e'), &inv),
            Outcome::Run(vec![(Stick, 1)])
        );
    }

    #[test]
    fn right_is_noop_when_focus_is_selected() {
        let mut exp = Experiment::default();
        let inv = inventory();
        exp.handle_key(KeyCode::Right, &inv); // select Stick
        exp.handle_key(KeyCode::Tab, &inv); // focus Selected
        exp.handle_key(KeyCode::Right, &inv); // no-op
        assert_eq!(
            exp.handle_key(KeyCode::Char('e'), &inv),
            Outcome::Run(vec![(Stick, 1)])
        );
    }

    #[test]
    fn left_decrements_then_removes_entry() {
        let mut exp = Experiment::default();
        let inv = inventory();
        exp.handle_key(KeyCode::Down, &inv); // Stone
        exp.handle_key(KeyCode::Right, &inv);
        exp.handle_key(KeyCode::Right, &inv); // Stone x2
        exp.handle_key(KeyCode::Tab, &inv); // focus Selected

        exp.handle_key(KeyCode::Left, &inv);
        assert_eq!(exp.selected, vec![(Stone, 1)]);

        exp.handle_key(KeyCode::Left, &inv);
        assert!(exp.selected.is_empty());
        assert_eq!(exp.cursor, 0);
    }

    #[test]
    fn left_is_noop_when_focus_is_available() {
        let mut exp = Experiment::default();
        let inv = inventory();
        exp.handle_key(KeyCode::Right, &inv); // select Stick
        exp.handle_key(KeyCode::Left, &inv); // focus still Available -> no-op
        assert_eq!(
            exp.handle_key(KeyCode::Char('e'), &inv),
            Outcome::Run(vec![(Stick, 1)])
        );
    }

    #[test]
    fn esc_and_q_cancel() {
        let inv = inventory();
        assert_eq!(
            Experiment::default().handle_key(KeyCode::Esc, &inv),
            Outcome::Cancel
        );
        assert_eq!(
            Experiment::default().handle_key(KeyCode::Char('q'), &inv),
            Outcome::Cancel
        );
    }

    #[test]
    fn run_takes_selection_and_empties_state() {
        let mut exp = Experiment::default();
        let inv = inventory();
        exp.handle_key(KeyCode::Right, &inv);
        assert_eq!(
            exp.handle_key(KeyCode::Char('e'), &inv),
            Outcome::Run(vec![(Stick, 1)])
        );
        assert!(exp.selected.is_empty());
    }

    #[test]
    fn available_subtracts_selection_saturating() {
        let mut exp = Experiment::default();
        let inv = inventory();
        exp.handle_key(KeyCode::Right, &inv); // Stick 1 -> remaining 0
        assert_eq!(exp.available(&inv), vec![(Stick, 0), (Stone, 3), (Vine, 2)]);
    }
}
