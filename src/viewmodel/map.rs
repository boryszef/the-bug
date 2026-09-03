//! The map prepared for display: every tile paired with its world coordinates.

use crate::game::{Map, MapTile};

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
}
