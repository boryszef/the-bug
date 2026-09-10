//! Steps for `tests/features/craft.feature` — building a known recipe.
//! Setup and storage/equipment assertions live in `common.rs`.

use cucumber::{then, when};
use the_bug::game::EventKind;

use crate::steps::world::{GameWorld, item};

#[when(regex = r#"^the player crafts "([^"]+)"$"#)]
async fn crafts(world: &mut GameWorld, recipe: String) {
    world.mark();
    world.game.craft(&recipe);
}

#[then(regex = r#"^a craft of "([^"]+)" is logged$"#)]
async fn craft_is_logged(world: &mut GameWorld, name: String) {
    let output = item(&name);
    assert_eq!(
        world.game.events().last().map(|e| e.kind()),
        Some(&EventKind::Crafted { output }),
        "last event is not a craft of {name}"
    );
}
