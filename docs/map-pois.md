# Points of interest are a tile overlay

## Why

Cave, Ruins and Village used to be `TerrainType` variants: a cave tile *was*
cave terrain, so it couldn't also be forest, and a search there turned up only
cave items. That coupled one idea (what does this ground look like) to another
(is there something notable here) and made "a cave in the woods"
unrepresentable.

Now a `MapTile` carries a terrain (`Meadow` / `Forest` / `Deadland`) **and** an
`Option<Poi>` — a point of interest sitting on top. `Poi` is `Cave`, `Ruins`
and `Village` (`Bridge` comes later). `Village` is a POI like the others; it's
special only in placement — always dead-centre, never random. A tile has at
most one POI: that's just the `Option`.

## What a POI does

- **Search yield.** A tile offers its terrain's items *and* its POI's, each
  tagged with where it came from (`FoundIn::Terrain` / `FoundIn::Poi`). The
  event log names the place: "a Branch in the forest", "a Stone in the cave".
  `Village` has no items of its own — searching it yields only its terrain's.
- **Quests.** `EventTypeID::VisitPoi(Poi)` fires when the player steps onto a
  tile with that POI; the "explore the ruins" quest now counts a `VisitPoi`,
  not a `VisitTerrain`.
- **Rendering.** The gui draws the POI's icon on the tile (and skips the wavy
  edge-trickle there, so the icon reads cleanly); the tui prints the POI's
  glyph in place of the terrain's. Both come from `Poi::symbol` /
  `MapTile`'s `Display`.

The item probabilities are unchanged from the old scatter terrain
(Cave → `Stone` 0.30; Ruins → `CopperWire` / `PlasticBottle` / `Umbrella` 0.20
each), in `POI_ITEMS` next to `TERRAIN_ITEMS` in `src/game/map.rs`.

## Generation

`mapgen` produces terrain only — every cell, no holes, no scatter step
(`docs/mapgen.md`). `Map::new` then lays the POI grid with `scatter_pois`:
`Village` on the centre cell, then `Cave` and `Ruins` on other cells picked
uniformly at random, at `CAVE_FRACTION` / `RUINS_FRACTION` of the tile count —
the same densities the scatter terrain used, still scaling with map size.

## Save format

`MapState` gains a `pois` grid parallel to `terrain`, one char per tile
(`poi_code`: `.`/space = none, `c` = cave, `r` = ruins, `v` = village). It's
`#[serde(default)]`, so a save without it (or a hand-made one) loads with every
tile POI-less.

**Clean break:** the old terrain codes `C` / `R` / `V` are gone. A save whose
`terrain` grid still uses them — or whose event log names `Cave` / `Ruins` as a
`Found` terrain — no longer loads (`io::ErrorKind::InvalidData`). Consistent
with the project's "no migration logic" stance (`docs/save-load.md`); the game
is pre-release.

## Not in scope

- `Poi::Bridge` and the road×river mechanic that creates it — after roads and
  rivers (`docs/gui-map.md` #2).
- Multiple POIs on one tile — the `Option` says one.
- A POI influencing terrain generation (e.g. ruins preferring deadland).
