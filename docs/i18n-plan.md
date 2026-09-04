# Design: internationalisation (i18n) — English + Polish

**Status: planned, not implemented.** Captured for later.

> Revised from the original draft: paths had drifted (`Game` moved from
> `src/game.rs` into `src/game/mod.rs` + submodules), a couple of "add this"
> steps were already done independently, two new panels (`Disassemble`,
> `Quests`) had shipped without translation in mind, and the localisation
> engine changed from hand-written `match` tables to **Project Fluent**. See
> the callouts below for what changed and why.

## Context

All player-facing text is hard-coded English, mostly built with `format!` in
`Game` (`search`/`craft`/`experiment`/`disassemble` push composed sentences),
plus hard-coded titles/hints/labels scattered across `src/ui/*.rs` and quest
name/description text baked into `src/game/quest.rs`'s `QUESTS` table. Polish
can't be produced by swapping words into an English template — it needs
grammatical case (genitive/locative), gendered forms, different prepositions,
and CLDR-style plural rules (`one`/`few`/`many`/`other`, not just
singular/plural).

Goal: a localisation layer with **English and Polish** to start, more
languages addable later. Everything user-facing is translated. Language is
chosen **at startup** — detected from the system locale, overridable on the
command line — with **no in-game switch and no on-screen indicator**.

Localisation engine: **Project Fluent**, via the `i18n-embed` +
`i18n-embed-fl` crates (plus `rust-embed`, `unic-langid`). Fluent's **Terms
with attributes** are a direct fit for Polish grammatical case (a term per
noun, an attribute per case), and its **plural selectors** cover Polish's
plural categories properly — both would otherwise need hand-rolled Rust
logic. The `fl!` macro from `i18n-embed-fl` checks that a message id exists
in the fallback-locale `.ftl` file at *compile* time, which recovers some of
the safety a hand-written `match` would give for free (see the note on
`EventKind` losing `Copy`-exhaustiveness checks, below).

## Approach

### 1. `src/game/` — events become structured data, not strings

`Game`'s logic now lives in `src/game/mod.rs`, with `Event`/`EventCategory`
in `src/game/event.rs`, `Recipe` in `src/game/recipe.rs`, `Item` in
`src/game/item.rs`, `Direction`/`TerrainType` in `src/game/map.rs`, and
`Quest`/`QuestID` in `src/game/quest.rs`. `Game` should describe *what
happened*, not phrase it.

**`EventCategory` already exists** (`General`/`Experiment`/`Crafting`,
`src/game/event.rs`) and is used two places the original draft didn't
account for: colouring the log in `render_events` (`src/ui/app.rs`), and
persistence in `save.rs`'s `EventState`. Rather than storing it alongside a
new `EventKind`, give `EventKind` a `category(&self) -> EventCategory`
method that derives it per variant — the field goes away, `render_events`
keeps working unchanged, and there's one less thing that can drift out of
sync.

New `pub enum EventKind`, one variant per `Game::log(...)` call site (there
are 10 today) plus the seed event:

```rust
pub enum EventKind {
    Awoke,
    Found { item: Item, terrain: TerrainType },
    QuestAccepted { quest: QuestID },
    QuestCompleted { quest: QuestID },
    UnknownRecipe { recipe: String },
    CraftShortage { needed: Item, output: Item },
    Crafted { output: Item },
    ExperimentShortage { items: Vec<(Item, u32)>, missing: Item, available: u32, needed: u32 },
    ExperimentFailed { items: Vec<(Item, u32)> },
    Experimented { items: Vec<(Item, u32)>, output: Item, newly_learned: bool },
    Disassembled { item: Item },
}
```

Corrections from the original draft's variant list:
- **`QuestAccepted`/`QuestCompleted`/`Disassembled` were missing entirely** —
  `accept_quest`, `complete_open_quest` and `disassemble` each log today
  (`src/game/mod.rs`) and were left out of the original enum.
- **`RecipeDiscovered`/`Experimented` collapse into one `Experimented`
  variant with a `newly_learned: bool` field.** `experiment()` has exactly
  one success log call, not two — the "(new recipe!)" suffix is a flag on
  that call, not a separate event. The Fluent message selects on
  `newly_learned` with a select expression, the same mechanism used for
  plurals.
- **`ExperimentShortage`/`ExperimentFailed`/`Experimented` carry a runtime
  `items: Vec<(Item, u32)>`** — the player's chosen combination for that
  attempt (`ItemSelection` in `src/viewmodel/selection.rs` is an unbounded
  running pick-list, not capped at a fixed size), plus, for the shortage
  case, which item/amount was missing. **This is why `EventKind` can no
  longer derive `Copy`** as the original draft assumed — a `Vec` isn't
  `Copy`. Derive `Clone, Debug, PartialEq` instead (`Serialize`/`Deserialize`
  too, for persistence — see below). This also means `Game::craft` does
  **not** need tightening from `&str` to `&'static str` (the original
  draft's reason for that change was preserving `EventKind: Copy`, which is
  moot now); `UnknownRecipe` just takes an owned `String`.
- **`Disassembled` only stores `item: Item`, not the recovered inputs.**
  What's recovered is fully determined by `item` via
  `reversible_recipe_for(item)` (`src/game/recipe.rs:93`, already exists) —
  storing it separately would be a derivable value duplicated in the enum.
  `i18n::event` (or the Fluent message itself, if it takes the resolved list
  as an arg) recomputes it via `reversible_recipe_for(item).inputs()` at
  render time.
- **No `Walked` variant.** `walk()` does not log anything today, and
  `walk_does_not_log`/`walking_onto_the_target_terrain_completes_the_quest`
  (`src/game/mod.rs`) assert that explicitly — walking is silent by design
  (avoids log spam every step), not an oversight. Adding a `Walked` event
  would be a behaviour change, not a refactor, and would break the "output
  identical to today" requirement for commit 1 (see *Execution &
  verification*). **Leave it out**; if footstep logging is wanted later,
  that's a separate decision from this i18n pass.

Remaining steps, corrected:
- `Event` (`src/game/event.rs`) becomes `{ kind: EventKind, elapsed:
  Duration }` — no stored `text`, no stored `category`. Replace `text()`
  with `pub fn kind(&self) -> &EventKind` and add `pub fn category(&self) ->
  EventCategory` that delegates to `self.kind.category()`.
- `Game::log` takes an `EventKind`; the 10 call sites push a variant instead
  of a `format!` string. `Default` seeds `EventKind::Awoke`.
- ~~Add `pub fn output(&self) -> Item` to `Recipe`~~ — **already exists**
  (`src/game/recipe.rs:20`), nothing to do here.
- **Remove** `impl Display for Item` (`src/game/item.rs:28`) and `impl
  Display for TerrainType` (`src/game/map.rs:56`) — there is no single
  canonical name any more; all wording lives in `i18n`. Keep
  `TerrainType::symbol()` (the map glyph, unrelated to naming).
  ~~and `Direction::name()`~~ — **doesn't exist**; `Direction` only has
  `delta()` (`src/game/map.rs:21`), so there's nothing to remove there.
  Direction wording (for a future footstep/compass feature) would be new
  i18n surface, not a replacement.
- Add `Serialize, Deserialize` derives to `Direction` and `TerrainType`
  (`src/game/map.rs`) — neither has them today, and `EventKind` needs to
  derive `Serialize`/`Deserialize` for save-file persistence (see below).
  `Item` and `QuestID` already derive both.

**Save-file impact (not addressed in the original draft at all).**
`save.rs`'s `EventState { category, text, elapsed_secs }` currently stores
already-rendered English text. Once `Event` holds `EventKind` instead, this
DTO must serialize the structured `EventKind` directly (e.g. serde's default
externally-tagged enum representation), not a category/text pair. This is a
**deliberate save-format break**: old save files' `events` array won't
deserialize against the new shape, and `load()` will surface that as the
`io::ErrorKind::InvalidData` it already returns for malformed JSON — no
compatibility shim, no migration path. Acceptable because this is a local,
single-player save file with no versioning guarantee today. Two existing
`save.rs` tests assert on the old shape and need to change or be deleted,
not left silently passing against a shape that no longer exists:
`event_without_category_defaults_to_general` and
`load_reads_a_hand_edited_file` (both construct hand-written JSON with a
`"text"` field).

### 2. `src/i18n/` (new module, `mod i18n;` in `src/main.rs`)

Fluent via `i18n-embed` + `i18n-embed-fl`, not hand-written `match` tables.

| File | Contents |
|---|---|
| `src/i18n/mod.rs` | `Language` enum + detection; `RustEmbed`-derived `Localizations` over `locales/`; a `fluent_language_loader!`; thin wrapper fns |
| `src/i18n/locales/en/main.ftl` | English messages/terms |
| `src/i18n/locales/pl/main.ftl` | Polish messages/terms |

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language { #[default] English, Polish }

/// CLI (`--lang xx` / `--lang=xx`) wins; then $LC_ALL / $LC_MESSAGES / $LANG
/// (skipping "C"/"POSIX"/empty); else English.
pub fn detect(args: impl Iterator<Item = String>, env: impl Fn(&str) -> Option<String>) -> Language;

pub fn event(kind: &EventKind, lang: Language) -> String;         // one log line
pub fn item(item: Item, lang: Language) -> String;                 // nominative, for lists/menus
pub fn quest_name(id: QuestID, lang: Language) -> String;
pub fn quest_description(id: QuestID, lang: Language) -> String;
pub fn ui(id: &str, lang: Language) -> String;                     // one-off titles/hints/labels
pub fn ui_args(id: &str, lang: Language, args: &FluentArgs) -> String; // labels with interpolation
```

No `Ui` struct any more — Fluent message ids already are the "one string per
concept" registry the original draft's struct was standing in for; a
parallel Rust struct would just duplicate that mapping. Panel code calls
`i18n::ui("craft-title", lang)` etc. directly.

Message id scheme, kebab-case, one per `EventKind` variant: `event-awoke`,
`event-found`, `event-quest-accepted`, `event-quest-completed`,
`event-unknown-recipe`, `event-craft-shortage`, `event-crafted`,
`event-experiment-shortage`, `event-experiment-failed`, `event-experimented`,
`event-disassembled`.

Grammatical case via Fluent **Terms with attributes** — one term per noun,
attributes for the cases Polish needs:

```ftl
-item-stick = Patyk
    .genitive = patyka
-terrain-forest = Las
    .locative = w lesie

event-found = Znajdujesz { -item-stick } { -terrain-forest.locative }.
```

referenced from messages as `{ -item-stick }` / `{ -item-stick.genitive }`.
English defines the same term ids with no case attributes (nothing reads
them). **Implementation note, not resolved by this doc:** Fluent Terms are
referenced *from* messages, not fetched standalone through the typical
`Loader`/`fl!` API — so `i18n::item(item, lang)` (used directly by menus/
inventory lists, not embedded in a sentence) should itself resolve a tiny
per-item *message* (`item-stick = { -item-stick }`), not try to fetch a Term
by id directly. Confirm this against `i18n-embed`'s actual API when
implementing.

Quantities (the old `describe_inputs()` output, inventory `"{item}
{quantity}"` lines) become Fluent messages with a `$count` arg and CLDR
plural selectors (Polish: `one`/`few`/`many`/`other`) instead of hand-rolled
pluralization — there was none before because English rarely needed it, but
Polish does.

Quest text: `src/game/quest.rs`'s `QUESTS` table hardcodes `name`/
`description` as English `&'static str`, read directly by
`src/ui/quests.rs` today — this was entirely missing from the original
draft. Add per-quest message ids (e.g. `quest-craft-arrows-name`,
`quest-craft-arrows-description`, mirroring `QuestID`'s variants), fetched
through the new `i18n::quest_name`/`quest_description`.

> The Polish `.ftl` content will be a first draft for a Polish speaker to
> review and correct.

### 3. `src/viewmodel/` — stays language-free, just carries structured data

- `events::recent` (`src/viewmodel/events.rs`) yields `RecentEvent {
  timestamp: String, kind: &EventKind }` instead of `{ category, text }`
  (`category` drops out the same way it does on `Event` — callers derive it
  from `kind.category()` if they need it for colouring); `compact()`
  timestamp logic unchanged.
- `crafting::CraftOption` (`src/viewmodel/crafting.rs`) — today it's `{
  name: &'static str, enabled: bool }`; change to `{ id: &'static str,
  output: Item, enabled: bool }` (`id = recipe.name()`, `output =
  recipe.output()`). `id` stays the lookup key passed to `Game::craft`;
  display text comes from `i18n::item(option.output, lang)`.
- `disassembly::options`/`quests::overview` (`src/viewmodel/disassembly.rs`,
  `src/viewmodel/quests.rs`) — untouched structurally, they already return
  `Item`s and `&'static Quest`s rather than pre-rendered text; the `ui`
  layer is what needs to stop reading `Quest::name`/`description` directly
  and go through `i18n::quest_name`/`quest_description` instead.

### 4. `src/ui/` — receives `Language`, renders through `i18n`

- `App` (`src/ui/app.rs`) gains `language: Language`. There is no
  `App::new` today — the real constructor is `App::with_game(game: Game) ->
  Self`; change it to `App::with_game(game: Game, language: Language) ->
  Self` and update its one call site in `src/main.rs`. Keep `Default` for
  tests (defaulting `language` to `Language::English`).
- `render_map` / `render_player` / `render_events` / `render_footer`
  (`src/ui/app.rs`) gain a `Language` arg. There is no `help` module —
  `render_footer` (private, `src/ui/app.rs`) is the closest thing to the
  original draft's `help::render`; it's the only place hint text is drawn.
- `Craft::render`/`handle_key` (`src/ui/craft.rs`) and
  `Experiment::render` (`src/ui/experiment.rs`) gain a `Language` arg, as
  the original draft had. **Also newly in scope, missed by the original
  draft**: `Disassemble::render`/`handle_key` (`src/ui/disassemble.rs`) and
  `Quests::render`/`handle_key` (`src/ui/quests.rs`) — both panels shipped
  after the original draft was written and have the same hard-coded
  titles/empty-state text/hints (e.g. "Nothing you're carrying can be taken
  apart.", "No quest accepted.") that need the same treatment.
- Titles/labels/hints come from `i18n::ui`/`ui_args`; event lines from
  `i18n::event(kind, lang)`; item names (`render_player`, `experiment`
  columns, `craft`/`disassemble` rows) from `i18n::item(item, lang)`; quest
  name/description in `quests.rs` from `i18n::quest_name`/
  `quest_description`.
- `Craft::handle_key` returns `Outcome::Craft(option.id)` once
  `CraftOption::name` is renamed to `id` (see §3) — today it's still
  `Outcome::Craft(option.name)`.
- No language-switch key, no language line anywhere.

### 5. `src/main.rs`

`main.rs` already parses a `clap` `Cli` with `--load <FILE>` — add `--lang`
alongside it rather than replacing the whole function:

```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, value_name = "FILE")]
    load: Option<PathBuf>,
    #[arg(long, value_name = "LANG")]
    lang: Option<String>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let language = i18n::Language::detect(std::env::args(), |k| std::env::var(k).ok());
    // ...existing --load / Game::default() branch...
    let mut app = ui::App::with_game(game, language);
    // ...existing run/save flow, unchanged...
}
```

(`Language::detect` re-parses `std::env::args()` rather than reading
`cli.lang` directly, to keep the `--lang xx` / `--lang=xx` parsing contract
from the original draft's spec self-contained in `i18n::detect`'s own
tests — reconcile this against how `clap` actually exposes `--lang` when
implementing; a `cli.lang` value could be threaded in instead if that proves
simpler.)

### 6. Docs

Replace/extend this file with `docs/i18n.md` on implementation — how
detection works and how to add a language (new `locales/<lang>/main.ftl` +
whatever message ids are missing, caught by the table test below).

## Reuse / references

- `viewmodel::events::compact` (`src/viewmodel/events.rs`) — timestamp
  format, untouched.
- `Recipe::name()` (`src/game/recipe.rs`) — `output()` already exists, no
  new step needed.
- `reversible_recipe_for` (`src/game/recipe.rs:93`) — already used by
  `disassemble()`/`viewmodel::disassembly`; reused again to derive
  `Disassembled`'s recovered-items text at render time instead of storing it.
- `src/game/quest.rs`'s `QUESTS` table — pattern for the new `QuestID` →
  Fluent message id mapping.
- `viewmodel` / thin-UI convention (see `CLAUDE.md` → `ARCHITECTURE.md`):
  viewmodel keeps structured data, `i18n` owns wording, `ui` composes.

## Not in scope

- In-game language switching / indicator / persistence (startup-only by
  request; persistence belongs with the "save game state" backlog item).
- Translating `docs/`, `README`, `STORY.md`, commit messages.
- Localising number / `(empty)` formatting beyond what a Fluent message
  handles directly.
- Sorting inventory by localised name (stays enum order).
- Footstep/walk logging (`EventKind::Walked` was dropped — see §1).

## Execution & verification

Two commits: (1) `EventKind` + `i18n` module with **English `.ftl` only**,
output identical to today (pure refactor — this is exactly why `Walked` was
dropped from scope, see §1); (2) add `locales/pl/main.ftl` + locale/CLI
detection in `main.rs`.

**Tests**
- `src/i18n/`:
  - `detect`: `--lang pl` / `--lang=pl` → Polish; `LANG=pl_PL.UTF-8` →
    Polish; `LC_ALL=C` + `LANG=pl_PL` → Polish (`C` skipped); nothing →
    English; CLI beats env.
  - `event(&Found { item: Stick, terrain: Forest }, English)` ==
    `"You find a Stick in the Forest."`; the Polish equivalent renders with
    the term's locative attribute; `event(&CraftShortage { needed: Stick,
    output: StoneAxe }, Polish)` reads correctly with genitive/nominative
    forms.
  - **Table test**: every `EventKind` variant, constructed with
    representative field values, × both `Language`s produces a non-empty,
    successful Fluent lookup. This is the safety net standing in for the
    `match`-exhaustiveness the original hand-written design got for free
    from the compiler — a missing Fluent key for a real variant no longer
    fails to compile, so this test is load-bearing, not optional.
- `src/game/mod.rs`: event-content assertions switch from string matching to
  `matches!(game.events().last().unwrap().kind(), EventKind::CraftShortage
  { needed: Item::Vine, output: Item::Cord })` etc. `item_display_names`
  (`src/game/item.rs`) and the terrain-`Display` test
  (`terrain_type_display_is_name_and_symbol_is_glyph`, `src/game/map.rs`)
  are removed along with the `Display` impls they test (naming moves to
  `i18n`); the `TerrainType::symbol` half of that test stays.
- `src/save.rs`: `event_without_category_defaults_to_general` and
  `load_reads_a_hand_edited_file` need rewriting against the new
  `EventState` shape (or deletion, if the scenario they cover — an
  old-format save loading cleanly — is no longer true by design); add a
  round-trip test confirming a **new** save/load cycle preserves `EventKind`
  correctly, and a test confirming an **old-format** save file fails to load
  with `io::ErrorKind::InvalidData` rather than silently misbehaving.
- `src/viewmodel/events.rs`: assert on `recent[0].kind` variant instead of
  `text`; `timestamp == "00:00"` unchanged.

**Manual**
- `cargo run` → English UI, `[00:00] You wake up and decide to go for a
  walk.`
- `cargo run -- --lang pl` → `[00:00] Budzisz się i postanawiasz wybrać się
  na spacer.`, pane titles `Mapa` / `Gracz` / `Zdarzenia`, footer hints in
  Polish, Craft/Experiment/**Disassemble/Quests** panels in Polish
  (including quest name/description text) — the last two panels weren't
  covered by the original draft's manual pass.
- `LANG=pl_PL.UTF-8 cargo run` → Polish without the flag.
- Load an old (pre-refactor) save file with `--load` → fails with a clear
  error, doesn't crash or silently drop the event log.

**Checks**: `cargo test`, `cargo clippy --all-targets -- -D warnings`,
`cargo fmt --all -- --check`. `rg -n '"You |format!\("[A-Z]' src/game/
src/ui/` — no English sentences left outside `src/i18n/`.
