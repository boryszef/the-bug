# Functional tests (Gherkin / cucumber-rs)

## Why

Alongside the unit tests colocated with each type, we keep black-box
**functional tests**: scenarios written in Gherkin that drive the game the
way a front end would — call a `Game` method, read the result back through
`viewmodel` — and never touch `gui`. They document behaviour in prose and
catch regressions across the `game` + `viewmodel` seam. Behaviour that reads
naturally as *given a state, when an action, then an outcome* lives here;
`src/game/mod.rs` keeps the unit tests (pool arithmetic, coordinate math,
event timestamps, log ordering, save-schema validation, anything RNG- or
`Map`-internal).

`cucumber-rs` is the Rust runner. It executes scenarios from its own test
binary (`harness = false`), which sees only the crate's **public** API —
which is why the shared core is a library
(`docs/adr/0003-library-target-for-functional-tests.md`).

## Layout

```
tests/
  cucumber.rs               # the runner — the [[test]] target in Cargo.toml
  steps/                    # a subdir, so Cargo doesn't treat it as its own test
    mod.rs                  # module wiring
    world.rs                # GameWorld, item()/quest(), tiny_map(), complete_quest()
    common.rs               # shared: new game, storage/equipment contents, craft menu, "no event"
    experiment.rs craft.rs disassemble.rs hunting.rs quests.rs equipment_storage.rs
  features/
    experiment.feature craft.feature disassemble.feature
    hunting.feature quests.feature equipment-storage.feature
```

`tests/cucumber.rs` is tiny:

```rust
pollster::block_on(GameWorld::cucumber().run_and_exit("tests/features"));
```

`pollster` is a minimal `block_on` — the game is synchronous, so there's no
async runtime. `run_and_exit` makes a failed scenario a non-zero exit, so
`cargo test` and `prek` fail on it.

`GameWorld` (`tests/steps/world.rs`) holds a `Game` plus a couple of
bookkeeping fields (`events_before` for the "no event was logged" check,
`accept_result` for the quest-rejection assertions). It's recreated per
scenario by `Given a new game`.

## Running

- `cargo test` — runs everything, functional suite included.
- `cargo test --test cucumber` — just the functional suite (prints each
  scenario with per-step ✔ / ✘).

## Conventions

- **Public API only.** Steps call `Game` methods and read `viewmodel`
  functions / public `Player` getters. Set-up state is forced through public
  fields — `player.inventory` / `player.equipment` / `player.coordinates` — or
  through `Player::grant_recipe` and `Map::from_terrain`, which are `pub`
  precisely so `tests/` can use them (both are also on the save path; see
  `ARCHITECTURE.md`'s visibility note). If a scenario needs something that
  isn't public, widen the API deliberately — don't bypass it.
- **Reach quest state through gameplay, not `restore_quest_state`.** That
  helper stays `pub(crate)` — it's save-internal. `complete_quest()` in
  `world.rs` accepts the quest and walks the player onto a Ruins on a small
  fixed [`tiny_map`], chaining prerequisites (the axe quest experiments a
  Stone Axe after exploring the ruins). "Stock Up" can't be completed this
  way (its condition is five RNG hunts) — no scenario needs it as a
  precondition.
- **No RNG-dependent scenarios.** `search` / `hunt` roll random items and
  `Map::new` generates terrain randomly; an outcome that depends on a roll is
  flaky by construction. Drive deterministic actions and, when geography
  matters, use [`tiny_map`] — a 3×3 fixed grid (village centre, Ruins one
  tile south) via the `pub` `Map::from_terrain`. Never poke `game.map.tiles`.
  When a quest's own condition is gated by `search()` (e.g.
  `OldCivilization`'s `FindItem`), widen the API instead of reaching past
  it: `Map::guarantee_find(pos, item)` is `pub` precisely so a step can
  force a `1.0` probability and get a deterministic find through the real
  `search()` call, rather than the scenario depending on a roll.
- **Item / quest names.** Gherkin uses the English `i18n` name (`Branch`,
  `Stone Axe`; the quest titles like `"The Digital Civilization"`);
  `world.rs`'s `item()` / `quest()` map them. Hand-maintained tables (the
  crate's `Item::ALL` is `#[cfg(test)]` and unreachable) — extend as
  features name new things. Multi-word names mean the item-bearing steps use
  `regex`, not `{word}`.
- **One file per feature area**, matched `features/<area>.feature` ⇄
  `tests/steps/<area>.rs`, with cross-cutting steps in `common.rs`.

## Current coverage

| feature | what it pins down |
|---|---|
| `experiment.feature` | recipe discovery, shortage / wrong-quantity failures still spending inputs, the tool gate, disassemble-only rejection, XP on discovery, the village gate |
| `craft.feature` | consumables from storage-then-equipment, tool required but kept, unknown-recipe and missing-tool refusals, the village gate |
| `disassemble.feature` | parts returned to storage (from storage or equipment), scavenged umbrella, craft-only ignored, parts stacking, the village gate |
| `hunting.feature` | the deterministic "unprepared" checks — no bow / no arrows / gear only in storage |
| `quests.feature` | accept rules (opening quest, one-at-a-time, unmet prerequisites), available/completed transitions, completion by crafting or experimenting the target item |
| `equipment-storage.feature` | `Game`-level transfers and drops — the village gate on transfers and drop-from-storage; drop-from-equipment has none |

## Out of scope (for now)

- Movement geometry and RNG actions (`search`, successful `hunt`s, the
  `sure_hunt` stock-up chain, walking onto a POI) — unit tests in
  `src/game/mod.rs`.
- `save.rs` round-trips — no save/load vocabulary in the scaffold.
- Any front-end (`gui`) behaviour — verified by running the app.
- Localised event text — `i18n` has its own tests; functional steps assert on
  `EventKind`, not rendered strings.
