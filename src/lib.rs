//! `the-bug`'s core, as a library so the functional test suite in `tests/`
//! can drive the game through its public API (`cucumber` runs from its own
//! binary and only sees what's `pub`). `src/main.rs` is a thin shim that
//! parses the CLI and hands off to the `#[cfg]`-selected front end below.
//!
//! Layering (see `ARCHITECTURE.md`): `game` ← `viewmodel` ← a front end
//! (`gui` / `tui`). `save`, `i18n` and `mapgen` sit alongside `game`.
//! See `docs/adr/0003-library-target-for-functional-tests.md`.

#[cfg(all(feature = "gui", feature = "tui"))]
compile_error!(
    "the-bug: enable exactly one front end — `gui` or `tui`, not both. \
     The default is `gui`; for the legacy terminal UI build with \
     `--no-default-features --features tui`."
);
#[cfg(not(any(feature = "gui", feature = "tui")))]
compile_error!(
    "the-bug: no front end selected. Build with the default `gui` feature, \
     or `--no-default-features --features tui` for the legacy terminal UI."
);

pub mod game;
pub mod i18n;
pub mod save;
pub mod viewmodel;

// Crate-internal: only `game::map` calls into it (`ARCHITECTURE.md` — the
// terrain generator sits beside `game`, not above it).
mod mapgen;

#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "tui")]
pub mod tui;
