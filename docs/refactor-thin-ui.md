# Refactor: keep the UI thin — `viewmodel` layer

## Why

The ratatui UI is temporary; the app may later move to a different front-end
(e.g. web). Anything that is not pure rendering or input-mapping should live
outside `src/ui/` so a future UI can reuse it. Several helpers currently sit in
the UI: inventory sorting, "10 most recent events", tile→world coordinate math,
and the item-selection rules of the experiment overlay. Player-facing text
also depends on Rust's `{:?}` (Debug) formatting of `Item` / `TerrainType`.

## Outcome

A new module `src/viewmodel/` sits between `game` and `ui` (`ui` → `viewmodel`
→ `game`) and owns the presentation-agnostic helpers and transient interaction
state. `game` gains human-readable `Display` for `Item` and a name/symbol
split for `TerrainType`. The ratatui layer keeps only rendering, key-mapping,
and overlay orchestration.

## Scope

### Behavior changes (commit 1)

- `Item`: `impl Display` (`StoneAxe` → `"Stone Axe"`), derive `Ord`
  (declaration order) for stable inventory sorting.
- `TerrainType`: `Display` emits the name (`"Forest"`); new `symbol() -> char`
  for the map glyph. `MapTile: Display` and map rendering use `symbol()`.
- Game event strings use `{}` (Display) instead of `{:?}` for `Item` /
  `TerrainType`.
- Player pane shows the inventory as `"Stick 1, Stone 3"` (sorted, Display)
  instead of the `HashMap` Debug output.

### Structural move (commit 2)

- `Map::tile_to_world(x, y) -> (i32, i32)` — inverse of the private
  `world_to_tile`, used by `viewmodel::map`.
- `src/viewmodel/inventory.rs` — `sorted(&Player) -> Vec<(Item, u32)>`
- `src/viewmodel/events.rs` — `recent(&Game, count) -> impl Iterator<Item = &str>`
  (newest first)
- `src/viewmodel/map.rs` — `world_tiles(&Map) -> impl Iterator<Item = ((i32, i32), &MapTile)>`
- `src/viewmodel/selection.rs` — `ItemSelection`: the quantity/cap/available
  rules extracted from `ui::experiment::Experiment`
- `ui::experiment::Experiment` keeps cursor/focus/`handle_key`/`render` and
  delegates the item rules to `ItemSelection`.

## Out of scope

- The crafting popup and removing the `game.craft("Cord")` stopgap on the `c`
  key (TODO backlog).
- Renaming `game` internals; widening/narrowing field visibility beyond what the
  new layer needs.
- `Direction` input mapping — legitimately a UI concern.
