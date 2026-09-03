# Design: internationalisation (i18n) — English + Polish

**Status: planned, not implemented.** Captured for later.

## Context

All player-facing text is hard-coded English, mostly built with `format!` in
`Game` (`walk`/`search`/`craft`/`experiment` push composed sentences). Polish
can't be produced by swapping words into an English template — it needs
grammatical case (genitive/locative), gendered forms and different prepositions.

Goal: a localisation layer with **English and Polish** to start, more languages
addable later. Everything user-facing is translated. Language is chosen **at
startup** — detected from the system locale, overridable on the command line —
with **no in-game switch and no on-screen indicator**.

## Approach

### 1. `src/game.rs` — events become structured data, not strings

`Game` should describe *what happened*, not phrase it.

- New `pub enum EventKind` (`#[derive(Clone, Copy, Debug)]`):
  `Awoke`, `Walked { direction: Direction, terrain: TerrainType }`,
  `Found { material: Material, terrain: TerrainType }`,
  `UnknownRecipe { recipe: &'static str }`,
  `CraftShortage { needed: Material, output: Material }`,
  `Crafted { output: Material }`,
  `ExperimentShortage { needed: Material }`, `ExperimentFailed`,
  `RecipeDiscovered { output: Material }`, `Experimented { output: Material }`.
- `Event` holds `kind: EventKind` (+ existing `elapsed`); replace `text()` with
  `pub fn kind(&self) -> EventKind`.
- `Game::log` takes an `EventKind`; the 10 call sites push a variant instead of
  a `format!` string. `Default` seeds `EventKind::Awoke`.
- `Game::craft(&mut self, recipe: &'static str)` (was `&str`) so `UnknownRecipe`
  can keep a `&'static str` and `EventKind` stays `Copy`. Callers already pass
  `&'static str` (`CraftOption` id, test literals).
- Add `pub fn output(&self) -> Material` to `Recipe`.
- **Remove** `impl Display for Material`, `impl Display for TerrainType` (name),
  and `Direction::name()` — there is no single canonical name any more; all
  wording lives in `i18n`. Keep `TerrainType::symbol()`.

### 2. `src/i18n/` (new module, `mod i18n;` in `src/main.rs`)

| File | Contents |
|---|---|
| `src/i18n/mod.rs` | `Language` enum + detection; dispatch fns; `Ui` struct |
| `src/i18n/en.rs` | English: `event()`, `material()`, `terrain()`, `direction()`, `static UI` |
| `src/i18n/pl.rs` | Polish: same, with `material_nominative` / `material_genitive`, `terrain_locative` / `terrain_genitive` |

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language { #[default] English, Polish }

/// CLI (`--lang xx` / `--lang=xx`) wins; then $LC_ALL / $LC_MESSAGES / $LANG
/// (skipping "C"/"POSIX"/empty); else English.
pub fn detect(args: impl Iterator<Item = String>, env: impl Fn(&str) -> Option<String>) -> Language;

pub fn event(kind: EventKind, lang: Language) -> String;   // one log line
pub fn material(m: Material, lang: Language) -> &'static str; // nominative, for lists/menus
pub fn ui(lang: Language) -> &'static Ui;                   // titles, help, hints
```

`Ui` is an all-`&'static str` struct (pane titles, popup titles, `level`,
`inventory`, `inventory_empty`, the craft/experiment hint lines, the "no recipes"
line, and `help_lines: &'static [&'static str]`). One `static EN: Ui` / `static
PL: Ui`.

`en::event` / `pl::event` are a `match kind { … }` each. Polish picks the case it
needs from its own noun tables — e.g. `Found` → `format!("Znajdujesz {} {}.",
material_nominative(m), terrain_locative(t))` where `terrain_locative` bakes in
the preposition (`"w lesie"`, `"na łące"`); `CraftShortage` →
`format!("Brakuje ci {}, aby wykonać {}.", material_genitive(needed),
material_nominative(output))`.

> The Polish strings will be a first draft for a Polish speaker to review and
> correct.

### 3. `src/viewmodel/` — stays language-free, just carries structured data

- `events::recent` yields `RecentEvent { timestamp: String, kind: EventKind }`
  (no more `text`); `compact()` timestamp logic unchanged.
- `crafting::CraftOption { id: &'static str, output: Material, enabled: bool }`
  (was `name: &'static str`); `options()` fills `id = r.name()`,
  `output = r.output()`.

### 4. `src/ui/` — receives `Language`, renders through `i18n`

- `App` gains `language: Language`; add `App::new(language) -> Self`
  (`{ language, ..Default::default() }`); keep `Default` for tests.
- `render_map` / `render_player` / `render_events` / `help::render` gain a
  `Language` arg; `Experiment::render` / `Craft::render` gain one too.
  Titles/labels come from `i18n::ui(lang)`; event lines from
  `i18n::event(re.kind, lang)`; material names (`render_player`,
  `experiment::material_list`, `craft` rows) from `i18n::material(m, lang)`.
- `Craft::handle_key` returns `Outcome::Craft(option.id)`.
- No `l` key, no language line anywhere.

### 5. `src/main.rs`

```rust
fn main() -> io::Result<()> {
    let language = i18n::Language::detect(std::env::args(), |k| std::env::var(k).ok());
    ratatui::run(|terminal| ui::App::new(language).run(terminal))
}
```

### 6. Docs

Replace/extend this file with `docs/i18n.md` on implementation — how detection
works and how to add a language (new `Ui` static + `event`/`material` arms).

## Reuse / references

- `viewmodel::events::compact` (`src/viewmodel/events.rs`) — timestamp format,
  untouched.
- `Recipe::name()` / `inputs()` (`src/game.rs`) — pattern for the new `output()`.
- `viewmodel` / thin-UI convention: viewmodel keeps structured data, `i18n` owns
  wording, `ui` composes.

## Not in scope

- In-game language switching / indicator / persistence (startup-only by request;
  persistence belongs with the "save game state" backlog item).
- Translating `docs/`, `README`, `STORY.md`, commit messages.
- Localising number / `(empty)` formatting beyond the `Ui` string.
- Sorting inventory by localised name (stays enum order).

## Execution & verification

Two commits: (1) `EventKind` + `i18n` module with **English only**, output
identical to today (pure refactor); (2) add `pl.rs` + locale/CLI detection in
`main.rs`.

**Tests**
- `src/i18n/`:
  - `detect`: `--lang pl` / `--lang=pl` → Polish; `LANG=pl_PL.UTF-8` → Polish;
    `LC_ALL=C` + `LANG=pl_PL` → Polish (C skipped); nothing → English; CLI beats
    env.
  - `en::event(Found { Stick, Forest })` == `"You find a Stick in the Forest."`;
    `pl::event(Found { Stick, Forest })` == `"Znajdujesz kij w lesie."`;
    `pl::event(CraftShortage { needed: Stick, output: StoneAxe })` ==
    `"Brakuje ci kija, aby wykonać kamienny topór."`.
  - every `EventKind` variant renders non-empty in both languages (table test).
- `src/game.rs`: the walk / `test_y_axis_inversion` / experiment-shortage
  assertions switch from string matching to
  `matches!(game.events().last().unwrap().kind(), EventKind::Walked { direction: Direction::North, terrain: TerrainType::Forest })`
  etc. `material_display_names` / terrain-Display tests move to `i18n` (the
  `TerrainType::symbol` check stays in `game.rs`).
- `src/viewmodel/events.rs`: assert on `recent[0].kind` variant instead of text;
  `timestamp == "00:00"` unchanged.

**Manual**
- `cargo run` → English UI, `[00:00] You wake up and decide to go for a walk.`
- `cargo run -- --lang pl` → `[00:00] Budzisz się i postanawiasz wybrać się na
  spacer.`, pane titles `Mapa` / `Gracz` / `Zdarzenia`, `?` help in Polish,
  Craft/Experiment popups in Polish; walk/search/craft/experiment lines read
  correctly.
- `LANG=pl_PL.UTF-8 cargo run` → Polish without the flag.

**Checks**: `cargo test`, `cargo clippy --all-targets -- -D warnings`,
`cargo fmt --all -- --check`. `rg -n '"You |format!\("[A-Z]' src/game.rs
src/ui/` — no English sentences left outside `src/i18n/`.
