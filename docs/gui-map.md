# Feature: graphical tile map in the egui front end

## Why

`docs/adr/0001-ui-framework-egui.md` picked egui specifically so the map
could render as "a custom `egui::Painter`-based tile grid, not a built-in
widget", replacing `TerrainType::symbol()`'s unicode glyphs. `docs/gui-frontend.md`
lists "map via `egui::Painter`" as the next unbuilt piece — the `gui` Map tab
currently shows only a placeholder heading (`src/gui/mod.rs`), while `tui`
draws glyphs on a ratatui `Canvas` (`src/tui/app.rs`).

The longer-term aim is a **predefined** (non-random) map with tile kinds that
connect on shared edges — roads and rivers, in shapes like straight / turn /
T-junction / crossroads / dead-end. This feature is the **first increment
only**: a graphical renderer for the map that already exists, shaped so the
predefined-map and road/river work land on top without reworking it.

## Scope decisions

- **Render technique — hybrid.** Draw procedurally now (`Painter` filled
  rectangles), no image assets. (POI *marks* became raster `painter.image`
  later — ADR 0001 amendment, `docs/icons.md`; terrain fills stay procedural.)
  Route tile data through a
  presentation-agnostic descriptor (`viewmodel::map::TileView`) so a
  sprite/texture backend can replace the drawing code later without touching
  `game/` or the (future) connection logic.
- **This increment — rendering only.** No changes to `game/`, the save
  format, or random map generation.
- **Feature model (future increment, not here) — overlay field.** Roads and
  rivers will be an `Option<Feature>` on `MapTile`; the connection *shape* is
  always derived from neighbours, never stored, so a predefined map can't be
  internally inconsistent.

## Behaviour

- The **Map** tab's central area shows the map as a grid of coloured
  squares, one per tile, colour determined by terrain type.
- The player's position is marked with a distinct icon drawn on top of its
  tile (originally a filled circle; see `docs/icons.md`).
- **Pan:** dragging inside the map area moves the view.
- **Zoom:** scroll wheel / pinch changes tile size, clamped to a min and max
  so the map can't vanish or fill the screen with one tile.
- On first launch the view is centred on the origin tile (the central
  village) at a default zoom.
- Only tiles that fall within the visible area are drawn.
- The other four tabs are unchanged (still placeholder headings); `[` `]`
  panel cycling and `q` quit are unchanged.
- The `tui` map is untouched.

## Design

### `src/viewmodel/map.rs` — tile descriptor + palette

- `TileView { world: (i32, i32), terrain: TerrainType }` — the per-tile
  data a renderer needs. Deliberately a plain owned struct, not `&MapTile`,
  so `feature` / `connections` fields can be added later for both the
  procedural and a sprite backend.
- `tile_views(&Map) -> impl Iterator<Item = TileView>` — for now a thin
  adapter over the existing `world_tiles` (`src/viewmodel/map.rs`), which
  already centralises the `tiles[y][x]` → world-coordinate transform.
- `terrain_rgb(TerrainType) -> (u8, u8, u8)` — the colour palette, kept as
  raw RGB (no `egui::Color32`) so it stays UI-agnostic, mirroring how
  `TerrainType::symbol()` centralises the glyph. `gui` converts via
  `Color32::from_rgb`.

### `src/gui/map.rs` — new module (transient view state + rendering)

Follows the shape `ARCHITECTURE.md` asks `gui` panels to mirror from `tui`:
a small struct holding only transient UI state, no game data.

- `MapView { center: egui::Vec2, tile_px: f32 }` — `center` is the world
  coordinate under the middle of the viewport; `tile_px` is the pixel size
  of one tile (zoom). Centres on `(0, 0)` at a fixed default `tile_px`.
  (Later gained a `PoiIcons` field and a `player_icon` texture, so
  construction moved from `Default` to `MapView::new(&egui::Context)` — the
  icon textures upload there. See `docs/icons.md`.)
- `MapView::ui(&mut self, ui, tiles: impl Iterator<Item = TileView>, player: (i32, i32))`
  — allocates a painter over the available area, applies drag to `center`
  and scroll/zoom to `tile_px` (clamped), culls to the visible tile range,
  draws one `rect_filled` per visible tile, then the player's icon.
- Pure helpers, unit-tested (the `ui` method itself isn't — rendering has no
  assertable output here, matching `ARCHITECTURE.md`'s "UI changes" note):
  - `world_to_screen` — the tile-to-pixel transform, with the Y flip (world
    Y increases northward: `Direction::North => (0, 1)`). No inverse is
    added until a consumer needs one (e.g. click-to-inspect).
  - `visible_tiles(viewport, center, tile_px)` — the inclusive world-coord
    range per axis that can appear.
  - `clamp_tile_px` — zoom clamped to `[MIN_TILE_PX, MAX_TILE_PX]`.

### `src/gui/mod.rs` — wiring

- `mod map;`, `map_view: MapView` field on `App`.
- The central panel's `Panel::Map` arm calls `self.map_view.ui(...)` with
  `viewmodel::map::tile_views(&self.game.map)` and
  `self.game.player.coordinates`; the other arms keep the placeholder
  heading.

## What is *not* built here

- No predefined map — generation is still random (`Map::new`).
- No road / river tiles, no `Feature` type, no edge-connection logic, no
  save-format change.
- No sprite/texture rendering — procedural fills only.
- No terrain legend, no per-tile tooltips or click-to-inspect. (Keyboard
  movement on the Map tab landed as a separate follow-up — see
  `docs/gui-frontend.md`.)
- `tui`'s map renderer is not changed or refactored.

## Follow-on increments (tracked, not in this change)

1. ~~**Predefined map:** load a bundled ASCII grid instead of the random
   fill.~~ Superseded: the map is still random, but is now built by the
   in-crate clustering generator `src/mapgen/` (called from `Map::new`),
   so terrain comes out in contiguous regions rather than per-tile noise —
   see `docs/mapgen.md`. A hand-authored fixed grid is no longer planned.
2. **Road / river feature layer:** `feature: Option<Feature>` on `MapTile`;
   `TileView` gains `feature` + a neighbour-derived edge mask computed in
   `viewmodel::map`; procedural stub-drawing in `gui/map.rs` (a segment from
   tile centre to each connected edge — straight / turn / T / crossroads /
   dead-end all emerge from which segments are drawn). Save format gains a
   parallel, versioned feature grid.
3. **Sprite atlas (optional):** swap procedural drawing for `painter.image`
   + atlas UV keyed on the edge mask; `TileView` unchanged. (Partly realised:
   the POI *marks* are raster `painter.image` as of the ADR 0001 amendment —
   `docs/icons.md`. Terrain fills stay procedural.)

## Code

- `src/viewmodel/map.rs` — `TileView`, `tile_views`, `terrain_rgb`.
- `src/gui/map.rs` — new; `MapView`, `world_to_screen`/`screen_to_world`,
  `visible_tiles`, clamping, the `ui` render method.
- `src/gui/mod.rs` — `mod map;`, `App::map_view`, `Panel::Map` render arm.
- `docs/gui-frontend.md` — Progress list updated (map-via-Painter item).
