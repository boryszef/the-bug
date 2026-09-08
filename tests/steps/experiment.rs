//! Steps for `tests/features/experiment.feature` — combining items to
//! discover and build a recipe. Setup and the storage/craft-menu assertions
//! are in `common.rs`.

use cucumber::{then, when};
use the_bug::game::Item;

use crate::steps::world::{GameWorld, item};

/// Parses `"2 Vine"`, `"1 Battery and 1 Speaker"`, `"1 Branch, 1 Stone and
/// 1 Cord"` into a `(Item, count)` combination.
fn item_list(spec: &str) -> Vec<(Item, u32)> {
    spec.replace(" and ", ", ")
        .split(',')
        .map(str::trim)
        .filter(|piece| !piece.is_empty())
        .map(|piece| {
            let (count, name) = piece.split_once(' ').expect("`<count> <item>`");
            (item(name), count.parse().expect("a numeric count"))
        })
        .collect()
}

#[when(regex = r"^the player experiments with (.+)$")]
async fn experiments_with(world: &mut GameWorld, spec: String) {
    let combo = item_list(&spec);
    world.mark();
    world.game.experiment(&combo);
}

#[then(regex = r#"^the recipe "([^"]+)" is known$"#)]
async fn recipe_is_known(world: &mut GameWorld, recipe: String) {
    assert!(
        world
            .game
            .player
            .known_recipes()
            .iter()
            .any(|r| r.name() == recipe),
        "the player does not know {recipe:?}"
    );
}

#[then(regex = r#"^the recipe "([^"]+)" is not known$"#)]
async fn recipe_is_not_known(world: &mut GameWorld, recipe: String) {
    assert!(
        world
            .game
            .player
            .known_recipes()
            .iter()
            .all(|r| r.name() != recipe),
        "the player unexpectedly knows {recipe:?}"
    );
}

#[then(regex = r"^the player has (\d+) experience$")]
async fn player_has_experience(world: &mut GameWorld, xp: u32) {
    assert_eq!(world.game.player.experience, xp);
}
