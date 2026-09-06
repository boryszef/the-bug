//! The map prepared for display: every tile paired with its world coordinates.

use crate::game::{Map, MapTile};
#[cfg(feature = "gui")]
use crate::game::{Poi, TerrainType};

/// One tile's data as a renderer needs it: its world coordinates and the
/// terrain to draw there.
///
/// Owned rather than a `&MapTile` reference so a later increment can add
/// `feature`/`connections` fields without a front end reaching back into the
/// domain model — see `docs/gui-map.md`.
///
/// Only the `gui` front end consumes this; `#[cfg]`-gated so a `tui`-only
/// build doesn't carry it as dead code.
#[cfg(feature = "gui")]
pub struct TileView {
    pub world: (i32, i32),
    pub terrain: TerrainType,
    /// The point of interest on this tile, if any.
    pub poi: Option<Poi>,
    /// Terrain of the four orthogonally-adjacent tiles, `None` past the map
    /// edge. Order: North, East, South, West (world space — North is `+y`).
    pub neighbours: [Option<TerrainType>; 4],
}

/// Every tile as a [`TileView`]. Thin adapter over [`world_tiles`] for now;
/// the indirection is what lets both a procedural and a sprite renderer
/// share one descriptor.
#[cfg(feature = "gui")]
pub fn tile_views(map: &Map) -> impl Iterator<Item = TileView> + '_ {
    world_tiles(map).map(move |((wx, wy), tile)| TileView {
        world: (wx, wy),
        terrain: tile.terrain_type,
        poi: tile.poi,
        neighbours: [
            map.get_tile((wx, wy + 1)).map(|t| t.terrain_type), // North
            map.get_tile((wx + 1, wy)).map(|t| t.terrain_type), // East
            map.get_tile((wx, wy - 1)).map(|t| t.terrain_type), // South
            map.get_tile((wx - 1, wy)).map(|t| t.terrain_type), // West
        ],
    })
}

/// The fill colour for a terrain type, as raw `(r, g, b)` so the palette
/// stays free of any toolkit's colour type — mirrors how
/// [`TerrainType::symbol`] centralises the glyph.
#[cfg(feature = "gui")]
pub fn terrain_rgb(terrain: TerrainType) -> (u8, u8, u8) {
    match terrain {
        TerrainType::Meadow => (0x7c, 0xb3, 0x42),
        TerrainType::Forest => (0x2f, 0x6d, 0x2f),
        TerrainType::Deadland => (0x33, 0x30, 0x2b),
    }
}

/// Every tile on the map together with its `(x, y)` world coordinates.
///
/// Centralises the `tiles[y][x]` → world-coordinate transform (see
/// [`Map::tile_to_world`]) so a UI never re-derives it.
pub fn world_tiles(map: &Map) -> impl Iterator<Item = ((i32, i32), &MapTile)> {
    map.tiles.iter().enumerate().flat_map(move |(y, row)| {
        row.iter()
            .enumerate()
            .map(move |(x, tile)| (map.tile_to_world(x, y), tile))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "gui")]
    use crate::game::TerrainType;
    use crate::game::{Player, Poi};

    #[test]
    fn visits_every_tile() {
        let map = Map::new(&Player::default());
        let side = map.tiles.len();
        assert_eq!(world_tiles(&map).count(), side * side);
    }

    #[test]
    fn origin_carries_the_village_poi() {
        let map = Map::new(&Player::default());
        let (_, tile) = world_tiles(&map)
            .find(|&(coords, _)| coords == (0, 0))
            .expect("a tile at the origin");
        assert_eq!(tile.poi, Some(Poi::Village));
    }

    #[cfg(feature = "gui")]
    #[test]
    fn tile_views_pairs_every_world_coord_with_its_terrain() {
        let map = Map::new(&Player::default());

        let from_views: Vec<_> = tile_views(&map).map(|t| (t.world, t.terrain)).collect();
        let from_world_tiles: Vec<_> = world_tiles(&map)
            .map(|(world, tile)| (world, tile.terrain_type))
            .collect();

        assert_eq!(from_views, from_world_tiles);
    }

    #[cfg(feature = "gui")]
    #[test]
    fn tile_views_reports_the_village_poi_at_the_origin() {
        let map = Map::new(&Player::default());

        let origin = tile_views(&map)
            .find(|t| t.world == (0, 0))
            .expect("a tile at the origin");

        assert_eq!(origin.poi, Some(Poi::Village));
    }

    #[cfg(feature = "gui")]
    #[test]
    fn tile_views_reports_each_tiles_four_neighbours() {
        use std::collections::HashMap;

        let map = Map::new(&Player::default());
        let by_coord: HashMap<(i32, i32), TerrainType> = world_tiles(&map)
            .map(|(world, tile)| (world, tile.terrain_type))
            .collect();

        let origin = tile_views(&map).find(|t| t.world == (0, 0)).unwrap();
        assert_eq!(
            origin.neighbours,
            [
                by_coord.get(&(0, 1)).copied(),
                by_coord.get(&(1, 0)).copied(),
                by_coord.get(&(0, -1)).copied(),
                by_coord.get(&(-1, 0)).copied(),
            ]
        );
    }

    #[cfg(feature = "gui")]
    #[test]
    fn tile_views_has_none_neighbours_past_the_map_edge() {
        let map = Map::new(&Player::default());
        let h = map.half;
        // The far (north-east) corner: only West and South are on the map.
        let corner = tile_views(&map).find(|t| t.world == (h, h)).unwrap();
        assert_eq!(corner.neighbours[0], None, "North off the map");
        assert_eq!(corner.neighbours[1], None, "East off the map");
        assert!(corner.neighbours[2].is_some(), "South on the map");
        assert!(corner.neighbours[3].is_some(), "West on the map");
    }

    #[cfg(feature = "gui")]
    #[test]
    fn terrain_rgb_is_distinct_per_terrain() {
        let colours = [
            terrain_rgb(TerrainType::Meadow),
            terrain_rgb(TerrainType::Forest),
            terrain_rgb(TerrainType::Deadland),
        ];

        for i in 0..colours.len() {
            for j in (i + 1)..colours.len() {
                assert_ne!(
                    colours[i], colours[j],
                    "terrains {i} and {j} share a colour"
                );
            }
        }
    }
}
