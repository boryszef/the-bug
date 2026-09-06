# Icons (map POI, later items)

The `gui` will draw point-of-interest marks from raster images rather than
procedural `egui::Painter` shapes — see `docs/adr/0001` "Amendment: raster
icon assets" for the why and the runtime plan.

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
| Village | `village` | two Mongolian-style gers (yurts) — felt cones, wooden crown wheels, a painted door |
| Cave | `cave` | a rocky knoll with a dark arched mouth, grass on top |
| Ruins | `ruins` | broken masonry — two wall fragments, a fallen lintel, rubble, a creeping vine |

`Poi::Bridge` gets one when it lands; item icons will follow the same pipeline.

## Not wired yet

`src/gui/map.rs` still draws the procedural `draw_village_icon` /
`draw_cave_icon` / `draw_ruins_icon` shapes. Replacing those with
`painter.image` — plus the `image` crate to decode the PNGs into a cached
`egui` texture per icon — is the next step (ADR amendment, "Consequences").
`tui` is unaffected: it keeps `Poi::symbol` glyphs, and `assets/` + the image
dep are `#[cfg(feature = "gui")]`.
