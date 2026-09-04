# 1. UI framework: egui/eframe

## Status

Accepted

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
