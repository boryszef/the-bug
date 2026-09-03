//! A running selection of items with quantities, bounded by what the
//! player owns. Drives the "experiment" flow and would drive a crafting flow
//! just the same.

use crate::game::Item;

/// Items picked so far, each with the quantity chosen. Quantities never
/// exceed what the player owns (the caller passes the owned amount in).
#[derive(Default, Debug)]
pub struct ItemSelection {
    picked: Vec<(Item, u32)>,
}

impl ItemSelection {
    /// The picked items, in the order they were first added.
    pub fn items(&self) -> &[(Item, u32)] {
        &self.picked
    }

    pub fn is_empty(&self) -> bool {
        self.picked.is_empty()
    }

    /// How many units of `item` are currently picked.
    pub fn quantity(&self, item: Item) -> u32 {
        self.picked
            .iter()
            .find(|(m, _)| *m == item)
            .map_or(0, |(_, quantity)| *quantity)
    }

    /// Picks one more unit of `item`, unless that would exceed `owned`.
    pub fn add(&mut self, item: Item, owned: u32) {
        if self.quantity(item) >= owned {
            return;
        }
        match self.picked.iter_mut().find(|(m, _)| *m == item) {
            Some((_, quantity)) => *quantity += 1,
            None => self.picked.push((item, 1)),
        }
    }

    /// Returns one unit of the entry at `index` to the player. Drops the entry
    /// once it reaches zero; returns `true` when that happened.
    pub fn decrement_at(&mut self, index: usize) -> bool {
        let Some((_, quantity)) = self.picked.get_mut(index) else {
            return false;
        };
        *quantity -= 1;
        if *quantity == 0 {
            self.picked.remove(index);
            true
        } else {
            false
        }
    }

    /// `inventory` with the picked amounts removed (saturating): what can still
    /// be added.
    pub fn available(&self, inventory: &[(Item, u32)]) -> Vec<(Item, u32)> {
        inventory
            .iter()
            .map(|&(item, owned)| (item, owned.saturating_sub(self.quantity(item))))
            .collect()
    }

    /// Takes the selection, leaving it empty.
    pub fn take(&mut self) -> Vec<(Item, u32)> {
        std::mem::take(&mut self.picked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Item::{Stick, Stone, Vine};

    fn inventory() -> Vec<(Item, u32)> {
        vec![(Stick, 1), (Stone, 3), (Vine, 2)]
    }

    #[test]
    fn add_accumulates_and_caps_at_owned() {
        let mut sel = ItemSelection::default();
        sel.add(Stone, 3);
        sel.add(Stone, 3);
        assert_eq!(sel.quantity(Stone), 2);

        for _ in 0..5 {
            sel.add(Stick, 1);
        }
        assert_eq!(sel.quantity(Stick), 1);
    }

    #[test]
    fn decrement_removes_entry_at_zero() {
        let mut sel = ItemSelection::default();
        sel.add(Stone, 3);
        sel.add(Stone, 3);

        assert!(!sel.decrement_at(0));
        assert_eq!(sel.items(), &[(Stone, 1)]);
        assert!(sel.decrement_at(0));
        assert!(sel.is_empty());
        assert!(!sel.decrement_at(0));
    }

    #[test]
    fn available_subtracts_selection_saturating() {
        let mut sel = ItemSelection::default();
        sel.add(Stick, 1);
        assert_eq!(
            sel.available(&inventory()),
            vec![(Stick, 0), (Stone, 3), (Vine, 2)]
        );
    }

    #[test]
    fn take_empties_the_selection() {
        let mut sel = ItemSelection::default();
        sel.add(Stick, 1);
        assert_eq!(sel.take(), vec![(Stick, 1)]);
        assert!(sel.is_empty());
    }
}
