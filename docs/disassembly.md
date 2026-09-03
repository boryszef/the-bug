# Feature: Disassembly popup

## Why

`STORY.md` §10 names "reverse engineering — disassembling an old device reveals
components" as one of the game's discovery paths, and it was the top backlog
item. Crafting turns inputs into an output; disassembly is the reverse. The
`Recipe` struct already carries a `reversible: bool` flag.

## Outcome

`d` opens a **Disassemble popup** listing every item in the inventory that a
**reversible** recipe produces. The player scrolls with `↑`/`↓` and confirms with
`Enter` (or `d`); `Esc`/`q` closes it. On confirm, `Game::disassemble` removes
one of that item and returns the recipe's inputs to the inventory, then logs a
precise line under the **Crafting** category (yellow):

```
[03:12] You take apart a Stone Axe, recovering 1 Stick + 1 Stone + 1 Cord.
```

Components are rendered by the existing `describe_inputs`, sorted by `Item`
order.

- The list shows **any** matching item — it is **not** limited to recipes the
  player has discovered, and disassembling does **not** teach the recipe.
- Disassembly grants no XP and does not count as a craft.
- Nothing reversible in the inventory → "Nothing you're carrying can be taken
  apart."

## Scope

- `src/game.rs`:
  - `reversible_recipe_for(output: Item) -> Option<Recipe>` — the shared lookup
    (`RECIPES` entry that is `reversible` and outputs `output`).
  - `Game::disassemble(&mut self, item: Item)` — no-op if no reversible recipe
    produces `item` or the player holds none; otherwise `spend` one and add the
    inputs back via `entry().or_insert()`.
- `src/viewmodel/disassembly.rs`: `options(player) -> Vec<Item>` — the
  zero-filtered, `Item`-ordered inventory narrowed to disassemblable items.
- `src/ui/disassemble.rs`: the overlay (`Disassemble`, `Outcome`) — cursor, key
  mapping and rendering only, mirroring `ui::craft::Craft` without the
  affordability/dimming logic (every listed row is actionable).
- `src/ui/app.rs`: `disassemble: Option<Disassemble>` field, `d` opens it,
  overlay dispatch and rendering alongside the other overlays.
- `src/ui/help.rs`: `d` line → "Disassemble".

## Out of scope

- Learning a recipe by taking apart an item you don't know how to make.
- Lossy / partial component return.
- A dedicated event category or colour for disassembly.
- Taking apart more than one unit at a time, or a quantity column in the popup.
- Any change to the save format (`Game::disassemble` adds no persistent state).
