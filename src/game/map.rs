use super::item::Item;
use super::player::Player;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::time::Instant;

const MAP_MIN_SIZE: u32 = 21;
const MAP_PER_LEVEL_INCREMENT: u32 = 2;
const DECAY_WINDOW_SECS: f64 = 60.0;

/// How strongly terrain clumps: `0.0` confetti .. `1.0` one blob per terrain.
/// See `docs/mapgen.md`.
const MAP_AFFINITY: f64 = 0.75;
/// Cluster shares, keeping the old 40:30:20 Deadland:Meadow:Forest ratio,
/// renormalised so the generator's "must sum to 100" holds.
const DEADLAND_PERCENT: f64 = 100.0 * 40.0 / 90.0;
const MEADOW_PERCENT: f64 = 100.0 * 30.0 / 90.0;
const FOREST_PERCENT: f64 = 100.0 * 20.0 / 90.0;
/// Scatter terrain as a fraction of the tile count (kept from the old per-tile
/// weights): the count scales with the map so density stays constant by level.
const CAVE_FRACTION: f64 = 0.07;
const RUINS_FRACTION: f64 = 0.03;
/// How many times to re-roll the RNG before giving up. The generator's clean
/// pre-melt layout check fails whenever a scatter tile splits a terrain slice,
/// which — with scatter at 10% of the map — is roughly a third of attempts; a
/// generous cap makes exhausting it (a panic) practically impossible while the
/// expected cost stays under two tries.
const MAPGEN_ATTEMPTS: usize = 32;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub(super) fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, 1),
            Direction::South => (0, -1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug)]
pub struct MapTile {
    pub terrain_type: TerrainType,
    pub(super) items: HashMap<Item, f64>,
    pub(super) last_search_time: Option<Instant>,
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
    pub(super) fn with_terrain(terrain_type: TerrainType) -> MapTile {
        MapTile {
            terrain_type,
            items: items_for_terrain(terrain_type),
            last_search_time: None,
        }
    }

    /// Rolls each of this tile's items against its search probability
    /// (decayed by how recently the tile was searched), returning what's
    /// found.
    pub(super) fn roll_found_items(&self, rng: &mut impl rand::Rng) -> Vec<Item> {
        self.items
            .iter()
            .filter(|&(_, &base)| {
                rng.random_range(0.0..1.0) < adjust_probability(base, self.last_search_time)
            })
            .map(|(&item, _)| item)
            .collect()
    }
}

#[derive(Debug)]
pub struct Map {
    pub tiles: Vec<Vec<MapTile>>,
    pub half: i32,
}

impl Map {
    pub fn new(player: &Player) -> Map {
        let size = (MAP_MIN_SIZE + player.level * MAP_PER_LEVEL_INCREMENT) as usize;
        let scatter_count = |fraction: f64| (fraction * (size * size) as f64).round() as u32;

        let spec = crate::mapgen::Spec {
            size,
            clusters: vec![
                (TerrainType::Deadland, DEADLAND_PERCENT),
                (TerrainType::Meadow, MEADOW_PERCENT),
                (TerrainType::Forest, FOREST_PERCENT),
            ],
            scatter: vec![
                (TerrainType::Cave, scatter_count(CAVE_FRACTION)),
                (TerrainType::Ruins, scatter_count(RUINS_FRACTION)),
            ],
            affinity: MAP_AFFINITY,
        };

        let mut rng = rand::rng();
        for _ in 0..MAPGEN_ATTEMPTS {
            if let Ok(grid) = crate::mapgen::generate(&spec, &mut rng) {
                return Map::from_terrain(grid);
            }
        }
        panic!("map generation failed with a hardcoded spec");
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

    pub(super) fn world_to_tile(&self, pos: (i32, i32)) -> (usize, usize) {
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

    /// Whether `pos` is within the map's boundary.
    pub(super) fn contains(&self, pos: (i32, i32)) -> bool {
        pos.0.abs() <= self.half && pos.1.abs() <= self.half
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

impl super::SaveState for Map {
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

impl super::RestoreState for Map {
    type Saved = crate::save::MapState;

    fn restore_state(saved: Self::Saved) -> io::Result<Map> {
        Ok(Map::from_terrain(crate::save::parse_terrain(
            &saved.terrain,
        )?))
    }
}

/// Scales `base_probability` down while the tile was searched recently
/// (within [`DECAY_WINDOW_SECS`]), so re-searching the same spot right away
/// rarely pays off.
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
    fn tile_to_world_round_trips_with_world_to_tile() {
        let map = Map::new(&Player::default());
        for pos in [(0, 0), (3, -2), (map.half, -map.half)] {
            let (x, y) = map.world_to_tile(pos);
            assert_eq!(map.tile_to_world(x, y), pos);
        }
    }

    #[test]
    fn symbol_is_the_map_glyph() {
        assert_eq!(TerrainType::Forest.symbol(), '𖠰');
        assert_eq!(TerrainType::Deadland.symbol(), ' ');
    }

    #[test]
    fn adjust_probability_tests() {
        use std::time::Duration;

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

    fn terrain_grid(map: &Map) -> Vec<Vec<TerrainType>> {
        map.tiles
            .iter()
            .map(|row| row.iter().map(|tile| tile.terrain_type).collect())
            .collect()
    }

    #[test]
    fn new_map_size_tracks_the_player_level() {
        for level in [1, 3, 7] {
            let mut player = Player::default();
            player.level = level;
            let map = Map::new(&player);
            let expected = (MAP_MIN_SIZE + level * MAP_PER_LEVEL_INCREMENT) as usize;
            assert_eq!(map.tiles.len(), expected);
            assert!(map.tiles.iter().all(|row| row.len() == expected));
            assert_eq!(
                map.get_tile((0, 0)).unwrap().terrain_type,
                TerrainType::Village
            );
        }
    }

    #[test]
    fn new_map_terrain_is_clustered_not_confetti() {
        // The old per-tile roll left Forest+Meadow+Deadland in ~200 specks
        // between them; at affinity 0.75 they average well under half that.
        // Averaged over a few maps so one unlucky melt can't flake the test.
        let total: usize = (0..5)
            .map(|_| {
                let grid = terrain_grid(&Map::new(&Player::default()));
                [
                    TerrainType::Forest,
                    TerrainType::Meadow,
                    TerrainType::Deadland,
                ]
                .into_iter()
                .map(|t| crate::mapgen::components(&grid, t))
                .sum::<usize>()
            })
            .sum();
        let average = total / 5;
        assert!(
            average < 130,
            "terrain barely clustered: {average} components on average across Forest+Meadow+Deadland"
        );
    }
}
