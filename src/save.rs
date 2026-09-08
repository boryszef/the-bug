//! Persisting the game to a JSON file and loading it back.
//!
//! The on-disk shape is an explicit set of DTOs rather than the game structs, so
//! the file stays small and stable enough to hand-edit during development.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::game::{
    Event, EventKind, Game, Item, Map, Player, Poi, QuestID, RestoreState, SaveState, TerrainType,
};

/// The game's semantic version, stamped into every save file.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Single-character codes for terrain in the save file, so a whole map row fits
/// on one line and the layout is easy to see and edit. Used by `Map`'s
/// `SaveState`/`RestoreState` impls in `game.rs`.
pub(crate) fn terrain_code(terrain: TerrainType) -> char {
    match terrain {
        TerrainType::Meadow => 'M',
        TerrainType::Forest => 'F',
        TerrainType::Deadland => '.',
    }
}

fn terrain_from_code(code: char) -> Option<TerrainType> {
    match code {
        'M' => Some(TerrainType::Meadow),
        'F' => Some(TerrainType::Forest),
        '.' => Some(TerrainType::Deadland),
        _ => None,
    }
}

/// Single-character codes for a tile's point of interest — `.` (or a space) for
/// none — kept in a grid parallel to `terrain` in the save file.
pub(crate) fn poi_code(poi: Option<Poi>) -> char {
    match poi {
        None => '.',
        Some(Poi::Cave) => 'c',
        Some(Poi::Ruins) => 'r',
        Some(Poi::Village) => 'v',
    }
}

fn poi_from_code(code: char) -> Option<Option<Poi>> {
    match code {
        '.' | ' ' => Some(None),
        'c' => Some(Some(Poi::Cave)),
        'r' => Some(Some(Poi::Ruins)),
        'v' => Some(Some(Poi::Village)),
        _ => None,
    }
}

/// The whole on-disk save shape: version stamp plus each sub-type's own saved
/// shape (see `Player`/`Map`/`Event`'s `SaveState`/`RestoreState` impls in
/// `game.rs`).
#[derive(Serialize, Deserialize)]
struct SaveFile {
    /// The game version that wrote this file. Absent in hand-made files.
    #[serde(default)]
    version: String,
    /// Total game time when the file was written, in seconds. Restored
    /// directly so the session clock doesn't snap back to the last event.
    /// Absent in older / hand-made saves — then it's inferred from the
    /// events, as it always was.
    #[serde(default)]
    elapsed_secs: f64,
    player: PlayerState,
    map: MapState,
    events: Vec<EventState>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PlayerState {
    pub(crate) level: u32,
    #[serde(default)]
    pub(crate) experience: u32,
    #[serde(default)]
    pub(crate) crafts_completed: u32,
    pub(crate) coordinates: (i32, i32),
    pub(crate) inventory: HashMap<Item, u32>,
    /// The bag's contents. Absent in older / hand-made saves — then the
    /// player starts with an empty bag.
    #[serde(default)]
    pub(crate) bag: HashMap<Item, u32>,
    pub(crate) recipes: Vec<String>,
    #[serde(default)]
    pub(crate) open_quest: Option<QuestID>,
    #[serde(default)]
    pub(crate) quest_progress: u32,
    #[serde(default)]
    pub(crate) quests_completed: Vec<QuestID>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct MapState {
    /// One string per map row, one character per tile (see `terrain_code`).
    /// The grid size is the map size.
    pub(crate) terrain: Vec<String>,
    /// Points of interest, a grid parallel to `terrain` (see `poi_code`).
    /// Absent in older / hand-made saves — then every tile has no POI.
    #[serde(default)]
    pub(crate) pois: Vec<String>,
}

/// The on-disk shape of one event-log entry. Old save files' `events` array
/// (which stored pre-rendered `{ category, text }`) is not compatible with
/// this shape by design — see docs/i18n-plan.md — so an old save fails to
/// load with `io::ErrorKind::InvalidData` rather than being migrated.
#[derive(Serialize, Deserialize)]
pub(crate) struct EventState {
    pub(crate) kind: EventKind,
    pub(crate) elapsed_secs: f64,
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
    let state: SaveFile =
        serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if !state.version.is_empty() && state.version != VERSION {
        eprintln!(
            "note: save file was written by the-bug {}, running {VERSION}",
            state.version
        );
    }
    restore(state)
}

fn capture(game: &Game) -> SaveFile {
    SaveFile {
        version: VERSION.to_string(),
        elapsed_secs: game.elapsed().as_secs_f64(),
        player: game.player.save_state(),
        map: game.map.save_state(),
        events: game.events().iter().map(Event::save_state).collect(),
    }
}

fn restore(state: SaveFile) -> io::Result<Game> {
    let map = Map::restore_state(state.map)?;
    let player = Player::restore_state(state.player)?;
    let events = state
        .events
        .into_iter()
        .map(Event::restore_state)
        .collect::<io::Result<Vec<_>>>()?;

    // The stored game time is the source of truth; fall back to the last
    // event for saves written before it existed (`elapsed_secs` == 0). A real
    // save always has `elapsed_secs` >= every event, so the max picks it.
    let last_event = events
        .iter()
        .map(Event::elapsed)
        .max()
        .unwrap_or(Duration::ZERO);
    let elapsed = Duration::from_secs_f64(state.elapsed_secs.max(0.0)).max(last_event);

    Ok(Game::from_saved(player, map, events, elapsed))
}

/// Parses the terrain rows into a grid, rejecting an empty / ragged grid or an
/// unknown terrain code. Used by `Map::restore_state` in `game.rs`.
pub(crate) fn parse_terrain(rows: &[String]) -> io::Result<Vec<Vec<TerrainType>>> {
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

/// Parses the POI rows into a grid parallel to a `height` × `width` terrain
/// grid. An empty `rows` (older / hand-made save) yields an all-`None` grid of
/// that size; otherwise the grid must match those dimensions and use known
/// codes. Used by `Map::restore_state` in `game.rs`.
pub(crate) fn parse_pois(
    rows: &[String],
    height: usize,
    width: usize,
) -> io::Result<Vec<Vec<Option<Poi>>>> {
    if rows.is_empty() {
        return Ok(vec![vec![None; width]; height]);
    }
    if rows.len() != height || rows.iter().any(|row| row.chars().count() != width) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "map pois grid must match the terrain grid's dimensions",
        ));
    }

    rows.iter()
        .map(|row| {
            row.chars()
                .map(|code| {
                    poi_from_code(code).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("unknown poi code {code:?}"),
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
        game.player.inventory.insert(Item::Vine, 5);
        game.player.inventory.insert(Item::Branch, 1);
        game.player.grant_recipe("Cord");

        let restored = roundtrip(&game);

        assert_eq!(restored.player.coordinates, (2, -1));
        assert_eq!(restored.player.inventory.get(&Item::Vine), Some(&5));
        assert_eq!(restored.player.inventory.get(&Item::Branch), Some(&1));
        let recipes: Vec<&str> = restored
            .player
            .known_recipes()
            .iter()
            .map(|r| r.name())
            .collect();
        assert_eq!(recipes, ["Cord"]);
    }

    #[test]
    fn bag_survives_round_trip() {
        let mut game = Game::default();
        game.player.bag.insert(Item::Vine, 4);
        game.player.bag.insert(Item::Branch, 1);

        let restored = roundtrip(&game);

        assert_eq!(restored.player.bag.get(&Item::Vine), Some(&4));
        assert_eq!(restored.player.bag.get(&Item::Branch), Some(&1));
    }

    #[test]
    fn player_without_a_bag_field_defaults_to_an_empty_bag() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["M"] },
            "events": []
        }"#;
        let game = restore(serde_json::from_str(json).unwrap()).unwrap();
        assert!(game.player.bag.is_empty());
    }

    /// `game.elapsed()` should be within `secs ± 2` of the target — a
    /// round-trip / load is near-instant, so the clock barely moves.
    fn assert_elapsed_near(game: &Game, secs: f64) {
        let got = game.elapsed().as_secs_f64();
        assert!(
            (got - secs).abs() < 2.0,
            "game.elapsed() = {got:.1}s, expected ~{secs:.1}s"
        );
    }

    #[test]
    fn game_time_is_restored_from_the_save_not_inferred_from_the_events() {
        let json = r#"{
            "elapsed_secs": 3600.0,
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["M"] },
            "events": [{ "kind": "Awoke", "elapsed_secs": 0.0 }]
        }"#;
        let game = restore(serde_json::from_str(json).unwrap()).unwrap();
        assert_elapsed_near(&game, 3600.0);
    }

    #[test]
    fn a_save_without_a_game_time_falls_back_to_the_last_event() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["M"] },
            "events": [{ "kind": "Awoke", "elapsed_secs": 4.5 }]
        }"#;
        let game = restore(serde_json::from_str(json).unwrap()).unwrap();
        assert_elapsed_near(&game, 4.5);
    }

    #[test]
    fn game_time_survives_a_round_trip() {
        let fresh = Game::default();
        let game = Game::from_saved(
            fresh.player,
            fresh.map,
            Vec::new(),
            Duration::from_secs(1000),
        );

        let restored = roundtrip(&game);

        assert_elapsed_near(&restored, 1000.0);
    }

    #[test]
    fn event_kind_survives_round_trip() {
        let mut game = Game::default();
        game.player.inventory.insert(Item::Vine, 2);
        game.experiment(&[(Item::Vine, 2)]);

        let restored = roundtrip(&game);
        let kind = restored.events().last().unwrap().kind();
        assert_eq!(
            kind,
            &EventKind::Experimented {
                items: vec![(Item::Vine, 2)],
                output: Item::Cord,
                newly_learned: true,
            }
        );
    }

    #[test]
    fn old_format_save_fails_to_load() {
        // Pre-`EventKind` save files stored pre-rendered `{ category, text }`
        // events; that shape is deliberately not migrated (docs/i18n-plan.md).
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["M"] },
            "events": [{ "category": "General", "text": "hi", "elapsed_secs": 0.0 }]
        }"#;
        let mut path = std::env::temp_dir();
        path.push(format!("the-bug-test-old-format-{}.json", now_epoch()));
        std::fs::write(&path, json).unwrap();

        let err = load(&path).unwrap_err();
        std::fs::remove_file(&path).ok();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn experience_and_craft_count_survive_round_trip() {
        let mut game = Game::default();
        game.player.experience = 37;
        game.player.crafts_completed = 4;

        let restored = roundtrip(&game);
        assert_eq!(restored.player.experience, 37);
        assert_eq!(restored.player.crafts_completed, 4);
    }

    #[test]
    fn player_without_experience_fields_defaults_to_zero() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["M"] },
            "events": []
        }"#;
        let game = restore(serde_json::from_str(json).unwrap()).unwrap();
        assert_eq!(game.player.experience, 0);
        assert_eq!(game.player.crafts_completed, 0);
    }

    #[test]
    fn unknown_recipe_names_are_skipped() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {},
                        "recipes": ["Cord", "Nonsense"] },
            "map": { "terrain": ["M"] },
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
    fn open_quest_and_progress_survive_round_trip() {
        let mut game = Game::default();
        game.player
            .restore_quest_state(Some(QuestID::CraftAxe), 3, vec![]);

        let restored = roundtrip(&game);

        assert_eq!(restored.player.open_quest(), Some(QuestID::CraftAxe));
        assert_eq!(restored.player.quest_progress(), 3);
    }

    #[test]
    fn completed_quests_survive_round_trip() {
        let mut game = Game::default();
        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftAxe, QuestID::ExploreRuins]);

        let restored = roundtrip(&game);

        assert_eq!(
            restored.player.completed_quests(),
            [QuestID::CraftAxe, QuestID::ExploreRuins]
        );
    }

    #[test]
    fn player_without_quest_fields_defaults_to_no_quests() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["M"] },
            "events": []
        }"#;
        let game = restore(serde_json::from_str(json).unwrap()).unwrap();
        assert_eq!(game.player.open_quest(), None);
        assert_eq!(game.player.quest_progress(), 0);
        assert!(game.player.completed_quests().is_empty());
    }

    #[test]
    fn ragged_terrain_grid_is_rejected() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["MF", "M"] },
            "events": []
        }"#;
        let err = restore(serde_json::from_str(json).unwrap()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn unknown_terrain_code_is_rejected() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["MX"] },
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
            TerrainType::Deadland,
        ] {
            assert_eq!(terrain_from_code(terrain_code(terrain)), Some(terrain));
        }
    }

    #[test]
    fn poi_codes_round_trip() {
        for poi in [None, Some(Poi::Cave), Some(Poi::Ruins), Some(Poi::Village)] {
            assert_eq!(poi_from_code(poi_code(poi)), Some(poi));
        }
    }

    #[test]
    fn pois_survive_a_full_save_round_trip() {
        let mut game = Game::default();
        let side = game.map.tiles.len();
        for tile in game.map.tiles.iter_mut().flatten() {
            tile.poi = None;
        }
        game.map.tiles[0][0].poi = Some(Poi::Cave);
        game.map.tiles[1][2].poi = Some(Poi::Ruins);
        game.map.tiles[side - 1][side - 1].poi = Some(Poi::Village);

        let restored = roundtrip(&game);

        assert_eq!(restored.map.tiles[0][0].poi, Some(Poi::Cave));
        assert_eq!(restored.map.tiles[1][2].poi, Some(Poi::Ruins));
        assert_eq!(
            restored.map.tiles[side - 1][side - 1].poi,
            Some(Poi::Village)
        );
        assert_eq!(restored.map.tiles[0][1].poi, None);
    }

    #[test]
    fn save_without_a_pois_grid_loads_with_no_pois() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["FMF", "M.M", "FMM"] },
            "events": []
        }"#;
        let game = restore(serde_json::from_str(json).unwrap()).unwrap();
        assert!(game.map.tiles.iter().flatten().all(|t| t.poi.is_none()));
    }

    #[test]
    fn load_reads_a_hand_edited_pois_grid() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": {
                "terrain": ["FMF", "M.M", "FM."],
                "pois":    ["c..", "...", "..v"]
            },
            "events": []
        }"#;
        let game = restore(serde_json::from_str(json).unwrap()).unwrap();
        assert_eq!(game.map.tiles[0][0].poi, Some(Poi::Cave));
        assert_eq!(game.map.tiles[2][2].poi, Some(Poi::Village));
        assert_eq!(game.map.tiles[1][1].poi, None);
    }

    #[test]
    fn a_pois_grid_that_does_not_match_the_terrain_is_rejected() {
        let json = r#"{
            "player": { "level": 1, "coordinates": [0, 0], "inventory": {}, "recipes": [] },
            "map": { "terrain": ["FMF", "M.M", "FMM"], "pois": ["c.", "..", ".."] },
            "events": []
        }"#;
        let err = restore(serde_json::from_str(json).unwrap()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
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
            "map": {
                "terrain": ["FMF", "M.M", "FMM"],
                "pois":    ["c..", "...", "..r"]
            },
            "events": [{ "kind": "Awoke", "elapsed_secs": 4.5 }]
        }"#;
        let mut path = std::env::temp_dir();
        path.push(format!("the-bug-test-edit-{}.json", now_epoch()));
        std::fs::write(&path, json).unwrap();

        let game = load(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(game.player.level, 3);
        assert_eq!(game.player.inventory.get(&Item::Vine), Some(&9));
        assert_eq!(game.map.tiles.len(), 3);
        assert_eq!(game.map.half, 1);
        assert_eq!(game.events().len(), 1);
        assert_eq!(game.events()[0].kind(), &EventKind::Awoke);
    }
}
