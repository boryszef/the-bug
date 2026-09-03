# Progression

Shown in the Player panel:

```
Level: 1
XP: 25
Recipes: 2/4
Inventory: Stick 3, Vine 2
```

## Recipe ratio

`Recipes: known/total` — `known` is how many recipes the player has discovered
(`Player::known_recipes()`), `total` is how many exist in the game (`RECIPES`).
Read via `Game::recipe_progress() -> (usize, usize)`.

## Experience points

`Player.experience` accumulates:

| Action | XP |
|---|---|
| An experiment that **discovers a new recipe** | +10 |
| Every **10 successful crafts** | +1 |

- `Player.crafts_completed` counts successful crafts (for the 1-per-10 rule);
  failed crafts (unknown recipe, not enough materials) don't count.
- No log line is written for XP — the panel counter is the only feedback.
- XP does **nothing** yet. `Player.level` is still fixed at 1 (it only affects
  starting map size). A level-up mechanic is a separate future item.

Both `experience` and `crafts_completed` are in the save file
(`#[serde(default)]` → `0` for older / hand-made files).

## Multi-resource search (confirmed)

`Game::search()` rolls each material a tile offers independently, so one search
can yield several different materials. This is exercised by
`search_yields_every_material_a_tile_offers`. In practice each terrain currently
lists only one material (`TERRAIN_MATERIALS`), so a search yields 0 or 1 until a
terrain gains a second material.
