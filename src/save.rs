//! Persisting the game to a JSON file and loading it back.
//!
//! The on-disk shape is an explicit set of DTOs rather than the game structs, so
//! the file stays small and stable enough to hand-edit during development.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::game::{Event, EventCategory, Game, Map, Material, Player, TerrainType};

/// The game's semantic version, stamped into every save file.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Single-character codes for terrain in the save file, so a whole map row fits
/// on one line and the layout is easy to see and edit.
fn terrain_code(terrain: TerrainType) -> char {
    match terrain {
        TerrainType::Meadow => 'M',
        TerrainType::Forest => 'F',
        TerrainType::Cave => 'C',
        TerrainType::Village => 'V',
        TerrainType::Deadland => '.',
    }
}

fn terrain_from_code(code: char) -> Option<TerrainType> {
    match code {
        'M' => Some(TerrainType::Meadow),
        'F' => Some(TerrainType::Forest),
        'C' => Some(TerrainType::Cave),
        'V' => Some(TerrainType::Village),
        '.' => Some(TerrainType::Deadland),
        _ => None,
    }
}

#[derive(Serialize, Deserialize)]
struct SaveState {
    /// The game version that wrote this file. Absent in hand-made files.
    #[serde(default)]
    version: String,
    player: PlayerState,
    map: MapState,
    events: Vec<EventState>,
}

#[derive(Serialize, Deserialize)]
struct PlayerState {
    level: u32,
    coordinates: (i32, i32),
    inventory: HashMap<Material, u32>,
    recipes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct MapState {
    /// One string per map row, one character per tile (see `terrain_code`).
    /// The grid size is the map size.
    terrain: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct EventState {
    /// Absent in older / hand-made files; defaults to `General`.
    #[serde(default)]
    category: EventCategory,
    text: String,
    elapsed_secs: f64,
}

/// Writes the game to `the-bug-save-<unix-seconds>.json` in the current
/// directory and returns the path it wrote.
pub fn save(game: &Game) -> io::Result<PathBuf> {
    let json = serde_json::to_string_pretty(&capture(game)).map_err(io::Error::other)?;
    let path = PathBuf::from(save_filename(now_epoch()));
    std::fs::write(&path, json)?;
    Ok(path)
}

/// Loads a game from a JSON save file.
pub fn load(path: &Path) -> io::Result<Game> {
    let json = std::fs::read_to_string(path)?;
    let state: SaveState =
        serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if !state.version.is_empty() && state.version != VERSION {
        eprintln!(
            "note: save file was written by the-bug {}, running {VERSION}",
            state.version
        );
    }
    restore(state)
}

fn capture(game: &Game) -> SaveState {
    SaveState {
        version: VERSION.to_string(),
        player: PlayerState {
            level: game.player.level,
            coordinates: game.player.coordinates,
            inventory: game.player.inventory.clone(),
            recipes: game
                .player
                .known_recipes()
                .iter()
                .map(|recipe| recipe.name().to_string())
                .collect(),
        },
        map: MapState {
            terrain: game
                .map
                .tiles
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|tile| terrain_code(tile.terrain_type))
                        .collect()
                })
                .collect(),
        },
        events: game
            .events()
            .iter()
            .map(|event| EventState {
                category: event.category(),
                text: event.text().to_string(),
                elapsed_secs: event.elapsed().as_secs_f64(),
            })
            .collect(),
    }
}

fn restore(state: SaveState) -> io::Result<Game> {
    let map = Map::from_terrain(parse_terrain(&state.map.terrain)?);

    let mut player = Player::default();
    player.level = state.player.level;
    player.coordinates = state.player.coordinates;
    player.inventory = state.player.inventory;
    for name in &state.player.recipes {
        player.grant_recipe(name);
    }

    let events = state
        .events
        .into_iter()
        .map(|event| {
            Event::new(
                event.category,
                event.text,
                Duration::from_secs_f64(event.elapsed_secs.max(0.0)),
            )
        })
        .collect();

    Ok(Game::from_saved(player, map, events))
}

/// Parses the terrain rows into a grid, rejecting an empty / ragged grid or an
/// unknown terrain code.
fn parse_terrain(rows: &[String]) -> io::Result<Vec<Vec<TerrainType>>> {
    let width = rows.first().map_or(0, |row| row.chars().count());
    if width == 0 || rows.iter().any(|row| row.chars().count() != width) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "map terrain must be a non-empty grid of equal-length rows",
        ));
    }

    rows.iter()
        .map(|row| {
            row.chars()
                .map(|code| {
                    terrain_from_code(code).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("unknown terrain code {code:?}"),
                        )
                    })
                })
                .collect()
        })
        .collect()
}

fn save_filename(epoch: u64) -> String {
    format!("the-bug-save-{epoch}.json")
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(game: &Game) -> Game {
        let json = serde_json::to_string(&capture(game)).unwrap();
        restore(serde_json::from_str(&json).unwrap()).unwrap()
    }

    #[test]
    fn save_filename_is_json_with_epoch() {
        assert_eq!(save_filename(1788436884), "the-bug-save-1788436884.json");
    }

    #[test]
    fn capture_stamps_the_current_version() {
        assert_eq!(capture(&Game::default()).version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn default_game_round_trips() {
        let game = Game::default();
        let restored = roundtrip(&game);

        assert_eq!(restored.player.level, game.player.level);
        assert_eq!(restored.player.coordinates, game.player.coordinates);
        assert_eq!(restored.events().len(), game.events().len());
        assert_eq!(restored.map.tiles.len(), game.map.tiles.len());
        assert_eq!(restored.map.half, game.map.half);
    }

    #[test]
    fn played_game_state_survives_round_trip() {
        let mut game = Game::default();
        game.player.coordinates = (2, -1);
        game.player.inventory.insert(Material::Vine, 5);
        game.player.inventory.insert(Material::Stick, 1);
        game.player.grant_recipe("Cord");

        let restored = roundtrip(&game);

        assert_eq!(restored.player.coordinates, (2, -1));
        assert_eq!(restored.player.inventory.get(&Material::Vine), Some(&5));
        assert_eq!(restored.player.inventory.get(&Material::Stick), Some(&1));
        let recipes: Vec<&str> = restored
            .player
            .known_recipes()
            .iter()
            .map(|r| r.name())
            .collect();
        assert_eq!(recipes, ["Cord"]);
    }

    #[test]
    fn event_category_survives_round_trip() {
        let mut game = Game::default();
        game.player.inventory.insert(Material::Vine, 2);
        game.experiment(&[(Material::Vine, 2)]);

        let restored = roundtrip(&game);
        assert_eq!(
            restored.events().last().unwrap().category(),
            EventCategory::Experiment
        );
    }

    #[test]
    fn event_without_category_defaults_to_general() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["V"] },
            "events": [{ "text": "hi", "elapsed_secs": 0.0 }]
        }"#;
        let game = restore(serde_json::from_str(json).unwrap()).unwrap();
        assert_eq!(
            game.events().last().unwrap().category(),
            EventCategory::General
        );
    }

    #[test]
    fn unknown_recipe_names_are_skipped() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {},
                        "recipes": ["Cord", "Nonsense"] },
            "map": { "terrain": ["V"] },
            "events": []
        }"#;
        let game = restore(serde_json::from_str(json).unwrap()).unwrap();
        let recipes: Vec<&str> = game
            .player
            .known_recipes()
            .iter()
            .map(|r| r.name())
            .collect();
        assert_eq!(recipes, ["Cord"]);
    }

    #[test]
    fn ragged_terrain_grid_is_rejected() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["VF", "M"] },
            "events": []
        }"#;
        let err = restore(serde_json::from_str(json).unwrap()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn unknown_terrain_code_is_rejected() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["VX"] },
            "events": []
        }"#;
        let err = restore(serde_json::from_str(json).unwrap()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn terrain_codes_round_trip() {
        for terrain in [
            TerrainType::Meadow,
            TerrainType::Forest,
            TerrainType::Cave,
            TerrainType::Village,
            TerrainType::Deadland,
        ] {
            assert_eq!(terrain_from_code(terrain_code(terrain)), Some(terrain));
        }
    }

    #[test]
    fn load_rejects_malformed_json() {
        let mut path = std::env::temp_dir();
        path.push(format!("the-bug-test-bad-{}.json", now_epoch()));
        std::fs::write(&path, "{ not json").unwrap();

        let err = load(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_reads_a_hand_edited_file() {
        let json = r#"{
            "player": {
                "level": 3,
                "coordinates": [0, 0],
                "inventory": { "Vine": 9, "Stone": 2 },
                "recipes": ["Cord"]
            },
            "map": { "terrain": ["FMC", "M.M", "CMV"] },
            "events": [{ "text": "loaded", "elapsed_secs": 4.5 }]
        }"#;
        let mut path = std::env::temp_dir();
        path.push(format!("the-bug-test-edit-{}.json", now_epoch()));
        std::fs::write(&path, json).unwrap();

        let game = load(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(game.player.level, 3);
        assert_eq!(game.player.inventory.get(&Material::Vine), Some(&9));
        assert_eq!(game.map.tiles.len(), 3);
        assert_eq!(game.map.half, 1);
        assert_eq!(game.events().len(), 1);
        assert_eq!(game.events()[0].text(), "loaded");
    }
}
