use super::item::Item;
use super::player::Player;
use rand::RngExt;
use rand::seq::SliceRandom;
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
/// POI density as a fraction of the tile count (kept from the old scatter
/// weights): the count scales with the map so density stays constant by level.
const CAVE_FRACTION: f64 = 0.07;
const RUINS_FRACTION: f64 = 0.03;
/// How many times to re-roll the RNG before giving up. The generator's
/// pre-melt layout check fails only when the single village cell splits a
/// terrain slice — rare — so a small cap is plenty; exhausting it is a panic.
const MAPGEN_ATTEMPTS: usize = 8;

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
    Village,
    Deadland,
}

impl TerrainType {
    /// The single-character glyph used to draw this terrain on the map.
    pub fn symbol(self) -> char {
        match self {
            TerrainType::Meadow => '𖧧',
            TerrainType::Forest => '𖠰',
            TerrainType::Village => '🛖',
            TerrainType::Deadland => ' ',
        }
    }
}

/// A point of interest sitting on a tile, on top of its terrain — a discrete
/// landmark, as opposed to the terrain fill. At most one per tile (it's an
/// `Option` on [`MapTile`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Poi {
    Cave,
    Ruins,
    Village,
}

impl Poi {
    /// The single-character glyph used to draw this POI on the map.
    pub fn symbol(self) -> char {
        match self {
            Poi::Cave => '🪨',
            Poi::Ruins => '🏙',
            Poi::Village => '🛖',
        }
    }
}

/// Where a searched-up item came from: the tile's terrain, or its POI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoundIn {
    Terrain(TerrainType),
    Poi(Poi),
}

#[derive(Debug)]
pub struct MapTile {
    pub terrain_type: TerrainType,
    pub poi: Option<Poi>,
    /// What a search here can turn up: item → (base probability, its source).
    pub(super) items: HashMap<Item, (f64, FoundIn)>,
    pub(super) last_search_time: Option<Instant>,
}

impl fmt::Display for MapTile {
    /// The tile's map glyph: its POI's if it has one, otherwise its terrain's.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let glyph = self
            .poi
            .map_or_else(|| self.terrain_type.symbol(), Poi::symbol);
        write!(f, "{glyph}")
    }
}

/// Items a tile's *terrain* can turn up when searched, with the base
/// probability per search.
const TERRAIN_ITEMS: &[(TerrainType, Item, f64)] = &[
    (TerrainType::Forest, Item::Stick, 0.5),
    (TerrainType::Meadow, Item::Vine, 0.3),
];

/// Items a tile's *POI* adds on top of its terrain's.
const POI_ITEMS: &[(Poi, Item, f64)] = &[
    (Poi::Cave, Item::Stone, 0.3),
    (Poi::Ruins, Item::CopperWire, 0.2),
    (Poi::Ruins, Item::PlasticBottle, 0.2),
    (Poi::Ruins, Item::Umbrella, 0.2),
];

/// Everything a tile can yield on a search: its terrain's items plus its POI's,
/// each tagged with where it came from. A POI entry overrides a terrain entry
/// for the same item (there are none today).
fn tile_items(terrain: TerrainType, poi: Option<Poi>) -> HashMap<Item, (f64, FoundIn)> {
    let mut items: HashMap<Item, (f64, FoundIn)> = TERRAIN_ITEMS
        .iter()
        .filter(|&&(t, _, _)| t == terrain)
        .map(|&(_, item, probability)| (item, (probability, FoundIn::Terrain(terrain))))
        .collect();
    if let Some(poi) = poi {
        for &(_, item, probability) in POI_ITEMS.iter().filter(|&&(p, _, _)| p == poi) {
            items.insert(item, (probability, FoundIn::Poi(poi)));
        }
    }
    items
}

impl MapTile {
    /// A tile with terrain but no POI — a convenience for tests; real tiles are
    /// built through [`with_terrain_and_poi`](Self::with_terrain_and_poi).
    #[cfg(test)]
    pub(super) fn with_terrain(terrain_type: TerrainType) -> MapTile {
        Self::with_terrain_and_poi(terrain_type, None)
    }

    pub(super) fn with_terrain_and_poi(terrain_type: TerrainType, poi: Option<Poi>) -> MapTile {
        MapTile {
            terrain_type,
            poi,
            items: tile_items(terrain_type, poi),
            last_search_time: None,
        }
    }

    /// Rolls each of this tile's items against its search probability
    /// (decayed by how recently the tile was searched), returning what's
    /// found and where each came from.
    pub(super) fn roll_found_items(&self, rng: &mut impl rand::Rng) -> Vec<(Item, FoundIn)> {
        self.items
            .iter()
            .filter(|&(_, &(base, _))| {
                rng.random_range(0.0..1.0) < adjust_probability(base, self.last_search_time)
            })
            .map(|(&item, &(_, source))| (item, source))
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

        let spec = crate::mapgen::Spec {
            size,
            clusters: vec![
                (TerrainType::Deadland, DEADLAND_PERCENT),
                (TerrainType::Meadow, MEADOW_PERCENT),
                (TerrainType::Forest, FOREST_PERCENT),
            ],
            affinity: MAP_AFFINITY,
        };

        let mut rng = rand::rng();
        for _ in 0..MAPGEN_ATTEMPTS {
            if let Ok(grid) = crate::mapgen::generate(&spec, &mut rng) {
                let pois = scatter_pois(grid.len(), &mut rng);
                return Map::from_terrain(grid, pois);
            }
        }
        panic!("map generation failed with a hardcoded spec");
    }

    /// Rebuilds a map from a saved terrain grid and its parallel POI grid. Tile
    /// items are recomputed from the terrain; per-tile search cooldowns start
    /// fresh.
    pub(crate) fn from_terrain(terrain: Vec<Vec<TerrainType>>, pois: Vec<Vec<Option<Poi>>>) -> Map {
        let half = (terrain.len() / 2) as i32;
        let tiles = terrain
            .into_iter()
            .zip(pois)
            .map(|(trow, prow)| {
                trow.into_iter()
                    .zip(prow)
                    .map(|(t, p)| MapTile::with_terrain_and_poi(t, p))
                    .collect()
            })
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
        let grid = |code: fn(&MapTile) -> char| {
            self.tiles
                .iter()
                .map(|row| row.iter().map(code).collect())
                .collect()
        };
        crate::save::MapState {
            terrain: grid(|t| crate::save::terrain_code(t.terrain_type)),
            pois: grid(|t| crate::save::poi_code(t.poi)),
        }
    }
}

impl super::RestoreState for Map {
    type Saved = crate::save::MapState;

    fn restore_state(saved: Self::Saved) -> io::Result<Map> {
        let terrain = crate::save::parse_terrain(&saved.terrain)?;
        let width = terrain.first().map_or(0, Vec::len);
        let pois = crate::save::parse_pois(&saved.pois, terrain.len(), width)?;
        Ok(Map::from_terrain(terrain, pois))
    }
}

/// Lays out the map's points of interest as a grid parallel to the terrain
/// grid: `Cave` and `Ruins` on cells picked uniformly at random, at the same
/// densities the old scatter terrain used. The centre cell is left clear for
/// the village.
fn scatter_pois(size: usize, rng: &mut impl rand::Rng) -> Vec<Vec<Option<Poi>>> {
    let mid = size / 2;
    let count = |fraction: f64| (fraction * (size * size) as f64).round() as usize;

    let mut cells: Vec<(usize, usize)> = (0..size)
        .flat_map(|y| (0..size).map(move |x| (x, y)))
        .filter(|&c| c != (mid, mid))
        .collect();
    cells.shuffle(rng);

    let mut pois = vec![vec![None; size]; size];
    let (caves, rest) = cells.split_at(count(CAVE_FRACTION));
    for &(x, y) in caves {
        pois[y][x] = Some(Poi::Cave);
    }
    for &(x, y) in &rest[..count(RUINS_FRACTION)] {
        pois[y][x] = Some(Poi::Ruins);
    }
    pois
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
    fn new_map_places_poi_overlays_at_the_expected_density() {
        let map = Map::new(&Player::default());
        let side = map.tiles.len();
        let count = |want: Poi| {
            map.tiles
                .iter()
                .flatten()
                .filter(|t| t.poi == Some(want))
                .count()
        };
        let expect = |frac: f64| (frac * (side * side) as f64).round() as usize;

        assert_eq!(count(Poi::Cave), expect(CAVE_FRACTION));
        assert_eq!(count(Poi::Ruins), expect(RUINS_FRACTION));
        // caves/ruins never land on the centre cell.
        assert_eq!(map.get_tile((0, 0)).unwrap().poi, None);
    }

    #[test]
    fn a_poi_tile_yields_its_terrain_items_and_its_poi_items() {
        // A cave on forest offers Stick (forest) and Stone (cave).
        let tile = MapTile::with_terrain_and_poi(TerrainType::Forest, Some(Poi::Cave));
        assert_eq!(
            tile.items.get(&Item::Stick).map(|&(_, s)| s),
            Some(FoundIn::Terrain(TerrainType::Forest))
        );
        assert_eq!(
            tile.items.get(&Item::Stone).map(|&(_, s)| s),
            Some(FoundIn::Poi(Poi::Cave))
        );
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
