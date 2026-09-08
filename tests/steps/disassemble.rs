//! Steps for `tests/features/disassemble.feature` — taking a carried item
//! apart. Setup and storage/equipment assertions live in `common.rs`.

use cucumber::{then, when};
use the_bug::game::EventKind;

use crate::steps::world::{GameWorld, item};

#[when(regex = r#"^the player disassembles "([^"]+)"$"#)]
async fn disassembles(world: &mut GameWorld, name: String) {
    world.mark();
    world.game.disassemble(item(&name));
}

#[then(regex = r#"^a disassembly of "([^"]+)" is logged$"#)]
async fn disassembly_is_logged(world: &mut GameWorld, name: String) {
    let taken = item(&name);
    assert_eq!(
        world.game.events().last().map(|e| e.kind()),
        Some(&EventKind::Disassembled { item: taken }),
        "last event is not a disassembly of {name}"
    );
}
