//! The Items tab: the player's Bag and Inventory (storage) prepared for
//! side-by-side display. Mirrors `viewmodel::inventory::sorted` for the
//! bag side rather than reshaping that already-shared function.

use std::collections::HashMap;

use crate::game::{Item, Player};

/// One side of the Items tab, sorted by [`Item`] order with exhausted
/// (quantity zero) entries omitted — same convention as
/// [`crate::viewmodel::inventory::sorted`].
pub struct Overview {
    pub bag: Vec<(Item, u32)>,
    /// `(items carried, bag capacity)`, both summed across every item type.
    pub bag_progress: (u32, u32),
    pub storage: Vec<(Item, u32)>,
}

pub fn overview(player: &Player) -> Overview {
    Overview {
        bag: sorted(&player.bag),
        bag_progress: player.bag_progress(),
        storage: crate::viewmodel::inventory::sorted(player),
    }
}

fn sorted(items: &HashMap<Item, u32>) -> Vec<(Item, u32)> {
    let mut sorted: Vec<(Item, u32)> = items
        .iter()
        .filter(|&(_, &quantity)| quantity > 0)
        .map(|(&item, &quantity)| (item, quantity))
        .collect();
    sorted.sort_by_key(|&(item, _)| item);
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Item::{Cord, Stick, Stone};

    #[test]
    fn empty_player_yields_empty_bag_and_storage() {
        let overview = overview(&Player::default());
        assert!(overview.bag.is_empty());
        assert!(overview.storage.is_empty());
        assert_eq!(
            overview.bag_progress,
            (0, Player::default().bag_progress().1)
        );
    }

    #[test]
    fn bag_and_storage_are_each_sorted_by_item_order_independently() {
        let mut player = Player::default();
        player.bag.insert(Cord, 1);
        player.bag.insert(Stick, 4);
        player.inventory.insert(Stone, 2);

        let overview = overview(&player);

        assert_eq!(overview.bag, vec![(Stick, 4), (Cord, 1)]);
        assert_eq!(overview.storage, vec![(Stone, 2)]);
    }

    #[test]
    fn zero_quantity_entries_are_omitted_from_the_bag() {
        let mut player = Player::default();
        player.bag.insert(Stick, 3);
        player.bag.insert(Cord, 0);

        assert_eq!(overview(&player).bag, vec![(Stick, 3)]);
    }

    #[test]
    fn bag_progress_reflects_the_bags_running_total() {
        let mut player = Player::default();
        player.bag.insert(Stick, 3);
        player.bag.insert(Stone, 2);

        let capacity = Player::default().bag_progress().1;
        assert_eq!(overview(&player).bag_progress, (5, capacity));
    }
}
