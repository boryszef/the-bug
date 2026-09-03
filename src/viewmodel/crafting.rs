//! The craft menu: the player's known recipes prepared for display.

use crate::game::Player;

/// One row of the craft menu.
pub struct CraftOption {
    pub name: &'static str,
    /// The player currently holds every input in the required amount.
    pub enabled: bool,
}

/// The player's known recipes as menu rows, in discovery order.
pub fn options(player: &Player) -> Vec<CraftOption> {
    player
        .known_recipes()
        .iter()
        .map(|recipe| CraftOption {
            name: recipe.name(),
            enabled: recipe
                .inputs()
                .iter()
                .all(|&(item, need)| player.inventory.get(&item).copied().unwrap_or(0) >= need),
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
        assert_eq!(opts[0].name, "Cord");
        assert!(!opts[0].enabled);

        game.player.inventory.insert(Item::Vine, 2);
        assert!(options(&game.player)[0].enabled);
    }
}
