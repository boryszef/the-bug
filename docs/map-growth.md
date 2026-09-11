# Feature: fixed-size map, revealed by level

## Why

`TODO.md`: "level up with increasing XP, when leveling up, uncover next
portions of the map... The whole map should be generated at the beginning
of the game, but hidden parts will be inaccessible." Before this, the map
was generated once at a size tied to the player's starting level
(`MAP_MIN_SIZE + player.level * MAP_PER_LEVEL_INCREMENT`) and never
changed again — `docs/progression.md` documented this as a known gap.

This is the first increment: a fixed-size map with a level-gated walkable
region, generated block by block as the player reaches each one — not the
full "whole map generated upfront" vision, and not yet "better resources,
more dangerous" difficulty scaling by region. Both are still open,
tracked in `TODO.md`.

## Design

The map is a fixed `51×51` grid (`MAP_SIZE`) — a `3×3` grid of `17×17`
blocks (`BLOCK_SIZE`). `51 = 17×3` is odd, which matters: `Map::half =
size/2` is exact for an odd size, giving the grid a true centre tile —
exactly the property the map already relied on (the Village always sits at
world `(0,0)`, the centre). An earlier draft of this design used two
`17×17` halves (`34×34`, even) and had to work around the resulting
off-centre grid — relocating the Village, recentring the map's camera,
fixing a boundary-check test. The `3×3` layout avoids all of that: nothing
about the coordinate system changes, only how much of it is walkable and
generated at a given time.

Blocks are addressed by `(col, row)`, each `0..3`, matching the existing
tile-index axes (`col` = east/west, `row` = north/south). The centre block,
`(1,1)`, holds the Village and is the only one populated at the start. The
rest reveal as the player levels up:

| Level | Blocks added |
|---|---|
| 1 | `(1,1)` — centre, holds the Village |
| 2 | `(1,0)` |
| 3 | `(0,0)`, `(0,1)` |
| 4 | `(2,0)`, `(2,1)` |
| 5 | `(0,2)`, `(1,2)`, `(2,2)` — completes the map |

(`BLOCK_UNLOCKS`, `src/game/map.rs`.)

### Generation is per-block, on demand

`Map::new()` builds the whole `51×51` grid as all-`Deadland`/no-POI, then
calls `reveal_for_level(1, &mut rng)`, which generates just the centre
block for real. Each later block is generated the first time the player's
level reaches its threshold — not upfront. `Map::reveal_for_level(level,
rng)` is idempotent: it checks every block unlocked at `level`, and
generates any that's still all-`Deadland`. "Still all-`Deadland`" is a
reliable, not probabilistic, signal that a block hasn't been through
`mapgen::generate` yet — the generator's quota step (`docs/mapgen.md`)
always allocates a nonzero Meadow/Forest count for any real run, so a
genuinely-generated block can never come back all-Deadland by chance. This
also means `reveal_for_level` correctly handles a single XP grant that
skips a level (e.g. 1→3 in one big grant) — it checks every block up to
the new level, not just the newest one — and calling it redundantly (the
same level twice, or on an already-fully-revealed map) is a cheap no-op.

Each block is generated with `mapgen::generate` at exactly `BLOCK_SIZE`
(17, odd — `mapgen::Spec` requires an odd size), using the same
`DEADLAND_PERCENT`/`MEADOW_PERCENT`/`FOREST_PERCENT`/`MAP_AFFINITY` tuning
`Map::new` always used, and its own `Cave`/`Ruins` scatter
(`scatter_pois`) at the same densities as before. Every block is always
exactly `17×17` — there's no rectangle/non-square case anywhere in this
design, since `mapgen` is square-only.

Two hooks call `reveal_for_level`:
- `Game::grant_award`'s `GrantType::Experience` branch, right where
  `EventKind::LeveledUp` is logged — the normal path.
- `Game::from_saved`, once, as a self-healing safety net: a no-op over
  already-revealed blocks, but it means a hand-edited save with a bumped
  `level` against never-regenerated terrain can't leave the player able to
  walk into blank, POI-less Deadland.

### Walkability

`Map::is_unlocked(pos, level)` replaces the bare `contains` check in
`Game::walk` — inside the grid *and* inside a block `level` has reached.
Refusal is silent (no event, no GUI message), matching how an out-of-bounds
move already behaved.

Both `is_unlocked` and `reveal_for_level` are no-ops (falling back to the
plain boundary check, or doing nothing) on any map that isn't exactly
`MAP_SIZE×MAP_SIZE`. This matters: the cucumber functional-test suite's
`tiny_map()` (a `3×3` fixture used by nearly every scenario,
`tests/steps/world.rs`) and several unit-test fixtures build much smaller
maps directly through `Map::from_terrain`, which stays fully generic. The
block scheme only makes sense for a genuine full-size map; a `3×3` map
being interpreted against `17`-wide block boundaries would silently lock
every tile, or — for `reveal_for_level` — index out of bounds.

### Persistence — no migration

`Map::restore_state` rejects a loaded grid that isn't exactly
`MAP_SIZE×MAP_SIZE` with `io::ErrorKind::InvalidData`. Old saves (variable,
level-scaled size, never exactly `51`) are naturally rejected — the same
precedent already used for the pre-`EventKind` event-log format break
(`docs/save-load.md`): a save whose shape no longer matches what the game
means by "a map" fails to load rather than being silently reinterpreted.

## What is *not* built here

- The whole map generated upfront — only the reachable block(s) are ever
  real; the rest waits as Deadland until reached. `TODO.md`'s original
  wording ("whole map... generated at the beginning") describes the
  eventual goal across future increments, not this one.
- Any difficulty/reward scaling by block ("better resources... possibly
  more dangerous") — every block uses identical tuning today.
- Any UI feedback when a new block unlocks (no event-log line, no map
  flash) — the new tiles becoming visible/walkable is the only signal, for
  now.
- Persisted "which level unlocked this" bookkeeping beyond the terrain
  itself — a block's state *is* its terrain (Deadland = not yet reached);
  there's no separate flag to drift out of sync with it.

## Code

- `src/game/map.rs` — `BLOCK_SIZE`, `MAP_SIZE`, `BLOCK_UNLOCKS`,
  `unlocked_blocks`, `block_bounds`, `Map::new` (now parameterless),
  `Map::reveal_for_level`/`block_is_ungenerated`/`generate_block`,
  `Map::is_unlocked`, `scatter_pois` (now block-scoped, with an `exclude`
  slot for the Village tile), the `MAP_SIZE` check in `Map::restore_state`.
- `src/game/mod.rs` — `Game::walk` reads `is_unlocked` instead of
  `contains`; `Game::grant_award` and `Game::from_saved` call
  `reveal_for_level`.
- `src/save.rs` (tests only) — `minimal_map_json`/`padded_grid_json`
  helpers, since several fixtures needed *some* valid `MAP_SIZE` grid
  without caring about its content.
