# Map generation: `src/mapgen/`

## Why

The map used to be per-tile confetti: every non-centre tile in `Map::new` was
an independent weighted roll, so forests, meadows and deadlands never formed
contiguous stretches. `src/mapgen/` replaces that with exact terrain
composition plus affinity-controlled *clustering*, while the map stays random
(no seed is stored — the generated grid is persisted verbatim in the save, so
reload is unchanged).

This started as a standalone dev tool, `tools/mapgen`, that authored candidate
grids for a human to eyeball and bundle. Once the game generates its map with
the algorithm directly there is nothing to author, so the tool was retired and
its `generator.rs` moved into the crate as `src/mapgen/`. Being in-crate, it
now uses `game::TerrainType` directly instead of duplicating the six-letter
terrain table (the thing ADR 0002's `pub(crate)` walls had forced on the
separate tool).

## Where the knobs live

`src/mapgen/` is policy-free — it takes a `Spec` and an RNG. The tuning is in
`src/game/map.rs`, as consts next to `Map::new`:

| const | value | meaning |
|---|---|---|
| `MAP_AFFINITY` | `0.75` | `0.0` confetti … `1.0` one contiguous block per terrain |
| `DEADLAND_PERCENT` / `MEADOW_PERCENT` / `FOREST_PERCENT` | `40 : 30 : 20` of 90, renormalised to sum 100 | clustering-terrain shares (the old per-tile weights, kept) |
| `CAVE_FRACTION` / `RUINS_FRACTION` | `0.07` / `0.03` of the tile count | POI density (`docs/map-pois.md`), placed by `Map::new`, not by `mapgen` |

`Map::new` builds the `Spec` from these, calls `mapgen::generate`, sprinkles
the POI overlay grid (`scatter_pois`), and hands terrain + POIs to the existing
`Map::from_terrain` (the same entry point save-load uses).

`mapgen` only knows about terrain — every cell gets one, no holes. The village,
caves and ruins used to be `TerrainType` variants placed here (village at the
centre, caves/ruins as a "scatter" category); they are all `Poi` overlays now
(`docs/map-pois.md`), so this generator has no scatter step and no special
centre cell.

## How it works

`src/mapgen/generator.rs`, in order:

1. **Quotas** — largest-remainder rounding turns the cluster percentages into
   exact per-terrain tile counts over every cell, so the counts always sum
   exactly.
2. **`affinity = 1` layout** — recursive rectilinear bisection ("slice and
   dice") cuts the cells into one contiguous block per terrain, each exactly
   its quota. The axis alternates by depth so blocks come out blocky, not
   striped. Over a full rectangle every slice is contiguous, so this can't
   fail.
3. **Melt** — for `affinity < 1`, pairs of cells are repeatedly
   swap-tested: a swap that doesn't increase the count of unlike orthogonal
   neighbours is always taken; one that does is taken with a flat probability
   `(1 - affinity)²`. Swaps never change tile counts, so composition stays
   exact. The `0.0` endpoint is short-cut to a plain shuffle. The knob is
   non-linear — clustering stays visible down to roughly `0.4`, then breaks
   up quickly.

## Not in scope

- A seed stored in the save / reproducible regeneration — the grid itself is
  persisted, which is all load needs.
- Roads, rivers, or any `Feature` overlay — `docs/gui-map.md` #2.
- Points of interest (village, caves, ruins) — a separate `Poi` overlay laid
  on by `Map::new`, see `docs/map-pois.md`.
- Biome realism (elevation, moisture, coastlines), non-square maps, terrain
  beyond the existing kinds.
