# Feature: village-gated crafting, experimenting, disassembly

## Why

`TODO.md` named this rule before it existed: "crafting/experimenting/
disassembly should only be possible at the village." Until now
`Game::craft`/`experiment`/`disassemble` acted from anywhere on the map — no
location mattered. This ties those three actions to a place, so the Village
(and, later, workshops — see "Out of scope") becomes somewhere the player has
a reason to return to, rather than a pure spawn point.

## Behaviour

`Game::at_craftable_location(&self) -> bool` is the single check: the tile
under `Player.coordinates` has `poi == Some(Poi::Village)`
(`self.map.get_tile(self.player.coordinates).and_then(|tile| tile.poi)`).
Cross-model (`Player` × `Map`), so it lives on `Game`, not either model alone —
same reasoning as every other action method.

Each of `craft`, `experiment`, `disassemble` checks it first — before any
recipe/inventory specifics — and does nothing if it fails, same as
`disassemble`'s two pre-existing guards (unknown disassembly recipe, item not
held):

- **`craft`**: before `find_known_recipe`. Away from the village, an unknown
  recipe name is still just a no-op — the location gate is more fundamental
  than the recipe's validity.
- **`experiment`**: after the existing `items.is_empty()` no-op check
  (unchanged — nothing to attempt regardless of location) but before
  `first_shortage`.
- **`disassemble`**: before its `disassembly_for(item)` lookup.

**No event is logged for this**, by design — see `docs/event-worthiness.md`:
an event is a record of something the game produced (a find, a craft, a
quest completing), not a reaction to the player's own doing. Being away
from the village when you click Craft isn't something that happened *to*
you; walk back and click again. At the time this feature shipped,
`CraftMissingTool`/`CraftShortage`/`UnknownRecipe`/`HuntUnprepared` predated
this principle and were left unaffected, folded into `TODO.md`'s "review
events" note as follow-up work; that follow-up has since landed and removed
all four (`docs/event-worthiness.md`).

### gui: the buttons themselves are disabled

Away from the village, the Craft/Experiment/Disassemble tabs disable their
action buttons (each recipe's Craft button, the Run button, each item's
Disassemble button) and show a standing hint (`village-required-hint`)
explaining why — so there's no enabled-looking button that quietly refuses on
click. Concretely: `at_village` (from `Game::at_craftable_location`) is
`&&`-ed into the existing enabled expression at each button
(`option.enabled && at_village`, `!self.selection.is_empty() && at_village`,
`at_village` alone for a Disassemble row) rather than becoming part of
`CraftOption::enabled` itself — the viewmodel's `enabled` still describes
recipe-affordability only, and the two are combined at the point each front
end actually draws a button. Picking items into the Experiment selection
stays enabled everywhere, since selecting has no effect until Run is
clicked. `tui` (frozen) gets neither the hint nor the disabling — clicking
there just does nothing now, matching its existing plain style for a no-op
guard.

## Scope

- `src/game/mod.rs` — `Game::at_craftable_location`; the silent guard in
  `craft`, `experiment`, `disassemble`.
- `src/i18n/mod.rs` + `locales/{en,pl}/main.ftl` — `village-required-hint`
  (the gui's standing hint; not an `event-*` id — this isn't an event).
- `src/gui/craft.rs`, `src/gui/experiment.rs`, `src/gui/disassemble.rs` — the
  `at_village: bool` parameter, the hint label, and disabling each button.
- `src/gui/mod.rs` — computes `at_village` once per relevant `Panel` arm and
  threads it through.

## Out of scope

- **Workshops.** `at_craftable_location`'s doc comment flags this
  deliberately: a future workshop POI extends the `matches!` (or becomes an
  `Option<Poi>` allow-list) without touching the three call sites or the
  gui's button-disabling. Not built now — no workshop POI exists yet.
  `TODO.md` carries the follow-on.
- **Broader event-system review.** `TODO.md`'s "review events" note was a
  wider epic (event lines should only ever record game-originated results,
  never a reflection of the player's own action) that this feature honored
  for its own new check without otherwise acting on it yet. That follow-up
  has since landed — see `docs/event-worthiness.md`.
