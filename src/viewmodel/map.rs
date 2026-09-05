//! The map prepared for display: every tile paired with its world coordinates.

#[cfg(feature = "gui")]
use crate::game::TerrainType;
use crate::game::{Map, MapTile};

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
}

/// Every tile as a [`TileView`]. Thin adapter over [`world_tiles`] for now;
/// the indirection is what lets both a procedural and a sprite renderer
/// share one descriptor.
#[cfg(feature = "gui")]
pub fn tile_views(map: &Map) -> impl Iterator<Item = TileView> + '_ {
    world_tiles(map).map(|(world, tile)| TileView {
        world,
        terrain: tile.terrain_type,
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
        TerrainType::Cave => (0x6b, 0x6b, 0x6b),
        TerrainType::Ruins => (0x9a, 0x8a, 0x74),
        TerrainType::Village => (0xc8, 0x8a, 0x3c),
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
    use crate::game::{Player, TerrainType};

    #[test]
    fn visits_every_tile() {
        let map = Map::new(&Player::default());
        let side = map.tiles.len();
        assert_eq!(world_tiles(&map).count(), side * side);
    }

    #[test]
    fn origin_maps_to_the_central_village() {
        let map = Map::new(&Player::default());
        let (_, tile) = world_tiles(&map)
            .find(|&(coords, _)| coords == (0, 0))
            .expect("a tile at the origin");
        assert_eq!(tile.terrain_type, TerrainType::Village);
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
    fn tile_views_reports_the_central_village_at_the_origin() {
        let map = Map::new(&Player::default());

        let origin = tile_views(&map)
            .find(|t| t.world == (0, 0))
            .expect("a tile at the origin");

        assert_eq!(origin.terrain, TerrainType::Village);
    }

    #[cfg(feature = "gui")]
    #[test]
    fn terrain_rgb_is_distinct_per_terrain() {
        let colours = [
            terrain_rgb(TerrainType::Meadow),
            terrain_rgb(TerrainType::Forest),
            terrain_rgb(TerrainType::Cave),
            terrain_rgb(TerrainType::Ruins),
            terrain_rgb(TerrainType::Village),
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
