//! The craft menu: the player's known recipes prepared for display.

use crate::game::{Item, Player};

/// One of a recipe's inputs, with how many the player has vs. needs.
pub struct CraftInput {
    pub item: Item,
    pub have: u32,
    pub need: u32,
}

impl CraftInput {
    /// Whether the player has enough of this input.
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
    /// The recipe's inputs with the player's current stock, in recipe order.
    pub inputs: Vec<CraftInput>,
    /// The player currently holds every input in the required amount.
    pub enabled: bool,
}

/// The player's known recipes as menu rows, in discovery order.
pub fn options(player: &Player) -> Vec<CraftOption> {
    player
        .known_recipes()
        .iter()
        .map(|recipe| {
            let inputs: Vec<CraftInput> = recipe
                .inputs()
                .iter()
                .map(|&(item, need)| CraftInput {
                    item,
                    need,
                    have: player.inventory.get(&item).copied().unwrap_or(0),
                })
                .collect();
            CraftOption {
                id: recipe.name(),
                output: recipe.output(),
                enabled: inputs.iter().all(CraftInput::met),
                inputs,
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
    fn option_is_enabled_only_when_inputs_are_affordable() {
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
    fn option_reports_each_input_with_have_and_need() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]); // discovers "Cord" (2x Vine), Vine now 0
        game.player.inventory.insert(Item::Vine, 5);

        let opt = options(&game.player).pop().unwrap();
        assert_eq!(opt.inputs.len(), 1);
        let vine = &opt.inputs[0];
        assert_eq!(vine.item, Item::Vine);
        assert_eq!((vine.have, vine.need), (5, 2));
        assert!(vine.met());
    }

    #[test]
    fn a_missing_input_is_reported_as_unmet() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]); // Vine drops to 0

        let opt = options(&game.player).pop().unwrap();
        let vine = &opt.inputs[0];
        assert_eq!((vine.have, vine.need), (0, 2));
        assert!(!vine.met());
        assert!(!opt.enabled);
    }
}
