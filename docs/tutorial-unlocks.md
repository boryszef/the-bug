# Feature: tutorial-style progressive tab unlocks

## Why

The GUI showed all six tabs (Map, Experiment, Craft, Disassemble, Items,
Quests) from the very first frame, which can overwhelm a new player before
they've learned any of the underlying mechanics. Instead, the game now
reveals one tab at a time as the player progresses through the opening
quests — each newly-visible tab lines up with the mechanic the quest that
unlocked it just taught. This needed a central "what has the player
unlocked" registry, since future milestones (more quests, possibly leveling
— see `docs/progression.md`) will keep adding to it.

## Design

`Unlocked` (`src/game/unlock.rs`) is a plain enum — one variant per
gate-able tab (`Map`, `Items`, `Craft`, `Disassemble`, `Experiment`). No
`Quests` variant: that tab is never gated, since it's the only thing a
brand-new player can interact with.

`Player.unlocked: Vec<Unlocked>` stores the set, checked with `.contains()`
— the same shape already used for `quests_completed: Vec<QuestID>` and
`recipes: Vec<Recipe>`. A `HashMap<Unlocked, bool>` or `HashSet<Unlocked>`
would work too, but nothing in `src/` uses either pattern for an enum-keyed
membership set, and the list stays small (five entries, ever), so there's
no reason to depart from the existing idiom. `Player::unlock(feature)` is
idempotent (a no-op if already present); `is_unlocked(feature)` and
`unlocked()` are the read side.

Each quest declares what it unlocks as static data on the `Quest` struct —
`unlocks_on_accept: &'static [Unlocked]` and
`unlocks_on_complete: &'static [Unlocked]` — the same style already used for
`dependencies`/`reward_xp`/`reward_items`. `Game::accept_quest` and
`Game::complete_open_quest` each loop their respective list and call
`player.unlock(...)`; neither method has any per-quest special-casing. This
keeps the unlock schedule readable in one place (the `QUESTS` table in
`src/game/quest.rs`) instead of scattered through `match quest.id { ... }`
branches, and means a future quest earns its own onboarding beat just by
filling in those two fields.

The current schedule: accepting "The Digital Civilization" (`ExploreRuins`)
unlocks Map; accepting "What the Ruins Kept" (`OldCivilization`, which
depends on `ExploreRuins`) unlocks Items — this is the quest that teaches
`search`, and Items is where the player sees what they've found; accepting
"Trouble in the East" (`CraftAxe`, which depends on `OldCivilization`)
unlocks Experiment; *completing* it unlocks Craft and Disassemble (by then
the player has actually discovered the Stone Axe recipe via Experiment, so
being able to craft it directly and take things apart both make sense).
"Stock Up for Hard Times" (`StockUp`) unlocks nothing further today.

`viewmodel::panel::Panel` gains `required_unlock()` (private — the
Panel-to-Unlocked mapping), `is_visible(unlocked)`, and
`visible(unlocked) -> Vec<Panel>` (the tab bar's actual contents).
`next`/`prev` take the player's `unlocked` slice and only land on visible
tabs — implemented by walking the full `ALL` array from the current panel
and skipping locked candidates, rather than indexing into the filtered
`visible` list directly. The latter would make "the current panel is always
in the set we're indexing" true only by convention (every `self.panel =
...` call site in `gui/mod.rs` happening to stay inside `visible`) instead
of true by construction, and a future edit could turn that into a silent
panic. `Panel::default()` moved from `Map` to `Quests`, since nothing else
is unlocked at game start.

## What is *not* built here

- No per-action gating within a tab — e.g. `search` itself isn't gated,
  only the tabs that surface its results. `OldCivilization` teaches search
  by requiring a specific find, but that's ordinary quest-condition
  plumbing (`EventTypeID::FindItem`, fired from `Game::search()`); it
  didn't need a new kind of gate, only a new `Unlocked` variant (`Items`)
  for what it reveals.
- No gating of the always-visible left column (player stats, event feed) or
  the theme/font-size controls — those aren't tabs, and cutting them off
  would remove feedback a new player still needs.
- Unlocking a feature does not add an entry to the on-screen event log
  (`EventKind`) — a new tab appearing is its own visible signal. This was a
  deliberate choice, not an oversight; revisit if playtesting shows the
  moment isn't noticeable enough.
- A returning player who has unlocked everything still opens on the Quests
  tab each launch — `panel` was never persisted (`App::panel` is
  session-only, like it always was), and this feature didn't change that.
- The `Unlocked` variants line up with today's `QuestID`s, but the two enums
  aren't otherwise coupled — nothing stops a future non-quest milestone
  (e.g. a level threshold) from calling `player.unlock(...)` directly from
  `Game::grant_award` the same way quest accept/complete do.

## Code

- `src/game/unlock.rs` — `Unlocked`.
- `src/game/player.rs` — `Player.unlocked`, `unlocked()`, `is_unlocked()`,
  `unlock()`, `restore_unlocked()`, `save_state`/`restore_state`.
- `src/game/quest.rs` — `Quest.unlocks_on_accept`/`.unlocks_on_complete`,
  populated in the `QUESTS` table.
- `src/game/mod.rs` — the unlock loops in `accept_quest`/
  `complete_open_quest`; `Game::search()` firing
  `EventTypeID::FindItem(item)` for the `OldCivilization` quest.
- `src/game/map.rs` — `Map::guarantee_find`, a `pub` test-support method
  that forces a guaranteed find so a `search()`-gated quest can be driven
  from a functional test (see `docs/functional-tests.md`).
- `src/save.rs` — `PlayerState.unlocked` (`#[serde(default)]`).
- `src/viewmodel/panel.rs` — `Panel::required_unlock`/`is_visible`/
  `visible`, gated `next`/`prev`/`cycle`, `Panel::default() = Quests`.
- `src/gui/mod.rs` — tab bar and `[`/`]` keybindings read
  `Panel::visible`/gated `next`/`prev`.
- `tests/features/quests.feature` + `tests/steps/quests.rs`/`world.rs` — the
  accept/complete unlock scenarios, and the `feature(name)` name→`Unlocked`
  lookup.
