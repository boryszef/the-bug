# Feature: egui/eframe gui front end

## Why

`docs/adr/0001-ui-framework-egui.md` picked egui/eframe as the framework for
a future graphical front end, but left its timing and scope for later. This
starts that work, alongside the existing ratatui front end rather than
replacing it outright, so the game stays playable throughout the migration.

## Structure

Single binary, no `src/lib.rs`. `mod tui;` and `mod gui;` sit side by side
in `main.rs`; a new `--gui` flag on the existing `clap` `Cli` picks which one
runs. Considered instead: a `src/lib.rs` + `src/bin/{tui,gui}.rs` split, so
each binary only links what it needs — rejected for now as more
restructuring than warranted before any `gui` code existed; worth
revisiting once `gui` is more than a scaffold.

`src/gui/` follows the same layering rule as `src/tui/`
(`ARCHITECTURE.md`): rendering and input-mapping only, everything else goes
through `viewmodel`/`game`.

## Progress

- Empty window (`src/gui/mod.rs`): an `eframe::App` with an empty
  `CentralPanel`, launched via `gui::run()` from `main` when `--gui` is
  passed.
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

Update this list as each subsequent piece lands (map via `egui::Painter`,
craft/disassemble/experiment/quests panel content — see `TODO.md`).

## What is *not* built here

- No map, or any of the craft/disassemble/experiment/quests panels' actual
  content — tab switching works, but every tab shows the same placeholder
  heading.
- Not yet decided whether `tui` is retired once `gui` reaches parity, or
  kept as a permanent alternate front end (ADR 0001 leaves this open).

## Code

- `Cargo.toml` — `eframe` dependency.
- `src/gui/mod.rs` — the `eframe::App`; `Panel`; `render_player`/
  `render_events`; the tab/quit toolbar and keyboard shortcuts in `App::ui`.
- `src/main.rs` — `mod gui;`, `Cli::gui` flag, shared `game`/`language`
  setup, dispatch in `main()`.
- `src/i18n/locales/{en,pl}/main.ftl` — `action-quit`.
