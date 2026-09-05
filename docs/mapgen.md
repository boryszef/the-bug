# Dev tool: `tools/mapgen`

## Why

`TODO.md` wants the map to stop being an independent per-tile random roll
(`Map::new`) and instead have terrain that **clusters** — forests, deadlands
and meadows in contiguous stretches rather than confetti. `docs/gui-map.md`'s
follow-on increment #1 (a bundled, predefined map loaded via
`save::parse_terrain` → `Map::from_terrain`) needs a hand-picked ASCII grid to
bundle.

`mapgen` is the authoring tool for that grid. It is a **dev-only** utility:
not shipped, not part of the `the-bug` crate, run a handful of times to
generate candidates that a human eyeballs and chooses between. It does not
touch the game — wiring a chosen grid into `Map::new` is a separate, later
change.

## Outcome

```
cargo mapgen -t Forest=45 -t Meadow=30 -t Deadland=25 -s Cave=8 -s Ruins=4 -a 0.75 --seed 3
```

prints a grid in the save file's letter codes (`M F C R V .`), one row per
line, `Village` dead-centre:

| flag | meaning |
|---|---|
| `-t, --terrain NAME=PCT` (repeatable, required) | a clustering terrain and its share. `Meadow`, `Forest` or `Deadland`. The percentages must sum to 100. |
| `-s, --scatter NAME=COUNT` (repeatable) | `Cave` or `Ruins`, an absolute tile count, placed uniformly at random *on top of* the 100 %. Never clustered. |
| `-a, --affinity FLOAT` (default `0.6`) | `0.0` fully random … `1.0` one contiguous block per clustering terrain. |
| `--size N` (default `23`) | edge length, odd. Game maps are `21 + 2 * level` (21, 23, 25, …); a non-game size still works but prints a warning. |
| `--seed U64` | reproducible output (default: OS entropy). |
| `--json FILE` | also write a minimal `the-bug --load`-able save wrapping the grid, so a candidate can be walked around in the GUI. |

The `cargo mapgen` alias is in `.cargo/config.toml`. Run it **from the repo
root** and pass its flags directly — the alias already supplies the `--`
separator, so `cargo mapgen -- --help` breaks; use `cargo mapgen --help`.

### Composition is exact

Scatter counts are placed first, as an exact number of tiles. The remaining
cells are split between the clustering terrains by largest-remainder rounding,
so their tile counts always match the requested percentages as closely as
integers allow, at every affinity. Worked example — `--size 27` is 729 tiles;
minus 1 village, minus `Cave=8 + Ruins=4` scatter leaves 716 for clusters;
`45/30/25` → 322 F / 215 M / 179 D.

## How it works

`tools/mapgen/src/generator.rs`, in order:

1. **Quotas** — largest-remainder rounding turns the percentages into exact
   per-terrain tile counts over the non-village, non-scatter cells.
2. **Scatter** — Cave/Ruins cells are sampled uniformly at random and set
   aside; they take no part in clustering, so they stay speckled at any
   affinity (a few will still touch by chance — that is not clustering).
3. **`affinity = 1` layout** — recursive rectilinear bisection ("slice and
   dice") cuts the clustering cells into one contiguous block per terrain,
   each exactly its quota. A prefix of cells sorted by `(x, y)` is a run of
   whole columns plus a partial one — always 4-connected — and the axis
   alternates by depth so blocks come out blocky, not striped.
4. **Melt** — for `affinity < 1`, pairs of clustering cells are repeatedly
   swap-tested: a swap that doesn't increase the count of unlike orthogonal
   neighbours is always taken; one that does is taken with a flat probability
   `(1 - affinity)²`. Swaps never change tile counts, so composition stays
   exact. Near `1.0` only the block edges soften; near `0.0` every swap goes
   through and the field mixes to uniform (that endpoint is also short-cut to
   a plain shuffle). The knob is non-linear — clustering stays visible down to
   roughly `0.4`, then breaks up quickly.

## Self-contained on purpose

ADR 0002 rejected splitting `the-bug` into a library plus binaries, so
`save::`/`game::` code (including `save::terrain_code`) is `pub(crate)` and
unreachable from a separate crate. `tools/mapgen` therefore re-declares the
six-letter terrain table in `src/terrain.rs`, with a comment pointing back at
`src/save.rs`. **If the game's terrain codes change, change both.**

`tools/mapgen` is its own crate (own `Cargo.toml` with an empty `[workspace]`
table, own `Cargo.lock`), outside the `the-bug` package. The repo-root
`cargo fmt`/`clippy`/`test` and the default `prek` hooks don't see it; `prek`
has dedicated `mapgen-fmt` / `mapgen-clippy` hooks that fire only when its
files change. Run its tests with
`cargo test --manifest-path tools/mapgen/Cargo.toml`.

Reproducibility is per-`Cargo.lock`: the same `--seed` gives the same map
until `rand` is updated.

## Not in scope

- Wiring a generated grid into the game (replacing `Map::new`) — later, per
  `docs/gui-map.md` #1.
- Roads, rivers, or any `Feature` overlay — `docs/gui-map.md` #2.
- Biome realism (elevation, moisture, coastlines), non-square maps, terrain
  beyond the existing six kinds.
