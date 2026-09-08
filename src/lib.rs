//! `the-bug`'s core, as a library so the functional test suite in `tests/`
//! can drive the game through its public API (`cucumber` runs from its own
//! binary and only sees what's `pub`). `src/main.rs` is a thin shim that
//! parses the CLI and hands off to `gui::run`.
//!
//! Layering (see `ARCHITECTURE.md`): `game` ← `viewmodel` ← `gui` (the
//! egui/eframe front end). `save`, `i18n` and `mapgen` sit alongside `game`.
//! See `docs/adr/0003-library-target-for-functional-tests.md`.

pub mod game;
pub mod gui;
pub mod i18n;
pub mod save;
pub mod viewmodel;

// Crate-internal: only `game::map` calls into it (`ARCHITECTURE.md` — the
// terrain generator sits beside `game`, not above it).
mod mapgen;
