# Improving the map's look

## Why

`docs/gui-map.md` built the first graphical map: one flat `rect_filled` per
tile, coloured by `viewmodel::map::terrain_rgb`, plus a circle for the player.
It reads like a spreadsheet — hard square cells, no markers. The `TODO.md`
"improve the map" EPIC softens that in small increments:

1. **POI icons** — a drawn mark on the two point-of-interest terrains, Cave and
   Ruins, so the player can pick them out.
2. **Wavy borders** — where two different *field* terrains meet, a little of
   the neighbour's colour trickles across the edge, so a forest/meadow border
   is ragged rather than a ruler-straight line.
3. **POI as an overlay** — Cave and Ruins stop being `TerrainType` variants and
   become an overlay on a tile, so a tile can be forest *and* a cave and yield
   findings from both. *Separate doc when built.*
4. **Roads and rivers** — another overlay on tiles, drawn as connected
   segments (see `docs/gui-map.md` follow-on #2). *Separate doc when built.*
5. **Bridge POI** — the tile where a road crosses a river becomes a point of
   interest yielding new salvage (steel bolt, rusty metal). *Separate doc.*

Increments 1 and 2 are covered here; 3–5 get their own docs. The overlay work
in 3 is the natural foundation for 4 and 5, so it likely lands first.

## 1. POI icons (Cave, Ruins)

`src/gui/map.rs` only — pure rendering, no `game`/`viewmodel` change.

After a tile's base square is painted, if the tile is `Cave` or `Ruins` and the
zoom is at least `POI_ICON_MIN_PX`, a small icon in one dark ink
(`POI_ICON_COLOR`) is drawn centred on the tile, sized `POI_ICON_RATIO` of the
tile:

- **Cave** — a filled triangle, apex up (a hill / a cave mouth).
- **Ruins** — four vertical bars of uneven height, bottoms aligned (a broken
  skyline).

The geometry comes from pure helpers (`cave_triangle`, `ruins_bars`) so it can
be unit-tested the way the rest of `src/gui/map.rs`'s helpers are; the drawing
calls themselves aren't tested (no assertable output — same rule as
`MapView::ui`).

### Not emoji

`TerrainType::symbol()` returns `🪨` / `🏙` and the `tui` prints those on its
canvas. The `gui` does **not** reuse them: egui 0.36 renders emoji monochrome,
its bundled fonts may not even carry `🪨` (U+1FAA8, 2020), and the repo has no
`FontDefinitions` customisation. A drawn shape is reliable and matches the
"procedural, no image assets" choice from `docs/adr/0001` / `docs/gui-map.md`.

## 2. Wavy borders (field terrains)

`viewmodel::map` change: `TileView` gains
`neighbours: [Option<TerrainType>; 4]` (North, East, South, West in world
space; `None` past the map edge), filled in `tile_views` from `Map::get_tile`.
This is the neighbour-derived data the front end needs but must not compute by
reaching into `Map` itself — the same principle `docs/gui-map.md` states for
the future road/river edge mask.

`src/gui/map.rs` change: after a tile's base square, if the tile is a **field**
terrain (`Meadow`/`Forest`/`Deadland`), for each edge whose neighbour is a
*different* field terrain, a few small rectangles ("teeth") in the neighbour's
`terrain_rgb` colour are drawn reaching **inward** from that edge. The teeth
stay inside the tile, so no draw-order juggling is needed — the single existing
tile loop just does one more thing. Both sides of a border grow teeth of the
other's colour, so the seam interlocks. The tooth pattern (count, offsets,
depths) is fixed — it only needs to break up the straight line, not model
anything. `edge_teeth` is a pure, tested helper.

Cave, Ruins and Village tiles are left as clean squares so their icon / colour
reads clearly.

### Deliberately out of scope (increments 1–2)

- Per-tile variation in the tooth pattern (every border uses the same shape).
- Diagonal-neighbour / corner treatment.
- Any `game` or save-format change — terrain and the grid are untouched.
- Sprites / textures — still procedural fills (`docs/gui-map.md` #3).
