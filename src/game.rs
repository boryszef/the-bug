use rand::RngExt;
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

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

    fn name(self) -> &'static str {
        match self {
            Direction::North => "north",
            Direction::South => "south",
            Direction::East => "east",
            Direction::West => "west",
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Material {
    Stick,
    Stone,
    Vine,
    Cord,
    StoneAxe,
}

impl fmt::Display for Material {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Material::Stick => "Stick",
            Material::Stone => "Stone",
            Material::Vine => "Vine",
            Material::Cord => "Cord",
            Material::StoneAxe => "Stone Axe",
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
struct Recipe {
    name: &'static str,
    inputs: &'static [(Material, u32)],
    output: Material,
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

const RECIPES: &[Recipe] = &[
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

#[derive(Debug)]
pub struct Game {
    pub player: Player,
    pub map: Map,
    pub events: Vec<String>,
}

impl Default for Game {
    fn default() -> Self {
        let player = Player::default();
        let map = Map::new(&player);
        Self {
            player,
            map,
            events: vec!["You wake up and decide to have a walk.".to_string()],
        }
    }
}

impl Game {
    pub fn walk(&mut self, dir: Direction) {
        let (dx, dy) = dir.delta();
        let (x, y) = self.player.coordinates;
        let (nx, ny) = (x + dx, y + dy);

        if nx.abs() > self.map.half || ny.abs() > self.map.half {
            return;
        }

        self.player.coordinates = (nx, ny);
        if let Some(tile) = self.map.get_tile((nx, ny)) {
            self.events.push(format!(
                "You walk {} and visit {}.",
                dir.name(),
                tile.terrain_type
            ));
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
            self.events
                .push(format!("You found a {material} in the {terrain}."));
        }
        self.map.update_tile_last_search_time(coords);
    }

    pub fn craft(&mut self, recipe_name: &str) {
        let Some(recipe) = self.player.recipes.iter().find(|r| r.name.eq(recipe_name)) else {
            self.events
                .push(format!("You don't know how to craft {recipe_name}"));
            return;
        };

        for &(material, amount) in recipe.inputs {
            let entry = self.player.inventory.entry(material).or_insert(0);
            if *entry < amount {
                self.events
                    .push(format!("Not enough {material} to craft {}.", recipe.name));
                return;
            }
        }

        for &(material, amount) in recipe.inputs {
            *self.player.inventory.get_mut(&material).unwrap() -= amount;
        }

        *self.player.inventory.entry(recipe.output).or_insert(0) += 1;
        self.events.push(format!("You crafted a {}.", recipe.name));
    }

    pub fn experiment(&mut self, materials: &[(Material, u32)]) {
        for &(material, amount) in materials {
            let available = self.player.inventory.get(&material).copied().unwrap_or(0);

            if available < amount {
                self.events
                    .push(format!("Not enough {material} to experiment."));
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
            self.events.push("The experiment failed.".to_string());
            return;
        };

        if !self.player.recipes.contains(recipe) {
            self.player.recipes.push(*recipe);
            self.events
                .push(format!("You discovered how to craft {}!", recipe.name));
        }

        *self.player.inventory.entry(recipe.output).or_insert(0) += 1;

        self.events.push(format!("You created a {}.", recipe.name));
    }
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
    fn walk_west_moves_and_logs_and_respects_boundary() {
        let mut game = Game::default();
        let initial = game.player.coordinates;
        game.walk(Direction::West);
        assert_eq!(game.player.coordinates, (initial.0 - 1, initial.1));
        assert_eq!(game.events.len(), 2);
        let last = game.events.last().unwrap();
        assert!(last.starts_with("You walk west"));

        // set to left boundary and ensure no move
        let h = game.map.half;
        game.player.coordinates = (-h, 0);
        let before_events = game.events.len();
        game.walk(Direction::West);
        assert_eq!(game.player.coordinates.0, -h);
        assert_eq!(game.events.len(), before_events);
    }

    #[test]
    fn walk_east_moves_and_logs_and_respects_boundary() {
        let mut game = Game::default();
        let initial = game.player.coordinates;
        game.walk(Direction::East);
        assert_eq!(game.player.coordinates, (initial.0 + 1, initial.1));
        assert_eq!(game.events.len(), 2);
        let last = game.events.last().unwrap();
        assert!(last.starts_with("You walk east"));

        // set to right boundary and ensure no move
        let h = game.map.half;
        game.player.coordinates = (h, 0);
        let before_events = game.events.len();
        game.walk(Direction::East);
        assert_eq!(game.player.coordinates.0, h);
        assert_eq!(game.events.len(), before_events);
    }

    #[test]
    fn walk_north_moves_and_logs_and_respects_boundary() {
        let mut game = Game::default();
        let initial = game.player.coordinates;
        game.walk(Direction::North);
        assert_eq!(game.player.coordinates, (initial.0, initial.1 + 1));
        assert_eq!(game.events.len(), 2);
        let last = game.events.last().unwrap();
        assert!(last.starts_with("You walk north"));

        // set to top boundary and ensure no move
        let h = game.map.half;
        game.player.coordinates = (0, h);
        let before_events = game.events.len();
        game.walk(Direction::North);
        assert_eq!(game.player.coordinates.1, h);
        assert_eq!(game.events.len(), before_events);
    }

    #[test]
    fn walk_south_moves_and_logs_and_respects_boundary() {
        let mut game = Game::default();
        let initial = game.player.coordinates;
        game.walk(Direction::South);
        assert_eq!(game.player.coordinates, (initial.0, initial.1 - 1));
        assert_eq!(game.events.len(), 2);
        let last = game.events.last().unwrap();
        assert!(last.starts_with("You walk south"));

        // set to bottom boundary and ensure no move
        let h = game.map.half;
        game.player.coordinates = (0, -h);
        let before_events = game.events.len();
        game.walk(Direction::South);
        assert_eq!(game.player.coordinates.1, -h);
        assert_eq!(game.events.len(), before_events);
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
        assert_eq!(
            game.events.last().unwrap(),
            "You walk north and visit Forest."
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

    #[test]
    fn experiment_shortage_message_uses_display_name() {
        let mut game = Game::default();
        game.experiment(&[(Material::StoneAxe, 1)]);
        assert_eq!(
            game.events.last().unwrap(),
            "Not enough Stone Axe to experiment."
        );
    }
}
