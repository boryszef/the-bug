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

#[then("the craft is refused as an unknown recipe")]
async fn refused_unknown(world: &mut GameWorld) {
    assert!(
        matches!(
            world.game.events().last().map(|e| e.kind()),
            Some(EventKind::UnknownRecipe { .. })
        ),
        "last event is not an unknown-recipe refusal"
    );
}

#[then(regex = r#"^the craft is refused for want of a "([^"]+)"$"#)]
async fn refused_missing_tool(world: &mut GameWorld, tool: String) {
    let want = item(&tool);
    assert!(
        matches!(
            world.game.events().last().map(|e| e.kind()),
            Some(EventKind::CraftMissingTool { tool, .. }) if *tool == want
        ),
        "last event is not a missing-{tool} refusal"
    );
}
