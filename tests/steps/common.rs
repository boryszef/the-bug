//! Steps shared across the feature files: game setup, storage/bag contents,
//! the craft menu, and the "nothing was logged" check. Feature-specific
//! actions and assertions live in the matching `tests/steps/<area>.rs`.
//!
//! State is forced through the crate's public surface only — the `pub`
//! fields `player.inventory` / `player.bag` / `player.coordinates`,
//! `Player::grant_recipe`, `Map::from_terrain` — and read back through
//! public getters and `viewmodel`. See `docs/functional-tests.md`.

use cucumber::{given, then};
use the_bug::viewmodel;

use crate::steps::world::{GameWorld, complete_quest, item, quest};

#[given("a new game")]
async fn new_game(world: &mut GameWorld) {
    *world = GameWorld::default();
}

// Regex rather than `{int} {word}` so multi-word item names ("Stone Axe")
// are captured whole.

#[given(regex = r"^the player has (\d+) (.+) in storage$")]
async fn has_in_storage(world: &mut GameWorld, count: u32, name: String) {
    world.game.player.inventory.insert(item(&name), count);
}

#[given(regex = r"^the player has (\d+) (.+) in the bag$")]
async fn has_in_bag(world: &mut GameWorld, count: u32, name: String) {
    world.game.player.bag.insert(item(&name), count);
}

#[given("the player is away from the village")]
async fn away_from_village(world: &mut GameWorld) {
    world.game.player.coordinates = (5, 5);
    assert!(!world.game.at_craftable_location());
}

#[given(regex = r#"^the player knows the "([^"]+)" recipe$"#)]
async fn knows_recipe(world: &mut GameWorld, recipe: String) {
    assert!(
        world.game.player.grant_recipe(&recipe),
        "no craftable recipe named {recipe:?}"
    );
}

#[given(regex = r#"^the player has completed "([^"]+)"$"#)]
async fn has_completed_quest(world: &mut GameWorld, name: String) {
    complete_quest(&mut world.game, quest(&name));
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

#[then(regex = r"^(?:the )?bag contains (\d+) (.+)$")]
async fn bag_contains(world: &mut GameWorld, count: u32, name: String) {
    let have = world
        .game
        .player
        .bag
        .get(&item(&name))
        .copied()
        .unwrap_or(0);
    assert_eq!(have, count, "bag holds {have} {name}, expected {count}");
}

#[then("craft menu is empty")]
async fn craft_menu_is_empty(world: &mut GameWorld) {
    let offered: Vec<&str> = viewmodel::crafting::options(&world.game.player)
        .iter()
        .map(|option| option.id)
        .collect();
    assert!(offered.is_empty(), "the craft menu offers {offered:?}");
}

#[then(regex = r#"^an? "([^"]+)" craft is offered in the craft menu$"#)]
async fn craft_is_offered(world: &mut GameWorld, recipe: String) {
    let offered = viewmodel::crafting::options(&world.game.player)
        .iter()
        .any(|option| option.id == recipe);
    assert!(offered, "the craft menu does not offer {recipe:?}");
}

#[then("no event was logged")]
async fn no_event_was_logged(world: &mut GameWorld) {
    assert!(
        !world.logged_something(),
        "expected no new event, got {:?}",
        &world.game.events()[world.events_before..]
    );
}
