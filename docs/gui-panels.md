# Feature: Experiment / Craft / Disassemble / Quests panels in the gui

## Why

`docs/gui-frontend.md` brought the egui front end up to a working Map tab
with tab switching, but the other four tabs (`Experiment`, `Craft`,
`Disassemble`, `Quests`) still render a placeholder heading. This fills them
in so `gui` reaches parity with `tui`'s panels.

## Shape

`tui`'s interactive panels are a keyboard state machine: a `State` struct
holding a cursor/focus, a `handle_key` returning an `Outcome`, and a
`render`. egui is immediate-mode and mouse-driven, so the equivalent is much
smaller — there is no cursor to track. Each panel is a `render` function
(one module per tab, mirroring `src/tui/`) that:

- takes the same `viewmodel` data and `Language` its `tui` counterpart does,
  never a `&Game` (per `ARCHITECTURE.md` layering), and
- returns an action value (`Option<…>` / a small `Outcome` enum) that
  `App::ui` applies to `game` — the same split `tui` uses.

Only **Experiment** carries state (an `ItemSelection`, exactly as
`tui::experiment` does), held as a field on `App`. The other three are
stateless. The **Map** tab follows the same return-an-action split —
`MapView::ui` returns `Option<MapCommand>` (`Walk`/`Search`/`Hunt`) from its
on-screen `←↑↓→` + Search + Hunt buttons or the arrow/`s`/`h` accelerators.

The panel bodies are egui rendering, which — like `render_player` /
`render_events` / the Map tab — is not unit-tested here (`ARCHITECTURE.md`,
"UI changes"): the testable logic they lean on (`ItemSelection`,
`viewmodel::crafting/disassembly/quests`) already has its own coverage.
Verification is running the app.

## Behaviour

### Experiment (`src/gui/experiment.rs`)
- Two columns, **Available** and **Selected**. Available lists every
  inventory item with its still-addable count (`owned` − already picked);
  a row is a button, disabled at count 0, that moves one unit right.
  Selected lists picked items; clicking a row returns one unit.
- A **Run experiment** button (disabled while nothing is selected) runs
  `Game::experiment` with the selection and clears it.
- State (`ItemSelection`) persists while switching tabs, like `tui`.

### Craft (`src/gui/craft.rs`)
- A button per known recipe, labelled with the output item; recipes the
  player can't currently afford are disabled
  (`viewmodel::crafting::CraftOption::enabled`). Click → `Game::craft`.
- Empty state: `craft-empty`.

### Disassemble (`src/gui/disassemble.rs`)
- A button per carried item that can be taken apart
  (`viewmodel::disassembly::options`). Click → `Game::disassemble`.
- Empty state: `disassemble-empty`.

### Quests (`src/gui/quests.rs`)
- **Active**: the open quest's name, `progress/goal`, and description; or
  `quests-active-none`.
- **Available**: while a quest is open, `quests-active-blocked`; otherwise a
  row per available quest — a collapsing section (header = name, body =
  description) with an **Accept** button → `Game::accept_quest`. Empty:
  `quests-available-empty`.
- **Completed**: `quests-completed` / `quests-completed-none`, same as
  `tui`.

## What is *not* built here

- No keyboard navigation inside these panels (no cursor) — `[`/`]` tab
  cycling and Map-tab movement are unchanged; panel actions are mouse-only.
- No new game logic — every action goes through an existing `Game` method.
- No confirmation dialogs, no undo, no per-recipe "what's missing"
  breakdown (that `TODO.md` item is separate).
- `tui` is untouched.

## Code

- `src/gui/{experiment,craft,disassemble,quests}.rs` — one `render` per tab.
- `src/gui/mod.rs` — `mod` declarations, `App::experiment` field, the
  per-`Panel` arms in `App::ui`'s central panel.
- `src/i18n/locales/{en,pl}/main.ftl` — `action-experiment`, `action-accept`
  (gui-only button labels); existing `*-empty` / `panel-*-title` strings
  reused.
- `docs/gui-frontend.md` — Progress list updated per tab.
