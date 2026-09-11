use super::item::Item;
use rand::RngExt;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::ops::Range;
use web_time::Instant;

/// One block's edge length — odd, since mapgen requires it. The fixed map
/// is a 3×3 grid of these, revealed by level. See docs/map-growth.md.
const BLOCK_SIZE: usize = 17;
/// The whole map's fixed edge length: 3 blocks wide, 3 tall (51). Odd, so
/// `half` is exact and the block grid has a true centre block. `pub(crate)`
/// so `save.rs` (and its tests) can build/expect a validly-sized grid.
pub(crate) const MAP_SIZE: usize = BLOCK_SIZE * 3;
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
const CAVE_FRACTION: f64 = 0.05;
const RUINS_FRACTION: f64 = 0.02;

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
    Deadland,
}

impl TerrainType {
    /// The single-character glyph used to draw this terrain on the map.
    pub fn symbol(self) -> char {
        match self {
            TerrainType::Meadow => '𖧧',
            TerrainType::Forest => '𖠰',
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
    /// What a hunt here can turn up: item → base probability. Empty on terrain
    /// with no fauna (Deadland), so a hunt there always comes back empty.
    pub(super) hunt_items: HashMap<Item, f64>,
    pub(super) last_search_time: Option<Instant>,
    pub(super) last_hunt_time: Option<Instant>,
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
    (TerrainType::Forest, Item::Branch, 0.5),
    (TerrainType::Meadow, Item::Vine, 0.3),
];

/// Items a tile's *POI* adds on top of its terrain's.
const POI_ITEMS: &[(Poi, Item, f64)] = &[
    (Poi::Cave, Item::Stone, 0.3),
    (Poi::Ruins, Item::CopperWire, 0.2),
    (Poi::Ruins, Item::PlasticBottle, 0.2),
    (Poi::Ruins, Item::Umbrella, 0.2),
    (Poi::Ruins, Item::ElectronicToy, 0.1),
];

/// What a *hunt* on a terrain can bring back, with the base probability per
/// hunt. Only Meadow and Forest carry game; Deadland has none, so a hunt there
/// comes back empty — the same way a search of barren ground turns up nothing.
const HUNT_ITEMS: &[(TerrainType, Item, f64)] = &[
    (TerrainType::Meadow, Item::Meat, 0.45),
    (TerrainType::Meadow, Item::Hide, 0.35),
    (TerrainType::Meadow, Item::Bone, 0.30),
    (TerrainType::Meadow, Item::Fur, 0.10),
    (TerrainType::Forest, Item::Meat, 0.50),
    (TerrainType::Forest, Item::Hide, 0.25),
    (TerrainType::Forest, Item::Bone, 0.35),
    (TerrainType::Forest, Item::Fur, 0.35),
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

/// What a hunt on this terrain can bring back: `item → base probability`.
/// Empty for terrain with no game.
fn tile_hunt_items(terrain: TerrainType) -> HashMap<Item, f64> {
    HUNT_ITEMS
        .iter()
        .filter(|&&(t, _, _)| t == terrain)
        .map(|&(_, item, probability)| (item, probability))
        .collect()
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
            hunt_items: tile_hunt_items(terrain_type),
            last_search_time: None,
            last_hunt_time: None,
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

    /// Rolls each of this tile's huntable items against its probability
    /// (decayed by how recently the tile was hunted). Mirrors
    /// [`roll_found_items`](Self::roll_found_items).
    pub(super) fn roll_hunted_items(&self, rng: &mut impl rand::Rng) -> Vec<Item> {
        self.hunt_items
            .iter()
            .filter(|&(_, &base)| {
                rng.random_range(0.0..1.0) < adjust_probability(base, self.last_hunt_time)
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

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    /// A fresh, fixed `MAP_SIZE`×`MAP_SIZE` map: every tile starts as
    /// Deadland with no POI, then the level-1 block (the centre, holding
    /// the Village) is generated for real. The rest stays Deadland until
    /// `reveal_for_level` reaches it.
    pub fn new() -> Map {
        let mut map = Map::from_terrain(
            vec![vec![TerrainType::Deadland; MAP_SIZE]; MAP_SIZE],
            vec![vec![None; MAP_SIZE]; MAP_SIZE],
        );
        map.reveal_for_level(1, &mut rand::rng());
        map
    }

    /// Generates real terrain/POIs for any block `level` newly reaches
    /// that hasn't been generated yet. Idempotent and safe to call
    /// redundantly — "already generated" is detected by the block still
    /// being all Deadland, a reliable (not probabilistic) signal: mapgen's
    /// quota step always allocates a nonzero Meadow/Forest count for any
    /// real generation run. Handles a single XP grant that skips a level
    /// (e.g. 1→3) by checking every block up to `level`, not just the
    /// newest. See docs/map-growth.md for the level→block schedule.
    ///
    /// A no-op on any map that isn't a genuine `MAP_SIZE` grid — the block
    /// scheme doesn't apply to a test fixture like `tiny_map()` (used by
    /// the cucumber suite), which is far smaller and would otherwise index
    /// out of bounds.
    pub fn reveal_for_level(&mut self, level: u32, rng: &mut impl rand::Rng) {
        if self.tiles.len() != MAP_SIZE {
            return;
        }
        for block in unlocked_blocks(level) {
            if self.block_is_ungenerated(block) {
                self.generate_block(block, rng);
            }
        }
    }

    fn block_is_ungenerated(&self, block: (usize, usize)) -> bool {
        let (mut rows, cols) = block_bounds(block);
        rows.all(|row| {
            cols.clone()
                .all(|col| self.tiles[row][col].terrain_type == TerrainType::Deadland)
        })
    }

    fn generate_block(&mut self, block: (usize, usize), rng: &mut impl rand::Rng) {
        let spec = crate::mapgen::Spec {
            size: BLOCK_SIZE,
            clusters: vec![
                (TerrainType::Deadland, DEADLAND_PERCENT),
                (TerrainType::Meadow, MEADOW_PERCENT),
                (TerrainType::Forest, FOREST_PERCENT),
            ],
            affinity: MAP_AFFINITY,
        };
        let local_terrain = crate::mapgen::generate(&spec, rng).expect("hardcoded spec is valid");
        // The centre block's Village tile is placed separately, right
        // below — excluded here so a Cave/Ruins roll never lands on it
        // and gets silently overwritten, which would make the density
        // tests (and the real game) lose a slot to chance.
        let village_local = (block == (1, 1)).then_some((BLOCK_SIZE / 2, BLOCK_SIZE / 2));
        let local_pois = scatter_pois(BLOCK_SIZE, village_local, rng);

        let (rows, cols) = block_bounds(block);
        for (local_row, row) in rows.enumerate() {
            for (local_col, col) in cols.clone().enumerate() {
                self.tiles[row][col] = MapTile::with_terrain_and_poi(
                    local_terrain[local_row][local_col],
                    local_pois[local_row][local_col],
                );
            }
        }

        if let Some((lx, ly)) = village_local {
            let (rows, cols) = block_bounds(block);
            self.tiles[rows.start + ly][cols.start + lx].poi = Some(Poi::Village);
        }
    }

    /// Whether `pos` is inside the map's boundary *and*, for a real
    /// full-grown map, inside a block `level` has unlocked. Any
    /// differently-sized map (test fixtures like `tiny_map()`, built
    /// directly through `from_terrain`) falls back to the plain boundary
    /// check — the block-unlock scheme only applies to a genuine
    /// `MAP_SIZE` grid.
    pub(super) fn is_unlocked(&self, pos: (i32, i32), level: u32) -> bool {
        if !self.contains(pos) {
            return false;
        }
        if self.tiles.len() != MAP_SIZE {
            return true;
        }
        let (x, y) = self.world_to_tile(pos);
        let block = (x / BLOCK_SIZE, y / BLOCK_SIZE);
        unlocked_blocks(level).any(|b| b == block)
    }

    /// Builds a map from an explicit terrain grid and its parallel POI grid.
    /// Tile items are recomputed from the terrain; per-tile search cooldowns
    /// start fresh. Used to rebuild a saved map, and `pub` so a functional
    /// test can stand up a small fixed map instead of a random one.
    pub fn from_terrain(terrain: Vec<Vec<TerrainType>>, pois: Vec<Vec<Option<Poi>>>) -> Map {
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

    /// Whether `pos` is within the map's boundary. Defined in terms of
    /// `get_tile` rather than re-deriving the boundary from `half`, so the
    /// two can never disagree — a map is always square today, but `half`
    /// alone doesn't distinguish an odd-sized map (where it's exact) from an
    /// even-sized one (only reachable via a hand-edited save), where a
    /// `half`-based check would admit a coordinate `get_tile` then refuses.
    pub(super) fn contains(&self, pos: (i32, i32)) -> bool {
        self.get_tile(pos).is_some()
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

    pub fn update_tile_last_hunt_time(&mut self, pos: (i32, i32)) {
        if let Some(tile) = self.get_tile_mut(pos) {
            tile.last_hunt_time = Some(Instant::now());
        }
    }

    /// Forces `item`'s find probability to `1.0` on the tile at `pos`, if
    /// it's among that tile's possible finds — a guaranteed find on the
    /// next `search()` there (decay aside). `pub` so a functional test can
    /// drive a quest gated by `search()` without relying on real
    /// randomness — see docs/functional-tests.md.
    pub fn guarantee_find(&mut self, pos: (i32, i32), item: Item) {
        if let Some(tile) = self.get_tile_mut(pos)
            && let Some(entry) = tile.items.get_mut(&item)
        {
            entry.0 = 1.0;
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
        if terrain.len() != MAP_SIZE || width != MAP_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "map must be {MAP_SIZE}x{MAP_SIZE} (got {}x{width})",
                    terrain.len()
                ),
            ));
        }
        let pois = crate::save::parse_pois(&saved.pois, terrain.len(), width)?;
        Ok(Map::from_terrain(terrain, pois))
    }
}

/// (level at which a block becomes reachable, its `(col, row)` index in
/// the 3×3 grid of `BLOCK_SIZE` blocks) — the exact growth sequence. See
/// docs/map-growth.md.
const BLOCK_UNLOCKS: &[(u32, (usize, usize))] = &[
    (1, (1, 1)),
    (2, (1, 0)),
    (3, (0, 0)),
    (3, (0, 1)),
    (4, (2, 0)),
    (4, (2, 1)),
    (5, (0, 2)),
    (5, (1, 2)),
    (5, (2, 2)),
];

/// The blocks unlocked at `level` — cumulative (every block from a lower
/// threshold stays included).
fn unlocked_blocks(level: u32) -> impl Iterator<Item = (usize, usize)> {
    BLOCK_UNLOCKS
        .iter()
        .filter(move |&&(lv, _)| lv <= level)
        .map(|&(_, block)| block)
}

/// The tile-index `(rows, cols)` ranges `block` occupies in the fixed grid.
fn block_bounds((col, row): (usize, usize)) -> (Range<usize>, Range<usize>) {
    (
        row * BLOCK_SIZE..(row + 1) * BLOCK_SIZE,
        col * BLOCK_SIZE..(col + 1) * BLOCK_SIZE,
    )
}

/// Scatters `Cave`/`Ruins` POI overlays across a `size`×`size` local grid,
/// at the same fractions the old whole-map scatter used — called once per
/// revealed block. `exclude`, when given, is skipped as a candidate (used
/// for the centre block, whose `Village` tile `Map::generate_block` places
/// separately — excluding it here keeps the Cave/Ruins counts exact rather
/// than occasionally losing a slot to a since-overwritten roll). Doesn't
/// place `Village` itself; that's the one-off step for the centre block.
fn scatter_pois(
    size: usize,
    exclude: Option<(usize, usize)>,
    rng: &mut impl rand::Rng,
) -> Vec<Vec<Option<Poi>>> {
    let count = |fraction: f64| (fraction * (size * size) as f64).round() as usize;

    let mut cells: Vec<(usize, usize)> = (0..size)
        .flat_map(|y| (0..size).map(move |x| (x, y)))
        .filter(|&c| Some(c) != exclude)
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

/// Scales `base_probability` down while the tile was worked recently (searched
/// or hunted, within [`DECAY_WINDOW_SECS`]), so going back to the same spot
/// right away rarely pays off.
fn adjust_probability(base_probability: f64, last_used: Option<Instant>) -> f64 {
    let time_elapsed = last_used.map_or(f64::INFINITY, |t| t.elapsed().as_secs_f64());

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
        let map = Map::new();
        let h = map.half;
        assert_eq!(map.world_to_tile((-h, -h)), (0usize, 0usize));
        let last = map.tiles.len() - 1;
        assert_eq!(map.world_to_tile((h, h)), (last, last));
    }

    #[test]
    fn world_to_tile_center_is_the_village() {
        let map = Map::new();
        // center in world coords is (0,0)
        let (cx, cy) = map.world_to_tile((0, 0));
        // the village POI sits on the centre tile
        let tile = map.get_tile((0, 0)).expect("center tile exists");
        assert_eq!(tile.poi, Some(Poi::Village));
        // and the indices point to the middle
        let mid = map.tiles.len() / 2;
        assert_eq!((cx, cy), (mid, mid));
    }

    #[test]
    fn world_to_tile_out_of_bounds() {
        let map = Map::new();
        // get_tile should return None for positions outside the boundary
        let outside = (map.half + 1, map.half + 1);
        assert!(map.get_tile(outside).is_none());
    }

    #[test]
    fn contains_agrees_with_get_tile_on_an_odd_map() {
        let map = Map::new();
        let h = map.half;
        // one step past the boundary in every direction, plus the boundary
        // itself and the centre, on both axes
        for pos in [
            (-h - 1, 0),
            (h + 1, 0),
            (0, -h - 1),
            (0, h + 1),
            (-h, -h),
            (h, h),
            (0, 0),
        ] {
            assert_eq!(map.contains(pos), map.get_tile(pos).is_some(), "at {pos:?}");
        }
    }

    /// A `half`-derived `contains` (the old implementation) disagrees with
    /// `get_tile` on an even-sized grid: `half = size / 2` rounds down, so
    /// `pos.abs() <= half` admits a coordinate that indexes one past the
    /// last row/column. Only reachable via a hand-edited save (`Map::new`
    /// always builds an odd-sized grid), but `contains` must still agree
    /// with `get_tile` there.
    #[test]
    fn contains_agrees_with_get_tile_on_an_even_map() {
        let terrain = vec![vec![TerrainType::Meadow; 4]; 4];
        let pois = vec![vec![None; 4]; 4];
        let map = Map::from_terrain(terrain, pois);
        assert_eq!(map.half, 2);

        for x in -3..=3 {
            for y in -3..=3 {
                assert_eq!(
                    map.contains((x, y)),
                    map.get_tile((x, y)).is_some(),
                    "at ({x}, {y})"
                );
            }
        }
        // the specific edge case: half itself is one past the last index
        assert!(!map.contains((2, 0)));
        assert!(map.contains((1, 0)));
    }

    #[test]
    fn tile_to_world_round_trips_with_world_to_tile() {
        let map = Map::new();
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
    fn new_map_is_fixed_size_with_only_the_centre_block_populated() {
        let map = Map::new();
        assert_eq!(map.tiles.len(), MAP_SIZE);
        assert!(map.tiles.iter().all(|row| row.len() == MAP_SIZE));
        assert_eq!(map.get_tile((0, 0)).unwrap().poi, Some(Poi::Village));

        // every block but the centre is still untouched Deadland
        for block in [
            (0, 0),
            (1, 0),
            (2, 0),
            (0, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ] {
            assert!(
                map.block_is_ungenerated(block),
                "block {block:?} should still be Deadland at level 1"
            );
        }
    }

    #[test]
    fn reveal_for_level_generates_every_newly_unlocked_block_and_is_idempotent() {
        let mut map = Map::new();
        assert!(!map.block_is_ungenerated((1, 1)));

        // a single jump straight to level 5 must not skip levels 2-4's blocks
        map.reveal_for_level(5, &mut rand::rng());
        for block in [
            (1, 0),
            (0, 0),
            (0, 1),
            (2, 0),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ] {
            assert!(
                !map.block_is_ungenerated(block),
                "block {block:?} should be generated by level 5"
            );
        }

        // calling it again (e.g. a second XP grant at the same level) must
        // not re-roll anything already generated
        let before = terrain_grid(&map);
        map.reveal_for_level(5, &mut rand::rng());
        assert_eq!(terrain_grid(&map), before);
    }

    #[test]
    fn is_unlocked_gates_by_level_on_a_real_map_but_not_on_a_test_fixture() {
        let map = Map::new();
        assert!(map.is_unlocked((0, 0), 1)); // the village, always reachable
        let (row_start, col_start) = {
            let (rows, cols) = block_bounds((1, 0));
            (rows.start, cols.start)
        };
        let level2_tile = map.tile_to_world(col_start, row_start);
        assert!(!map.is_unlocked(level2_tile, 1));
        assert!(map.is_unlocked(level2_tile, 2));

        // a small hand-built map (any size but MAP_SIZE) is never gated
        let small = Map::from_terrain(
            vec![vec![TerrainType::Meadow; 3]; 3],
            vec![vec![None; 3]; 3],
        );
        assert!(small.is_unlocked((0, 0), 1));
        assert!(small.is_unlocked((1, 1), 1));
    }

    #[test]
    fn revealing_every_block_places_poi_overlays_at_the_expected_density() {
        let mut map = Map::new();
        map.reveal_for_level(5, &mut rand::rng());

        let count = |want: Poi| {
            map.tiles
                .iter()
                .flatten()
                .filter(|t| t.poi == Some(want))
                .count()
        };
        // each of the 9 blocks rolls its own quota independently, so the
        // expected total is the sum of 9 per-block roundings, not one
        // rounding over the whole MAP_SIZE grid.
        let expect = |frac: f64| (frac * (BLOCK_SIZE * BLOCK_SIZE) as f64).round() as usize * 9;

        assert_eq!(count(Poi::Cave), expect(CAVE_FRACTION));
        assert_eq!(count(Poi::Ruins), expect(RUINS_FRACTION));
        // the centre is always the village, never a cave or ruin.
        assert_eq!(map.get_tile((0, 0)).unwrap().poi, Some(Poi::Village));
    }

    #[test]
    fn a_poi_tile_yields_its_terrain_items_and_its_poi_items() {
        // A cave on forest offers Branch (forest) and Stone (cave).
        let tile = MapTile::with_terrain_and_poi(TerrainType::Forest, Some(Poi::Cave));
        assert_eq!(
            tile.items.get(&Item::Branch).map(|&(_, s)| s),
            Some(FoundIn::Terrain(TerrainType::Forest))
        );
        assert_eq!(
            tile.items.get(&Item::Stone).map(|&(_, s)| s),
            Some(FoundIn::Poi(Poi::Cave))
        );
    }

    #[test]
    fn hunt_items_are_the_terrains_fauna_and_deadland_has_none() {
        let meadow: std::collections::HashSet<Item> = MapTile::with_terrain(TerrainType::Meadow)
            .hunt_items
            .into_keys()
            .collect();
        assert_eq!(
            meadow,
            [Item::Meat, Item::Hide, Item::Bone, Item::Fur]
                .into_iter()
                .collect()
        );
        assert!(
            MapTile::with_terrain(TerrainType::Deadland)
                .hunt_items
                .is_empty()
        );
    }

    #[test]
    fn roll_hunted_items_returns_everything_at_full_odds_and_nothing_when_barren() {
        let mut sure = MapTile::with_terrain(TerrainType::Forest);
        for p in sure.hunt_items.values_mut() {
            *p = 1.0;
        }
        let bag: std::collections::HashSet<Item> = sure
            .roll_hunted_items(&mut rand::rng())
            .into_iter()
            .collect();
        assert_eq!(bag, sure.hunt_items.keys().copied().collect());

        let barren = MapTile::with_terrain(TerrainType::Deadland);
        assert!(barren.roll_hunted_items(&mut rand::rng()).is_empty());
    }

    #[test]
    fn revealed_blocks_have_clustered_not_confetti_terrain() {
        // Each block is clustered independently (its own mapgen::generate
        // call), so a fully-revealed map has roughly 9x one block's
        // component count, not the single-block figure the old whole-map
        // test used. Averaged over a few maps so one unlucky melt can't
        // flake the test.
        let total: usize = (0..5)
            .map(|_| {
                let mut map = Map::new();
                map.reveal_for_level(5, &mut rand::rng());
                let grid = terrain_grid(&map);
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
            average < 130 * 9,
            "terrain barely clustered: {average} components on average across Forest+Meadow+Deadland"
        );
    }
}
