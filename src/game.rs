use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::time::{Duration, Instant};

const MAP_MIN_SIZE: u32 = 21;
const MAP_PER_LEVEL_INCREMENT: u32 = 2;
const DECAY_WINDOW_SECS: f64 = 60.0;

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
pub struct Player {
    pub level: u32,
    pub experience: u32,
    /// Successful crafts so far — every tenth grants a point of experience.
    pub crafts_completed: u32,
    pub coordinates: (i32, i32),
    pub inventory: HashMap<Item, u32>,
    recipes: Vec<Recipe>,
    open_quest: Option<QuestID>,
    /// Occurrences of the open quest's condition seen since it was accepted.
    /// Meaningless while `open_quest` is `None`.
    quest_progress: u32,
    quests_completed: Vec<QuestID>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            level: 1,
            experience: 0,
            crafts_completed: 0,
            coordinates: (0, 0),
            inventory: HashMap::new(),
            recipes: Vec::new(),
            open_quest: None,
            quest_progress: 0,
            quests_completed: Vec::new(),
        }
    }
}

impl Player {
    /// The recipes the player has discovered, in discovery order.
    pub fn known_recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    /// The currently active quest, if any.
    pub fn open_quest(&self) -> Option<QuestID> {
        self.open_quest
    }

    /// Occurrences of the open quest's condition seen so far. `0` when no
    /// quest is open.
    pub fn quest_progress(&self) -> u32 {
        self.quest_progress
    }

    /// Quests completed so far, in completion order.
    pub fn completed_quests(&self) -> &[QuestID] {
        &self.quests_completed
    }

    /// Sets quest state directly (used when loading a save).
    pub(crate) fn restore_quest_state(
        &mut self,
        open_quest: Option<QuestID>,
        quest_progress: u32,
        quests_completed: Vec<QuestID>,
    ) {
        self.open_quest = open_quest;
        self.quest_progress = quest_progress;
        self.quests_completed = quests_completed;
    }

    /// Removes `amount` of `item` from the inventory, dropping the entry
    /// entirely once it hits zero so exhausted items don't linger. Callers must
    /// have already checked the player holds enough.
    fn spend(&mut self, item: Item, amount: u32) {
        if let Some(remaining) = self.inventory.get_mut(&item) {
            *remaining -= amount;
            if *remaining == 0 {
                self.inventory.remove(&item);
            }
        }
    }

    /// Marks the recipe with the given name as known (used when loading a save).
    /// Returns `false` for an unrecognised name, which the caller can ignore.
    pub(crate) fn grant_recipe(&mut self, name: &str) -> bool {
        match RECIPES.iter().find(|recipe| recipe.name == name).copied() {
            Some(recipe) => {
                if !self.recipes.contains(&recipe) {
                    self.recipes.push(recipe);
                }
                true
            }
            None => false,
        }
    }
}

impl SaveState for Player {
    type Saved = crate::save::PlayerState;

    fn save_state(&self) -> Self::Saved {
        crate::save::PlayerState {
            level: self.level,
            experience: self.experience,
            crafts_completed: self.crafts_completed,
            coordinates: self.coordinates,
            inventory: self.inventory.clone(),
            recipes: self
                .known_recipes()
                .iter()
                .map(|recipe| recipe.name().to_string())
                .collect(),
            open_quest: self.open_quest,
            quest_progress: self.quest_progress,
            quests_completed: self.quests_completed.clone(),
        }
    }
}

impl RestoreState for Player {
    type Saved = crate::save::PlayerState;

    fn restore_state(saved: Self::Saved) -> io::Result<Player> {
        let mut player = Player {
            level: saved.level,
            experience: saved.experience,
            crafts_completed: saved.crafts_completed,
            coordinates: saved.coordinates,
            inventory: saved.inventory,
            ..Player::default()
        };
        for name in &saved.recipes {
            player.grant_recipe(name);
        }
        player.restore_quest_state(
            saved.open_quest,
            saved.quest_progress,
            saved.quests_completed,
        );
        Ok(player)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, 1),
            Direction::South => (0, -1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainType {
    Meadow,
    Forest,
    Cave,
    Ruins,
    Village,
    Deadland,
}

impl TerrainType {
    /// The single-character glyph used to draw this terrain on the map.
    pub fn symbol(self) -> char {
        match self {
            TerrainType::Meadow => '𖧧',
            TerrainType::Forest => '𖠰',
            TerrainType::Cave => '🪨',
            TerrainType::Ruins => '🏙',
            TerrainType::Village => '🛖',
            TerrainType::Deadland => ' ',
        }
    }
}

impl fmt::Display for TerrainType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            TerrainType::Meadow => "Meadow",
            TerrainType::Forest => "Forest",
            TerrainType::Cave => "Cave",
            TerrainType::Ruins => "Ruins",
            TerrainType::Village => "Village",
            TerrainType::Deadland => "Deadland",
        };
        write!(f, "{name}")
    }
}

const RANDOM_TERRAIN_TYPES: &[(TerrainType, u32)] = &[
    (TerrainType::Meadow, 30),
    (TerrainType::Forest, 20),
    (TerrainType::Deadland, 40),
    (TerrainType::Cave, 7),
    (TerrainType::Ruins, 3),
];

fn choose_weighted<T: Copy>(choices: &[(T, u32)], rng: &mut impl rand::Rng) -> T {
    let total: u32 = choices.iter().map(|(_, weight)| weight).sum();
    let mut n = rng.random_range(0..total);

    for &(value, weight) in choices {
        if n < weight {
            return value;
        }
        n -= weight;
    }

    unreachable!()
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Item {
    Stick,
    Stone,
    Vine,
    Cord,
    StoneAxe,
    Arrow,
    WoodenBow,
    PlasticBottle,
    CopperWire,
    Coil,
    Pole,
    Microcontroller,
    Speaker,
    MetalDetector,
    Battery,
    SolarPanel,
    SolarCharger,
    CircuitBoard,
    Umbrella,
    Fabric,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Item::Stick => "Stick",
            Item::Stone => "Stone",
            Item::Vine => "Vine",
            Item::Cord => "Cord",
            Item::StoneAxe => "Stone Axe",
            Item::Arrow => "Arrow",
            Item::WoodenBow => "Wooden Bow",
            Item::PlasticBottle => "Plastic Bottle",
            Item::CopperWire => "Copper Wire",
            Item::Coil => "Coil",
            Item::Pole => "Pole",
            Item::Speaker => "Speaker",
            Item::Microcontroller => "Microcontroller",
            Item::MetalDetector => "Metal Detector",
            Item::Battery => "Battery",
            Item::SolarPanel => "Solar Panel",
            Item::SolarCharger => "Solar Charger",
            Item::CircuitBoard => "Circuit Board",
            Item::Umbrella => "Umbrella",
            Item::Fabric => "Fabric",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug)]
pub struct MapTile {
    pub terrain_type: TerrainType,
    items: HashMap<Item, f64>,
    last_search_time: Option<Instant>,
}

impl fmt::Display for MapTile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.terrain_type.symbol())
    }
}

const TERRAIN_ITEMS: &[(TerrainType, Item, f64)] = &[
    (TerrainType::Forest, Item::Stick, 0.5),
    (TerrainType::Cave, Item::Stone, 0.3),
    (TerrainType::Meadow, Item::Vine, 0.3),
    (TerrainType::Ruins, Item::CopperWire, 0.2),
    (TerrainType::Ruins, Item::PlasticBottle, 0.2),
    (TerrainType::Ruins, Item::Umbrella, 0.2),
];

fn items_for_terrain(terrain: TerrainType) -> HashMap<Item, f64> {
    TERRAIN_ITEMS
        .iter()
        .filter(|&&(t, _, _)| t == terrain)
        .map(|&(_, item, probability)| (item, probability))
        .collect()
}

impl MapTile {
    fn with_terrain(terrain_type: TerrainType) -> MapTile {
        MapTile {
            terrain_type,
            items: items_for_terrain(terrain_type),
            last_search_time: None,
        }
    }

    fn new() -> MapTile {
        Self::with_terrain(choose_weighted(RANDOM_TERRAIN_TYPES, &mut rand::rng()))
    }
}

#[derive(Debug)]
pub struct Map {
    pub tiles: Vec<Vec<MapTile>>,
    pub half: i32,
}

impl Map {
    pub fn new(player: &Player) -> Map {
        let size = MAP_MIN_SIZE + player.level * MAP_PER_LEVEL_INCREMENT;
        let middle = (size / 2, size / 2);

        let tiles: Vec<Vec<MapTile>> = (0..size)
            .map(|y| {
                (0..size)
                    .map(|x| {
                        if (x, y) == middle {
                            MapTile::with_terrain(TerrainType::Village)
                        } else {
                            MapTile::new()
                        }
                    })
                    .collect()
            })
            .collect();

        Map {
            tiles,
            half: (size / 2) as i32,
        }
    }

    /// Rebuilds a map from a saved terrain grid. Tile items are recomputed
    /// from the terrain; per-tile search cooldowns start fresh.
    pub(crate) fn from_terrain(grid: Vec<Vec<TerrainType>>) -> Map {
        let half = (grid.len() / 2) as i32;
        let tiles = grid
            .into_iter()
            .map(|row| row.into_iter().map(MapTile::with_terrain).collect())
            .collect();
        Map { tiles, half }
    }

    fn world_to_tile(&self, pos: (i32, i32)) -> (usize, usize) {
        ((pos.0 + self.half) as usize, (pos.1 + self.half) as usize)
    }

    /// Inverse of [`world_to_tile`](Self::world_to_tile): the world coordinates
    /// of the tile at row/column indices `(x, y)`.
    pub fn tile_to_world(&self, x: usize, y: usize) -> (i32, i32) {
        (x as i32 - self.half, y as i32 - self.half)
    }

    pub fn get_tile(&self, pos: (i32, i32)) -> Option<&MapTile> {
        let (x, y) = self.world_to_tile(pos);
        self.tiles.get(y)?.get(x)
    }

    fn get_tile_mut(&mut self, pos: (i32, i32)) -> Option<&mut MapTile> {
        let (x, y) = self.world_to_tile(pos);
        self.tiles.get_mut(y)?.get_mut(x)
    }

    pub fn update_tile_last_search_time(&mut self, pos: (i32, i32)) {
        if let Some(tile) = self.get_tile_mut(pos) {
            tile.last_search_time = Some(Instant::now());
        }
    }
}

impl SaveState for Map {
    type Saved = crate::save::MapState;

    fn save_state(&self) -> Self::Saved {
        crate::save::MapState {
            terrain: self
                .tiles
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|tile| crate::save::terrain_code(tile.terrain_type))
                        .collect()
                })
                .collect(),
        }
    }
}

impl RestoreState for Map {
    type Saved = crate::save::MapState;

    fn restore_state(saved: Self::Saved) -> io::Result<Map> {
        Ok(Map::from_terrain(crate::save::parse_terrain(
            &saved.terrain,
        )?))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Recipe {
    name: &'static str,
    inputs: &'static [(Item, u32)],
    output: Item,
    reversible: bool,
}

impl Recipe {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn inputs(&self) -> &'static [(Item, u32)] {
        self.inputs
    }
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

const RECIPES: &[Recipe] = &[
    Recipe {
        name: "Arrow",
        inputs: &[(Item::Stick, 1)],
        output: Item::Arrow,
        reversible: false,
    },
    Recipe {
        name: "Wooden Bow",
        inputs: &[(Item::Stick, 1), (Item::Cord, 1)],
        output: Item::WoodenBow,
        reversible: true,
    },
    Recipe {
        name: "Cord",
        inputs: &[(Item::Vine, 2)],
        output: Item::Cord,
        reversible: false,
    },
    Recipe {
        name: "Stone Axe",
        inputs: &[(Item::Stick, 1), (Item::Stone, 1), (Item::Cord, 1)],
        output: Item::StoneAxe,
        reversible: true,
    },
    Recipe {
        name: "Coil",
        inputs: &[(Item::CopperWire, 2), (Item::PlasticBottle, 1)],
        output: Item::Coil,
        reversible: false,
    },
    Recipe {
        name: "Metal Detector",
        inputs: &[
            (Item::Coil, 1),
            (Item::Pole, 1),
            (Item::Speaker, 1),
            (Item::Microcontroller, 1),
        ],
        output: Item::MetalDetector,
        reversible: true,
    },
    Recipe {
        name: "Solar Charger",
        inputs: &[
            (Item::CopperWire, 1),
            (Item::SolarPanel, 1),
            (Item::CircuitBoard, 1),
        ],
        output: Item::SolarCharger,
        reversible: true,
    },
    Recipe {
        name: "Umbrella",
        inputs: &[(Item::Fabric, 1), (Item::Pole, 1)],
        output: Item::Umbrella,
        reversible: true,
    },
];

/// The reversible recipe that produces `output`, if any. Drives disassembly:
/// the player can take such an item apart to recover the recipe's inputs.
pub(crate) fn reversible_recipe_for(output: Item) -> Option<Recipe> {
    RECIPES
        .iter()
        .find(|recipe| recipe.reversible && recipe.output == output)
        .copied()
}

/// Which part of the game an event belongs to. Used to colour the log.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    #[default]
    General,
    Experiment,
    Crafting,
}

/// A message in the event log, tagged with its category and how far into the
/// session it happened.
#[derive(Debug)]
pub struct Event {
    category: EventCategory,
    text: String,
    elapsed: Duration,
}

impl Event {
    pub(crate) fn new(
        category: EventCategory,
        text: impl Into<String>,
        elapsed: Duration,
    ) -> Event {
        Event {
            category,
            text: text.into(),
            elapsed,
        }
    }

    pub fn category(&self) -> EventCategory {
        self.category
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Time from the start of the session to when this event was logged.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

impl SaveState for Event {
    type Saved = crate::save::EventState;

    fn save_state(&self) -> Self::Saved {
        crate::save::EventState {
            category: self.category(),
            text: self.text().to_string(),
            elapsed_secs: self.elapsed().as_secs_f64(),
        }
    }
}

impl RestoreState for Event {
    type Saved = crate::save::EventState;

    fn restore_state(saved: Self::Saved) -> io::Result<Event> {
        Ok(Event::new(
            saved.category,
            saved.text,
            Duration::from_secs_f64(saved.elapsed_secs.max(0.0)),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestID {
    CraftArrows,
    ExploreRuins,
}

/// Failure reasons for [`Game::accept_quest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestError {
    /// Another quest is already open; only one can be active at a time.
    AnotherQuestActive,
    /// This quest is already in `Player::completed_quests()`.
    AlreadyCompleted,
    /// Not every quest in `Quest::dependencies` has been completed yet.
    DependenciesNotMet,
}

/// A game action that can count toward an open quest's [`QuestCondition`].
/// Fired at the moment the action happens (see `Game::grant_item`/`walk`) —
/// not stored or replayed, see docs/quest-system.md for why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventTypeID {
    CraftItem(Item),
    VisitTerrain(TerrainType),
}

/// What it takes to complete a quest: `count` occurrences of `event`.
struct QuestCondition {
    event: EventTypeID,
    count: u32,
}

pub struct Quest {
    pub id: QuestID,
    pub name: &'static str,
    pub description: &'static str,
    pub dependencies: &'static [QuestID],
    condition: QuestCondition,
    reward_xp: u32,
    reward_items: &'static [(Item, u32)],
}

const QUESTS: &[Quest] = &[
    Quest {
        id: QuestID::CraftArrows,
        name: "Craft Arrows",
        description: "Group of local hunters is preparing for a hunt. They asked you to create 5 arrows for them. Visit the forrest to gather sticks and exeriment with them to learn how to craft arrows.",
        dependencies: &[],
        condition: QuestCondition {
            event: EventTypeID::CraftItem(Item::Arrow),
            count: 5,
        },
        reward_xp: 20,
        reward_items: &[],
    },
    Quest {
        id: QuestID::ExploreRuins,
        name: "Explore the Ruins",
        description: "A passing traveler told you about some ruins nearby. They said that there are some old artifacts there. You should go and explore.",
        dependencies: &[],
        condition: QuestCondition {
            event: EventTypeID::VisitTerrain(TerrainType::Ruins),
            count: 1,
        },
        reward_xp: 10,
        reward_items: &[],
    },
];

/// Looks up a quest by id. Panics if `QUESTS` is missing a variant — a bug in
/// the static table, not a runtime condition.
fn quest_for(id: QuestID) -> &'static Quest {
    QUESTS
        .iter()
        .find(|quest| quest.id == id)
        .expect("QUESTS must contain every QuestID")
}

/// Whether every quest in `quest.dependencies` is in `completed`.
fn dependencies_met(quest: &Quest, completed: &[QuestID]) -> bool {
    quest.dependencies.iter().all(|dep| completed.contains(dep))
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

    /// `(recipes discovered, recipes that exist)`.
    pub fn recipe_progress(&self) -> (usize, usize) {
        (self.player.known_recipes().len(), RECIPES.len())
    }

    /// Accepts `id` as the player's open quest. Fails if another quest is
    /// already open, `id` was already completed, or its dependencies aren't
    /// all in `Player::completed_quests()` yet.
    pub fn accept_quest(&mut self, id: QuestID) -> Result<(), QuestError> {
        if self.player.open_quest.is_some() {
            return Err(QuestError::AnotherQuestActive);
        }
        if self.player.quests_completed.contains(&id) {
            return Err(QuestError::AlreadyCompleted);
        }
        let quest = quest_for(id);
        if !dependencies_met(quest, &self.player.quests_completed) {
            return Err(QuestError::DependenciesNotMet);
        }

        self.player.open_quest = Some(id);
        self.player.quest_progress = 0;
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
            .filter(|quest| self.player.open_quest != Some(quest.id))
            .filter(|quest| !self.player.quests_completed.contains(&quest.id))
            .filter(|quest| dependencies_met(quest, &self.player.quests_completed))
            .collect()
    }

    /// Records that `item` was produced through crafting or experimenting,
    /// and reports it toward the open quest's condition. `search()` (found in
    /// the wild) and `disassemble()` (recovered) do not go through this.
    fn grant_item(&mut self, item: Item, amount: u32) {
        *self.player.inventory.entry(item).or_insert(0) += amount;
        self.note_quest_event(EventTypeID::CraftItem(item));
    }

    /// Routes a game action to the open quest's condition, advancing
    /// `Player::quest_progress` and completing the quest once it's met.
    fn note_quest_event(&mut self, event: EventTypeID) {
        let Some(quest_id) = self.player.open_quest else {
            return;
        };
        let quest = quest_for(quest_id);
        if quest.condition.event != event {
            return;
        }

        self.player.quest_progress += 1;
        if self.player.quest_progress >= quest.condition.count {
            self.complete_open_quest(quest);
        }
    }

    /// Completes `quest`: grants its reward, moves it into
    /// `Player::completed_quests()`, and clears `open_quest`/`quest_progress`.
    fn complete_open_quest(&mut self, quest: &'static Quest) {
        self.player.open_quest = None;
        self.player.quest_progress = 0;
        self.player.quests_completed.push(quest.id);
        self.player.experience += quest.reward_xp;
        for &(item, amount) in quest.reward_items {
            *self.player.inventory.entry(item).or_insert(0) += amount;
        }
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
        let (dx, dy) = dir.delta();
        let (x, y) = self.player.coordinates;
        let (nx, ny) = (x + dx, y + dy);

        if nx.abs() > self.map.half || ny.abs() > self.map.half {
            return;
        }

        self.player.coordinates = (nx, ny);
        if let Some(tile) = self.map.get_tile((nx, ny)) {
            self.note_quest_event(EventTypeID::VisitTerrain(tile.terrain_type));
        }
    }

    pub fn search(&mut self) {
        let coords = self.player.coordinates;
        let Some(tile) = self.map.get_tile(coords) else {
            return;
        };
        let terrain = tile.terrain_type;
        let last_search = tile.last_search_time;

        let mut rng = rand::rng();
        let found: Vec<Item> = tile
            .items
            .iter()
            .filter(|&(_, &base)| {
                rng.random_range(0.0..1.0) < adjust_probability(base, last_search)
            })
            .map(|(&item, _)| item)
            .collect();

        for item in found {
            *self.player.inventory.entry(item).or_insert(0) += 1;
            self.log(
                EventCategory::General,
                format!("You find a {item} in the {terrain}."),
            );
        }
        self.map.update_tile_last_search_time(coords);
    }

    pub fn craft(&mut self, recipe_name: &str) {
        let Some(recipe) = self
            .player
            .recipes
            .iter()
            .find(|r| r.name.eq(recipe_name))
            .copied()
        else {
            self.log(
                EventCategory::Crafting,
                format!("You don't know how to craft a {recipe_name}."),
            );
            return;
        };

        for &(item, amount) in recipe.inputs {
            if self.player.inventory.get(&item).copied().unwrap_or(0) < amount {
                self.log(
                    EventCategory::Crafting,
                    format!("You don't have enough {item} to craft a {}.", recipe.name),
                );
                return;
            }
        }

        for &(item, amount) in recipe.inputs {
            self.player.spend(item, amount);
        }

        self.grant_item(recipe.output, 1);
        self.log(
            EventCategory::Crafting,
            format!("You craft a {}.", recipe.name),
        );

        self.player.crafts_completed += 1;
        if self.player.crafts_completed.is_multiple_of(10) {
            self.player.experience += 1;
        }
    }

    pub fn experiment(&mut self, items: &[(Item, u32)]) {
        if items.is_empty() {
            return;
        }

        let inputs = describe_inputs(items);

        for &(item, amount) in items {
            let available = self.player.inventory.get(&item).copied().unwrap_or(0);
            if available < amount {
                self.log(
                    EventCategory::Experiment,
                    format!(
                        "Experiment: {inputs} → not enough {item} (have {available}, need {amount})"
                    ),
                );
                return;
            }
        }

        for &(item, amount) in items {
            self.player.spend(item, amount);
        }

        let Some(recipe) = RECIPES.iter().find(|recipe| {
            recipe.inputs.len() == items.len()
                && recipe.inputs.iter().all(|input| items.contains(input))
        }) else {
            self.log(
                EventCategory::Experiment,
                format!("Experiment: {inputs} → nothing"),
            );
            return;
        };

        let newly_learned = !self.player.recipes.contains(recipe);
        if newly_learned {
            self.player.recipes.push(*recipe);
            self.player.experience += 10;
        }

        self.grant_item(recipe.output, 1);

        let suffix = if newly_learned { " (new recipe!)" } else { "" };
        self.log(
            EventCategory::Experiment,
            format!("Experiment: {inputs} → {}{suffix}", recipe.name),
        );
    }

    /// Takes one `item` apart, returning the inputs of the reversible recipe
    /// that makes it. Does nothing if no reversible recipe produces `item` or
    /// the player is not carrying one.
    pub fn disassemble(&mut self, item: Item) {
        let Some(recipe) = reversible_recipe_for(item) else {
            return;
        };
        if self.player.inventory.get(&item).copied().unwrap_or(0) == 0 {
            return;
        }

        self.player.spend(item, 1);
        for &(input, amount) in recipe.inputs {
            *self.player.inventory.entry(input).or_insert(0) += amount;
        }

        self.log(
            EventCategory::Crafting,
            format!(
                "You take apart a {item}, recovering {}.",
                describe_inputs(recipe.inputs)
            ),
        );
    }
}

/// A stable, human-readable rendering of a set of items, e.g.
/// `"1 Stick + 1 Stone + 1 Cord"`.
fn describe_inputs(items: &[(Item, u32)]) -> String {
    let mut sorted = items.to_vec();
    sorted.sort_by_key(|&(item, _)| item);
    sorted
        .iter()
        .map(|(item, quantity)| format!("{quantity} {item}"))
        .collect::<Vec<_>>()
        .join(" + ")
}

fn adjust_probability(base_probability: f64, last_search_time: Option<Instant>) -> f64 {
    let time_elapsed = last_search_time.map_or(f64::INFINITY, |t| t.elapsed().as_secs_f64());

    if time_elapsed < DECAY_WINDOW_SECS {
        base_probability * (time_elapsed / DECAY_WINDOW_SECS)
    } else {
        base_probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_tile_corners() {
        let player = Player::default();
        let map = Map::new(&player);
        let h = map.half;
        assert_eq!(map.world_to_tile((-h, -h)), (0usize, 0usize));
        let last = map.tiles.len() - 1;
        assert_eq!(map.world_to_tile((h, h)), (last, last));
    }

    #[test]
    fn world_to_tile_center_is_village() {
        let player = Player::default();
        let map = Map::new(&player);
        // center in world coords is (0,0)
        let (cx, cy) = map.world_to_tile((0, 0));
        // ensure center tile is the village created at middle
        let tile = map.get_tile((0, 0)).expect("center tile exists");
        match tile.terrain_type {
            TerrainType::Village => (),
            other => panic!("expected Village at center, found {other:?}"),
        }
        // also ensure indices point to the middle
        let mid = map.tiles.len() / 2;
        assert_eq!((cx, cy), (mid, mid));
    }

    #[test]
    fn world_to_tile_out_of_bounds() {
        let player = Player::default();
        let map = Map::new(&player);
        // get_tile should return None for positions outside the boundary
        let outside = (map.half + 1, map.half + 1);
        assert!(map.get_tile(outside).is_none());
    }

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
        let tile_coordinates = game.map.world_to_tile(game.player.coordinates);

        game.map.tiles[tile_coordinates.0 - 1][tile_coordinates.1] =
            MapTile::with_terrain(TerrainType::Meadow);
        game.map.tiles[tile_coordinates.0 + 1][tile_coordinates.1] =
            MapTile::with_terrain(TerrainType::Forest);

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

    #[test]
    fn adjust_probability_tests() {
        use std::time::{Duration, Instant};

        let base = 0.6f64;

        // None => returns base_probability
        let p_none = adjust_probability(base, None);
        assert!((p_none - base).abs() < f64::EPSILON);

        // recent search (about 30s ago) scales probability down
        let last_recent = Instant::now() - Duration::from_secs(30);
        let elapsed = last_recent.elapsed().as_secs_f64();
        let expected_recent = base * (elapsed / DECAY_WINDOW_SECS);
        let p_recent = adjust_probability(base, Some(last_recent));
        assert!((p_recent - expected_recent).abs() < 1e-6);

        // old search (>= 60s) returns base_probability unchanged
        let last_old = Instant::now() - Duration::from_secs(120);
        let p_old = adjust_probability(base, Some(last_old));
        assert!((p_old - base).abs() < f64::EPSILON);
    }

    #[test]
    fn item_display_names() {
        assert_eq!(Item::Stick.to_string(), "Stick");
        assert_eq!(Item::StoneAxe.to_string(), "Stone Axe");
    }

    #[test]
    fn terrain_type_display_is_name_and_symbol_is_glyph() {
        assert_eq!(TerrainType::Forest.to_string(), "Forest");
        assert_eq!(TerrainType::Forest.symbol(), '𖠰');
        assert_eq!(TerrainType::Deadland.symbol(), ' ');
    }

    #[test]
    fn tile_to_world_round_trips_with_world_to_tile() {
        let map = Map::new(&Player::default());
        for pos in [(0, 0), (3, -2), (map.half, -map.half)] {
            let (x, y) = map.world_to_tile(pos);
            assert_eq!(map.tile_to_world(x, y), pos);
        }
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
        assert_eq!(game.recipe_progress(), (0, RECIPES.len()));

        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]);
        assert_eq!(game.recipe_progress().0, 1);
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
    //
    // Failing tests written ahead of the implementation (see
    // docs/quest-system.md). `CraftArrows`/`ExploreRuins` in `QUESTS` are
    // used directly for the integration-style tests below; `dependencies_met`
    // and `complete_open_quest` also get isolated tests against a local
    // fixture `Quest`, since the real `QUESTS` table currently has no
    // dependency chain or non-empty rewards (placeholder game content).

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
    fn dependencies_met_is_false_when_a_dependency_is_missing() {
        assert!(!dependencies_met(&FIXTURE_QUEST, &[]));
    }

    #[test]
    fn dependencies_met_is_true_once_every_dependency_is_completed() {
        assert!(dependencies_met(&FIXTURE_QUEST, &[QuestID::ExploreRuins]));
    }

    #[test]
    fn complete_open_quest_grants_reward_and_resets_quest_state() {
        let mut game = Game::default();
        game.player.open_quest = Some(FIXTURE_QUEST.id);
        game.player.quest_progress = 1;
        let xp_before = game.player.experience;

        game.complete_open_quest(&FIXTURE_QUEST);

        assert_eq!(game.player.open_quest, None);
        assert_eq!(game.player.quest_progress, 0);
        assert_eq!(game.player.quests_completed, [FIXTURE_QUEST.id]);
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
