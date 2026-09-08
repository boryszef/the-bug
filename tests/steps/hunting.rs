//! Steps for `tests/features/hunting.feature`. Only the deterministic
//! "unprepared" checks live here — a successful hunt rolls the tile's game
//! and stays a unit test. Setup and equipment assertions are in `common.rs`.

use cucumber::{then, when};
use the_bug::game::EventKind;

use crate::steps::world::{GameWorld, item};

#[when("the player hunts")]
async fn hunts(world: &mut GameWorld) {
    world.mark();
    world.game.hunt();
}

#[then(regex = r#"^the hunt reports a missing "([^"]+)"$"#)]
async fn hunt_reports_missing(world: &mut GameWorld, name: String) {
    let want = item(&name);
    assert!(
        matches!(
            world.game.events().last().map(|e| e.kind()),
            Some(EventKind::HuntUnprepared { missing }) if *missing == want
        ),
        "last event is not an unprepared hunt missing {name}"
    );
}
