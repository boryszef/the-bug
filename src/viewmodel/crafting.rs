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

/// A tool the recipe requires (held, not consumed), and whether the player
/// currently has one.
pub struct CraftTool {
    pub item: Item,
    pub present: bool,
}

/// One row of the craft menu.
pub struct CraftOption {
    /// The recipe name, passed to [`crate::game::Game::craft`] — a lookup
    /// key, not display text.
    pub id: &'static str,
    pub output: Item,
    /// The recipe's consumables with the player's current stock, in recipe order.
    pub consumables: Vec<CraftConsumable>,
    /// The recipe's required tools and whether the player holds each, in recipe
    /// order.
    pub tools: Vec<CraftTool>,
    /// The player holds every consumable in the required amount and every tool.
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
                    // Storage and the equipment combined — craft/experiment now
                    // draw on either (storage first, the equipment for any
                    // remainder). Tools, below, stay storage-only: a tool
                    // is kept at the workshop where it's used, unlike a
                    // consumable.
                    have: player.inventory.get(&item).copied().unwrap_or(0)
                        + player.equipment.get(&item).copied().unwrap_or(0),
                })
                .collect();
            let tools: Vec<CraftTool> = recipe
                .tools()
                .iter()
                .map(|&item| CraftTool {
                    item,
                    present: player.inventory.get(&item).copied().unwrap_or(0) > 0,
                })
                .collect();
            CraftOption {
                id: recipe.name(),
                output: recipe.output(),
                enabled: consumables.iter().all(CraftConsumable::met)
                    && tools.iter().all(|t| t.present),
                consumables,
                tools,
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

    #[test]
    fn a_consumable_split_across_storage_and_the_equipment_still_counts_combined() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]); // discovers "Cord" (2x Vine), Vine now 0
        game.player.inventory.insert(Item::Vine, 1);
        game.player.equipment.insert(Item::Vine, 1);

        let opt = options(&game.player).pop().unwrap();
        let vine = &opt.consumables[0];
        assert_eq!((vine.have, vine.need), (2, 2));
        assert!(vine.met());
        assert!(opt.enabled);
    }

    #[test]
    fn a_missing_tool_disables_the_option_even_with_every_consumable() {
        let mut game = Game::default();
        game.player.grant_recipe("Wooden Bow"); // needs a Stone Axe tool
        game.player.inventory.insert(Item::Branch, 1);
        game.player.inventory.insert(Item::Cord, 1);

        let opt = options(&game.player).pop().unwrap();
        assert_eq!(opt.tools.len(), 1);
        assert_eq!(opt.tools[0].item, Item::StoneAxe);
        assert!(!opt.tools[0].present);
        assert!(!opt.enabled);

        game.player.inventory.insert(Item::StoneAxe, 1);
        let opt = options(&game.player).pop().unwrap();
        assert!(opt.tools[0].present);
        assert!(opt.enabled);
    }
}
