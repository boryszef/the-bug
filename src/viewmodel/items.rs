//! The Items tab: the player's Equipment and Inventory (storage) prepared for
//! side-by-side display. Mirrors `viewmodel::inventory::sorted` for the
//! equipment side rather than reshaping that already-shared function.

use std::collections::HashMap;

use crate::game::{Item, Player};

/// One side of the Items tab, sorted by [`Item`] order with exhausted
/// (quantity zero) entries omitted — same convention as
/// [`crate::viewmodel::inventory::sorted`].
pub struct Overview {
    pub equipment: Vec<(Item, u32)>,
    /// `(items carried, equipment capacity)`, both summed across every item type.
    pub equipment_progress: (u32, u32),
    pub storage: Vec<(Item, u32)>,
}

pub fn overview(player: &Player) -> Overview {
    Overview {
        equipment: sorted(&player.equipment),
        equipment_progress: player.equipment_progress(),
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
    use crate::game::Item::{Branch, Cord, Stone};

    #[test]
    fn empty_player_yields_empty_equipment_and_storage() {
        let overview = overview(&Player::default());
        assert!(overview.equipment.is_empty());
        assert!(overview.storage.is_empty());
        assert_eq!(
            overview.equipment_progress,
            (0, Player::default().equipment_progress().1)
        );
    }

    #[test]
    fn equipment_and_storage_are_each_sorted_by_item_order_independently() {
        let mut player = Player::default();
        player.equipment.insert(Cord, 1);
        player.equipment.insert(Branch, 4);
        player.inventory.insert(Stone, 2);

        let overview = overview(&player);

        assert_eq!(overview.equipment, vec![(Branch, 4), (Cord, 1)]);
        assert_eq!(overview.storage, vec![(Stone, 2)]);
    }

    #[test]
    fn zero_quantity_entries_are_omitted_from_the_equipment() {
        let mut player = Player::default();
        player.equipment.insert(Branch, 3);
        player.equipment.insert(Cord, 0);

        assert_eq!(overview(&player).equipment, vec![(Branch, 3)]);
    }

    #[test]
    fn equipment_progress_reflects_the_running_total() {
        let mut player = Player::default();
        player.equipment.insert(Branch, 3);
        player.equipment.insert(Stone, 2);

        let capacity = Player::default().equipment_progress().1;
        assert_eq!(overview(&player).equipment_progress, (5, capacity));
    }
}
