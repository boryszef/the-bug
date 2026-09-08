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

    /// Searches the player's current tile. Each item the roll turns up is
    /// added to the bag if there's room; one that doesn't fit is lost —
    /// logged as `BagFull` rather than `Found`, since nothing was actually
    /// gained (see docs/bag-and-storage.md).
    pub fn search(&mut self) {
        let coords = self.player.coordinates;
        let Some(tile) = self.map.get_tile(coords) else {
            return;
        };
        let found = tile.roll_found_items(&mut rand::rng());

        for (item, source) in found {
            if self.player.add_to_bag(item, 1) {
                self.log(EventKind::Found { item, source });
            } else {
                self.log(EventKind::BagFull { item });
            }
        }
        self.map.update_tile_last_search_time(coords);
    }

    /// Hunts the player's current tile: needs a Wooden Bow *and* an Arrow in
    /// the bag — gear must be carried to be used in the field, unlike a
    /// craft/experiment tool sitting in storage at the village — spends the
    /// Arrow, then rolls the tile's game. Like `search` there's no terrain
    /// gate — a tile with no fauna just comes back empty. Each hit is added
    /// to the bag if there's room; one that doesn't fit is lost, logged as
    /// `BagFull` rather than folded into the haul. A hunt that brings back
    /// at least one *kept* item counts toward an open quest; a wasted arrow,
    /// or a catch the bag had no room for, does not.
    pub fn hunt(&mut self) {
        let coords = self.player.coordinates;
        if self.map.get_tile(coords).is_none() {
            return;
        }
        if !self.player.has_item_in_bag(Item::WoodenBow) {
            self.log(EventKind::HuntUnprepared {
                missing: Item::WoodenBow,
            });
            return;
        }
        if !self.player.has_item_in_bag(Item::Arrow) {
            self.log(EventKind::HuntUnprepared {
                missing: Item::Arrow,
            });
            return;
        }
        self.player.spend_from_bag(Item::Arrow, 1);

        let catch = self
            .map
            .get_tile(coords)
            .expect("checked above")
            .roll_hunted_items(&mut rand::rng());
        if catch.is_empty() {
            self.log(EventKind::HuntMissed);
        } else {
            let mut gained = Vec::new();
            for &item in &catch {
                if self.player.add_to_bag(item, 1) {
                    gained.push(item);
                } else {
                    self.log(EventKind::BagFull { item });
                }
            }
            if !gained.is_empty() {
                self.log(EventKind::Hunted {
                    items: gained.iter().map(|&item| (item, 1)).collect(),
                });
                self.note_quest_event(EventTypeID::Hunt);
            }
        }
        self.map.update_tile_last_hunt_time(coords);
    }

    /// Whether the player is somewhere crafting, experimenting, and
    /// disassembling are allowed — the Village today; a future workshop POI
    /// extends this without the three methods below (or the gui, which
    /// disables their buttons using this same query) needing to change.
    pub fn at_craftable_location(&self) -> bool {
        matches!(
            self.map
                .get_tile(self.player.coordinates)
                .and_then(|tile| tile.poi),
            Some(Poi::Village)
        )
    }

    /// Crafts a known recipe by name. Consumables are drawn from storage
    /// first, the bag for any remainder (`docs/bag-and-storage.md`) — tools
    /// stay storage-only, a tool is kept at the workshop where it's used.
    /// Does nothing away from the Village (or a future workshop) — see
    /// [`at_craftable_location`](Self::at_craftable_location). Not logged:
    /// the gui disables the button so this is normally unreachable, and
    /// being away from the village is the player's own doing, not a result
    /// the game produced — see docs/village-crafting.md.
    pub fn craft(&mut self, recipe_name: &str) {
        if !self.at_craftable_location() {
            return;
        }

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

        self.player.spend_all_storage_then_bag(recipe.consumables());

        // Logged before `grant_item`, whose `note_quest_event` may log the
        // quest's own completion line right behind it — the event log reads
        // newest-first, so the craft has to land first or "Quest complete"
        // would appear to precede the craft that caused it.
        self.log(EventKind::Crafted {
            output: recipe.output(),
        });
        self.grant_item(recipe.output(), 1);

        self.player.record_successful_craft();
    }

    /// Tries the given items as an experiment — drawn from storage first,
    /// the bag for any remainder, same as `craft`. Does nothing away from
    /// the Village (or a future workshop), same as `craft` — see
    /// [`at_craftable_location`](Self::at_craftable_location).
    pub fn experiment(&mut self, items: &[(Item, u32)]) {
        if items.is_empty() {
            return;
        }

        if !self.at_craftable_location() {
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

        self.player.spend_all_storage_then_bag(items);

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

        // Logged before `grant_item`, same reasoning as in `craft` above.
        self.log(EventKind::Experimented {
            items: items.to_vec(),
            output: recipe.output(),
            newly_learned,
        });
        self.grant_item(recipe.output(), 1);
    }

    /// Takes one `item` apart, returning the consumables of the recipe it
    /// decomposes into. `item` itself may be held in storage or the bag
    /// (storage first); the recovered consumables always go to storage.
    /// Does nothing if no recipe lets `item` be taken apart, the player is
    /// not carrying one (in either pool), or they're away from the Village
    /// (or a future workshop) — see
    /// [`at_craftable_location`](Self::at_craftable_location).
    pub fn disassemble(&mut self, item: Item) {
        if !self.at_craftable_location() {
            return;
        }

        let Some(recipe) = disassembly_for(item) else {
            return;
        };
        if !self.player.has_item_combined(item) {
            return;
        }

        self.player.spend_storage_then_bag(item, 1);
        self.player.add_all_to_inventory(recipe.consumables());

        self.log(EventKind::Disassembled { item });
    }

    /// Moves `amount` of `item` from the bag to storage. Does nothing away
    /// from the Village (or a future workshop) — see
    /// [`at_craftable_location`](Self::at_craftable_location) — or if the
    /// bag doesn't hold `amount`. Not logged: a transfer is the player's own
    /// bookkeeping between two pools they already own, not a result the game
    /// produced — see docs/bag-and-storage.md.
    #[allow(dead_code)] // wired up by the gui's Items tab (next commit)
    pub fn transfer_to_storage(&mut self, item: Item, amount: u32) {
        if !self.at_craftable_location() {
            return;
        }
        self.player.transfer_to_storage(item, amount);
    }

    /// Moves `amount` of `item` from storage to the bag. Does nothing away
    /// from the Village, or if storage doesn't hold `amount`, or if the bag
    /// has no room for it. Not logged — see
    /// [`transfer_to_storage`](Self::transfer_to_storage).
    #[allow(dead_code)] // wired up by the gui's Items tab (next commit)
    pub fn transfer_to_bag(&mut self, item: Item, amount: u32) {
        if !self.at_craftable_location() {
            return;
        }
        self.player.transfer_to_bag(item, amount);
    }

    /// Drops `amount` of `item` from the bag — allowed anywhere. Does
    /// nothing if the bag doesn't hold `amount`.
    #[allow(dead_code)] // wired up by the gui's Items tab (next commit)
    pub fn drop_from_bag(&mut self, item: Item, amount: u32) {
        if self.player.bag_count(item) < amount {
            return;
        }
        self.player.spend_from_bag(item, amount);
        self.log(EventKind::Dropped { item });
    }

    /// Drops `amount` of `item` from storage. Does nothing away from the
    /// Village, or if storage doesn't hold `amount`.
    #[allow(dead_code)] // wired up by the gui's Items tab (next commit)
    pub fn drop_from_storage(&mut self, item: Item, amount: u32) {
        if !self.at_craftable_location() {
            return;
        }
        if self.player.inventory_count(item) < amount {
            return;
        }
        self.player.spend(item, amount);
        self.log(EventKind::Dropped { item });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quest::QuestCondition;
    use recipe::RECIPES;
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
        game.player.inventory.insert(Item::Branch, 1);
        game.player.inventory.insert(Item::Vine, 1);
        game.experiment(&[(Item::Vine, 1), (Item::Branch, 1)]);
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::ExperimentFailed {
                items: vec![(Item::Vine, 1), (Item::Branch, 1)],
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

    // The default player spawns at the world origin, which is always the
    // Village (`scatter_pois` fixes it there) — so every craft / experiment /
    // disassemble test in this file exercises the "at the village" path
    // implicitly. The *away*-from-the-village no-ops are covered by the
    // `craft`/`experiment`/`disassemble` Gherkin features (see
    // `docs/functional-tests.md`); this checks the query they gate on.

    #[test]
    fn at_craftable_location_is_true_at_the_default_spawn_and_false_away_from_it() {
        let mut game = Game::default();
        assert!(game.at_craftable_location());

        game.player.coordinates = (5, 5);
        assert!(!game.at_craftable_location());
    }

    // --- Bag and storage ---------------------------------------------------
    //
    // `Player.inventory` is the unlimited village Storage (unchanged in
    // role — every craft/experiment/disassemble/quest-reward test in this
    // file already exercises it). `Player.bag` is new: a capacity-limited
    // pool the player carries, tested directly here. See
    // docs/bag-and-storage.md.

    #[test]
    fn bag_total_sums_every_item_in_the_bag() {
        let mut game = Game::default();
        assert_eq!(game.player.bag_total(), 0);

        game.player.add_to_bag(Item::Branch, 3);
        game.player.add_to_bag(Item::Stone, 2);

        assert_eq!(game.player.bag_total(), 5);
    }

    #[test]
    fn add_to_bag_fits_within_capacity() {
        let mut game = Game::default();

        assert!(game.player.add_to_bag(Item::Branch, player::BAG_CAPACITY));

        assert_eq!(game.player.bag_count(Item::Branch), player::BAG_CAPACITY);
        assert_eq!(game.player.bag_total(), player::BAG_CAPACITY);
    }

    #[test]
    fn add_to_bag_rejects_an_amount_that_would_exceed_capacity() {
        let mut game = Game::default();
        game.player
            .add_to_bag(Item::Branch, player::BAG_CAPACITY - 1);

        // one more than fits: rejected entirely, not partially added
        assert!(!game.player.add_to_bag(Item::Stone, 2));

        assert_eq!(game.player.bag_count(Item::Stone), 0);
        assert_eq!(game.player.bag_total(), player::BAG_CAPACITY - 1);
    }

    #[test]
    fn has_item_in_bag_and_bag_count_reflect_the_bag_not_storage() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Branch, 5); // storage, not bag

        assert!(!game.player.has_item_in_bag(Item::Branch));
        assert_eq!(game.player.bag_count(Item::Branch), 0);

        game.player.add_to_bag(Item::Branch, 1);

        assert!(game.player.has_item_in_bag(Item::Branch));
        assert_eq!(game.player.bag_count(Item::Branch), 1);
    }

    #[test]
    fn transfer_to_storage_moves_items_from_bag_to_storage() {
        let mut game = Game::default();
        game.player.add_to_bag(Item::Vine, 5);

        assert!(game.player.transfer_to_storage(Item::Vine, 3));

        assert_eq!(game.player.bag_count(Item::Vine), 2);
        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&3));
    }

    #[test]
    fn transfer_to_storage_fails_without_enough_in_the_bag() {
        let mut game = Game::default();
        game.player.add_to_bag(Item::Vine, 1);

        assert!(!game.player.transfer_to_storage(Item::Vine, 2));

        assert_eq!(game.player.bag_count(Item::Vine), 1);
        assert!(game.player.inventory.is_empty());
    }

    #[test]
    fn transfer_to_bag_moves_items_from_storage_to_bag() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 5);

        assert!(game.player.transfer_to_bag(Item::Vine, 3));

        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&2));
        assert_eq!(game.player.bag_count(Item::Vine), 3);
    }

    #[test]
    fn transfer_to_bag_fails_without_enough_in_storage() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 1);

        assert!(!game.player.transfer_to_bag(Item::Vine, 2));

        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&1));
        assert!(game.player.bag.is_empty());
    }

    #[test]
    fn transfer_to_bag_fails_when_the_bag_has_no_room_and_leaves_storage_untouched() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 5);
        game.player.add_to_bag(Item::Branch, player::BAG_CAPACITY); // bag full

        assert!(!game.player.transfer_to_bag(Item::Vine, 1));

        // storage untouched — nothing left in limbo
        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&5));
        assert_eq!(game.player.bag_count(Item::Vine), 0);
    }

    #[test]
    fn spend_from_bag_removes_exhausted_entries() {
        let mut game = Game::default();
        game.player.add_to_bag(Item::Vine, 2);

        game.player.spend_from_bag(Item::Vine, 2);

        assert_eq!(game.player.bag.get(&Item::Vine), None);
    }

    #[test]
    fn spend_storage_then_bag_draws_storage_first() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 3);
        game.player.add_to_bag(Item::Vine, 5);

        game.player.spend_storage_then_bag(Item::Vine, 3);

        assert_eq!(game.player.inventory.get(&Item::Vine), None);
        assert_eq!(game.player.bag_count(Item::Vine), 5); // untouched
    }

    #[test]
    fn spend_storage_then_bag_spills_the_remainder_into_the_bag() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 2);
        game.player.add_to_bag(Item::Vine, 5);

        game.player.spend_storage_then_bag(Item::Vine, 4);

        assert_eq!(game.player.inventory.get(&Item::Vine), None);
        assert_eq!(game.player.bag_count(Item::Vine), 3); // 5 - (4 - 2)
    }

    #[test]
    fn combined_count_and_has_item_combined_sum_both_pools() {
        let mut game = Game::default();
        assert!(!game.player.has_item_combined(Item::Vine));

        game.player.inventory.insert(Item::Vine, 2);
        game.player.add_to_bag(Item::Vine, 3);

        assert_eq!(game.player.combined_count(Item::Vine), 5);
        assert!(game.player.has_item_combined(Item::Vine));
    }

    // --- Game-level transfer and drop --------------------------------------
    //
    // `Player::transfer_to_storage`/`transfer_to_bag` are already tested for
    // correctness above; these exercise `Game`'s thin wrappers — the
    // village gate on transfers and drop-from-storage, and that
    // drop-from-bag has none.

    #[test]
    fn game_transfer_to_storage_requires_the_village() {
        let mut game = Game::default();
        game.player.add_to_bag(Item::Vine, 3);
        game.player.coordinates = (5, 5);

        game.transfer_to_storage(Item::Vine, 3);

        assert_eq!(game.player.bag_count(Item::Vine), 3);
        assert!(game.player.inventory.is_empty());
    }

    #[test]
    fn game_transfer_to_storage_moves_items_at_the_village() {
        let mut game = Game::default();
        game.player.add_to_bag(Item::Vine, 3);

        game.transfer_to_storage(Item::Vine, 3);

        assert_eq!(game.player.bag_count(Item::Vine), 0);
        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&3));
    }

    #[test]
    fn game_transfer_to_bag_requires_the_village() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 3);
        game.player.coordinates = (5, 5);

        game.transfer_to_bag(Item::Vine, 3);

        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&3));
        assert!(game.player.bag.is_empty());
    }

    #[test]
    fn game_transfer_to_bag_moves_items_at_the_village() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 3);

        game.transfer_to_bag(Item::Vine, 3);

        assert_eq!(game.player.inventory.get(&Item::Vine), None);
        assert_eq!(game.player.bag_count(Item::Vine), 3);
    }

    #[test]
    fn drop_from_bag_works_anywhere() {
        let mut game = Game::default();
        game.player.add_to_bag(Item::Vine, 3);
        game.player.coordinates = (5, 5); // away from the village

        game.drop_from_bag(Item::Vine, 1);

        assert_eq!(game.player.bag_count(Item::Vine), 2);
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::Dropped { item: Item::Vine }
        );
    }

    #[test]
    fn drop_from_bag_without_enough_does_nothing() {
        let mut game = Game::default();
        let before = game.events().len();

        game.drop_from_bag(Item::Vine, 1);

        assert_eq!(game.events().len(), before);
    }

    #[test]
    fn drop_from_storage_requires_the_village() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 3);
        game.player.coordinates = (5, 5);
        let before = game.events().len();

        game.drop_from_storage(Item::Vine, 1);

        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&3));
        assert_eq!(game.events().len(), before);
    }

    #[test]
    fn drop_from_storage_removes_one_unit_and_logs_at_the_village() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 3);

        game.drop_from_storage(Item::Vine, 1);

        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&2));
        assert_eq!(
            last_event(&game).kind(),
            &EventKind::Dropped { item: Item::Vine }
        );
    }

    #[test]
    fn failed_craft_does_not_insert_zero_entries() {
        let mut game = Game::default();
        game.player.grant_recipe("Stone Axe");

        game.craft("Stone Axe"); // empty inventory

        assert!(game.player.inventory.is_empty());
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
                (Item::Branch, (1.0, FoundIn::Terrain(TerrainType::Forest))),
                (Item::Stone, (1.0, FoundIn::Poi(Poi::Cave))),
            ]),
            ..MapTile::with_terrain_and_poi(TerrainType::Forest, Some(Poi::Cave))
        };
        let before = game.events().len();

        game.search();

        // found items go into the bag, not storage
        assert_eq!(game.player.bag.get(&Item::Branch), Some(&1));
        assert_eq!(game.player.bag.get(&Item::Stone), Some(&1));
        assert!(game.player.inventory.is_empty());
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

    #[test]
    fn a_full_bag_loses_the_search_find_and_logs_bag_full_instead() {
        let mut game = Game::default();
        game.player.bag.insert(Item::Stone, player::BAG_CAPACITY); // no room left
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);
        let mut tile = MapTile::with_terrain(TerrainType::Forest); // yields Branch
        for (probability, _) in tile.items.values_mut() {
            *probability = 1.0; // a sure find, not a coin flip
        }
        game.map.tiles[ty][tx] = tile;

        game.search();

        assert_eq!(
            last_event(&game).kind(),
            &EventKind::BagFull { item: Item::Branch }
        );
        assert_eq!(game.player.bag.get(&Item::Branch), None);
    }

    /// Puts the player on a Meadow tile whose game is a sure thing (every
    /// probability forced to `1.0`) and hands them a bow *in the bag*, so a
    /// hunt's outcome turns only on whether they carry an arrow too.
    fn armed_on_a_meadow() -> Game {
        let mut game = Game::default();
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);
        let mut tile = MapTile::with_terrain(TerrainType::Meadow);
        for probability in tile.hunt_items.values_mut() {
            *probability = 1.0;
        }
        game.map.tiles[ty][tx] = tile;
        game.player.bag.insert(Item::WoodenBow, 1);
        game
    }

    #[test]
    fn a_successful_hunt_spends_one_arrow_keeps_the_bow_and_logs_the_haul() {
        let mut game = armed_on_a_meadow();
        game.player.bag.insert(Item::Arrow, 2);

        game.hunt();

        assert_eq!(game.player.bag.get(&Item::Arrow), Some(&1));
        assert_eq!(game.player.bag.get(&Item::WoodenBow), Some(&1));
        for item in [Item::Meat, Item::Hide, Item::Bone, Item::Fur] {
            assert_eq!(game.player.bag.get(&item), Some(&1), "{item:?}");
        }
        assert!(matches!(last_event(&game).kind(), EventKind::Hunted { .. }));
    }

    #[test]
    fn a_hunt_that_catches_nothing_still_spends_the_arrow_and_logs_a_miss() {
        let mut game = Game::default();
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);
        game.map.tiles[ty][tx] = MapTile::with_terrain(TerrainType::Deadland);
        game.player.bag.insert(Item::WoodenBow, 1);
        game.player.bag.insert(Item::Arrow, 1);

        game.hunt();

        assert_eq!(game.player.bag.get(&Item::Arrow), None);
        assert_eq!(last_event(&game).kind(), &EventKind::HuntMissed);
    }

    #[test]
    fn a_hunt_that_only_partly_fits_logs_bag_full_and_hunted_and_still_counts() {
        let mut game = armed_on_a_meadow();
        game.player.bag.insert(Item::Arrow, 1);
        // room for exactly one more item beyond the bow+arrow already carried
        let carried = game.player.bag_total();
        game.player
            .bag
            .insert(Item::Branch, player::BAG_CAPACITY - carried - 1);

        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftAxe]);
        game.accept_quest(QuestID::StockUp).unwrap();

        game.hunt();

        let kinds: Vec<&EventKind> = game.events().iter().map(Event::kind).collect();
        assert!(
            kinds.iter().any(|k| matches!(k, EventKind::BagFull { .. })),
            "one catch didn't fit"
        );
        assert!(
            kinds.iter().any(|k| matches!(k, EventKind::Hunted { .. })),
            "the rest still counted as a haul"
        );
        assert_eq!(game.player.quest_progress(), 1);
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
    fn disassemble_without_the_item_does_nothing() {
        let mut game = Game::default();
        let before = game.events().len();

        game.disassemble(Item::StoneAxe);

        assert!(game.player.inventory.is_empty());
        assert_eq!(game.events().len(), before);
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
        game.player.inventory.insert(Item::Branch, 1);
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
    fn crafting_the_target_item_logs_the_craft_before_the_quest_completion() {
        // The log is newest-first, so the cause (`Crafted`) must be pushed
        // before its effect (`QuestCompleted`) for the pane to read top-down
        // as "quest complete" above "you craft the axe".
        let mut game = game_with_the_axe_quest_unlocked();
        game.player.grant_recipe("Stone Axe");
        game.player.inventory.insert(Item::Branch, 1);
        game.player.inventory.insert(Item::Stone, 1);
        game.player.inventory.insert(Item::Cord, 1);
        game.accept_quest(QuestID::CraftAxe).unwrap();

        game.craft("Stone Axe");

        let kinds: Vec<&EventKind> = game
            .events()
            .iter()
            .rev()
            .take(2)
            .map(Event::kind)
            .collect();
        assert_eq!(
            kinds,
            [
                &EventKind::QuestCompleted {
                    quest: QuestID::CraftAxe
                },
                &EventKind::Crafted {
                    output: Item::StoneAxe
                },
            ]
        );
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
        game.player.inventory.insert(Item::Branch, 1);
        game.player.inventory.insert(Item::Stone, 1);
        game.player.inventory.insert(Item::Cord, 1);
        game.accept_quest(QuestID::CraftAxe).unwrap();

        game.experiment(&[(Item::Branch, 1), (Item::Stone, 1), (Item::Cord, 1)]);

        // experimenting the target item advances the quest, same as crafting it
        assert!(game.player.completed_quests().contains(&QuestID::CraftAxe));
    }

    #[test]
    fn experimenting_the_target_item_logs_the_experiment_before_the_quest_completion() {
        let mut game = game_with_the_axe_quest_unlocked();
        game.player.inventory.insert(Item::Branch, 1);
        game.player.inventory.insert(Item::Stone, 1);
        game.player.inventory.insert(Item::Cord, 1);
        game.accept_quest(QuestID::CraftAxe).unwrap();

        game.experiment(&[(Item::Branch, 1), (Item::Stone, 1), (Item::Cord, 1)]);

        let kinds: Vec<&EventKind> = game
            .events()
            .iter()
            .rev()
            .take(2)
            .map(Event::kind)
            .collect();
        assert_eq!(
            kinds,
            [
                &EventKind::QuestCompleted {
                    quest: QuestID::CraftAxe
                },
                &EventKind::Experimented {
                    items: vec![(Item::Branch, 1), (Item::Stone, 1), (Item::Cord, 1)],
                    output: Item::StoneAxe,
                    newly_learned: true,
                },
            ]
        );
    }

    #[test]
    fn stock_up_needs_trouble_in_the_east_first() {
        let mut game = Game::default();
        assert_eq!(
            game.accept_quest(QuestID::StockUp),
            Err(QuestError::DependenciesNotMet)
        );
    }

    /// Accepts "Stock Up for Hard Times" and hands the player a bow — in the
    /// bag, since hunting gear must be carried.
    fn game_with_the_stock_up_quest_open() -> Game {
        let mut game = Game::default();
        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftAxe]);
        game.accept_quest(QuestID::StockUp).unwrap();
        game.player.bag.insert(Item::WoodenBow, 1);
        game
    }

    /// Forces the player's tile to a Meadow whose game is a sure thing, then
    /// hunts — resets `last_hunt_time` each call so repeated hunts all land.
    fn sure_hunt(game: &mut Game) {
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);
        let mut tile = MapTile::with_terrain(TerrainType::Meadow);
        for probability in tile.hunt_items.values_mut() {
            *probability = 1.0;
        }
        game.map.tiles[ty][tx] = tile;
        game.hunt();
    }

    #[test]
    fn five_successful_hunts_complete_the_stock_up_quest() {
        let mut game = game_with_the_stock_up_quest_open();
        game.player.bag.insert(Item::Arrow, 5);

        for _ in 0..5 {
            sure_hunt(&mut game);
        }

        assert_eq!(game.player.open_quest(), None);
        assert!(game.player.completed_quests().contains(&QuestID::StockUp));
        assert_eq!(game.player.experience, 30);
    }

    #[test]
    fn hunts_short_of_the_goal_leave_the_stock_up_quest_open() {
        let mut game = game_with_the_stock_up_quest_open();
        game.player.bag.insert(Item::Arrow, 3);

        for _ in 0..3 {
            sure_hunt(&mut game);
        }

        assert_eq!(game.player.open_quest(), Some(QuestID::StockUp));
        assert_eq!(game.player.quest_progress(), 3);
    }

    #[test]
    fn a_hunt_that_catches_nothing_does_not_count_toward_the_stock_up_quest() {
        let mut game = game_with_the_stock_up_quest_open();
        game.player.bag.insert(Item::Arrow, 1);
        // a barren tile: the hunt still spends the arrow, but catches nothing
        let (tx, ty) = game.map.world_to_tile(game.player.coordinates);
        game.map.tiles[ty][tx] = MapTile::with_terrain(TerrainType::Deadland);

        game.hunt();

        assert_eq!(last_event(&game).kind(), &EventKind::HuntMissed);
        assert_eq!(game.player.quest_progress(), 0);
        assert_eq!(game.player.open_quest(), Some(QuestID::StockUp));
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
