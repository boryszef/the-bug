# Code review, September 2026

A full read of `src/` (~7.5k lines) looking for dead code, logic bugs,
cleanliness and test coverage. This file is the standing record of what that
review found and what was decided about each item — the fixes themselves are
described in the sections they touch.

The codebase was in good shape going in: layering (`game` / `viewmodel` / front
end) is respected, 158 (gui) / 168 (tui) tests pass, clippy is clean under both
feature sets. None of what follows is compiler- or clippy-visible.

Three decisions were taken up front:

- The unreachable-recipe gap (below) is **reported, not fixed** — only the doc
  that misstates it is corrected.
- The duplicated `Panel` enum **is** extracted to `viewmodel`, even though that
  touches the frozen `tui`.
- `prek.toml` is **left alone** — the manual `tui` test pass stays manual.

---

## Findings

### Bug 1 — reachable panic in the Experiment tab (`src/gui/experiment.rs:40`)

```rust
let addable = owned - self.selection.quantity(item);
```

Unchecked `u32` subtraction. `ItemSelection` lives on `App` and survives tab
switches, but `owned` is re-read from the live inventory every frame, so the
selection can outlive the stock it was capped against:

1. Hold 3 Branches. On Experiment, click Branch three times → selection `(Stick, 3)`.
2. Switch to Craft, craft an Arrow (spends 1 Branch) → stock is 2.
3. Switch back to Experiment → `2 - 3` underflows.

Debug builds panic (`attempt to subtract with overflow`); release wraps to
`4294967293` and renders that as the button label. The same staleness also lets
a doomed experiment be submitted — it just logs a shortage, which is harmless.

`ItemSelection::available()` (`viewmodel/selection.rs`) already does this
correctly with `saturating_sub`; the gui simply doesn't use it. Reachable via
Craft, Disassemble, or Hunt — any action that spends an item while a selection
is held.

**Fix.** Give `ItemSelection` a `clamp_to(inventory)` that trims every picked
quantity to what is actually owned and drops entries that hit zero, and call it
at the top of `Experiment::render` before drawing. That fixes the underflow at
its source and keeps the panel honest, rather than papering over it with
`saturating_sub` at the call site. `tui::experiment` gets the same call — it is
reachable there too.

### Bug 2 — quest completion is logged before the action that caused it

`Game::craft` (`src/game/mod.rs:262`) and `Game::experiment` (`:307`) both call
`grant_item` — which routes to `note_quest_event` and can log
`QuestCompleted` — *before* logging `Crafted` / `Experimented`. The log ends up
holding the effect ahead of its cause, and since the pane renders newest-first
the player reads "You craft a Stone Axe" with "Quest complete" sitting above it.

`hunt` already has this right (it logs `Hunted`, then notes the quest); `walk`
is unaffected (walking isn't logged).

**Fix.** In both `craft` and `experiment`, move the `log(...)` of the action
above the `grant_item(...)` call. Inventory and quest state are untouched — only
the order of two pushes onto `events` changes.

### Cleanups

- **`Panel` is duplicated verbatim** between `src/gui/mod.rs:47-92` and
  `src/tui/app.rs:29-58` — variants, `next`, `prev`, and all three unit tests.
  Only one compiles at a time so clippy can't see it. Per `CLAUDE.md` /
  `ARCHITECTURE.md` this is UI-agnostic and belongs in `viewmodel`.
- **`Player::spend_all`** (`player.rs:131`) underflows on a duplicated entry
  (`[(Vine,2),(Vine,2)]` against 3 Vine): `first_shortage` checks each entry
  independently against the full stock, so it passes, then the second `spend`
  underflows. Not reachable today — `ItemSelection` dedupes and every static
  recipe is duplicate-free — so this is a latent trap in a
  caller-must-check API, not a live bug.
- **`Map::contains`** (`map.rs:279`) tests `|pos| <= half` rather than the
  actual grid, so on an even-sized map (only reachable from a hand-edited save)
  it admits a coordinate `get_tile` then returns `None` for; `walk` moves the
  player onto a tile that doesn't exist.
- **`mapgen::generate`** builds a whole `HashMap<Cell, TerrainType>`
  (`generator.rs:135-138`) only to copy it straight into the grid — `bisect` can
  write into the grid directly. Also, the `affinity <= EPS` branch shuffles both
  `cells` and `bag` when one shuffle suffices; `boundary_ratio` (`:332`) is
  `pub(crate)` but used only inside its own module's tests; and `bisect`'s doc
  comment still mentions "the rare 1-cell village/scatter split, which the
  caller checks for", which stopped being true when POIs became an overlay.
- **`Spec::validate`** doesn't reject negative percentages; `[150.0, -50.0]`
  sums to 100 and then underflows `n - assigned` in `cluster_quotas`. Const-only
  input today, so this is hardening, not a live bug.
- **`EventKind::Hunted`** carries `Vec<(Item, u32)>` where the count is always
  `1`. Left alone deliberately — `describe_items` wants the pair shape and the
  save format would change.

### Reported, not fixed

`Item` declares `MetalKnife`, `ElectricMotor`, `SteelBolt`, `RustyMetal` — no
recipe and no terrain/POI table mentions any of them (the last two are on
`TODO.md` for the future bridge POI). Worse, `Microcontroller`, `SolarPanel` and
`CircuitBoard` are *inputs* to Metal Detector and Solar Charger but have no
source either, so **both recipes are permanently uncraftable** while still
counting in the `recipe_progress()` denominator.

`docs/progression.md` claims the opposite — "100% stays reachable". Only that
sentence is corrected; the item and recipe tables stay as they are, and the gap
goes on `TODO.md`.

### Stale docs

- `docs/event-log.md` describes `EventCategory` and `Event { category, text }`,
  and points at `src/game.rs` and `src/ui/app.rs`. All four are gone — it's
  `EventKind` + i18n rendering, in `src/game/event.rs`, `src/gui/mod.rs` and
  `src/tui/app.rs`. Its example lines also use `→`, which
  `every_event_kind_renders_in_every_language` now forbids as unrenderable in
  the gui.
- `docs/progression.md` cites a test named `search_yields_every_item_a_tile_offers`;
  it is `search_yields_terrain_and_poi_items_and_names_each_source`.

### Test gaps

Coverage is good where it exists — mapgen, save/load, quests and the gui map's
geometry helpers are all well covered. Missing:

1. A selection held across an inventory drop — the Bug 1 scenario. Nothing
   exercises `ItemSelection` against a shrinking inventory.
2. Event **ordering** around quest completion — the Bug 2 scenario. The existing
   quest tests assert state and event *counts*, never the sequence.
3. Every `Item` / `TerrainType` / `Poi` rendering in every language. The
   equivalent `EventKind` test uses a hand-maintained sample list, so a new
   `Item` with a missing `.ftl` key compiles and silently renders
   "Unknown localization". (The `.ftl` files are complete today — all 29 item
   keys were diffed against both locales.)
4. `Map::contains` agreeing with `get_tile` (the even-grid case).
5. `adjust_probability` at elapsed ≈ 0 — a search immediately after a search
   should be able to yield nothing.

---

## Work

Sequenced so each commit is one kind of change, per `ARCHITECTURE.md`'s
refactor/behaviour split.

### 1. Fix the Experiment-panel panic

- `src/viewmodel/selection.rs` — add
  `pub fn clamp_to(&mut self, inventory: &[(Item, u32)])`: trim each picked
  quantity to the owned amount (items absent from `inventory` count as zero) and
  drop entries that reach zero. Mirrors the existing `available()`.
- `src/gui/experiment.rs` — call it at the top of `render`, before the columns
  are drawn; `addable` is then always safe.
- `src/tui/experiment.rs` — same call on its render/key path.
- Tests in `selection.rs`: quantity trimmed when stock shrinks; entry dropped
  when stock hits zero; untouched when stock is unchanged or grows.

### 2. Fix the quest-completion log order

- `src/game/mod.rs` — in `craft`, move `log(EventKind::Crafted { .. })` above
  `grant_item(...)`; same for `Experimented` in `experiment`.
- Test in `game/mod.rs`: with the axe quest open, craft a Stone Axe and assert
  the last two events are `Crafted` then `QuestCompleted`, in that order. Add
  the experiment counterpart.

### 3. Harden the caller-must-check APIs

Behaviour-preserving for every reachable path:

- `Player::spend` — `debug_assert!` the stock covers the amount, and use
  `saturating_sub` so a release build can't wrap.
- `Map::contains` — derive from the grid (`self.tiles.get(y).is_some_and(|row|
  x < row.len())` via `world_to_tile`) instead of `half`, so it can never
  disagree with `get_tile`. Add a test pairing the two over a hand-built
  even-sized map.
- `Spec::validate` — reject a negative percentage with a `GenError` variant;
  extend `rejects_bad_specs`.

### 4. Extract `Panel` into the viewmodel

New `src/viewmodel/panel.rs` holding the enum, `ALL`, `next`, `prev` and the
three cycling tests (currently duplicated in both front ends). Both
`src/gui/mod.rs` and `src/tui/app.rs` import it and keep only what is genuinely
theirs — the gui's `title_id`, the tui's `footer-*` id mapping. `next` and
`prev` derive from `ALL`'s index rather than restating the cycle in two match
arms. Pure refactor: the existing tests move with the code and must pass
unmodified under both feature sets.

### 5. Tidy `mapgen`

- `bisect` writes into `&mut Vec<Vec<TerrainType>>` directly; drop the
  intermediate `HashMap` and the now-unused `std::collections::HashMap` import.
- Drop the redundant `cells.shuffle` in the `affinity <= EPS` branch.
- Make `boundary_ratio` private (`components` stays `pub(crate)` — `game/map.rs`
  tests use it).
- Correct `bisect`'s stale doc comment about a village/scatter split.

`deterministic_given_a_seed` will need re-baselining if removing the `HashMap`
changes the RNG draw sequence — confirm whether it does, and if so say so in the
commit rather than quietly editing the assertion.

### 6. Widen the i18n safety net

Add `Item::ALL` (a `const [Item; N]`, the same shape as `Panel::ALL`) and assert
in `src/i18n/mod.rs` tests that every `Item`, `TerrainType` and `Poi` renders
non-empty and free of "Unknown localization" in both languages — the coverage
`every_event_kind_renders_in_every_language` gives `EventKind`.

### 7. Docs

- `docs/event-log.md` — rewrite the "Code" section and the category paragraph
  for `EventKind` + i18n; replace `→` in the examples.
- `docs/progression.md` — correct the reachability claim (naming Metal Detector
  and Solar Charger and why), and the stale test name.
- `docs/gui-panels.md` — a one-paragraph note on the `Panel` extraction.
- `TODO.md` — add the unreachable-recipe gap under the story EPIC.

---

## Verification

Per `ARCHITECTURE.md`, everything runs under **both** feature sets:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --no-default-features --features tui -- -D warnings
cargo test
cargo test --no-default-features --features tui
```

(`--all-features` is expected to fail — the two front ends are mutually
exclusive by design.)

Beyond the suite:

- **Bug 1, by hand** — `cargo run`, search until you hold 3 Branches, select all
  three on Experiment, craft an Arrow on the Craft tab, return to Experiment.
  Before the fix a debug build panics there; after, the row reads `2 Branch`
  with the selection trimmed to 2.
- **Bug 2, by hand** — accept "Trouble in the East" → the axe quest, craft the
  Stone Axe, and read the Events pane top-down: the completion line must sit
  above (i.e. after) the craft line.
- **Panel extraction** — cycle `[` / `]` through all five tabs in both
  directions in the gui, and in the tui under `tmux`
  (`--no-default-features --features tui`), confirming order and wrap-around are
  unchanged.
- **mapgen** — `new_map_terrain_is_clustered_not_confetti` and the generator's
  own clustering tests are the regression net for step 5; run them a few times,
  since two of them are stochastic.

---

## Status

All of the "Work" items above are done, each as its own commit on
`code-review-fixes`. `cargo fmt --all -- --check`, `cargo clippy --all-targets
-- -D warnings` and `cargo test` are clean under both `gui` (166 tests, was
158) and `--no-default-features --features tui` (176, was 168).

One planned cleanup turned out not to hold up: the `.to_vec()` clone in
`gui/experiment.rs`'s "Selected" column loop (flagged above as unnecessary)
is in fact required — `decrement_at` needs `&mut self.selection` while the
loop is still borrowing it immutably to iterate, so dropping the clone fails
to borrow-check. Left as-is, with a comment explaining why.
