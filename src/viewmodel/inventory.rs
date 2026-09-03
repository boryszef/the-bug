//! The player's inventory prepared for display.

use crate::game::{Material, Player};

/// The player's inventory as a list sorted by [`Material`] order, for a stable
/// display order and index-addressable cursor navigation. Exhausted items
/// (quantity zero) are left out.
pub fn sorted(player: &Player) -> Vec<(Material, u32)> {
    let mut materials: Vec<(Material, u32)> = player
        .inventory
        .iter()
        .filter(|&(_, &quantity)| quantity > 0)
        .map(|(&material, &quantity)| (material, quantity))
        .collect();
    materials.sort_by_key(|&(material, _)| material);
    materials
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Material::{Cord, Stick, Stone};

    #[test]
    fn empty_inventory_yields_empty_list() {
        assert!(sorted(&Player::default()).is_empty());
    }

    #[test]
    fn entries_are_sorted_by_material_order() {
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
}
