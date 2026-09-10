# Progression

Shown in the Player panel:

```
Level: 1
XP: 25
Recipes: 2/4
Inventory: Branch 3, Vine 2
```

## Recipe ratio

`Recipes: known/total` — `known` is how many recipes the player has discovered
(`Player::known_recipes()`), `total` is how many **craftable** recipes exist
(`RECIPES` filtered by `Recipe::craftable()`). Disassemble-only recipes
(`docs/disassembly.md`) are decompositions, not recipes to discover, so they're
left out of the total. Read via `Game::recipe_progress() -> (usize, usize)`.

100% is **not** currently reachable: Metal Detector needs a Microcontroller
and Solar Charger needs a Solar Panel and a Circuit Board, and none of the
three has a source anywhere — no recipe produces one and no terrain/POI
search yields one. Both recipes are permanently uncraftable while still
counting toward `total`. See `docs/code-review-2026-09.md`.

## Experience points

`Player.experience` accumulates:

| Action | XP |
|---|---|
| An experiment that **discovers a new recipe** | +10 (`RECIPE_DISCOVERY_XP`) |
| Every **10 successful crafts** | +1 |
| Completing a quest | the quest's `reward_xp` (varies) |

- `Player.crafts_completed` counts successful crafts (for the 1-per-10 rule);
  failed crafts (unknown recipe, not enough items) don't count.
- No log line is written for XP itself — only for a level change it causes
  (below). The panel counter is the feedback for XP on its own.

Both `experience` and `crafts_completed` are in the save file
(`#[serde(default)]` → `0` for older / hand-made files).

## Levels

`Player.level` is a stored field (also in the save file), kept in sync by
`Player::add_experience` as a side effect of every XP grant — not derived
fresh on each read. Level 1 below 100 XP; level *N* (*N* ≥ 2) once XP
reaches `100 × 3^(N-2)` — each threshold triples the last: 100, 300, 900,
2700, .... `level_for_xp(xp)` (`src/game/player.rs`) is the pure formula;
`add_experience` calls it after adding XP and returns the new level if it
changed.

Crossing a threshold logs `EventKind::LeveledUp { level }` — "You levelled
up! You are now level *N*." Every XP grant in `Game` (`craft`, `experiment`,
quest completion) routes through one funnel, `Game::grant_award(GrantType)`
(currently just `GrantType::Experience(u32)` — room for another award kind
later without building it now): it applies the grant via `add_experience`
and logs `LeveledUp` if it reports a change, so the "what happens when XP
is granted" logic — including detecting a level-up — lives in exactly one
place rather than being duplicated at each of the three call sites.

`level` still only affects starting map size (`Map::new`,
`MAP_PER_LEVEL_INCREMENT`) — a fixed-at-creation formula, so levelling up
mid-game doesn't retroactively resize the current map. Anything more
(unlocking content, progressively revealing the map) is a separate, larger
future item (`TODO.md`).

## Multi-resource search (confirmed)

`Game::search()` rolls each item a tile offers independently, so one search can
yield several different items. This is exercised by
`search_yields_terrain_and_poi_items_and_names_each_source`.

`Game::hunt()` works the same way over a separate `HUNT_ITEMS` table (Meadow and
Forest only — Meat / Hide / Bone / Fur, ~0.1–0.5 each), with its own
`last_hunt_time` decay. See `docs/hunting.md`.
