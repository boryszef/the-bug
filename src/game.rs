use rand::prelude::IndexedRandom;
use std::fmt;

const MAP_MIN_SIZE: u32 = 15;
const MAP_PER_LEVEL_INCREMENT: u32 = 2;

#[derive(Debug)]
pub struct Player {
    pub level: u32,
    pub coordinates: (i32, i32),
    resources: Vec<Resource>,
}

impl Default for Player {
    fn default() -> Player {
        Player {
            level: 1,
            coordinates: (0, 0),
            resources: Vec::new(),
        }
    }
}

impl Player {
    pub fn walk_west(&mut self, boundaries: (i32, i32, i32, i32)) {
        if self.coordinates.0 > boundaries.0 {
            self.coordinates.0 -= 1;
        }
    }

    pub fn walk_east(&mut self, boundaries: (i32, i32, i32, i32)) {
        if self.coordinates.0 < boundaries.1 {
            self.coordinates.0 += 1;
        }
    }

    pub fn walk_north(&mut self, boundaries: (i32, i32, i32, i32)) {
        if self.coordinates.1 < boundaries.3 {
            self.coordinates.1 += 1;
        }
    }

    pub fn walk_south(&mut self, boundaries: (i32, i32, i32, i32)) {
        if self.coordinates.1 > boundaries.2 {
            self.coordinates.1 -= 1;
        }
    }
}

#[derive(Debug)]
enum Resource {
    Wood,
}

#[derive(Clone, Copy, Debug)]
pub enum TerrainType {
    Meadow,
    Forest,
    Village,
    Deadland,
}

impl fmt::Display for TerrainType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let symbol = match self {
            TerrainType::Meadow => '𖧧',
            TerrainType::Forest => '𖠰',
            TerrainType::Village => '🛖',
            TerrainType::Deadland => ' ',
        };
        write!(f, "{symbol}")
    }
}

const TERRAIN_TYPES: &[TerrainType] = &[
    TerrainType::Meadow,
    TerrainType::Forest,
    TerrainType::Deadland,
];

#[derive(Debug)]
pub struct MapTile {
    pub terrain_type: TerrainType,
    resources: Vec<Resource>,
}

impl fmt::Display for MapTile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let terrain = self.terrain_type;
        write!(f, "{terrain}")
    }
}

impl MapTile {
    fn new() -> MapTile {
        let mut rng = rand::rng();
        let terrain_type = *TERRAIN_TYPES.choose(&mut rng).unwrap();

        MapTile {
            terrain_type,
            resources: Vec::new(),
        }
    }

    fn generate(terrain_type: TerrainType) -> MapTile {
        MapTile {
            terrain_type,
            resources: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct RegionMap {
    pub tiles: Vec<Vec<MapTile>>,
    pub size: (u32, u32),
    pub boundaries: (i32, i32, i32, i32), // (-max_x, max_x, -max_y, max_y)
}

impl RegionMap {
    pub fn new(player: &Player) -> RegionMap {
        let size = MAP_MIN_SIZE + player.level * MAP_PER_LEVEL_INCREMENT;
        let mut map = Vec::new();
        let middle = (size / 2, size / 2);
        for x in 0..size {
            let mut row = Vec::new();
            for y in 0..size {
                let tile = match (x, y) {
                    pos if pos == middle => MapTile::generate(TerrainType::Village),
                    _ => MapTile::new(),
                };
                row.push(tile);
            }
            map.push(row);
        }
        RegionMap {
            tiles: map,
            size: (size, size),
            boundaries: (
                -(middle.0 as i32),
                middle.1 as i32,
                -(middle.0 as i32),
                middle.1 as i32,
            ),
        }
    }

    fn world_to_tile(&self, pos: (i32, i32)) -> (usize, usize) {
        let x = (pos.0 - self.boundaries.0) as usize;
        let y = (pos.1 - self.boundaries.2) as usize;

        (x, y)
    }

    pub fn get_tile(&self, pos: (i32, i32)) -> Option<&MapTile> {
        let (x, y) = self.world_to_tile(pos);
        if x < self.size.0 as usize && y < self.size.1 as usize {
            Some(&self.tiles[y][x])
        } else {
            None
        }
    }
}
