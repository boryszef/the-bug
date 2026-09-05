# 2. Front end selected at build time (Cargo features)

## Status

Accepted. Supersedes the "Structure" section of `docs/gui-frontend.md`.

## Context

ADR 0001 chose egui/eframe for a graphical front end; `docs/gui-frontend.md`
built it alongside the existing ratatui (`tui`) front end, picked at runtime
by a `--gui` flag, with *both* toolkits linked into the single binary. The
`gui` front end has since reached feature parity with `tui`.

The decision now: `tui` is **frozen** — kept working, not developed further —
and `gui` is the one that moves forward. Given that, linking both toolkits
into every build is pure cost:

- `eframe` drags in a windowing/GL stack (x11/wayland/OpenGL system
  libraries) that a terminal-only build has no use for;
- `ratatui` + `crossterm` are dead weight in a graphical build;
- the `--gui` flag implies both are always available, which is no longer
  true of what we want to ship.

`docs/gui-frontend.md` already floated a `src/lib.rs` + `src/bin/{tui,gui}.rs`
split and rejected it as "more restructuring than warranted". Cargo features
on the existing single binary get the same dependency partitioning for far
less churn.

## Decision

The front end is a **compile-time choice** via two mutually exclusive Cargo
features:

- `gui` (in `default`) → pulls `eframe`; builds the egui front end.
- `tui` → pulls `ratatui` + `crossterm`; builds the legacy terminal front
  end. `cargo build --no-default-features --features tui`.

`src/main.rs` `#[cfg]`-gates `mod gui;` / `mod tui;` and dispatches to a
`#[cfg]`-selected `run_frontend`. Enabling **both** features, or **neither**,
is a hard `compile_error!` with a message pointing at the right build
command. The `--gui` CLI flag is removed.

Considered and rejected:

- **Runtime flag, both linked (status quo)** — keeps the unwanted deps.
- **`lib.rs` + `src/bin/` split** — cleaner separation but a bigger
  restructure than features, and `docs/gui-frontend.md` already deferred it.
- **Additive (non-exclusive) features, `gui` wins if both** — keeps
  `cargo {test,clippy} --all-features` working in one pass, but "either/or"
  is exactly the intent, and a real terminal-only binary still needs
  `--no-default-features` anyway.

## Consequences

- **Two-config verification.** `cargo test` / `cargo clippy` cover only the
  active feature set, so the full check is two runs: the default (`gui`) and
  `--no-default-features --features tui`. `prek.toml` runs `clippy` for both;
  `ARCHITECTURE.md`'s Workflow section says the same for `test`.
- **`viewmodel::map`'s gui-only helpers** (`TileView`, `tile_views`,
  `terrain_rgb`) are `#[cfg(feature = "gui")]` — under a `tui`-only build
  they'd be dead code. `world_tiles` and the rest of `viewmodel` stay shared
  and ungated. The layer is still conceptually front-end-agnostic; the gate
  is dead-code hygiene, not a design boundary.
- **`.ftl` locale files are unchanged.** Unused message ids under either
  build (`footer-*` for `gui`, `gui-hint*` for `tui`) are harmless, and
  `i18n::ui` takes a runtime `&str` so nothing is compile-checked away.
- **The saved-game format is identical** across both builds — same
  `save.rs`, same on-disk JSON.
- **Not decided here:** whether `tui` is eventually deleted. For now it
  stays as a build option.
