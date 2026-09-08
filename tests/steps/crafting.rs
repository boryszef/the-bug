//! Steps for `tests/features/crafting.feature`.
//!
//! Every step touches only the crate's public API: `Game::experiment`, the
//! public `Player.inventory` field, and `viewmodel::crafting::options`. State
//! is forced through public fields and asserted through public getters — the
//! convention for this suite (`docs/functional-tests.md`).

use cucumber::{given, then, when};
use the_bug::game::Game;
use the_bug::viewmodel;

use crate::steps::world::{GameWorld, item};

#[given("a new game")]
async fn new_game(world: &mut GameWorld) {
    world.game = Game::default();
}

// Regex rather than `{int} {word}` so multi-word item names ("Stone Axe")
// are captured whole, and so "storage" reads with or without a leading "the".

#[given(regex = r"^the player has (\d+) (.+) in storage$")]
async fn player_has_in_storage(world: &mut GameWorld, count: u32, name: String) {
    world.game.player.inventory.insert(item(&name), count);
}

#[when(regex = r"^the player experiments with (\d+) (.+)$")]
async fn player_experiments_with(world: &mut GameWorld, count: u32, name: String) {
    world.game.experiment(&[(item(&name), count)]);
}

#[then(regex = r"^(?:the )?storage contains (\d+) (.+)$")]
async fn storage_contains(world: &mut GameWorld, count: u32, name: String) {
    let have = world
        .game
        .player
        .inventory
        .get(&item(&name))
        .copied()
        .unwrap_or(0);
    assert_eq!(have, count, "storage holds {have} {name}, expected {count}");
}

#[then(regex = r#"^an? "([^"]+)" craft is offered in the craft menu$"#)]
async fn craft_is_offered(world: &mut GameWorld, recipe: String) {
    let offered = viewmodel::crafting::options(&world.game.player)
        .iter()
        .any(|option| option.id == recipe);
    assert!(offered, "the craft menu does not offer {recipe:?}");
}

#[then("craft menu is empty")]
async fn craft_menu_is_empty(world: &mut GameWorld) {
    let offered: Vec<&str> = viewmodel::crafting::options(&world.game.player)
        .iter()
        .map(|option| option.id)
        .collect();
    assert!(offered.is_empty(), "the craft menu offers {offered:?}");
}
