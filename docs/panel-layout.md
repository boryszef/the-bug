# Feature: cycled panels instead of popups

## Why

The UI used four independent overlay flags (`show_help: bool`,
`experiment`/`craft`/`disassemble: Option<T>`) gating centered popups drawn
on top of an always-visible Map/Player/Events screen, with hand-written
mutual exclusivity via early returns. `docs/craft-popup.md` already flagged
consolidating these into one enum as deferred tech debt.

The user wants popups gone: Player + Events stay always visible on the
left; the right side cycles through full panels instead of popping
overlays on top of a fixed Map view.

## Behaviour

- Left column (Player panel, Events panel) is unchanged and always visible.
- Right column shows exactly one of five panels at a time: **Map**,
  **Experiment**, **Craft**, **Disassemble**, **Quests** — in that order.
- `]` cycles to the next panel, `[` to the previous, wrapping at both ends.
- A one-line footer at the bottom of the screen shows the key hints for
  whichever panel is active, replacing the old Help popup and the `?` key
  entirely (there is no help screen/panel anymore — the footer *is* help).
- `q` quits immediately, from any panel.
- Movement (arrow keys), search (`s`) and hunt (`h`) only respond while the
  Map panel is active — arrows are needed for list navigation on the other
  panels.
- Craft/Disassemble/Experiment keep their exact existing interaction
  (cursor movement, Experiment's `Tab`-based Available/Selected column
  focus, confirm keys) unchanged — only *how* they're framed on screen
  changes (filling their panel instead of a centered popup). Their state
  (cursor position, Experiment's in-progress selection) now persists across
  cycling away and back, since panels aren't opened/closed anymore.
- Quests panel shows the active quest's progress (or an "Available" list,
  navigable with `↑↓`/`Enter` to accept, while none is open) and completed
  quests — wired to `Game::available_quests()`/`accept_quest()`
  (`docs/quest-system.md`). `↑↓`/`Enter` are no-ops while a quest is active,
  since only one can be open at a time.

## Design

- `Panel` enum (`src/ui/app.rs`): `Map` (default), `Experiment`, `Craft`,
  `Disassemble`, `Quests`, with `next()`/`prev()` wrapping through that
  order.
- `App` replaces `show_help`/`Option<Craft>`/`Option<Disassemble>`/
  `Option<Experiment>` with `panel: Panel` and plain (non-`Option`)
  `craft: Craft`, `disassemble: Disassemble`, `experiment: Experiment`
  fields — always constructed, shown/hidden by `panel`, not
  created/destroyed.
- `src/ui/mod.rs`'s `centered_rect`/`popup_frame` (which centered a rect and
  `Clear`-ed it over an already-drawn screen) are replaced by
  `panel_frame(area, title, buf) -> Rect`, which draws a titled bordered
  block filling the *given* area — panels now own their whole region, so
  there's nothing underneath to clear.
- `craft.rs`/`disassemble.rs`/`experiment.rs`: only the `render()` call to
  `popup_frame` changes (to `panel_frame`, no percentage sizing). Their
  `handle_key`/`Outcome` and all existing unit tests are untouched.
- `help.rs` is deleted; its content becomes per-panel strings in `app.rs`'s
  new footer-rendering function.
- `quests.rs` (new) follows the same `State`/`Outcome`/`handle_key`/`render`
  shape as `craft.rs`/`disassemble.rs`: owns only a cursor over the
  "available to accept" list; `Outcome::Accept(QuestID)` drives
  `Game::accept_quest`. Quest data comes from
  `viewmodel::quests::overview(&Game)` (new — `Overview { active:
  Option<ActiveQuest>, available, completed }`, all `&'static Quest`).
  `Quest` gained a `goal()` accessor and `Game` gained
  `quest(id) -> &'static Quest` to support this without exposing
  `QuestCondition`/`EventTypeID`/`QUESTS` outside `game::quest`. (`Quest`
  also briefly had `reward_xp()`/`reward_items()` accessors; the panel never
  displayed rewards, so they were later removed as unused.)

## What is *not* built here

- No change to `craft`/`disassemble`/`experiment`'s own state machines,
  key bindings, or tests.
- No UI for locked quests (unmet dependencies) or quest rewards — see
  `docs/quest-system.md`.

## Code

- `src/ui/app.rs` — `Panel`, `App`, key dispatch, layout/render, footer.
- `src/ui/mod.rs` — `panel_frame` (replaces `centered_rect`/`popup_frame`).
- `src/ui/{craft,disassemble,experiment}.rs` — one-line `render()` change
  each.
- `src/ui/quests.rs` — new, interactive panel (accept a quest).
- `src/viewmodel/quests.rs` — new, `Overview`/`ActiveQuest`/`overview()`.
- `src/game/quest.rs`, `src/game/mod.rs` — `Quest::goal`, `Game::quest(id)`.
- `src/ui/help.rs` — deleted.
