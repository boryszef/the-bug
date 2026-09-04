mod event;
mod item;
mod map;
mod player;
mod quest;
mod recipe;

pub use event::{Event, EventCategory};
pub use item::Item;
pub use map::{Direction, Map, MapTile, TerrainType};
pub use player::Player;
pub use quest::{Quest, QuestError, QuestID};
pub(crate) use recipe::reversible_recipe_for;

use quest::{EventTypeID, QUESTS, dependencies_met, quest_for};
use recipe::{describe_inputs, find_matching};
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
            events: vec![Event::new(
                EventCategory::General,
                "You wake up and decide to go for a walk.",
                Duration::ZERO,
            )],
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
        self.log(
            EventCategory::General,
            format!("Quest accepted: {}", quest.name),
        );
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
        self.log(
            EventCategory::General,
            format!("Quest complete: {}!", quest.name),
        );
    }

    /// Appends a message to the event log, timestamped with the current session
    /// elapsed time.
    fn log(&mut self, category: EventCategory, text: impl Into<String>) {
        self.events
            .push(Event::new(category, text, self.started.elapsed()));
    }

    pub fn walk(&mut self, dir: Direction) {
        let next = self.player.coordinates_after(dir);
        if !self.map.contains(next) {
            return;
        }

        self.player.coordinates = next;
        if let Some(tile) = self.map.get_tile(next) {
            self.note_quest_event(EventTypeID::VisitTerrain(tile.terrain_type));
        }
    }

    pub fn search(&mut self) {
        let coords = self.player.coordinates;
        let Some(tile) = self.map.get_tile(coords) else {
            return;
        };
        let terrain = tile.terrain_type;
        let found = tile.roll_found_items(&mut rand::rng());

        for item in found {
            self.player.add_to_inventory(item, 1);
            self.log(
                EventCategory::General,
                format!("You find a {item} in the {terrain}."),
            );
        }
        self.map.update_tile_last_search_time(coords);
    }

    pub fn craft(&mut self, recipe_name: &str) {
        let Some(recipe) = self.player.find_known_recipe(recipe_name) else {
            self.log(
                EventCategory::Crafting,
                format!("You don't know how to craft a {recipe_name}."),
            );
            return;
        };

        if let Some((item, ..)) = self.player.first_shortage(recipe.inputs()) {
            self.log(
                EventCategory::Crafting,
                format!("You don't have enough {item} to craft a {}.", recipe.name()),
            );
            return;
        }

        self.player.spend_all(recipe.inputs());

        self.grant_item(recipe.output(), 1);
        self.log(
            EventCategory::Crafting,
            format!("You craft a {}.", recipe.name()),
        );

        self.player.record_successful_craft();
    }

    pub fn experiment(&mut self, items: &[(Item, u32)]) {
        if items.is_empty() {
            return;
        }

        let inputs = describe_inputs(items);

        if let Some((item, available, amount)) = self.player.first_shortage(items) {
            self.log(
                EventCategory::Experiment,
                format!(
                    "Experiment: {inputs} → not enough {item} (have {available}, need {amount})"
                ),
            );
            return;
        }

        self.player.spend_all(items);

        let Some(recipe) = find_matching(items) else {
            self.log(
                EventCategory::Experiment,
                format!("Experiment: {inputs} → nothing"),
            );
            return;
        };

        let newly_learned = self.player.learn_recipe(*recipe);

        self.grant_item(recipe.output(), 1);

        let suffix = if newly_learned { " (new recipe!)" } else { "" };
        self.log(
            EventCategory::Experiment,
            format!("Experiment: {inputs} → {}{suffix}", recipe.name()),
        );
    }

    /// Takes one `item` apart, returning the inputs of the reversible recipe
    /// that makes it. Does nothing if no reversible recipe produces `item` or
    /// the player is not carrying one.
    pub fn disassemble(&mut self, item: Item) {
        let Some(recipe) = reversible_recipe_for(item) else {
            return;
        };
        if !self.player.has_item(item) {
            return;
        }

        self.player.spend(item, 1);
        self.player.add_all_to_inventory(recipe.inputs());

        self.log(
            EventCategory::Crafting,
            format!(
                "You take apart a {item}, recovering {}.",
                describe_inputs(recipe.inputs())
            ),
        );
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
            last_event(&game).text(),
            "Experiment: 5 Stone → not enough Stone (have 1, need 5)"
        );
        assert_eq!(last_event(&game).category(), EventCategory::Experiment);

        // failure: shows the inputs and "nothing"
        let mut game = Game::default();
        game.player.inventory.insert(Item::Stick, 1);
        game.player.inventory.insert(Item::Vine, 1);
        game.experiment(&[(Item::Vine, 1), (Item::Stick, 1)]);
        assert_eq!(
            last_event(&game).text(),
            "Experiment: 1 Stick + 1 Vine → nothing"
        );

        // success + discovery, then success without
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 4);
        game.experiment(&[(Item::Vine, 2)]);
        assert_eq!(
            last_event(&game).text(),
            "Experiment: 2 Vine → Cord (new recipe!)"
        );
        game.experiment(&[(Item::Vine, 2)]);
        assert_eq!(last_event(&game).text(), "Experiment: 2 Vine → Cord");
    }

    #[test]
    fn empty_experiment_does_nothing() {
        let mut game = Game::default();
        let before = game.events().len();
        game.experiment(&[]);
        assert_eq!(game.events().len(), before);
    }

    #[test]
    fn crafting_removes_exhausted_inputs() {
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
    fn craft_events_are_categorised_crafting() {
        let mut game = Game::default();
        game.craft("Cord"); // unknown recipe
        assert_eq!(last_event(&game).category(), EventCategory::Crafting);

        game.player.grant_recipe("Cord");
        game.player.inventory.insert(Item::Vine, 2);
        game.craft("Cord");
        assert_eq!(last_event(&game).text(), "You craft a Cord.");
        assert_eq!(last_event(&game).category(), EventCategory::Crafting);
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
    fn recipe_progress_reports_known_and_total() {
        let mut game = Game::default();
        assert_eq!(game.player.recipe_progress(), (0, RECIPES.len()));

        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]);
        assert_eq!(game.player.recipe_progress().0, 1);
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
    fn search_yields_every_item_a_tile_offers() {
        let mut game = Game::default();
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);
        game.map.tiles[ty][tx] = MapTile {
            terrain_type: TerrainType::Forest,
            items: HashMap::from([(Item::Stick, 1.0), (Item::Vine, 1.0)]),
            last_search_time: None,
        };
        let before = game.events().len();

        game.search();

        assert_eq!(game.player.inventory.get(&Item::Stick), Some(&1));
        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&1));
        assert_eq!(game.events().len(), before + 2);
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
    fn disassemble_logs_a_precise_crafting_line() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::StoneAxe, 1);

        game.disassemble(Item::StoneAxe);

        assert_eq!(
            last_event(&game).text(),
            "You take apart a Stone Axe, recovering 1 Stick + 1 Stone + 1 Cord."
        );
        assert_eq!(last_event(&game).category(), EventCategory::Crafting);
    }

    #[test]
    fn disassemble_ignores_irreversible_recipes() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Arrow, 1); // Arrow recipe is not reversible
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
        id: QuestID::CraftArrows,
        name: "Fixture",
        description: "",
        dependencies: &[QuestID::ExploreRuins],
        condition: QuestCondition {
            event: EventTypeID::CraftItem(Item::Arrow),
            count: 1,
        },
        reward_xp: 7,
        reward_items: &[(Item::Cord, 2)],
    };

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
        assert_eq!(last_event(&game).text(), "Quest complete: Fixture!");
    }

    #[test]
    fn accept_quest_succeeds_for_an_available_quest() {
        let mut game = Game::default();
        assert_eq!(game.accept_quest(QuestID::CraftArrows), Ok(()));
        assert_eq!(game.player.open_quest(), Some(QuestID::CraftArrows));
        assert_eq!(game.player.quest_progress(), 0);
    }

    #[test]
    fn accept_quest_rejects_a_second_concurrent_quest() {
        let mut game = Game::default();
        game.accept_quest(QuestID::CraftArrows).unwrap();
        assert_eq!(
            game.accept_quest(QuestID::ExploreRuins),
            Err(QuestError::AnotherQuestActive)
        );
    }

    #[test]
    fn accept_quest_rejects_an_already_completed_quest() {
        let mut game = Game::default();
        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftArrows]);
        assert_eq!(
            game.accept_quest(QuestID::CraftArrows),
            Err(QuestError::AlreadyCompleted)
        );
    }

    #[test]
    fn available_quests_excludes_the_open_and_completed_quests() {
        let mut game = Game::default();
        assert_eq!(game.available_quests().len(), QUESTS.len());

        game.accept_quest(QuestID::CraftArrows).unwrap();
        let available: Vec<QuestID> = game.available_quests().iter().map(|q| q.id).collect();
        assert!(!available.contains(&QuestID::CraftArrows));
    }

    #[test]
    fn crafting_the_target_item_enough_times_completes_the_quest() {
        let mut game = Game::default();
        game.player.grant_recipe("Arrow");
        game.player.inventory.insert(Item::Stick, 5);
        game.accept_quest(QuestID::CraftArrows).unwrap();

        for _ in 0..5 {
            game.craft("Arrow");
        }

        assert_eq!(game.player.open_quest(), None);
        assert_eq!(game.player.completed_quests(), [QuestID::CraftArrows]);
        assert_eq!(game.player.experience, 20);
    }

    #[test]
    fn crafting_a_different_item_does_not_advance_quest_progress() {
        let mut game = Game::default();
        game.player.grant_recipe("Cord");
        game.player.inventory.insert(Item::Vine, 2);
        game.accept_quest(QuestID::CraftArrows).unwrap();

        game.craft("Cord");

        assert_eq!(game.player.quest_progress(), 0);
        assert_eq!(game.player.open_quest(), Some(QuestID::CraftArrows));
    }

    #[test]
    fn experimenting_the_target_item_also_advances_quest_progress() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Stick, 1);
        game.accept_quest(QuestID::CraftArrows).unwrap();

        game.experiment(&[(Item::Stick, 1)]);

        assert_eq!(game.player.quest_progress(), 1);
    }

    #[test]
    fn walking_onto_the_target_terrain_completes_the_quest() {
        let mut game = Game::default();
        game.player.coordinates = (0, 0);
        let (tx, ty) = game.map.world_to_tile((0, 1));
        game.map.tiles[ty][tx] = MapTile::with_terrain(TerrainType::Ruins);
        game.accept_quest(QuestID::ExploreRuins).unwrap();
        let events_before = game.events().len();

        game.walk(Direction::North);

        assert_eq!(game.player.open_quest(), None);
        assert_eq!(game.player.completed_quests(), [QuestID::ExploreRuins]);
        // walking itself still logs nothing; only the completion line is new.
        assert_eq!(game.events().len(), events_before + 1);
    }

    #[test]
    fn walking_onto_non_matching_terrain_does_not_advance_progress() {
        let mut game = Game::default();
        game.player.coordinates = (0, 0);
        let (tx, ty) = game.map.world_to_tile((0, 1));
        game.map.tiles[ty][tx] = MapTile::with_terrain(TerrainType::Meadow);
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
