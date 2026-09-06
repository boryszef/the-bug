# Feature: Craft popup

## Why

The `c` key called `game.craft("Cord")` directly — a hardcoded stopgap from when
the experiment popup was built (see `refactor-thin-ui.md`). The crafting backend
(`Game::craft`) is complete; only the UI was missing.

## Outcome

`c` opens a **Craft popup** listing the recipes the player has discovered
(`Player::known_recipes`). The player scrolls with `↑`/`↓` and confirms with
`Enter` (or `c`); `Esc`/`q` closes it. The picked recipe name is handed to
`Game::craft`, which subtracts the consumables, adds the output, and logs the
result.

Recipes the player cannot currently afford are shown **dimmed and are not
selectable**. Whether a recipe is affordable is decided in
`viewmodel::crafting::options`, not in the UI:

```rust
pub struct CraftOption { pub name: &'static str, pub enabled: bool }
pub fn options(player: &Player) -> Vec<CraftOption>
```

`ui::craft::Craft` owns only the cursor, key mapping, and rendering — mirroring
`ui::experiment::Experiment`.

## Scope

- `src/game.rs`: `Recipe` made `pub` with `name()` / `consumables()` accessors
  (`consumables()` was `inputs()` originally); `Player::known_recipes()`.
- `src/viewmodel/crafting.rs`: `CraftOption`, `options()`.
- `src/ui/craft.rs`: the overlay (`Craft`, `Outcome`).
- `src/ui/app.rs`: `craft: Option<Craft>` field, `c` opens it, overlay dispatch
  and rendering alongside the experiment overlay.
- `src/ui/help.rs`: `c` line → "Craft menu".

## Out of scope

- Merging the `show_help` / `experiment` / `craft` overlay flags into one enum.
- Skipping disabled rows while navigating (reachable but inert).
- Any change to recipe discovery, recipe definitions, or `Game::craft`.

## Update: show what's missing (later)

The dimmed/enabled state alone didn't tell the player *what* they were short on.
`CraftOption` now also carries
`consumables: Vec<CraftConsumable { item, have, need }>` (with `met()`), still
computed in `viewmodel::crafting::options`. The `gui` Craft tab keeps one row
per recipe — the button, then each consumable as `have/need <item>` at the same
text size, met ones muted and short ones in the error colour
(`Coil  3/2 Copper Wire  0/1 Plastic Bottle`). The `tui` panel appends the same
list to each row's text (`Coil  (3/2 Copper Wire, 0/1 Plastic Bottle)`).

## Update: recipe tools (later)

`CraftOption` also carries `tools: Vec<CraftTool { item, present }>`, and
`enabled` now requires every consumable met **and** every tool held. The panels
render tools with no `have/need` number — muted when held, in the error colour
when missing. See `recipe-tools.md`. (`consumables` / `CraftConsumable` were
`inputs` / `CraftInput` before this — tools made "inputs" ambiguous.)

Paths since the original: `src/game/recipe.rs`, `src/viewmodel/crafting.rs`,
`src/gui/craft.rs` (the `src/game.rs` / `src/ui/` names above predate the
module split).
