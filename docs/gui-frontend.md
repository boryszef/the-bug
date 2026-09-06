# Feature: egui/eframe gui front end

## Why

`docs/adr/0001-ui-framework-egui.md` picked egui/eframe as the framework for
a future graphical front end, but left its timing and scope for later. This
starts that work, alongside the existing ratatui front end rather than
replacing it outright, so the game stays playable throughout the migration.

## Structure

Single binary, no `src/lib.rs`. The front end is a **compile-time choice**
between two mutually exclusive Cargo features — `gui` (default, pulls
`eframe`) and `tui` (legacy, pulls `ratatui`/`crossterm`) — not a runtime
flag. `src/main.rs` `#[cfg]`-gates `mod gui;` / `mod tui;` and its
`run_frontend`; both features on (or neither) is a `compile_error!`. See
**`docs/adr/0002-frontend-selected-at-build-time.md`** — that ADR supersedes
this section. (Historically this was a `--gui` flag with both toolkits
linked; the flag is gone.)

`src/gui/` follows the same layering rule as `src/tui/`
(`ARCHITECTURE.md`): rendering and input-mapping only, everything else goes
through `viewmodel`/`game`.

## Progress

- Empty window (`src/gui/mod.rs`): an `eframe::App` with an empty
  `CentralPanel`, launched via `gui::run()` from `main`.
- `Game`/`Language`/save wiring: `main` now builds `game`/`language` the
  same way for both front ends (load-or-default, detect), then either hands
  them to `tui::App::with_game` or `gui::run(game, language)`. `gui::App`
  saves on close via `eframe::App::on_exit`, mirroring `tui`'s
  save-after-`ratatui::run` in `main` — this only fires on a real window
  close (not a `kill`), so it's not exercised by an automated test.
- Player + event-log panel (`render_player`/`render_events` in
  `src/gui/mod.rs`): same `viewmodel::inventory`/`viewmodel::events` calls
  and `i18n::*` strings as `tui::app`'s equivalents, laid out in an
  `egui::Panel::left` instead of a ratatui `Paragraph`. Event lines get the
  same per-`EventKind` colour-coding (`Color32` instead of ratatui's
  `Color`).
- Tab/quit toolbar: a `Panel` enum (`Map`/`Experiment`/`Craft`/
  `Disassemble`/`Quests`) mirroring `tui::app::Panel`, shown as a row of
  `egui::Panel::top` buttons (click any tab to jump to it) plus a Quit
  button. `[`/`]` cycle `prev`/`next` and `q` quits, matching `tui`'s key
  bindings, checked via `egui::Context::input` each frame. Quitting sends
  `ViewportCommand::Close`, which triggers the same `on_exit` save path as
  closing the window normally. The central area still has no per-panel
  content — it just shows the active tab's title as a placeholder heading.
- Map tab tile grid (`src/gui/map.rs`, `MapView`): the **Map** tab draws
  the map via `egui::Painter` — one `rect_filled` per visible tile coloured
  by `viewmodel::map::terrain_rgb`, plus a player marker; drag pans, scroll/
  pinch zooms (clamped). Tiles arrive as `viewmodel::map::TileView`s
  (`tile_views`), a renderer-neutral descriptor with room for a later
  `feature`/`connections` field and a sprite backend. Still the random map;
  no predefined maps or road/river tiles yet — see `docs/gui-map.md`. The
  other four tabs keep the placeholder heading.
- Map tab controls (`MapView::ui` → `Option<MapCommand>`): a row of
  `←`/`↑`/`↓`/`→` buttons and a **Search** button above the grid, plus the
  arrow keys and `s` as accelerators (read in `map.rs`, not `App`). All
  produce a `MapCommand::{Walk, Search}` that `App::ui` applies via
  `Game::walk` / `Game::search`. The arrows are drawn in the Monospace font
  (`RichText::monospace`) — egui's default proportional font has no arrow
  coverage, but bundled Hack does. The view stays where the user panned it —
  it does not recentre on the player.
- Hint bar (`egui::Panel::bottom`, `hint_text`): one persistent line —
  `gui-hint` (`[ ] switch tabs   q quit`) always, plus `gui-hint-map`
  (`drag to pan   scroll to zoom`) on the Map tab. The gui's counterpart of
  `tui`'s per-panel footer, collapsed to one line since every other panel
  is self-evident buttons.
- Panel content, one mouse-driven `render` per tab (`src/gui/*.rs`), each
  taking the same `viewmodel` data as its `tui` counterpart and returning an
  action `App::ui` applies to `game` — see `docs/gui-panels.md`:
  - **Experiment** (`experiment.rs`): two columns (Available / Selected) of
    click-to-move item buttons plus a run button; holds an `ItemSelection`.
  - **Craft** (`craft.rs`): a button per known recipe, disabled when its
    inputs aren't affordable (`viewmodel::crafting::CraftOption::enabled`).
  - **Disassemble** (`disassemble.rs`): a button per carried item that can be
    taken apart (`viewmodel::disassembly::options`).
  - **Quests** (`quests.rs`): active quest progress + description; available
    quests as collapsing sections (description + Accept button), or the
    blocked/empty message; the completed-quests line.

With this, every tab has its content and the hint bar replaces `tui`'s
per-panel footer — `gui` reaches feature parity with `tui`. Update this
list as further gui work lands (see `TODO.md`).

## What is *not* built here

- No per-panel footer. `tui`'s footer strings for Craft/Disassemble/
  Experiment/Quests describe keyboard list-navigation the gui replaced with
  clicking, so only the universal + Map hints are surfaced (in the hint
  bar). `footer-*` i18n strings stay `tui`-only.
- Not yet decided whether `tui` is retired now that `gui` has reached
  parity, or kept as a permanent alternate front end (ADR 0001 leaves this
  open).

## Code

- `Cargo.toml` — `eframe` as an optional dep behind the `gui` feature
  (default); see ADR 0002.
- `src/gui/mod.rs` — the `eframe::App`; `Panel`; `render_player`/
  `render_events`; the tab/quit toolbar; the hint bar (`hint_text`); the
  per-`Panel` render arms in `App::ui`.
- `src/gui/map.rs` — `MapView` (pan/zoom + `egui::Painter` tile grid), the
  `←↑↓→` + Search controls, `MapCommand`, arrow/`s` key accelerators.
- `src/gui/{experiment,craft,disassemble,quests}.rs` — one `render` per tab.
- `src/i18n/locales/{en,pl}/main.ftl` — `action-*` / `gui-hint*` gui strings.
- `src/viewmodel/map.rs` — `TileView`, `tile_views`, `terrain_rgb`
  (`#[cfg(feature = "gui")]`).
- `src/main.rs` — `#[cfg(feature = "gui")] mod gui;`, shared `game`/
  `language` setup, `#[cfg]`-selected `run_frontend`.
