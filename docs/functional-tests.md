# Functional tests (Gherkin / cucumber-rs)

## Why

Alongside the unit tests colocated with each type, we want black-box
**functional tests**: scenarios written in Gherkin that drive the game the
way a front end would — call a `Game` method, read the result back through
`viewmodel` — and never touch `gui`. They document behaviour in prose and
catch regressions across the `game` + `viewmodel` seam.

`cucumber-rs` is the Rust runner for this. It executes scenarios from its own
test binary (`harness = false`), which sees only the crate's **public** API —
which is why the shared core is now a library
(`docs/adr/0003-library-target-for-functional-tests.md`).

## Layout

```
tests/
  cucumber.rs            # the runner — the [[test]] target in Cargo.toml
  steps/                 # a subdir, so Cargo doesn't treat it as its own test
    mod.rs               # module wiring
    world.rs             # GameWorld + the item(name) helper
    crafting.rs          # step definitions, grouped by feature
  features/
    crafting.feature
```

`tests/cucumber.rs` is tiny:

```rust
pollster::block_on(GameWorld::cucumber().run_and_exit("tests/features"));
```

`pollster` is a minimal `block_on` — the game is synchronous, so there's no
async runtime. `run_and_exit` makes a failed scenario a non-zero exit, so
`cargo test` and `prek` fail on it.

`GameWorld` (`tests/steps/world.rs`) is just `{ game: Game }`; `Game` already
derives `Debug` and implements `Default`, which is all `#[derive(World)]`
needs. It is recreated per scenario by the `Given a new game` step.

## Running

- `cargo test` — runs everything, functional suite included.
- `cargo test --test cucumber` — just the functional suite (prints each
  scenario with per-step ✔ / ✘).

## Conventions

- **Public API only.** Steps call `Game` methods and read `viewmodel`
  functions / public `Player` getters. Set-up state is forced through the
  public `player.inventory` / `player.bag` / `player.coordinates` fields.
  Don't reach for crate internals — if a scenario needs something that isn't
  public, that's a signal to widen an API deliberately, not to bypass it.
- **No RNG-dependent scenarios.** `search` / `hunt` roll random items and
  map generation is random; a scenario whose outcome depends on a roll is
  flaky by construction. Drive deterministic actions (`experiment`, `craft`,
  `walk`, `accept_quest`, transfers) and force the surrounding state
  explicitly. (The unit tests that *do* exercise `search` / `hunt` force
  every tile probability to `1.0` first — they can, because they're inside
  the crate and can touch `MapTile`; functional tests can't and shouldn't.)
- **Item names.** Gherkin refers to items by their English `i18n` name
  (`Branch`, not `Stick`; `Stone Axe`); `tests/steps/world.rs::item()` maps
  them to the enum. It's a hand-maintained table (the crate's own `Item::ALL`
  is `#[cfg(test)]` and unreachable here) — extend it as features name new
  items. Multi-word names mean the item-bearing steps use `regex` rather than
  a `{word}` cucumber expression.
- **One file per feature area**, both for `features/*.feature` and the
  matching `tests/steps/*.rs`.

## Current coverage

- `crafting.feature` — experiment outcomes read back through
  `viewmodel::crafting::options`: recipe discovery, shortage / wrong-quantity
  failures still spending the inputs, and the tool gate (an experiment that
  matches a recipe but lacks its tool learns nothing).

## Out of scope (for now)

- Breadth — movement, hunting, quests, disassembly, save/load. The step
  library grows with each feature area added.
- Any front-end (`gui`) behaviour — rendering and input mapping are
  verified by running the app, not here.
- Localised event text — `i18n` has its own tests; functional steps assert on
  `EventKind`, not rendered strings.
