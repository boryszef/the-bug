# Feature: Disassembly popup

## Why

`STORY.md` §10 names "reverse engineering — disassembling an old device reveals
components" as one of the game's discovery paths, and it was the top backlog
item. Crafting turns inputs into an output; disassembly is the reverse.

### Recipe flow (`RecipeFlow`)

A `Recipe` declares a `flow: RecipeFlow`, exposed as two predicates:

| `RecipeFlow` | `craftable()` | `disassemblable()` | example |
|---|---|---|---|
| `CraftOnly` | ✓ | — | Arrow, Cord, Coil |
| `Both` | ✓ | ✓ | Wooden Bow, Stone Axe, Metal Detector, Solar Charger |
| `DisassembleOnly` | — | ✓ | Umbrella, Electronic Toy |

`DisassembleOnly` recipes are **scavenged old-world objects** — you can take a
found one apart, but its parts don't reassemble into it. This started as a
single `reversible: bool` (`false` ≙ `CraftOnly`, `true` ≙ `Both`); the third
state was added so `experiment([battery, speaker])` can't conjure an "Electronic
Toy". Consequence: taking apart a found **Umbrella** is currently the only
source of `Fabric` and `Pole`.

`Game::craft` and experiment discovery (`find_matching`) only ever touch
`craftable()` recipes; `Game::disassemble` and the disassembly menu only touch
`disassemblable()` ones. `Player::grant_recipe` refuses a non-craftable name, so
a recipe in `Player::known_recipes()` is always craftable, and the
`Recipes: known/total` ratio counts only craftable recipes as the total.

## Outcome

`d` opens a **Disassemble popup** listing every inventory item that some
`disassemblable()` recipe produces. The player scrolls with `↑`/`↓` and confirms
with `Enter` (or `d`); `Esc`/`q` closes it. On confirm, `Game::disassemble`
removes one of that item and returns the recipe's inputs to the inventory, then
logs a precise line under the **Crafting** category (yellow):

```
[03:12] You take apart a Stone Axe, recovering 1 Stick + 1 Stone + 1 Cord.
```

Components are rendered by the existing `describe_items`, sorted by `Item`
order.

- The list shows **any** matching item — it is **not** limited to recipes the
  player has discovered, and disassembling does **not** teach the recipe.
- Disassembly grants no XP and does not count as a craft.
- Nothing in the inventory can be taken apart → "Nothing you're carrying can be
  taken apart."

## Scope

- `src/game/recipe.rs`:
  - `RecipeFlow` + `Recipe::craftable()` / `Recipe::disassemblable()`.
  - `disassembly_for(output: Item) -> Option<Recipe>` — the shared lookup
    (`RECIPES` entry that is `disassemblable()` and outputs `output`).
  - `find_matching` gated on `craftable()`.
- `src/game/mod.rs` `Game::disassemble(&mut self, item: Item)` — no-op if no
  recipe lets `item` be taken apart or the player holds none; otherwise `spend`
  one and add the inputs back.
- `src/game/player.rs` — `grant_recipe` refuses non-craftable names;
  `recipe_progress` total = craftable count.
- `src/viewmodel/disassembly.rs`: `options(player) -> Vec<Item>` — the
  zero-filtered, `Item`-ordered inventory narrowed to disassemblable items.
- `src/viewmodel/disassembly.rs`: `options(player) -> Vec<Item>` — the
  zero-filtered, `Item`-ordered inventory narrowed to disassemblable items.
- `src/{tui,gui}/disassemble.rs`: the panel — cursor / click and rendering
  only, mirroring `craft` without the affordability/dimming logic (every listed
  row is actionable). (`src/ui/…` names above predate the front-end split.)

## Out of scope

- Learning a recipe by taking apart an item you don't know how to make (the
  fuller `STORY.md` §10 reverse-engineering-as-discovery is still future work).
- Lossy / partial component return.
- A dedicated event category or colour for disassembly.
- Taking apart more than one unit at a time, or a quantity column in the popup.
- Any change to the save format (`Game::disassemble` adds no persistent state).
