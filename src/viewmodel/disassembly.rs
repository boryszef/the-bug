//! The disassembly menu: items that can be taken apart.

use super::inventory;
use crate::game::{Item, Player, disassembly_for};

/// The items the player is carrying — storage and the bag combined, since
/// disassembling now draws on either — that can be taken apart, in
/// [`Item`] order (a stable, index-addressable list for the popup cursor).
pub fn options(player: &Player) -> Vec<Item> {
    inventory::combined_sorted(player)
        .into_iter()
        .map(|(item, _)| item)
        .filter(|&item| disassembly_for(item).is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_when_nothing_is_disassemblable() {
        let mut player = Player::default();
        player.inventory.insert(Item::Vine, 3);
        player.inventory.insert(Item::Arrow, 1); // the Arrow recipe is craft-only

        assert!(options(&player).is_empty());
    }

    #[test]
    fn lists_disassemblable_outputs_in_item_order() {
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

    #[test]
    fn an_item_held_only_in_the_bag_is_still_offered() {
        let mut player = Player::default();
        player.bag.insert(Item::StoneAxe, 1);

        assert_eq!(options(&player), vec![Item::StoneAxe]);
    }
}
