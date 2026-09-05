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
  passed. No `Game`/`Language` wiring yet.

Update this list as each subsequent piece lands (game/save wiring, player +
event-log panel, map via `egui::Painter`, craft/disassemble/experiment/quests
panels — see `TODO.md`).

## What is *not* built here

- No `Game` or save/load wiring — `--gui` currently ignores `--load` and
  never saves.
- No map, player panel, event log, or any of the craft/disassemble/
  experiment/quests panels yet.
- Not yet decided whether `tui` is retired once `gui` reaches parity, or
  kept as a permanent alternate front end (ADR 0001 leaves this open).

## Code

- `Cargo.toml` — `eframe` dependency.
- `src/gui/mod.rs` — new, the `eframe::App` scaffold.
- `src/main.rs` — `mod gui;`, `Cli::gui` flag, dispatch in `main()`.
