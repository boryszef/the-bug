# Architecture

The project's standing design decisions — not a specific feature's rationale
(those live in `docs/`), but the rules that should hold across all of them.
When a new piece of code doesn't obviously fit one of the modules below, or a
refactor changes where logic lives, this is the doc to check and update.

## Layering: front end → `viewmodel` → `game`

Three layers, each only depending on the one below it:

- **`src/game/`** — the domain model. Pure Rust, no rendering, no key
  handling, no knowledge that a terminal (or a window) exists. `Player`,
  `Map`, `Item`, `Recipe`, `Event`, `Quest`, and `Game` (the orchestrator)
  all live here.
- **`src/viewmodel/`** — presentation-agnostic helpers that shape `game`
  state for display: sorting, filtering, formatting, coordinate transforms,
  transient interaction state (e.g. `selection::ItemSelection`). No
  ratatui or egui types — this is the layer both front ends below share.
- **A front-end module** — rendering and input-mapping only, one per UI
  toolkit: **`src/tui/`** (ratatui, terminal) and **`src/gui/`** (egui,
  graphical — see `docs/adr/0001-ui-framework-egui.md`). Exactly **one** is
  compiled, chosen by the mutually exclusive `gui` (default) / `tui` Cargo
  features — `src/main.rs` `#[cfg]`-gates the modules; both or neither is a
  `compile_error!`. `tui` is frozen (kept building, not developed); `gui` is
  where new work goes. See `docs/adr/0002-frontend-selected-at-build-time.md`.

All of the above is a **library crate** (`src/lib.rs`, `the_bug`); `src/main.rs`
is a thin CLI shim that parses args and hands off to the `#[cfg]`-selected
front end. The split exists so the functional test suite in `tests/` can drive
the game through its public API — see
`docs/adr/0003-library-target-for-functional-tests.md` and
`docs/functional-tests.md`. It is still one binary, still feature-selected; it
is *not* the `src/bin/{gui,tui}` split ADR 0002 rejected.

The rule of thumb: if a different front-end would also need it, it doesn't
belong in a front-end module. See `docs/refactor-thin-ui.md` for the
refactor that established this (written when there was only `src/ui/`;
the same rule now applies to both `tui` and `gui`).

`src/mapgen/` sits beside `game`: a policy-free terrain generator that
`Map::new` calls with a hardcoded `Spec` (`docs/mapgen.md`). It depends on
`game::TerrainType` and nothing else.

## The UI is temporary — keep it thin

No front-end module is the product. Concretely:

- Anything that isn't rendering or input-mapping moves to `viewmodel` (or
  `game`, if it's really domain logic). A front-end module's files should
  be small enough that swapping — or adding — a front end mostly means
  writing that module and leaving `game`/`viewmodel` untouched.
- Interactive `tui` modules (`craft.rs`, `disassemble.rs`, `experiment.rs`,
  `quests.rs`) follow one shape: a small `State` struct holding only
  transient UI state (cursor, focus — never game data), a `handle_key`
  method, an `Outcome` enum describing what `App` should do next, and a
  `render` method. They take `game`/`viewmodel` data as parameters rather
  than holding a reference to `Game`, which keeps them independently
  testable without a terminal. `gui` should follow the equivalent shape for
  its own toolkit as its panels are built out.
- `tui::App` (`src/tui/app.rs`) owns key dispatch and layout only — see
  `docs/panel-layout.md` for the current panel-cycling structure.

## `Game` is a thin orchestrator — models own their own logic

`Game` (`src/game/mod.rs`) should contain only logic where two or more
models genuinely need to cooperate, or where `Game`'s own private state
(the event log) is the reason a call has to go through it. Anything that
only concerns one model's own fields — a computation, a validation, a state
transition — belongs as a method on that model (`player.rs`, `map.rs`,
`quest.rs`, `recipe.rs`), not inlined in `Game`.

- A model may freely consult another model's **static, read-only catalog**
  (`Player` methods already call into `RECIPES`/`grant_recipe`/
  `find_known_recipe`) — that's not cross-model intersection, since only
  one model's *mutable* state is involved.
- The one thing only `Game` can do is append to its own event log
  (`self.log(...)`), since `events`/`started` are private to `Game`. A
  model method that needs to report something log-worthy returns a value
  describing it (e.g. `Player::note_quest_event` returns `Option<&'static
  Quest>` for "this quest just completed") and lets `Game` decide
  whether/what to log — the model itself stays silent.
- Genuine intersections that *do* belong in `Game`: `walk` (Player's next
  position × Map's bounds × the quest system), `search` (Player's coords ×
  Map's tile × Player's inventory × per-item logging), anything wrapping a
  model call only to log its outcome.
- Prefer deleting a `Game`-level passthrough entirely over keeping a thin
  wrapper that adds nothing — e.g. `Player::recipe_progress()` and
  `Player::available_quests()` are called directly as `game.player.foo()`
  rather than re-exposed on `Game`, matching how `Player::open_quest()`/
  `completed_quests()` already work. Only keep a `Game`-level method when it
  also needs to log or otherwise touch another model.

### Models don't reach into each other's internals

A model may hold and pass around another model's **bare identifier**
(`Player` stores `Option<QuestID>`/`Vec<QuestID>`), but not its rich type.
`Player` never sees `Quest`, `QUESTS`, `QuestCondition`, or `EventTypeID` —
those are `game::quest`'s own internals, `pub(super)` at most. Resolving a
`QuestID` into a `Quest` (name, description, reward, progress goal) is
`Game`'s job (`Game::quest(id)`), because that resolution is exactly the
kind of cross-model lookup `Game` exists for. The same principle generalizes:
before adding a field or import that lets one model's code name another
model's *type* (not just its ID), ask whether the logic should move to
`Game` instead.

## Functions live next to the structs they operate on

When logic is genuinely about one struct, its `impl` block — including
trait impls — lives in the same file as the struct, not in whichever file
happens to call it. This was the driving idea behind splitting `game.rs`
into `src/game/{player,map,item,recipe,event,quest}.rs`, and behind the
`SaveState`/`RestoreState` traits: each type's save/restore mapping is an
`impl` next to that type, not one large function elsewhere that has to
remember every field.

## Visibility: as narrow as the actual audience

Default to fully private. Widen only as far as the real caller requires:

- **`pub(super)`** — needed only by the parent module (typically `Game` in
  `game/mod.rs`) and that module's other children. This is the common case
  for cross-file-but-internal-to-`game` sharing (e.g. `Map::contains`,
  `Player::spend`, `Recipe::find_matching`).
- **`pub(crate)`** — genuinely needed elsewhere in the crate, most often
  `save.rs` (`Player::grant_recipe`, `Event::new`, `Map::from_terrain`) or
  the `SaveState`/`RestoreState` traits themselves.
- **`pub`** — real public API: consumed by `viewmodel`/`ui`, or (for
  `game`'s own DTO-adjacent types like `Quest`'s `id`/`name`/`description`)
  data meant to be read widely without a dedicated accessor for every field.

When a module split or a new caller breaks a private-field assumption,
prefer adding a narrowly-scoped accessor/mutator method over reaching for
`pub(crate)` by default — see `Player::add_to_inventory`/`inventory_count`/
`first_shortage` etc. replacing direct `HashMap` manipulation from `Game`.

## Persistence: on-disk shape is decoupled from live structs

`src/save.rs` owns an explicit DTO schema (`PlayerState`, `MapState`,
`EventState`, wrapped in `SaveFile`) rather than serializing the domain
structs directly, so the save file stays small, stable, and hand-editable
across internal refactors. Each domain type maps to its DTO via `impl
SaveState`/`impl RestoreState` colocated with that type (see "functions
live next to the structs" above); `save.rs` itself only holds the DTO
shapes and the whole-`Game` glue that doesn't belong to any single type
(version stamping, reconstructing `started` from the saved events). See
`docs/save-load.md`.

## `tools/` holds standalone dev utilities, not part of the game

If a future dev-only utility is ever heavy enough to want its own crate,
`tools/<name>/` is the place: a self-contained crate (own `Cargo.toml` +
`Cargo.lock`, empty `[workspace]` table), built and run on its own, never
shipped. It could `path`-depend on the `the_bug` library for its **public**
API (ADR 0003), but not reach crate internals — for anything private it would
re-declare the small amount it needs, with a comment pointing back. It gets
its own `prek` fmt/clippy hooks since root `fmt`/`clippy`/`test` don't reach
outside the root package.

There is currently no such tool. The first one, `tools/mapgen`, was folded
back into the crate as `src/mapgen/` once the game generated its map with the
algorithm directly (`docs/mapgen.md`) — the duplication the split had forced
(terrain letters) went away with it.

## Docs record the *why*, not just the *what*

Each `docs/*.md` file is a short, standing record for one feature/decision:
why it was needed, what changed, and — often the most useful part — what
was deliberately left out of scope. Code comments explain non-obvious local
detail; `docs/` explains why the surrounding shape exists at all. Write one
when a change is more than a small fix, before implementing it if the
design needs to be pinned down first.

## Workflow

- **New feature**: plan → a short `docs/*.md` capturing it → failing tests
  for the genuinely new logic, reviewed → implement.
- **Refactor**: confirm existing tests already cover the behavior (add
  characterization tests first if not) → change → re-run the full suite
  unmodified as proof nothing changed. Keep refactor commits separate from
  behavior-changing ones.
- **Before every commit**: `cargo fmt --all -- --check`, plus `cargo test`
  and `cargo clippy --all-targets -- -D warnings` for **both** front-end
  feature sets — the default (`gui`) and
  `--no-default-features --features tui` — since the shared `game`/
  `viewmodel`/`save`/`i18n` layers compile under both. `prek`'s hook runs
  `clippy` for both; run the `tui` `test` pass yourself when touching shared
  code. `cargo test` also builds and runs the `cucumber` functional suite
  (`docs/functional-tests.md`), which compiles under either feature set.
- **UI changes**: also run the affected front end and drive it through the
  change — rendering isn't unit-tested here, so this is the only real
  verification for layout/visual correctness. The `tui` needs a real pty
  (wrap in `screen`/`tmux`); build it with `--no-default-features --features
  tui`. The `gui` is the default build (`cargo run`).
