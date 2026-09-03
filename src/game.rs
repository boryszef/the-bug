use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

const MAP_MIN_SIZE: u32 = 15;
const MAP_PER_LEVEL_INCREMENT: u32 = 2;
const DECAY_WINDOW_SECS: f64 = 60.0;

#[derive(Debug)]
pub struct Player {
    pub level: u32,
    pub coordinates: (i32, i32),
    pub inventory: HashMap<Material, u32>,
    recipes: Vec<Recipe>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            level: 1,
            coordinates: (0, 0),
            inventory: HashMap::new(),
            recipes: Vec::new(),
        }
    }
}

impl Player {
    /// The recipes the player has discovered, in discovery order.
    pub fn known_recipes(&self) -> &[Recipe] {
        &self.recipes
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TerrainType {
    Meadow,
    Forest,
    Cave,
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
            TerrainType::Village => "Village",
            TerrainType::Deadland => "Deadland",
        };
        write!(f, "{name}")
    }
}

const RANDOM_TERRAIN_TYPES: &[(TerrainType, u32)] = &[
    (TerrainType::Meadow, 30),
    (TerrainType::Forest, 15),
    (TerrainType::Deadland, 50),
    (TerrainType::Cave, 5),
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
pub enum Material {
    Stick,
    Stone,
    Vine,
    Cord,
    StoneAxe,
    Arrow,
    WoodenBow,
}

impl fmt::Display for Material {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Material::Stick => "Stick",
            Material::Stone => "Stone",
            Material::Vine => "Vine",
            Material::Cord => "Cord",
            Material::StoneAxe => "Stone Axe",
            Material::Arrow => "Arrow",
            Material::WoodenBow => "Wooden Bow",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug)]
pub struct MapTile {
    pub terrain_type: TerrainType,
    materials: HashMap<Material, f64>,
    last_search_time: Option<Instant>,
}

impl fmt::Display for MapTile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.terrain_type.symbol())
    }
}

const TERRAIN_MATERIALS: &[(TerrainType, Material, f64)] = &[
    (TerrainType::Forest, Material::Stick, 0.5),
    (TerrainType::Cave, Material::Stone, 0.3),
    (TerrainType::Meadow, Material::Vine, 0.2),
];

fn materials_for_terrain(terrain: TerrainType) -> HashMap<Material, f64> {
    TERRAIN_MATERIALS
        .iter()
        .filter(|&&(t, _, _)| t == terrain)
        .map(|&(_, material, probability)| (material, probability))
        .collect()
}

impl MapTile {
    fn with_terrain(terrain_type: TerrainType) -> MapTile {
        MapTile {
            terrain_type,
            materials: materials_for_terrain(terrain_type),
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

    /// Rebuilds a map from a saved terrain grid. Tile materials are recomputed
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

#[derive(Copy, Clone, Debug)]
pub struct Recipe {
    name: &'static str,
    inputs: &'static [(Material, u32)],
    output: Material,
}

impl Recipe {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn inputs(&self) -> &'static [(Material, u32)] {
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
        inputs: &[(Material::Stick, 1)],
        output: Material::Arrow,
    },
    Recipe {
        name: "Wooden Bow",
        inputs: &[(Material::Stick, 1), (Material::Cord, 1)],
        output: Material::WoodenBow,
    },
    Recipe {
        name: "Cord",
        inputs: &[(Material::Vine, 2)],
        output: Material::Cord,
    },
    Recipe {
        name: "Stone Axe",
        inputs: &[
            (Material::Stick, 1),
            (Material::Stone, 1),
            (Material::Cord, 1),
        ],
        output: Material::StoneAxe,
    },
];

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
    }

    pub fn search(&mut self) {
        let coords = self.player.coordinates;
        let Some(tile) = self.map.get_tile(coords) else {
            return;
        };
        let terrain = tile.terrain_type;
        let last_search = tile.last_search_time;

        let mut rng = rand::rng();
        let found: Vec<Material> = tile
            .materials
            .iter()
            .filter(|&(_, &base)| {
                rng.random_range(0.0..1.0) < adjust_probability(base, last_search)
            })
            .map(|(&material, _)| material)
            .collect();

        for material in found {
            *self.player.inventory.entry(material).or_insert(0) += 1;
            self.log(
                EventCategory::General,
                format!("You find a {material} in the {terrain}."),
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

        for &(material, amount) in recipe.inputs {
            let entry = self.player.inventory.entry(material).or_insert(0);
            if *entry < amount {
                self.log(
                    EventCategory::Crafting,
                    format!(
                        "You don't have enough {material} to craft a {}.",
                        recipe.name
                    ),
                );
                return;
            }
        }

        for &(material, amount) in recipe.inputs {
            *self.player.inventory.get_mut(&material).unwrap() -= amount;
        }

        *self.player.inventory.entry(recipe.output).or_insert(0) += 1;
        self.log(
            EventCategory::Crafting,
            format!("You craft a {}.", recipe.name),
        );
    }

    pub fn experiment(&mut self, materials: &[(Material, u32)]) {
        if materials.is_empty() {
            return;
        }

        let inputs = describe_inputs(materials);

        for &(material, amount) in materials {
            let available = self.player.inventory.get(&material).copied().unwrap_or(0);
            if available < amount {
                self.log(
                    EventCategory::Experiment,
                    format!(
                        "Experiment: {inputs} → not enough {material} (have {available}, need {amount})"
                    ),
                );
                return;
            }
        }

        for &(material, amount) in materials {
            *self.player.inventory.get_mut(&material).unwrap() -= amount;
        }

        let Some(recipe) = RECIPES.iter().find(|recipe| {
            recipe.inputs.len() == materials.len()
                && recipe.inputs.iter().all(|input| materials.contains(input))
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
        }

        *self.player.inventory.entry(recipe.output).or_insert(0) += 1;

        let suffix = if newly_learned { " (new recipe!)" } else { "" };
        self.log(
            EventCategory::Experiment,
            format!("Experiment: {inputs} → {}{suffix}", recipe.name),
        );
    }
}

/// A stable, human-readable rendering of a set of materials, e.g.
/// `"1 Stick + 1 Stone + 1 Cord"`.
fn describe_inputs(materials: &[(Material, u32)]) -> String {
    let mut sorted = materials.to_vec();
    sorted.sort_by_key(|&(material, _)| material);
    sorted
        .iter()
        .map(|(material, quantity)| format!("{quantity} {material}"))
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
    fn material_display_names() {
        assert_eq!(Material::Stick.to_string(), "Stick");
        assert_eq!(Material::StoneAxe.to_string(), "Stone Axe");
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
        game.player.inventory.insert(Material::Stone, 1);
        game.experiment(&[(Material::Stone, 5)]);
        assert_eq!(
            last_event(&game).text(),
            "Experiment: 5 Stone → not enough Stone (have 1, need 5)"
        );
        assert_eq!(last_event(&game).category(), EventCategory::Experiment);

        // failure: shows the inputs and "nothing"
        let mut game = Game::default();
        game.player.inventory.insert(Material::Stick, 1);
        game.player.inventory.insert(Material::Vine, 1);
        game.experiment(&[(Material::Vine, 1), (Material::Stick, 1)]);
        assert_eq!(
            last_event(&game).text(),
            "Experiment: 1 Stick + 1 Vine → nothing"
        );

        // success + discovery, then success without
        let mut game = Game::default();
        game.player.inventory.insert(Material::Vine, 4);
        game.experiment(&[(Material::Vine, 2)]);
        assert_eq!(
            last_event(&game).text(),
            "Experiment: 2 Vine → Cord (new recipe!)"
        );
        game.experiment(&[(Material::Vine, 2)]);
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
    fn craft_events_are_categorised_crafting() {
        let mut game = Game::default();
        game.craft("Cord"); // unknown recipe
        assert_eq!(last_event(&game).category(), EventCategory::Crafting);

        game.player.grant_recipe("Cord");
        game.player.inventory.insert(Material::Vine, 2);
        game.craft("Cord");
        assert_eq!(last_event(&game).text(), "You craft a Cord.");
        assert_eq!(last_event(&game).category(), EventCategory::Crafting);
    }

    #[test]
    fn known_recipes_starts_empty_and_grows_on_discovery() {
        let mut game = Game::default();
        assert!(game.player.known_recipes().is_empty());

        game.player.inventory.insert(Material::Vine, 2);
        game.experiment(&[(Material::Vine, 2)]);

        let known: Vec<&str> = game
            .player
            .known_recipes()
            .iter()
            .map(Recipe::name)
            .collect();
        assert_eq!(known, ["Cord"]);
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
}
