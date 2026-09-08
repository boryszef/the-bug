# 3. A library target for the shared core

## Status

Accepted. Amends ADR 0002's aside that a `src/lib.rs` split was "more
restructuring than warranted".

## Context

We want functional (BDD) tests in Gherkin, run by `cucumber-rs`
(`docs/functional-tests.md`). `cucumber` executes its scenarios from its own
test binary (`harness = false`), which — like any `tests/` integration
target — can only see the crate's **public** API.

`the-bug` was binary-only: `src/main.rs` privately declared `mod game;`,
`mod viewmodel;`, etc. Nothing under `tests/` could reach any of it, so a
functional suite was impossible without a structural change.

ADR 0002 considered a `src/lib.rs` + `src/bin/{gui,tui}.rs` split and rejected
it — but that was rejecting splitting the **two front ends** into separate
binaries, as a heavier alternative to the `gui`/`tui` Cargo features. It was
not a decision about whether the shared core should be a library.

## Decision

Move everything except the CLI shim into a library crate, `src/lib.rs`:

- `pub mod game; pub mod viewmodel; pub mod save; pub mod i18n;` — the shared
  layers, now reachable from `tests/`.
- `mod mapgen;` — crate-internal, only `game::map` uses it.
- `#[cfg(feature = "gui")] pub mod gui;` /
  `#[cfg(feature = "tui")] pub mod tui;` — the front ends stay **in the
  library**, feature-gated exactly as before. Keeping them here (rather than
  in the binary) means their `#[cfg(test)]` unit tests keep their existing
  `pub(crate)` / `pub(super)` access to core internals — no visibility
  widening anywhere.
- The two `compile_error!` front-end guards move to `lib.rs`.

`src/main.rs` becomes a ~60-line shim: parse `Cli`, load or start a `Game`,
call `the_bug::gui::run` / `the_bug::tui::App`. It keeps its own `Cli` unit
tests.

Still **one** binary, still selected by the `gui` / `tui` features. This is
not the `src/bin/` split ADR 0002 rejected.

## Consequences

- `cargo test` now also builds and runs the `cucumber` target. It compiles
  under both feature sets (it only touches `game` + `viewmodel`), so the
  existing two-config check (`ARCHITECTURE.md` Workflow) covers it with no
  new command.
- `cargo clippy --all-targets` now lints `tests/` too, under both feature
  sets — `prek` already runs clippy for both.
- New dev-dependencies: `cucumber` and `pollster` (a minimal `block_on`;
  the game is fully synchronous). Both are dev-only — not in the shipped
  binary.
- Doc-tests are now possible (the crate has a lib); there are none yet.
- The on-disk save format, the feature flags, and the run-time behaviour are
  all unchanged.
- Not decided here: whether `main.rs` should own `run_frontend` long-term or
  hand a fully-formed front end back from the library.
