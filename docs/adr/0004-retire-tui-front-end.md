# 4. Retire the `tui` front end

## Status

Accepted. Supersedes `docs/adr/0002-frontend-selected-at-build-time.md`.

## Context

ADR 0002 kept two front ends behind mutually-exclusive Cargo features:
`gui` (egui/eframe, the developed one) and `tui` (ratatui/crossterm),
declared **frozen** — "kept working, not developed further". That was a
transitional state while the gui caught up.

The gui reached parity long ago and has taken every feature since (the Items
tab, hunting, the bag/storage split, the map rework). Nothing — no user, no
test, no tooling — depends on the tui. What it still costs:

- a `[features]` table and `#[cfg(feature = …)]` gates threaded through
  `src/viewmodel/` (`map.rs`'s `TileView`/`tile_views`/`terrain_rgb`,
  `mod.rs`'s `items`, `selection.rs`'s `available`);
- a second `clippy` pass in `prek.toml` and a second `cargo test` config a
  contributor has to remember (`ARCHITECTURE.md` Workflow);
- ~1230 lines in `src/tui/` and a set of tui-only Fluent keys carried in
  both locales;
- `compile_error!` guards for the "both / neither feature" cases.

## Decision

Delete the tui. Concretely:

- Remove `src/tui/` and the `.cargo/config.toml` tui aliases.
- `Cargo.toml`: drop `ratatui` and `crossterm`; make `eframe` a normal
  (non-`optional`) dependency; **remove the `[features]` table entirely** —
  there is nothing left to select.
- `src/lib.rs`: `pub mod gui;` unconditional; drop both `compile_error!`
  guards.
- `src/main.rs`: one `run_frontend`, no `#[cfg]`.
- `src/viewmodel/`: delete every front-end `#[cfg]` gate; delete
  `ItemSelection::available` (only `tui::experiment` called it).
- `prek.toml`: one `cargo-clippy` hook.
- Drop the tui-only Fluent keys (`footer-*`, `craft-hint` / `experiment-hint`
  / `disassemble-hint`, `items-not-in-tui`, the joined `quests-completed`
  line).

Still one binary, still `the_bug` lib + `main.rs` shim (ADR 0003). There are
now **no front-end Cargo features**.

## Consequences

- One build, one `cargo test`, one `cargo clippy` — the two-config dance in
  `ARCHITECTURE.md`'s Workflow section is gone.
- `src/viewmodel/` is plain code again; the layer is still conceptually
  front-end-agnostic (a second front end could still be added), just no
  longer `#[cfg]`-partitioned for a front end that exists.
- `docs/gui-frontend.md` and `docs/panel-layout.md` become historical
  records of the two-front-end era; ADR 0002 is superseded; ADR 0003's
  `#[cfg]` front-end modules collapse to one.
- The dependency graph loses ratatui's tree (~40 crates: crossterm, mio,
  signal-hook, compact_str, …).
- The saved-game format, the CLI, and all runtime behaviour are unchanged.
- Reversible only by `git revert` — but the intent is that a future
  alternate front end (e.g. web/wasm via `eframe`) would be *new* work in
  its own module, not a resurrection of the ratatui code.
