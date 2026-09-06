//! The craft menu: the player's known recipes prepared for display.

use crate::game::{Item, Player};

/// One of a recipe's consumables, with how many the player has vs. needs.
pub struct CraftConsumable {
    pub item: Item,
    pub have: u32,
    pub need: u32,
}

impl CraftConsumable {
    /// Whether the player has enough of this consumable.
    pub fn met(&self) -> bool {
        self.have >= self.need
    }
}

/// One row of the craft menu.
pub struct CraftOption {
    /// The recipe name, passed to [`crate::game::Game::craft`] — a lookup
    /// key, not display text.
    pub id: &'static str,
    pub output: Item,
    /// The recipe's consumables with the player's current stock, in recipe order.
    pub consumables: Vec<CraftConsumable>,
    /// The player currently holds every consumable in the required amount.
    pub enabled: bool,
}

/// The player's known recipes as menu rows, in discovery order.
pub fn options(player: &Player) -> Vec<CraftOption> {
    player
        .known_recipes()
        .iter()
        .map(|recipe| {
            let consumables: Vec<CraftConsumable> = recipe
                .consumables()
                .iter()
                .map(|&(item, need)| CraftConsumable {
                    item,
                    need,
                    have: player.inventory.get(&item).copied().unwrap_or(0),
                })
                .collect();
            CraftOption {
                id: recipe.name(),
                output: recipe.output(),
                enabled: consumables.iter().all(CraftConsumable::met),
                consumables,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, Item};

    #[test]
    fn no_recipes_yields_no_options() {
        assert!(options(&Player::default()).is_empty());
    }

    #[test]
    fn option_is_enabled_only_when_consumables_are_affordable() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]); // discovers "Cord", consumes the Vine

        let opts = options(&game.player);
        assert_eq!(opts.len(), 1);
        assert_eq!(opts[0].id, "Cord");
        assert_eq!(opts[0].output, Item::Cord);
        assert!(!opts[0].enabled);

        game.player.inventory.insert(Item::Vine, 2);
        assert!(options(&game.player)[0].enabled);
    }

    #[test]
    fn option_reports_each_consumable_with_have_and_need() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]); // discovers "Cord" (2x Vine), Vine now 0
        game.player.inventory.insert(Item::Vine, 5);

        let opt = options(&game.player).pop().unwrap();
        assert_eq!(opt.consumables.len(), 1);
        let vine = &opt.consumables[0];
        assert_eq!(vine.item, Item::Vine);
        assert_eq!((vine.have, vine.need), (5, 2));
        assert!(vine.met());
    }

    #[test]
    fn a_missing_consumable_is_reported_as_unmet() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]); // Vine drops to 0

        let opt = options(&game.player).pop().unwrap();
        let vine = &opt.consumables[0];
        assert_eq!((vine.have, vine.need), (0, 2));
        assert!(!vine.met());
        assert!(!opt.enabled);
    }
}
