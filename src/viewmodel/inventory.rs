//! The player's inventory (storage) prepared for display.

use std::collections::HashMap;

use crate::game::{Item, Player};

/// The player's storage as a list sorted by [`Item`] order, for a stable
/// display order and index-addressable cursor navigation. Exhausted items
/// (quantity zero) are left out.
pub fn sorted(player: &Player) -> Vec<(Item, u32)> {
    let mut items: Vec<(Item, u32)> = player
        .inventory
        .iter()
        .filter(|&(_, &quantity)| quantity > 0)
        .map(|(&item, &quantity)| (item, quantity))
        .collect();
    items.sort_by_key(|&(item, _)| item);
    items
}

/// Storage and the bag combined into one list, sorted by [`Item`] order —
/// what craft/experiment/disassemble now draw on (storage first, the bag
/// for any remainder — see `docs/bag-and-storage.md`), so this is what
/// their panels offer the player to pick from or check against. Exhausted
/// entries (combined quantity zero) are left out.
pub fn combined_sorted(player: &Player) -> Vec<(Item, u32)> {
    let mut totals: HashMap<Item, u32> = player.inventory.clone();
    for (&item, &quantity) in &player.bag {
        *totals.entry(item).or_insert(0) += quantity;
    }
    let mut items: Vec<(Item, u32)> = totals.into_iter().filter(|&(_, q)| q > 0).collect();
    items.sort_by_key(|&(item, _)| item);
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Item::{Cord, Stick, Stone};

    #[test]
    fn empty_inventory_yields_empty_list() {
        assert!(sorted(&Player::default()).is_empty());
    }

    #[test]
    fn entries_are_sorted_by_item_order() {
        let mut player = Player::default();
        player.inventory.insert(Cord, 1);
        player.inventory.insert(Stick, 4);
        player.inventory.insert(Stone, 2);

        assert_eq!(sorted(&player), vec![(Stick, 4), (Stone, 2), (Cord, 1)]);
    }

    #[test]
    fn zero_quantity_entries_are_omitted() {
        let mut player = Player::default();
        player.inventory.insert(Stick, 3);
        player.inventory.insert(Cord, 0);

        assert_eq!(sorted(&player), vec![(Stick, 3)]);
    }

    #[test]
    fn combined_sorted_merges_storage_and_bag_quantities_for_the_same_item() {
        let mut player = Player::default();
        player.inventory.insert(Stick, 3);
        player.bag.insert(Stick, 2);
        player.bag.insert(Stone, 1);

        assert_eq!(combined_sorted(&player), vec![(Stick, 5), (Stone, 1)]);
    }

    #[test]
    fn combined_sorted_omits_entries_that_are_zero_in_both_pools() {
        let mut player = Player::default();
        player.inventory.insert(Cord, 0);
        player.bag.insert(Cord, 0);
        player.bag.insert(Stick, 2);

        assert_eq!(combined_sorted(&player), vec![(Stick, 2)]);
    }
}
