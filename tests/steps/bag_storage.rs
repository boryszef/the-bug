//! Steps for `tests/features/bag-storage.feature` — `Game`-level transfers
//! between the bag and storage, and dropping. Setup and the contains/no-event
//! assertions are in `common.rs`.

use cucumber::{then, when};
use the_bug::game::EventKind;

use crate::steps::world::{GameWorld, item};

#[when(regex = r"^the player transfers (\d+) (.+) to storage$")]
async fn transfers_to_storage(world: &mut GameWorld, count: u32, name: String) {
    world.mark();
    world.game.transfer_to_storage(item(&name), count);
}

#[when(regex = r"^the player transfers (\d+) (.+) to the bag$")]
async fn transfers_to_bag(world: &mut GameWorld, count: u32, name: String) {
    world.mark();
    world.game.transfer_to_bag(item(&name), count);
}

#[when(regex = r"^the player drops (\d+) (.+) from storage$")]
async fn drops_from_storage(world: &mut GameWorld, count: u32, name: String) {
    world.mark();
    world.game.drop_from_storage(item(&name), count);
}

#[when(regex = r"^the player drops (\d+) (.+) from the bag$")]
async fn drops_from_bag(world: &mut GameWorld, count: u32, name: String) {
    world.mark();
    world.game.drop_from_bag(item(&name), count);
}

#[then(regex = r#"^a drop of "([^"]+)" is logged$"#)]
async fn drop_is_logged(world: &mut GameWorld, name: String) {
    let dropped = item(&name);
    assert_eq!(
        world.game.events().last().map(|e| e.kind()),
        Some(&EventKind::Dropped { item: dropped }),
        "last event is not a drop of {name}"
    );
}
