use the_bug::game::{Game, Item};

/// The scenario state: one `Game`, recreated fresh per scenario. `Game`
/// already derives `Debug` and implements `Default`, which is all the
/// `World` derive needs.
#[derive(cucumber::World, Debug, Default)]
pub struct GameWorld {
    pub game: Game,
}

/// Resolves a Gherkin item name (its `Display` form, e.g. `"Stone Axe"`) to
/// the `Item` variant.
///
/// The crate's own `Item::ALL` is `#[cfg(test)]` and so unreachable from this
/// external test target; this table covers the items the feature files
/// currently name and grows as scenarios are added.
pub fn item(name: &str) -> Item {
    use Item::*;
    match name {
        "Stick" => Stick,
        "Stone" => Stone,
        "Vine" => Vine,
        "Cord" => Cord,
        other => panic!("no Item mapping for {other:?} — add it to tests/steps/world.rs"),
    }
}
