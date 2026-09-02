use rand::RngExt;
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

#[derive(Debug)]
enum Resource {
    Wood,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TerrainType {
    Meadow,
    Forest,
    Cave,
    Village,
    Deadland,
}

impl fmt::Display for TerrainType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let symbol = match self {
            TerrainType::Meadow => '𖧧',
            TerrainType::Forest => '𖠰',
            TerrainType::Cave => '🪨',
            TerrainType::Village => '🛖',
            TerrainType::Deadland => ' ',
        };
        write!(f, "{symbol}")
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
        //        let terrain_type = *RANDOM_TERRAIN_TYPES.choose(&mut rng).unwrap();
        let terrain_type = choose_weighted(RANDOM_TERRAIN_TYPES, &mut rng);

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
pub struct Map {
    pub tiles: Vec<Vec<MapTile>>,
    size: (u32, u32),
    pub boundary: (i32, i32, i32, i32), // (-max_x, max_x, -max_y, max_y)
}

impl Map {
    pub fn new(player: &Player) -> Map {
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
        Map {
            tiles: map,
            size: (size, size),
            boundary: (
                -(middle.0 as i32),
                middle.1 as i32,
                -(middle.0 as i32),
                middle.1 as i32,
            ),
        }
    }

    fn world_to_tile(&self, pos: (i32, i32)) -> (usize, usize) {
        let x = (pos.0 - self.boundary.0) as usize;
        let y = (pos.1 - self.boundary.2) as usize;

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

#[derive(Debug)]
pub struct Game {
    pub player: Player,
    pub map: Map,
    pub events: Vec<String>,
}

impl Default for Game {
    fn default() -> Game {
        let player = Player::default();
        let map = Map::new(&player);
        let events = vec!["You wake up.".to_string()];
        Game {
            player,
            map,
            events,
        }
    }
}

impl Game {
    pub fn walk_west(&mut self) {
        if self.player.coordinates.0 > self.map.boundary.0 {
            self.player.coordinates.0 -= 1;
            let current_tile = self.map.get_tile(self.player.coordinates);
            self.events.push(format!(
                "You walk west and visit {:?}.",
                current_tile.unwrap().terrain_type
            ));
        }
    }

    pub fn walk_east(&mut self) {
        if self.player.coordinates.0 < self.map.boundary.1 {
            self.player.coordinates.0 += 1;
            let current_tile = self.map.get_tile(self.player.coordinates);
            self.events.push(format!(
                "You walk east and visit {:?}.",
                current_tile.unwrap().terrain_type
            ));
        }
    }

    pub fn walk_north(&mut self) {
        if self.player.coordinates.1 < self.map.boundary.3 {
            self.player.coordinates.1 += 1;
            let current_tile = self.map.get_tile(self.player.coordinates);
            self.events.push(format!(
                "You walk north and visit {:?}.",
                current_tile.unwrap().terrain_type
            ));
        }
    }

    pub fn walk_south(&mut self) {
        if self.player.coordinates.1 > self.map.boundary.2 {
            self.player.coordinates.1 -= 1;
            let current_tile = self.map.get_tile(self.player.coordinates);
            self.events.push(format!(
                "You walk south and visit {:?}.",
                current_tile.unwrap().terrain_type
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_tile_corners() {
        let player = Player::default();
        let map = Map::new(&player);
        let (min_x, max_x, min_y, max_y) = map.boundary;
        assert_eq!(map.world_to_tile((min_x, min_y)), (0usize, 0usize));
        let size_x = map.size.0 as usize;
        let size_y = map.size.1 as usize;
        assert_eq!(map.world_to_tile((max_x, max_y)), (size_x - 1, size_y - 1));
    }

    #[test]
    fn world_to_tile_center_is_village() {
        let player = Player::default();
        let map = Map::new(&player);
        // center in world coords is (0,0)
        let center_idx = map.world_to_tile((0, 0));
        let (cx, cy) = center_idx;
        // ensure center tile is the village created at middle
        let tile = map.get_tile((0, 0)).expect("center tile exists");
        match tile.terrain_type {
            TerrainType::Village => (),
            other => panic!("expected Village at center, found {:?}", other),
        }
        // also ensure indices point to the middle
        let middle = ((map.size.0 / 2) as usize, (map.size.1 / 2) as usize);
        assert_eq!((cx, cy), middle);
    }

    #[test]
    fn world_to_tile_out_of_bounds() {
        let player = Player::default();
        let map = Map::new(&player);
        // pick a coordinate just outside the boundary to see mapping still returns index beyond size
        let (_min_x, max_x, _min_y, max_y) = map.boundary;
        let outside = (max_x + 1, max_y + 1);
        // get_tile should return None for outside positions
        assert!(map.get_tile(outside).is_none());
    }

    #[test]
    fn walk_west_moves_and_logs_and_respects_boundary() {
        let mut game = Game::default();
        let initial = game.player.coordinates;
        // move west once
        game.walk_west();
        assert_eq!(game.player.coordinates, (initial.0 - 1, initial.1));
        assert_eq!(game.events.len(), 2);
        let last = game.events.last().unwrap();
        assert!(last.starts_with("You walk west"));

        // set to left boundary and ensure no move
        let (min_x, _max_x, _min_y, _max_y) = game.map.boundary;
        game.player.coordinates = (min_x, 0);
        let before_events = game.events.len();
        game.walk_west();
        assert_eq!(game.player.coordinates.0, min_x);
        assert_eq!(game.events.len(), before_events);
    }

    #[test]
    fn walk_east_moves_and_logs_and_respects_boundary() {
        let mut game = Game::default();
        let initial = game.player.coordinates;
        game.walk_east();
        assert_eq!(game.player.coordinates, (initial.0 + 1, initial.1));
        assert_eq!(game.events.len(), 2);
        let last = game.events.last().unwrap();
        assert!(last.starts_with("You walk east"));

        // set to right boundary and ensure no move
        let (_min_x, max_x, _min_y, _max_y) = game.map.boundary;
        game.player.coordinates = (max_x, 0);
        let before_events = game.events.len();
        game.walk_east();
        assert_eq!(game.player.coordinates.0, max_x);
        assert_eq!(game.events.len(), before_events);
    }

    #[test]
    fn walk_north_moves_and_logs_and_respects_boundary() {
        let mut game = Game::default();
        let initial = game.player.coordinates;
        game.walk_north();
        assert_eq!(game.player.coordinates, (initial.0, initial.1 + 1));
        assert_eq!(game.events.len(), 2);
        let last = game.events.last().unwrap();
        assert!(last.starts_with("You walk north"));

        // set to top boundary and ensure no move
        let (_min_x, _max_x, _min_y, max_y) = game.map.boundary;
        game.player.coordinates = (0, max_y);
        let before_events = game.events.len();
        game.walk_north();
        assert_eq!(game.player.coordinates.1, max_y);
        assert_eq!(game.events.len(), before_events);
    }

    #[test]
    fn walk_south_moves_and_logs_and_respects_boundary() {
        let mut game = Game::default();
        let initial = game.player.coordinates;
        game.walk_south();
        assert_eq!(game.player.coordinates, (initial.0, initial.1 - 1));
        assert_eq!(game.events.len(), 2);
        let last = game.events.last().unwrap();
        assert!(last.starts_with("You walk south"));

        // set to bottom boundary and ensure no move
        let (_min_x, _max_x, min_y, _max_y) = game.map.boundary;
        game.player.coordinates = (0, min_y);
        let before_events = game.events.len();
        game.walk_south();
        assert_eq!(game.player.coordinates.1, min_y);
        assert_eq!(game.events.len(), before_events);
    }

    #[test]
    fn test_y_axis_inversion() {
        let mut game = Game::default();

        game.player.coordinates = (0, 0);
        let tile_coordinates = game.map.world_to_tile(game.player.coordinates);

        game.map.tiles[tile_coordinates.0 - 1][tile_coordinates.1] =
            MapTile::generate(TerrainType::Meadow);
        game.map.tiles[tile_coordinates.0 + 1][tile_coordinates.1] =
            MapTile::generate(TerrainType::Forest);

        game.walk_north();
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
}
