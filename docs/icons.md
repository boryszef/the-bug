# Icons (map POI, later items)

The `gui` draws point-of-interest marks from raster images rather than
procedural `egui::Painter` shapes — see `docs/adr/0001` "Amendment: raster
icon assets" for the why.

## Layout

- **`assets/icons/<name>.svg`** — the source, hand-authored. Shared 64×64
  viewBox and a shared palette: a warm near-black outline (`#463a2b`), a soft
  ground-shadow ellipse, flat fills with a light/shade pass, and a green plant
  accent. The subject sits on a ~y53 baseline in a ~44-wide footprint so the
  set reads at one visual weight on the map.
- **`assets/icons/<name>.png`** — generated from the `.svg`, 256×256 RGBA,
  transparent background. Committed: the game will `include_bytes!` it (so it
  must be in the tree) and a committed PNG keeps the shipped pixels reviewable
  in a diff.

## Regenerating

```
scripts/render-icons.sh
```

Run after editing any `.svg`, then commit the changed `.png` alongside it.
Needs `rsvg-convert` (`apt install librsvg2-bin`, preferred) or `inkscape`.

## The set

| POI | file | subject |
|---|---|---|
| Village | `village` | two Mongolian-style gers (yurts) — felt domes, small wooden crown wheels, a painted door |
| Cave | `cave` | a rocky knoll with a dark arched mouth, grass on top |
| Ruins | `ruins` | a broken 21st-century concrete building — a storey or two of jagged wall, empty window openings, exposed rebar, a snapped-off floor slab, rubble, growth creeping back over it |
| Player | `player` | a hooded scavenger with a backpack, belt, and a foraged sprig tucked in the pack |

`Poi::Bridge` gets one when it lands; item icons will follow the same pipeline.

The player icon isn't a `Poi` — it's the always-visible position marker, not
an overlay that can be absent, so (unlike the POI icons) it isn't gated by
`POI_ICON_MIN_PX`: it stays on screen at every zoom level, just shrinking
with the tile.

## Runtime wiring

`src/gui/map.rs`:

- `image = { version = "0.25.10", default-features = false, features =
  ["png"] }` is a direct dependency (`eframe` already pulls the same crate +
  feature in transitively, so it costs nothing).
- `struct PoiIcons` holds one `egui::TextureHandle` per `Poi` kind. Each PNG
  is `include_bytes!`'d, decoded with `image::load_from_memory_with_format`,
  and uploaded via `ctx.load_texture(_, _, TextureOptions::LINEAR)` — the
  256 px source is only ever minified onto 12–96 px tiles, so linear
  filtering, not nearest.
- `PoiIcons` is built once in `MapView::new(&egui::Context)`, called from the
  `eframe` app-creator closure (`src/gui/mod.rs`). It lives on `MapView`,
  alongside a `player_icon: TextureHandle` loaded the same way from
  `assets/icons/player.png`.
- In the tile loop, a POI tile draws `painter.image(handle.id(),
  icon_rect(centre, tile_px, POI_ICON_RATIO), …)` — a square fraction of the
  tile, centred — instead of the old `draw_*_icon` polygon calls.
  `POI_ICON_MIN_PX` still gates it out when zoomed too far. The player marker
  draws the same way with `PLAYER_ICON_RATIO`, ungated, after the tile loop.
  `icon_rect` is the one shared helper both call sites use.

A decode failure `panic!`s: the bytes are compiled in, so a bad PNG is a
broken asset in the tree, not a runtime condition.
