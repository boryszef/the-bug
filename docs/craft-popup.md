# Feature: Craft popup

## Why

The `c` key called `game.craft("Cord")` directly — a hardcoded stopgap from when
the experiment popup was built (see `refactor-thin-ui.md`). The crafting backend
(`Game::craft`) is complete; only the UI was missing.

## Outcome

`c` opens a **Craft popup** listing the recipes the player has discovered
(`Player::known_recipes`). The player scrolls with `↑`/`↓` and confirms with
`Enter` (or `c`); `Esc`/`q` closes it. The picked recipe name is handed to
`Game::craft`, which subtracts the inputs, adds the output, and logs the result.

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

- `src/game.rs`: `Recipe` made `pub` with `name()` / `inputs()` accessors;
  `Player::known_recipes()`.
- `src/viewmodel/crafting.rs`: `CraftOption`, `options()`.
- `src/ui/craft.rs`: the overlay (`Craft`, `Outcome`).
- `src/ui/app.rs`: `craft: Option<Craft>` field, `c` opens it, overlay dispatch
  and rendering alongside the experiment overlay.
- `src/ui/help.rs`: `c` line → "Craft menu".

## Out of scope

- Merging the `show_help` / `experiment` / `craft` overlay flags into one enum.
- Skipping disabled rows while navigating (reachable but inert).
- Any change to recipe discovery, recipe definitions, or `Game::craft`.
