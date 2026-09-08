# Feature: save on quit / load on startup

## Why

There was no persistence — every run started from `Game::default()`. This adds a
JSON save written on quit and an optional `--load` on startup, which also serves
as a development knob: hand-edit a save file to set up a scenario, then load it.

## Behaviour

- **On quit** (`q`, or closing the window): the game is written to
  `the-bug-save-<unix-seconds>.json` in the current directory, and the path
  is printed: `Game saved to the-bug-save-1788436884.json`. A save failure
  prints a warning but does not fail the process (the game already ran).
- **On startup**: `the-bug --load <file>` resumes from that file instead of a
  new game. A missing or invalid file is a hard error (exit 1).
- `the-bug --help` / `--version` come from clap.

## What is saved

`src/save.rs` owns an explicit DTO schema (not the game structs) so the file
stays small and editable:

```json
{
  "version": "0.1.0",
  "elapsed_secs": 512.4,
  "player": {
    "level": 1,
    "coordinates": [0, 1],
    "inventory": { "Vine": 3, "Branch": 1 },
    "recipes": ["Cord"]
  },
  "map": {
    "terrain": [
      "F..F....M.F.M.MMM",
      "M.M..F.FMMMM..M.M",
      "...",
      "MM.MM.M.M.M.MM..F"
    ],
    "pois": [
      "................",
      ".....c..........",
      "...",
      "........v......r"
    ]
  },
  "events": [
    { "text": "You wake up and decide to go for a walk.", "elapsed_secs": 0.0 }
  ]
}
```

**Terrain codes** — the `terrain` grid, one character per tile, one string per
row:

| code | terrain  |
|------|----------|
| `M`  | Meadow   |
| `F`  | Forest   |
| `.`  | Deadland |

**POI codes** — the optional `pois` grid, same shape, one point of interest per
tile (`docs/map-pois.md`). Absent (or omitted) means no POIs anywhere.

| code       | POI     |
|------------|---------|
| `.` or ` ` | none    |
| `c`        | Cave    |
| `r`        | Ruins   |
| `v`        | Village |

The `terrain` grid's dimensions define the map size (`half = rows / 2`); the
`pois` grid, if present, must match them. Rows must all be the same length; an
unknown code or a ragged grid is rejected.

Caves, ruins and the village used to be terrain codes `C` / `R` / `V`; a save
that still uses them no longer loads (`docs/map-pois.md` — clean break, no
migration). The village is now a `v` in the `pois` grid.

`version` is the game's semantic version (`CARGO_PKG_VERSION`) at save time.
Loading a file written by a different version prints a note but still loads;
hand-made files may omit the field. There is no migration logic yet.

`elapsed_secs` is the total game time (session-relative, carried across
save/load) when the file was written — `Game::elapsed()`. It's restored
directly, so reloading no longer snaps the session clock back to the last
logged event (walking, or just sitting idle, isn't an event, and that time
used to be lost). A save without the field — older or hand-made — falls back
to the old behaviour: the time is taken from the latest event. A real save
always has `elapsed_secs` at least as large as every event, so the loader
uses `max(elapsed_secs, latest event)` and both cases fall out of that.

## What is *not* saved (reset on load)

- **Per-tile search cooldown** (`MapTile.last_search_time`) — a transient ~60 s
  decay timer that needs a wall clock and expires across any real gap. Every
  tile loads as "not recently searched".
- **Tile item tables** (`MapTile.items`) — recomputed from the terrain and POI,
  so editing either grid also changes what can be found there.
- Unknown recipe names in `recipes` are silently skipped.

## Editing tips

- Bump `level`, add `"inventory": { "Cord": 5 }`, add `"Stone Axe"` to
  `recipes` to jump ahead.
- Redraw the `terrain` grid to build a specific map; add a `pois` grid (same
  size) with a `v` for the village (put one somewhere — the game expects it)
  and `c` / `r` for caves / ruins.
- `events` can be trimmed to `[]` or a single line for a clean log.

## Code

- `src/save.rs` — DTOs, `capture` / `restore`, `save` / `load`, terrain/POI
  codes, the `elapsed_secs` reconciliation.
- `src/game/mod.rs` — `Game::from_saved` (takes the restored elapsed),
  `Game::elapsed` (both `pub(crate)`). `Map::from_terrain` /
  `Player::grant_recipe` are `pub` (also used by the functional tests);
  `Player::restore_quest_state` / `Event::new` are `pub(crate)`.
- `src/main.rs` — clap `Cli { load: Option<PathBuf> }`; on `--load` calls
  `save::load`, otherwise `Game::default()`.
- `src/gui/mod.rs` — `App::on_exit` writes the save on window close.
