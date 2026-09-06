mod event;
mod item;
mod map;
mod player;
mod quest;
mod recipe;

pub use event::{Event, EventKind};
pub use item::Item;
pub use map::{Direction, FoundIn, Map, MapTile, Poi, TerrainType};
pub use player::Player;
pub use quest::{Quest, QuestError, QuestID};
pub(crate) use recipe::disassembly_for;

use quest::{EventTypeID, QUESTS, dependencies_met, quest_for};
use recipe::find_matching;
use std::io;
use std::time::{Duration, Instant};

/// Maps a live game type to its on-disk save shape (the DTO types live in
/// `save.rs`, colocating the mapping with the type it applies to keeps each
/// struct's own fields and its save-shape mapping from drifting apart).
pub(crate) trait SaveState {
    type Saved;
    fn save_state(&self) -> Self::Saved;
}

/// Rebuilds a live game type from its on-disk save shape.
pub(crate) trait RestoreState: Sized {
    type Saved;
    fn restore_state(saved: Self::Saved) -> io::Result<Self>;
}

#[derive(Debug)]
pub struct Game {
    pub player: Player,
    pub map: Map,
    events: Vec<Event>,
    started: Instant,
}

impl Default for Game {
    fn default() -> Self {
        let player = Player::default();
        let map = Map::new(&player);
        Self {
            player,
            map,
            events: vec![Event::new(EventKind::Awoke, Duration::ZERO)],
            started: Instant::now(),
        }
    }
}

impl Game {
    /// Rebuilds a game from saved parts. `started` is placed in the past so that
    /// events logged after the load stay ordered after the restored ones.
    pub(crate) fn from_saved(player: Player, map: Map, events: Vec<Event>) -> Game {
        let latest = events
            .iter()
            .map(Event::elapsed)
            .max()
            .unwrap_or(Duration::ZERO);
        let started = Instant::now()
            .checked_sub(latest)
            .unwrap_or_else(Instant::now);

        Game {
            player,
            map,
            events,
            started,
        }
    }

    /// The event log, oldest first.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Accepts `id` as the player's open quest. Fails if another quest is
    /// already open, `id` was already completed, or its dependencies aren't
    /// all in `Player::completed_quests()` yet.
    pub fn accept_quest(&mut self, id: QuestID) -> Result<(), QuestError> {
        if self.player.open_quest().is_some() {
            return Err(QuestError::AnotherQuestActive);
        }
        if self.player.completed_quests().contains(&id) {
            return Err(QuestError::AlreadyCompleted);
        }
        let quest = quest_for(id);
        if !dependencies_met(quest, self.player.completed_quests()) {
            return Err(QuestError::DependenciesNotMet);
        }

        self.player.open_quest_as(id);
        self.log(EventKind::QuestAccepted { quest: id });
        Ok(())
    }

    /// Quests not currently open, not yet completed, whose dependencies are
    /// all satisfied.
    pub fn available_quests(&self) -> Vec<&'static Quest> {
        QUESTS
            .iter()
            .filter(|quest| self.player.open_quest() != Some(quest.id))
            .filter(|quest| !self.player.completed_quests().contains(&quest.id))
            .filter(|quest| dependencies_met(quest, self.player.completed_quests()))
            .collect()
    }

    /// Looks up a quest by id, e.g. to show the name/description behind
    /// `Player::open_quest()`/`completed_quests()`.
    pub fn quest(&self, id: QuestID) -> &'static Quest {
        quest_for(id)
    }

    /// `(quests completed, quests that exist)`.
    pub fn quest_progress(&self) -> (usize, usize) {
        (self.player.completed_quests().len(), QUESTS.len())
    }

    /// Records that `item` was produced through crafting or experimenting,
    /// and reports it toward the open quest's condition. `search()` (found in
    /// the wild) and `disassemble()` (recovered) do not go through this.
    fn grant_item(&mut self, item: Item, amount: u32) {
        self.player.add_to_inventory(item, amount);
        self.note_quest_event(EventTypeID::CraftItem(item));
    }

    /// Routes a game action to the open quest's condition, advancing
    /// `Player::quest_progress` and completing the quest once it's met.
    fn note_quest_event(&mut self, event: EventTypeID) {
        let Some(quest_id) = self.player.open_quest() else {
            return;
        };
        let quest = quest_for(quest_id);
        if !quest.condition.matches(event) {
            return;
        }

        let progress = self.player.advance_quest_progress();
        if quest.condition.is_satisfied_by(progress) {
            self.complete_open_quest(quest);
        }
    }

    /// Completes `quest`: grants its reward, moves it into
    /// `Player::completed_quests()`, and clears `open_quest`/`quest_progress`.
    fn complete_open_quest(&mut self, quest: &'static Quest) {
        self.player.complete_quest(quest.id);
        self.player
            .grant_reward(quest.reward_xp, quest.reward_items);
        self.log(EventKind::QuestCompleted { quest: quest.id });
    }

    /// Appends an event to the log, timestamped with the current session
    /// elapsed time.
    fn log(&mut self, kind: EventKind) {
        self.events.push(Event::new(kind, self.started.elapsed()));
    }

    pub fn walk(&mut self, dir: Direction) {
        let next = self.player.coordinates_after(dir);
        if !self.map.contains(next) {
            return;
        }

        self.player.coordinates = next;
        if let Some(tile) = self.map.get_tile(next) {
            let (terrain, poi) = (tile.terrain_type, tile.poi);
            self.note_quest_event(EventTypeID::VisitTerrain(terrain));
            if let Some(poi) = poi {
                self.note_quest_event(EventTypeID::VisitPoi(poi));
            }
        }
    }

    pub fn search(&mut self) {
        let coords = self.player.coordinates;
        let Some(tile) = self.map.get_tile(coords) else {
            return;
        };
        let found = tile.roll_found_items(&mut rand::rng());

        for (item, source) in found {
            self.player.add_to_inventory(item, 1);
            self.log(EventKind::Found { item, source });
        }
        self.map.update_tile_last_search_time(coords);
    }

    /// Hunts the player's current tile: needs a Wooden Bow held (kept) and
    /// spends one Arrow, then rolls the tile's game. Like `search` there's no
    /// terrain gate — a tile with no fauna just comes back empty. Counts as one
    /// hunt toward an open quest whether or not it brought anything back.
    pub fn hunt(&mut self) {
        let coords = self.player.coordinates;
        if self.map.get_tile(coords).is_none() {
            return;
        }
        if !self.player.has_item(Item::WoodenBow) {
            self.log(EventKind::HuntUnprepared {
                missing: Item::WoodenBow,
            });
            return;
        }
        if !self.player.has_item(Item::Arrow) {
            self.log(EventKind::HuntUnprepared {
                missing: Item::Arrow,
            });
            return;
        }
        self.player.spend(Item::Arrow, 1);

        let bag = self
            .map
            .get_tile(coords)
            .expect("checked above")
            .roll_hunted_items(&mut rand::rng());
        if bag.is_empty() {
            self.log(EventKind::HuntMissed);
        } else {
            for &item in &bag {
                self.player.add_to_inventory(item, 1);
            }
            self.log(EventKind::Hunted {
                items: bag.iter().map(|&item| (item, 1)).collect(),
            });
        }
        self.map.update_tile_last_hunt_time(coords);
        self.note_quest_event(EventTypeID::Hunt);
    }

    pub fn craft(&mut self, recipe_name: &str) {
        let Some(recipe) = self.player.find_known_recipe(recipe_name) else {
            self.log(EventKind::UnknownRecipe {
                recipe: recipe_name.to_string(),
            });
            return;
        };

        if let Some((item, ..)) = self.player.first_shortage(recipe.consumables()) {
            self.log(EventKind::CraftShortage {
                needed: item,
                output: recipe.output(),
            });
            return;
        }

        if let Some(tool) = self.player.first_missing_tool(recipe.tools()) {
            self.log(EventKind::CraftMissingTool {
                tool,
                output: recipe.output(),
            });
            return;
        }

        self.player.spend_all(recipe.consumables());

        self.grant_item(recipe.output(), 1);
        self.log(EventKind::Crafted {
            output: recipe.output(),
        });

        self.player.record_successful_craft();
    }

    pub fn experiment(&mut self, items: &[(Item, u32)]) {
        if items.is_empty() {
            return;
        }

        if let Some((item, available, needed)) = self.player.first_shortage(items) {
            self.log(EventKind::ExperimentShortage {
                items: items.to_vec(),
                missing: item,
                available,
                needed,
            });
            return;
        }

        self.player.spend_all(items);

        let Some(recipe) = find_matching(items) else {
            self.log(EventKind::ExperimentFailed {
                items: items.to_vec(),
            });
            return;
        };

        if let Some(tool) = self.player.first_missing_tool(recipe.tools()) {
            // The combination was right, but a required tool isn't in hand —
            // the recipe isn't learned and nothing is produced (the
            // consumables are already spent, like any failed experiment).
            self.log(EventKind::CraftMissingTool {
                tool,
                output: recipe.output(),
            });
            return;
        }

        let newly_learned = self.player.learn_recipe(*recipe);

        self.grant_item(recipe.output(), 1);

        self.log(EventKind::Experimented {
            items: items.to_vec(),
            output: recipe.output(),
            newly_learned,
        });
    }

    /// Takes one `item` apart, returning the consumables of the recipe it
    /// decomposes into. Does nothing if no recipe lets `item` be taken apart,
    /// or the player is not carrying one.
    pub fn disassemble(&mut self, item: Item) {
        let Some(recipe) = disassembly_for(item) else {
            return;
        };
        if !self.player.has_item(item) {
            return;
        }

        self.player.spend(item, 1);
        self.player.add_all_to_inventory(recipe.consumables());

        self.log(EventKind::Disassembled { item });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quest::QuestCondition;
    use recipe::{RECIPES, Recipe};
    use std::collections::HashMap;

    #[test]
    fn walk_moves_and_respects_boundary() {
        for (dir, delta) in [
            (Direction::West, (-1, 0)),
            (Direction::East, (1, 0)),
            (Direction::North, (0, 1)),
            (Direction::South, (0, -1)),
        ] {
            let mut game = Game::default();
            let (x, y) = game.player.coordinates;
            game.walk(dir);
            assert_eq!(game.player.coordinates, (x + delta.0, y + delta.1));

            // at the boundary the move is refused
            let h = game.map.half;
            game.player.coordinates = (delta.0 * h, delta.1 * h);
            game.walk(dir);
            assert_eq!(game.player.coordinates, (delta.0 * h, delta.1 * h));
        }
    }

    #[test]
    fn walk_does_not_log() {
        let mut game = Game::default();
        let before = game.events().len();
        game.walk(Direction::North);
        game.walk(Direction::East);
        assert_eq!(game.events().len(), before);
    }

    #[test]
    fn test_y_axis_inversion() {
        let mut game = Game::default();

        game.player.coordinates = (0, 0);
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);

        game.map.tiles[ty - 1][tx] = MapTile::with_terrain(TerrainType::Meadow);
        game.map.tiles[ty + 1][tx] = MapTile::with_terrain(TerrainType::Forest);

        game.walk(Direction::North);
        assert_eq!(game.player.coordinates, (0, 1));
        assert_eq!(
            game.map
                .get_tile(game.player.coordinates)
                .unwrap()
                .terrain_type,
            TerrainType::Forest
        );
    }

    fn last_event(game: &Game) -> &Event {
        game.events().last().unwrap()
    }

    #[test]
    fn experiment_logs_are_precise() {
        // shortage: shows have / need
        let mut game = Game::default();
        game.player.inventory.insert(Item::Stone, 1);
        game.experiment(&[(Item::Stone, 5)]);
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::ExperimentShortage {
                items: vec![(Item::Stone, 5)],
                missing: Item::Stone,
                available: 1,
                needed: 5,
            }
        );

        // failure: shows the items tried
        let mut game = Game::default();
        game.player.inventory.insert(Item::Stick, 1);
        game.player.inventory.insert(Item::Vine, 1);
        game.experiment(&[(Item::Vine, 1), (Item::Stick, 1)]);
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::ExperimentFailed {
                items: vec![(Item::Vine, 1), (Item::Stick, 1)],
            }
        );

        // success + discovery, then success without
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 4);
        game.experiment(&[(Item::Vine, 2)]);
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::Experimented {
                items: vec![(Item::Vine, 2)],
                output: Item::Cord,
                newly_learned: true,
            }
        );
        game.experiment(&[(Item::Vine, 2)]);
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::Experimented {
                items: vec![(Item::Vine, 2)],
                output: Item::Cord,
                newly_learned: false,
            }
        );
    }

    #[test]
    fn empty_experiment_does_nothing() {
        let mut game = Game::default();
        let before = game.events().len();
        game.experiment(&[]);
        assert_eq!(game.events().len(), before);
    }

    #[test]
    fn crafting_removes_exhausted_consumables() {
        let mut game = Game::default();
        game.player.grant_recipe("Cord");
        game.player.inventory.insert(Item::Vine, 2); // exactly one Cord

        game.craft("Cord");

        assert_eq!(game.player.inventory.get(&Item::Vine), None);
        assert_eq!(game.player.inventory.get(&Item::Cord), Some(&1));
    }

    #[test]
    fn failed_craft_does_not_insert_zero_entries() {
        let mut game = Game::default();
        game.player.grant_recipe("Stone Axe");

        game.craft("Stone Axe"); // empty inventory

        assert!(game.player.inventory.is_empty());
    }

    #[test]
    fn experiment_removes_exhausted_inputs() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 2);

        game.experiment(&[(Item::Vine, 2)]);

        assert_eq!(game.player.inventory.get(&Item::Vine), None);
    }

    #[test]
    fn craft_without_the_required_tool_is_blocked() {
        let mut game = Game::default();
        game.player.grant_recipe("Wooden Bow");
        game.player.inventory.insert(Item::Stick, 1);
        game.player.inventory.insert(Item::Cord, 1);

        game.craft("Wooden Bow"); // needs a Stone Axe, not holding one

        assert_eq!(
            last_event(&game).kind(),
            &EventKind::CraftMissingTool {
                tool: Item::StoneAxe,
                output: Item::WoodenBow,
            }
        );
        assert_eq!(game.player.inventory.get(&Item::WoodenBow), None);
        // craft checks before spending — the consumables are untouched
        assert_eq!(game.player.inventory.get(&Item::Stick), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::Cord), Some(&1));
    }

    #[test]
    fn craft_does_not_consume_the_tool() {
        let mut game = Game::default();
        game.player.grant_recipe("Wooden Bow");
        game.player.inventory.insert(Item::Stick, 1);
        game.player.inventory.insert(Item::Cord, 1);
        game.player.inventory.insert(Item::StoneAxe, 1);

        game.craft("Wooden Bow");

        assert_eq!(game.player.inventory.get(&Item::WoodenBow), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::Stick), None);
        assert_eq!(game.player.inventory.get(&Item::Cord), None);
        assert_eq!(game.player.inventory.get(&Item::StoneAxe), Some(&1));
    }

    #[test]
    fn experiment_matching_a_recipe_without_its_tool_fails() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Stick, 1);
        game.player.inventory.insert(Item::Cord, 1);

        game.experiment(&[(Item::Stick, 1), (Item::Cord, 1)]); // Wooden Bow, no axe

        assert_eq!(
            last_event(&game).kind(),
            &EventKind::CraftMissingTool {
                tool: Item::StoneAxe,
                output: Item::WoodenBow,
            }
        );
        assert_eq!(game.player.inventory.get(&Item::WoodenBow), None);
        // a failed experiment still spends the combination
        assert_eq!(game.player.inventory.get(&Item::Stick), None);
        assert_eq!(game.player.inventory.get(&Item::Cord), None);
        assert!(
            game.player
                .known_recipes()
                .iter()
                .all(|r| r.output() != Item::WoodenBow)
        );
    }

    #[test]
    fn experiment_with_the_tool_present_discovers_and_builds() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Stick, 1);
        game.player.inventory.insert(Item::Cord, 1);
        game.player.inventory.insert(Item::StoneAxe, 1);

        game.experiment(&[(Item::Stick, 1), (Item::Cord, 1)]);

        assert_eq!(game.player.inventory.get(&Item::WoodenBow), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::StoneAxe), Some(&1));
        assert!(
            game.player
                .known_recipes()
                .iter()
                .any(|r| r.output() == Item::WoodenBow)
        );
    }

    #[test]
    fn craft_logs_unknown_recipe_then_crafted() {
        let mut game = Game::default();
        game.craft("Cord"); // unknown recipe
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::UnknownRecipe {
                recipe: "Cord".to_string()
            }
        );

        game.player.grant_recipe("Cord");
        game.player.inventory.insert(Item::Vine, 2);
        game.craft("Cord");
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::Crafted { output: Item::Cord }
        );
    }

    #[test]
    fn known_recipes_starts_empty_and_grows_on_discovery() {
        let mut game = Game::default();
        assert!(game.player.known_recipes().is_empty());

        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]);

        let known: Vec<&str> = game
            .player
            .known_recipes()
            .iter()
            .map(Recipe::name)
            .collect();
        assert_eq!(known, ["Cord"]);
    }

    #[test]
    fn recipe_progress_reports_known_and_craftable_total() {
        let mut game = Game::default();
        let craftable = RECIPES.iter().filter(|r| r.craftable()).count();
        assert!(
            craftable < RECIPES.len(),
            "some recipes are disassemble-only"
        );
        assert_eq!(game.player.recipe_progress(), (0, craftable));

        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]);
        assert_eq!(game.player.recipe_progress().0, 1);
    }

    #[test]
    fn quest_progress_reports_completed_and_total() {
        let mut game = Game::default();
        assert_eq!(game.quest_progress(), (0, QUESTS.len()));

        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftAxe]);
        assert_eq!(game.quest_progress(), (1, QUESTS.len()));
    }

    #[test]
    fn experiment_discovery_grants_experience() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 4);

        game.experiment(&[(Item::Vine, 2)]);
        assert_eq!(game.player.experience, 10);

        game.experiment(&[(Item::Vine, 2)]); // already known -> no XP
        assert_eq!(game.player.experience, 10);
    }

    #[test]
    fn crafting_grants_one_xp_per_ten_successful_crafts() {
        let mut game = Game::default();
        game.player.grant_recipe("Cord");

        game.craft("Unknown"); // does not count
        game.craft("Cord"); // known but no Vine -> shortage, does not count
        assert_eq!(game.player.crafts_completed, 0);

        for _ in 0..10 {
            game.player.inventory.insert(Item::Vine, 2);
            game.craft("Cord");
        }
        assert_eq!(game.player.crafts_completed, 10);
        assert_eq!(game.player.experience, 1);
    }

    #[test]
    fn search_yields_terrain_and_poi_items_and_names_each_source() {
        let mut game = Game::default();
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);
        game.map.tiles[ty][tx] = MapTile {
            items: HashMap::from([
                (Item::Stick, (1.0, FoundIn::Terrain(TerrainType::Forest))),
                (Item::Stone, (1.0, FoundIn::Poi(Poi::Cave))),
            ]),
            ..MapTile::with_terrain_and_poi(TerrainType::Forest, Some(Poi::Cave))
        };
        let before = game.events().len();

        game.search();

        assert_eq!(game.player.inventory.get(&Item::Stick), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::Stone), Some(&1));
        assert_eq!(game.events().len(), before + 2);

        let sources: Vec<FoundIn> = game.events()[before..]
            .iter()
            .filter_map(|e| match e.kind() {
                EventKind::Found { source, .. } => Some(*source),
                _ => None,
            })
            .collect();
        assert!(sources.contains(&FoundIn::Terrain(TerrainType::Forest)));
        assert!(sources.contains(&FoundIn::Poi(Poi::Cave)));
    }

    /// Puts the player on a Meadow tile whose game is a sure thing (every
    /// probability forced to `1.0`) and hands them a bow, so a hunt's outcome
    /// turns only on whether they have an arrow.
    fn armed_on_a_meadow() -> Game {
        let mut game = Game::default();
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);
        let mut tile = MapTile::with_terrain(TerrainType::Meadow);
        for probability in tile.hunt_items.values_mut() {
            *probability = 1.0;
        }
        game.map.tiles[ty][tx] = tile;
        game.player.inventory.insert(Item::WoodenBow, 1);
        game
    }

    #[test]
    fn hunt_without_a_bow_logs_unprepared_and_spends_nothing() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Arrow, 3);

        game.hunt();

        assert_eq!(
            last_event(&game).kind(),
            &EventKind::HuntUnprepared {
                missing: Item::WoodenBow
            }
        );
        assert_eq!(game.player.inventory.get(&Item::Arrow), Some(&3));
    }

    #[test]
    fn hunt_without_arrows_logs_unprepared() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::WoodenBow, 1);

        game.hunt();

        assert_eq!(
            last_event(&game).kind(),
            &EventKind::HuntUnprepared {
                missing: Item::Arrow
            }
        );
    }

    #[test]
    fn a_successful_hunt_spends_one_arrow_keeps_the_bow_and_logs_the_haul() {
        let mut game = armed_on_a_meadow();
        game.player.inventory.insert(Item::Arrow, 2);

        game.hunt();

        assert_eq!(game.player.inventory.get(&Item::Arrow), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::WoodenBow), Some(&1));
        for item in [Item::Meat, Item::Hide, Item::Bone, Item::Fur] {
            assert_eq!(game.player.inventory.get(&item), Some(&1), "{item:?}");
        }
        assert!(matches!(last_event(&game).kind(), EventKind::Hunted { .. }));
    }

    #[test]
    fn a_hunt_that_catches_nothing_still_spends_the_arrow_and_logs_a_miss() {
        let mut game = Game::default();
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);
        game.map.tiles[ty][tx] = MapTile::with_terrain(TerrainType::Deadland);
        game.player.inventory.insert(Item::WoodenBow, 1);
        game.player.inventory.insert(Item::Arrow, 1);

        game.hunt();

        assert_eq!(game.player.inventory.get(&Item::Arrow), None);
        assert_eq!(last_event(&game).kind(), &EventKind::HuntMissed);
    }

    #[test]
    fn disassemble_returns_components_and_consumes_the_item() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::StoneAxe, 1);

        game.disassemble(Item::StoneAxe);

        assert_eq!(game.player.inventory.get(&Item::StoneAxe), None);
        assert_eq!(game.player.inventory.get(&Item::Stick), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::Stone), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::Cord), Some(&1));
    }

    #[test]
    fn disassemble_logs_the_item_taken_apart() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::StoneAxe, 1);

        game.disassemble(Item::StoneAxe);

        assert_eq!(
            last_event(&game).kind(),
            &EventKind::Disassembled {
                item: Item::StoneAxe
            }
        );
    }

    #[test]
    fn disassemble_ignores_craft_only_recipes() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Arrow, 1); // the Arrow recipe is craft-only
        let before = game.events().len();

        game.disassemble(Item::Arrow);

        assert_eq!(game.player.inventory.get(&Item::Arrow), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::Stick), None);
        assert_eq!(game.events().len(), before);
    }

    #[test]
    fn disassemble_without_the_item_does_nothing() {
        let mut game = Game::default();
        let before = game.events().len();

        game.disassemble(Item::StoneAxe);

        assert!(game.player.inventory.is_empty());
        assert_eq!(game.events().len(), before);
    }

    #[test]
    fn disassemble_stacks_components_onto_existing_entries() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::StoneAxe, 1);
        game.player.inventory.insert(Item::Stick, 2);

        game.disassemble(Item::StoneAxe);

        assert_eq!(game.player.inventory.get(&Item::Stick), Some(&3));
    }

    #[test]
    fn disassemble_a_scavenged_object_returns_its_parts() {
        // The Umbrella recipe is disassemble-only — you can take a found one
        // apart, and that is the only way to get Fabric and Pole.
        let mut game = Game::default();
        game.player.inventory.insert(Item::Umbrella, 1);

        game.disassemble(Item::Umbrella);

        assert_eq!(game.player.inventory.get(&Item::Umbrella), None);
        assert_eq!(game.player.inventory.get(&Item::Fabric), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::Pole), Some(&1));
    }

    #[test]
    fn a_disassemble_only_recipe_cannot_be_experimented_into_existence() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Battery, 1);
        game.player.inventory.insert(Item::Speaker, 1);

        game.experiment(&[(Item::Battery, 1), (Item::Speaker, 1)]);

        assert_eq!(
            last_event(&game).kind(),
            &EventKind::ExperimentFailed {
                items: vec![(Item::Battery, 1), (Item::Speaker, 1)],
            }
        );
        assert_eq!(game.player.inventory.get(&Item::ElectronicToy), None);
        assert!(
            game.player
                .known_recipes()
                .iter()
                .all(|r| r.output() != Item::ElectronicToy)
        );
        // experiment still consumes the inputs, like any failed experiment
        assert!(game.player.inventory.is_empty());
    }

    #[test]
    fn events_are_timestamped_in_non_decreasing_order() {
        let mut game = Game::default();
        assert!(game.events()[0].elapsed() < Duration::from_secs(1));

        game.craft("Cord"); // unknown recipe -> one log line
        game.craft("Cord");

        let elapsed: Vec<Duration> = game.events().iter().map(Event::elapsed).collect();
        assert!(elapsed.len() >= 3);
        assert!(elapsed.windows(2).all(|w| w[0] <= w[1]));
    }

    // --- Quest system -----------------------------------------------------

    const FIXTURE_QUEST: Quest = Quest {
        id: QuestID::CraftAxe,
        dependencies: &[QuestID::ExploreRuins],
        condition: QuestCondition {
            event: EventTypeID::CraftItem(Item::StoneAxe),
            count: 1,
        },
        reward_xp: 7,
        reward_items: &[(Item::Cord, 2)],
    };

    /// A game with `ExploreRuins` already completed, so the axe quest — which
    /// depends on it — can be accepted.
    fn game_with_the_axe_quest_unlocked() -> Game {
        let mut game = Game::default();
        game.player
            .restore_quest_state(None, 0, vec![QuestID::ExploreRuins]);
        game
    }

    #[test]
    fn complete_open_quest_grants_reward_and_resets_quest_state() {
        let mut game = Game::default();
        game.player
            .restore_quest_state(Some(FIXTURE_QUEST.id), 1, vec![]);
        let xp_before = game.player.experience;

        game.complete_open_quest(&FIXTURE_QUEST);

        assert_eq!(game.player.open_quest(), None);
        assert_eq!(game.player.quest_progress(), 0);
        assert_eq!(game.player.completed_quests(), [FIXTURE_QUEST.id]);
        assert_eq!(game.player.experience, xp_before + FIXTURE_QUEST.reward_xp);
        assert_eq!(game.player.inventory.get(&Item::Cord), Some(&2));
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::QuestCompleted {
                quest: FIXTURE_QUEST.id
            }
        );
    }

    #[test]
    fn accept_quest_succeeds_for_an_available_quest() {
        let mut game = Game::default();
        assert_eq!(game.accept_quest(QuestID::ExploreRuins), Ok(()));
        assert_eq!(game.player.open_quest(), Some(QuestID::ExploreRuins));
        assert_eq!(game.player.quest_progress(), 0);
    }

    #[test]
    fn accept_quest_rejects_a_second_concurrent_quest() {
        let mut game = Game::default();
        game.accept_quest(QuestID::ExploreRuins).unwrap();
        assert_eq!(
            game.accept_quest(QuestID::CraftAxe),
            Err(QuestError::AnotherQuestActive)
        );
    }

    #[test]
    fn accept_quest_rejects_an_already_completed_quest() {
        let mut game = Game::default();
        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftAxe]);
        assert_eq!(
            game.accept_quest(QuestID::CraftAxe),
            Err(QuestError::AlreadyCompleted)
        );
    }

    #[test]
    fn accept_quest_rejects_a_quest_with_unmet_dependencies() {
        let mut game = Game::default();
        assert_eq!(
            game.accept_quest(QuestID::CraftAxe),
            Err(QuestError::DependenciesNotMet)
        );
    }

    #[test]
    fn available_quests_tracks_open_completed_and_dependencies() {
        let ids = |game: &Game| -> Vec<QuestID> {
            game.available_quests().iter().map(|q| q.id).collect()
        };

        let mut game = Game::default();
        // only the dependency-free quest is available at the start
        assert_eq!(ids(&game), [QuestID::ExploreRuins]);

        game.accept_quest(QuestID::ExploreRuins).unwrap();
        assert!(ids(&game).is_empty(), "the open quest drops off the list");

        game.player
            .restore_quest_state(None, 0, vec![QuestID::ExploreRuins]);
        // completing it takes it off the list and unlocks its dependent
        assert_eq!(ids(&game), [QuestID::CraftAxe]);
    }

    #[test]
    fn crafting_the_target_item_completes_the_quest() {
        let mut game = game_with_the_axe_quest_unlocked();
        game.player.grant_recipe("Stone Axe");
        game.player.inventory.insert(Item::Stick, 1);
        game.player.inventory.insert(Item::Stone, 1);
        game.player.inventory.insert(Item::Cord, 1);
        game.accept_quest(QuestID::CraftAxe).unwrap();

        game.craft("Stone Axe");

        assert_eq!(game.player.open_quest(), None);
        assert_eq!(
            game.player.completed_quests(),
            [QuestID::ExploreRuins, QuestID::CraftAxe]
        );
        assert_eq!(game.player.experience, 20);
    }

    #[test]
    fn crafting_a_different_item_does_not_advance_quest_progress() {
        let mut game = game_with_the_axe_quest_unlocked();
        game.player.grant_recipe("Cord");
        game.player.inventory.insert(Item::Vine, 2);
        game.accept_quest(QuestID::CraftAxe).unwrap();

        game.craft("Cord");

        assert_eq!(game.player.quest_progress(), 0);
        assert_eq!(game.player.open_quest(), Some(QuestID::CraftAxe));
    }

    #[test]
    fn experimenting_the_target_item_also_counts_toward_the_quest() {
        let mut game = game_with_the_axe_quest_unlocked();
        game.player.inventory.insert(Item::Stick, 1);
        game.player.inventory.insert(Item::Stone, 1);
        game.player.inventory.insert(Item::Cord, 1);
        game.accept_quest(QuestID::CraftAxe).unwrap();

        game.experiment(&[(Item::Stick, 1), (Item::Stone, 1), (Item::Cord, 1)]);

        // experimenting the target item advances the quest, same as crafting it
        assert!(game.player.completed_quests().contains(&QuestID::CraftAxe));
    }

    #[test]
    fn stock_up_needs_trouble_in_the_east_first() {
        let mut game = Game::default();
        assert_eq!(
            game.accept_quest(QuestID::StockUp),
            Err(QuestError::DependenciesNotMet)
        );
    }

    #[test]
    fn hunting_five_times_completes_the_stock_up_quest() {
        let mut game = Game::default();
        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftAxe]);
        game.accept_quest(QuestID::StockUp).unwrap();
        game.player.inventory.insert(Item::WoodenBow, 1);
        game.player.inventory.insert(Item::Arrow, 5);

        for _ in 0..5 {
            game.hunt();
        }

        assert_eq!(game.player.open_quest(), None);
        assert!(game.player.completed_quests().contains(&QuestID::StockUp));
        assert_eq!(game.player.experience, 30);
    }

    #[test]
    fn hunts_short_of_the_goal_leave_the_stock_up_quest_open() {
        let mut game = Game::default();
        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftAxe]);
        game.accept_quest(QuestID::StockUp).unwrap();
        game.player.inventory.insert(Item::WoodenBow, 1);
        game.player.inventory.insert(Item::Arrow, 3);

        for _ in 0..3 {
            game.hunt();
        }

        assert_eq!(game.player.open_quest(), Some(QuestID::StockUp));
        assert_eq!(game.player.quest_progress(), 3);
    }

    #[test]
    fn walking_onto_the_target_poi_completes_the_quest() {
        let mut game = Game::default();
        game.player.coordinates = (0, 0);
        let (tx, ty) = game.map.world_to_tile((0, 1));
        game.map.tiles[ty][tx].poi = Some(Poi::Ruins);
        game.accept_quest(QuestID::ExploreRuins).unwrap();
        let events_before = game.events().len();

        game.walk(Direction::North);

        assert_eq!(game.player.open_quest(), None);
        assert_eq!(game.player.completed_quests(), [QuestID::ExploreRuins]);
        // walking itself still logs nothing; only the completion line is new.
        assert_eq!(game.events().len(), events_before + 1);
    }

    #[test]
    fn walking_onto_a_tile_without_the_target_poi_does_not_advance_progress() {
        let mut game = Game::default();
        game.player.coordinates = (0, 0);
        let (tx, ty) = game.map.world_to_tile((0, 1));
        game.map.tiles[ty][tx].poi = None;
        game.accept_quest(QuestID::ExploreRuins).unwrap();

        game.walk(Direction::North);

        assert_eq!(game.player.quest_progress(), 0);
        assert_eq!(game.player.open_quest(), Some(QuestID::ExploreRuins));
    }

    #[test]
    fn walking_without_an_open_quest_does_nothing() {
        let mut game = Game::default();
        assert_eq!(game.player.open_quest(), None);
        game.walk(Direction::North);
        assert_eq!(game.player.quest_progress(), 0);
    }
}
