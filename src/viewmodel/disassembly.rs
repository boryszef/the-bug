//! The disassembly menu: inventory items that can be taken apart.

use super::inventory;
use crate::game::{Item, Player, reversible_recipe_for};

/// The items the player is carrying that some reversible recipe produces, in
/// [`Item`] order (a stable, index-addressable list for the popup cursor).
pub fn options(player: &Player) -> Vec<Item> {
    inventory::sorted(player)
        .into_iter()
        .map(|(item, _)| item)
        .filter(|&item| reversible_recipe_for(item).is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_when_nothing_is_disassemblable() {
        let mut player = Player::default();
        player.inventory.insert(Item::Vine, 3);
        player.inventory.insert(Item::Arrow, 1); // Arrow recipe is not reversible

        assert!(options(&player).is_empty());
    }

    #[test]
    fn lists_reversible_outputs_in_item_order() {
        let mut player = Player::default();
        player.inventory.insert(Item::Vine, 2); // filtered out
        player.inventory.insert(Item::StoneAxe, 1);
        player.inventory.insert(Item::WoodenBow, 1);

        assert_eq!(options(&player), vec![Item::StoneAxe, Item::WoodenBow]);
    }

    #[test]
    fn exhausted_items_are_omitted() {
        let mut player = Player::default();
        player.inventory.insert(Item::StoneAxe, 0);

        assert!(options(&player).is_empty());
    }
}
