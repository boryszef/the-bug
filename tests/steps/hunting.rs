//! Steps for `tests/features/hunting.feature`. A successful hunt rolls the
//! tile's game and stays a unit test. Setup and equipment assertions, and
//! the "no event was logged" refusal check, are in `common.rs`.

use cucumber::when;

use crate::steps::world::GameWorld;

#[when("the player hunts")]
async fn hunts(world: &mut GameWorld) {
    world.mark();
    world.game.hunt();
}
