# Feature: save on quit / load on startup

## Why

There was no persistence — every run started from `Game::default()`. This adds a
JSON save written on quit and an optional `--load` on startup, which also serves
as a development knob: hand-edit a save file to set up a scenario, then load it.

## Behaviour

- **On quit** (`q`): the game is written to `the-bug-save-<unix-seconds>.json`
  in the current directory. After the TUI exits the path is printed:
  `Game saved to the-bug-save-1788436884.json`. A save failure prints a warning
  but does not fail the process (the game already ran).
- **On startup**: `the-bug --load <file>` resumes from that file instead of a
  new game. A missing or invalid file is a hard error (exit 1, no TUI).
- `the-bug --help` / `--version` come from clap.

## What is saved

`src/save.rs` owns an explicit DTO schema (not the game structs) so the file
stays small and editable:

```json
{
  "version": "0.1.0",
  "player": {
    "level": 1,
    "coordinates": [0, 1],
    "inventory": { "Vine": 3, "Stick": 1 },
    "recipes": ["Cord"]
  },
  "map": {
    "terrain": [
      "F..F....M.F.M.MMM",
      "M.M..F.FMMMM..M.M",
      "...",
      "MM.MM.M.V.M.MM..F"
    ]
  },
  "events": [
    { "text": "You wake up and decide to go for a walk.", "elapsed_secs": 0.0 }
  ]
}
```

**Terrain codes** (one character per tile, one string per map row):

| code | terrain  |
|------|----------|
| `M`  | Meadow   |
| `F`  | Forest   |
| `C`  | Cave     |
| `V`  | Village  |
| `.`  | Deadland |

The grid's dimensions define the map size (`half = rows / 2`). Rows must all be
the same length; an unknown code or a ragged grid is rejected.

`version` is the game's semantic version (`CARGO_PKG_VERSION`) at save time.
Loading a file written by a different version prints a note but still loads;
hand-made files may omit the field. There is no migration logic yet.

## What is *not* saved (reset on load)

- **Per-tile search cooldown** (`MapTile.last_search_time`) — a transient ~60 s
  decay timer that needs a wall clock and expires across any real gap. Every
  tile loads as "not recently searched".
- **Tile item tables** (`MapTile.items`) — recomputed from the terrain, so
  editing a tile's terrain code also changes what can be found there.
- Unknown recipe names in `recipes` are silently skipped.

## Editing tips

- Bump `level`, add `"inventory": { "Cord": 5 }`, add `"Stone Axe"` to
  `recipes` to jump ahead.
- Redraw the `terrain` grid to build a specific map; put `V` somewhere for the
  village.
- `events` can be trimmed to `[]` or a single line for a clean log.

## Code

- `src/save.rs` — DTOs, `capture` / `restore`, `save` / `load`, terrain codes.
- `src/game.rs` — `Map::from_terrain`, `Player::grant_recipe`,
  `Game::from_saved`, `Event::new` (all `pub(crate)`).
- `src/main.rs` — clap `Cli { load: Option<PathBuf> }`; wraps `App::with_game`
  and the save-on-exit.
- `src/ui/app.rs` — `App::with_game` / `App::game`.
