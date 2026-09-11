use the_bug::game::{Direction, Game, Item, Map, Poi, QuestError, QuestID, TerrainType, Unlocked};

/// The scenario state: one `Game`, recreated fresh per scenario. `Game`
/// already derives `Debug` and implements `Default`, which is all the
/// `World` derive needs.
#[derive(cucumber::World, Debug, Default)]
pub struct GameWorld {
    pub game: Game,
    /// Event-log length captured at the start of the `When` step, so a
    /// `Then` can assert "nothing new was logged".
    pub events_before: usize,
    /// The outcome of the most recent `accept_quest`, for the "refused
    /// because …" assertions.
    pub accept_result: Option<Result<(), QuestError>>,
}

impl GameWorld {
    /// Snapshot the event-log length. Every `When` step calls this first.
    pub fn mark(&mut self) {
        self.events_before = self.game.events().len();
    }

    /// Whether the `When` step added any event-log line.
    pub fn logged_something(&self) -> bool {
        self.game.events().len() > self.events_before
    }
}

/// Resolves a Gherkin item name (its English `i18n` display text) to the
/// `Item` variant.
///
/// The crate's own `Item::ALL` is `#[cfg(test)]` and so unreachable from this
/// external test target; this table covers the items the feature files name
/// and grows as scenarios are added.
pub fn item(name: &str) -> Item {
    use Item::*;
    match name {
        "Branch" => Branch,
        "Stone" => Stone,
        "Vine" => Vine,
        "Cord" => Cord,
        "Stone Axe" => StoneAxe,
        "Arrow" => Arrow,
        "Wooden Bow" => WoodenBow,
        "Umbrella" => Umbrella,
        "Fabric" => Fabric,
        "Pole" => Pole,
        "Battery" => Battery,
        "Speaker" => Speaker,
        "Electronic Toy" => ElectronicToy,
        "Satchel" => Satchel,
        other => panic!("no Item mapping for {other:?} — add it to tests/steps/world.rs"),
    }
}

/// Resolves a Gherkin quest name (its English `i18n` title, or a short alias)
/// to the `QuestID`.
pub fn quest(name: &str) -> QuestID {
    match name {
        "The Digital Civilization" | "the ruins quest" => QuestID::ExploreRuins,
        "What the Ruins Kept" | "the search quest" => QuestID::OldCivilization,
        "Follow the Thread" | "the cord quest" => QuestID::CraftCord,
        "Trouble in the East" | "the axe quest" => QuestID::CraftAxe,
        "One Man's Trash" | "the umbrella quest" => QuestID::DisassembleUmbrella,
        "Stock Up for Hard Times" | "the stock-up quest" => QuestID::StockUp,
        other => panic!("no QuestID mapping for {other:?} — add it to tests/steps/world.rs"),
    }
}

/// Resolves a Gherkin tab/feature name to the `Unlocked` variant that gates
/// it.
pub fn feature(name: &str) -> Unlocked {
    match name {
        "Map" => Unlocked::Map,
        "Items" => Unlocked::Items,
        "Craft" => Unlocked::Craft,
        "Disassemble" => Unlocked::Disassemble,
        "Experiment" => Unlocked::Experiment,
        other => panic!("no Unlocked mapping for {other:?} — add it to tests/steps/world.rs"),
    }
}

/// A 3×3 fixed map: Meadow everywhere, the Village dead-centre (world origin,
/// where the player spawns) and a Ruins one tile south of it at world
/// `(0, -1)`. Deterministic geography for the quest scenarios — `Map::new`'s
/// mapgen output is random.
pub fn tiny_map() -> Map {
    let terrain = vec![vec![TerrainType::Meadow; 3]; 3];
    // pois[y][x]; tile (x, y) is world (x - 1, y - 1), so (1, 1) is the origin.
    let mut pois = vec![vec![None; 3]; 3];
    pois[1][1] = Some(Poi::Village);
    pois[0][1] = Some(Poi::Ruins); // world (0, -1)
    Map::from_terrain(terrain, pois)
}

/// Accepts `id` unless it's already the open quest.
fn accept_if_needed(game: &mut Game, id: QuestID) {
    if game.player.open_quest() != Some(id) {
        game.accept_quest(id)
            .unwrap_or_else(|e| panic!("could not accept {id:?}: {e:?}"));
    }
}

/// Drives `id` to completion through public gameplay only — no
/// `restore_quest_state` shortcut. Swaps in [`tiny_map`] for deterministic
/// geography and leaves the player back at the village (world origin).
/// Idempotent and tolerant of the quest already being open.
pub fn complete_quest(game: &mut Game, id: QuestID) {
    if game.player.completed_quests().contains(&id) {
        return;
    }
    match id {
        QuestID::ExploreRuins => {
            game.map = tiny_map();
            game.player.coordinates = (0, 0);
            accept_if_needed(game, QuestID::ExploreRuins);
            game.walk(Direction::South); // onto the Ruins at (0, -1)
        }
        QuestID::OldCivilization => {
            complete_quest(game, QuestID::ExploreRuins);
            game.player.coordinates = (0, -1); // the Ruins tile
            accept_if_needed(game, QuestID::OldCivilization);
            game.map.guarantee_find((0, -1), Item::CopperWire);
            game.search();
        }
        QuestID::CraftCord => {
            complete_quest(game, QuestID::OldCivilization);
            game.player.coordinates = (0, 0); // the village
            accept_if_needed(game, QuestID::CraftCord);
            game.player.inventory.insert(Item::Vine, 2);
            game.experiment(&[(Item::Vine, 2)]);
        }
        QuestID::CraftAxe => {
            complete_quest(game, QuestID::CraftCord);
            game.player.coordinates = (0, 0);
            accept_if_needed(game, QuestID::CraftAxe);
            for (it, n) in [(Item::Branch, 1), (Item::Stone, 1), (Item::Cord, 1)] {
                game.player.inventory.insert(it, n);
            }
            game.experiment(&[(Item::Branch, 1), (Item::Stone, 1), (Item::Cord, 1)]);
        }
        QuestID::DisassembleUmbrella => {
            complete_quest(game, QuestID::CraftAxe);
            game.player.coordinates = (0, 0); // the village
            accept_if_needed(game, QuestID::DisassembleUmbrella);
            game.player.inventory.insert(Item::Umbrella, 1);
            game.disassemble(Item::Umbrella);
        }
        QuestID::StockUp => {
            panic!("StockUp completion needs RNG-free hunting — not drivable from a step")
        }
    }
    game.player.coordinates = (0, 0);
    assert!(
        game.player.completed_quests().contains(&id),
        "failed to drive {id:?} to completion"
    );
}
