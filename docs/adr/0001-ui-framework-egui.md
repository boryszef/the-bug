# 1. UI framework: egui/eframe

## Status

Accepted. Amended 2026-09-06 (see "Amendment: raster icon assets" below).

## Context

`src/ui/` is a ratatui (terminal) front end, deliberately kept thin and
swappable — see `ARCHITECTURE.md`'s "UI is temporary" section, which already
frames a front-end swap as "rewriting `ui/` and leaving `game`/`viewmodel`
untouched." Today the map renders as unicode glyphs
(`TerrainType::symbol()`); we want simple graphical map tiles and icons
instead, which a terminal can't give us reliably.

Requirements considered:
- Minimal graphics support — simple map tiles and icons, not a terminal's
  glyph set.
- Easy to implement.
- Either has Rust support, or is a fully separate front end (e.g. a web
  app) talking to Rust underneath.

Four options were compared:

| Option | Maturity | Fit for this UI | Effort |
|---|---|---|---|
| **macroquad** | Solid, smaller maintainer base | Game/sprite-first; the-bug's UI is mostly panels/lists, not sprites | Low |
| **egui/eframe** | Most widely used pure-Rust GUI; large active community | Widget-first (panels, lists, buttons) matches the-bug's actual UI shape; map tiles via `egui::Painter` | Low |
| Fully separate web app (Rust via `wasm-bindgen` + hand-written JS/HTML) | N/A (bespoke) | Most flexible, most control over a "real" web app | Highest — separate build pipeline, hand-written UI layer |
| `ratatui-image` add-on | Niche | Keeps current terminal UI, adds real icons only where the terminal supports an image protocol | Lowest, but graphics aren't guaranteed everywhere |

Both macroquad and egui/eframe compile the same Rust source to either a
native binary or `wasm32-unknown-unknown` (a browser-hosted build, no
server involved) — either satisfies "Rust support ... or a fully separate
web app" without actually requiring a separate web app.

## Decision

Use **egui**, via **eframe** as the windowing/application harness.

The-bug's UI today is almost entirely panel- and list-shaped: Player stats,
an Events log, and four cursor-driven menus (Craft, Experiment, Disassemble,
Quests). egui's immediate-mode widget model (`ui.button(...)`,
`ui.label(...)`, list/table widgets) maps onto that directly. The map — a
grid of tiles — isn't a built-in widget in either library, but is no harder
to hand-draw via egui's `Painter` API than via macroquad's sprite calls.

macroquad's advantage is lowest-friction sprite/tilemap/animation work,
which matters more for a UI that's becoming more game-like (camera movement,
animated sprites); that's not the direction this UI is headed. egui also has
a clear edge in maturity, ecosystem size, and default visual polish.

## Consequences

- `src/ui/` will be rewritten against `egui`/`eframe`; `src/game/` and
  `src/viewmodel/` are unaffected, per the existing layering
  (`ARCHITECTURE.md`).
- The Craft/Experiment/Disassemble/Quests panels map onto egui
  windows/panels and widgets instead of hand-rolled ratatui cursor/list
  code.
- The map renders via a custom `egui::Painter`-based tile grid, not a
  built-in widget — tile icons (images) replace `TerrainType::symbol()`'s
  unicode glyphs.
- A web build becomes available later via `eframe`'s
  `wasm32-unknown-unknown` target, entirely client-side (static files, no
  server) — not scoped by this decision.
- Not yet decided: the timing and scope of the actual `src/ui/` rewrite.
  That's a separate future plan.

## Amendment: raster icon assets (2026-09-06)

### Context

This ADR always anticipated "tile icons (images)" (see Consequences). In
practice the map's points of interest were drawn as procedural `egui::Painter`
shapes — `docs/gui-map.md` "Draw procedurally now, no image assets" and
`docs/map-improvements.md` "Not emoji" — with a real image backend left as
`docs/gui-map.md`'s optional follow-on #3. Those procedural POI marks
(`draw_cave_icon` / `draw_ruins_icon` / `draw_village_icon`) top out at a few
flat polygons; we want icons with more character and better legibility, and a
matching set for items later.

### Decision

Adopt **pre-rendered raster icons**, authored as **SVG**:

- **Source of truth: SVG** in `assets/icons/*.svg` — hand-authored, one file
  per icon (`village.svg`, `cave.svg`, `ruins.svg`, …), a shared 64×64
  viewBox and a shared palette.
- **Shipped asset: PNG.** A committed script (`scripts/render-icons.sh`,
  `resvg`/`inkscape` at a fixed working size, e.g. 256×256) renders every
  SVG to `assets/icons/*.png`. Both the `.svg` and the generated `.png` are
  committed; the script is the regen path, **not** a `build.rs` step.
- **Runtime.** The `gui` build `include_bytes!`s the PNGs, decodes them with
  the `image` crate (`default-features = false`, `features = ["png"]`) into
  `egui::ColorImage`, and uploads one `egui::TextureHandle` per icon (built
  once, cached). `gui/map.rs` draws the POI tile with `painter.image(...)`
  instead of the `draw_*_icon` polygon calls. No runtime SVG renderer
  (`resvg`/`usvg`) is linked.

### Why PNG-at-build rather than SVG-at-runtime

`resvg` + `usvg` + `tiny-skia` is a large dependency tree and compile-time
cost for what is a handful of never-animated icons at a small on-screen size.
Pre-rendering keeps the only new runtime dependency a PNG decoder, makes the
shipped pixels deterministic, and still leaves the SVGs editable.

### Scope

- **POI icons first** (Village, Cave, Ruins; Bridge when it lands). Item
  icons follow the same pipeline when added.
- **Terrain fills stay procedural** — `viewmodel::map::terrain_rgb` +
  `painter.rect_filled`, plus the wavy edge-trickle. Only the POI *mark*
  becomes an image.
- **`tui` is unchanged** — it keeps `Poi::symbol` / `TerrainType::symbol`
  glyphs; `assets/` and the `image` dep are `#[cfg(feature = "gui")]`.
- `viewmodel::map::TileView` is unchanged — it already carries `poi`, which
  is all the renderer needs.

### Consequences

- New `assets/icons/` directory (SVG + generated PNG, both committed) and
  `scripts/render-icons.sh` + a short `docs/icons.md` on the pipeline.
- One new `gui`-only runtime dependency: `image` (png only).
- `docs/gui-map.md` #3 and `docs/map-improvements.md` "Not emoji" are updated
  to point here; the procedural `draw_*_icon` helpers and their geometry
  tests in `src/gui/map.rs` are removed when the image path lands.
- Not decided here: item icons' exact set, and whether the map ever needs
  more than one raster size per icon (texture filtering on a single 256px
  source is expected to be enough across the zoom range).
