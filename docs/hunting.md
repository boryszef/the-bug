# Feature: hunting

## Why

From `TODO.md`'s "EPIC: the story":

> implement hunting: player needs a bow and arrows. Hunting is possible on
> meadows and in the forest; it is activated by pressing `[h]` and uses one
> arrow each time. Hunting, like searching, gives some items with certain
> probability: meat, bone, hide, fur.

It also partly delivers the sibling "items become enhancers" line — the Wooden
Bow becomes useful (it lets you hunt) and Arrows get spent doing it.

## Behaviour

`Game::hunt()` is a world action on the player's current tile, built to mirror
`Game::search()`:

- **No terrain gate.** Only Meadow and Forest have rows in `HUNT_ITEMS`, so a
  hunt on Deadland just comes back empty — the same way searching barren ground
  turns up nothing. (An earlier draft gated on terrain; dropped for symmetry
  with search.)
- **Gear.** A hunt needs a `WoodenBow` *held* (not consumed) and **spends one
  `Arrow`**. Missing either logs `EventKind::HuntUnprepared { missing }` and
  spends nothing — the checks come before the spend, like `Game::craft`.
- **Yield.** Each of the tile's `hunt_items` is rolled independently against its
  base probability, decayed by `last_hunt_time` through the shared
  `adjust_probability` helper (re-hunting the same spot right away rarely pays).
  Products: `Meat`, `Hide`, `Bone`, `Fur` — `HUNT_ITEMS` in `src/game/map.rs`,
  ~0.1–0.5 each, Forest a bit richer than Meadow. One unit per product per hunt.
- **Events.** `Hunted { items }` (green) for a haul, `HuntMissed` (yellow) when
  the arrow bought nothing. Rendered `Hunt: 1 Meat + 1 Hide.` /
  `Hunt: the quarry got away.` (Polish `Polowanie: …`), in the experiment-log
  style.
- **Quest hook.** A *successful* hunt (one that brings something back) fires
  `EventTypeID::Hunt`, which the "Stock Up for Hard Times" quest counts — a
  wasted arrow doesn't count. See `docs/quest-system.md`.

### Controls

`h` on the Map tab, or the **Hunt (h)** button next to **Search (s)**. The
button is always clickable; `Game::hunt` decides and logs the outcome
(`MapCommand::Hunt` → `Game::hunt` in both front ends; the `tui` maps
`KeyCode::Char('h')`).

## Scope

- `src/game/item.rs` — `Meat`, `Bone`, `Hide`, `Fur` appended to `Item`.
- `src/game/map.rs` — `HUNT_ITEMS`, `tile_hunt_items`, `MapTile.hunt_items` +
  `MapTile.last_hunt_time`, `roll_hunted_items`, `Map::update_tile_last_hunt_time`;
  `adjust_probability`'s parameter renamed `last_used` (serves both).
- `src/game/event.rs` — `Hunted`, `HuntMissed`, `HuntUnprepared` variants.
- `src/game/quest.rs` — `EventTypeID::Hunt`.
- `src/game/mod.rs` — `Game::hunt()`.
- `src/i18n/` — `item-{meat,bone,hide,fur}` (en + pl with case attributes),
  `event-hunted` / `event-hunt-missed` / `event-hunt-unprepared`,
  `action-hunt`, `footer-map` (tui) gains `h hunt`.
- `src/gui/map.rs` + `src/gui/mod.rs` + `src/tui/app.rs` — the `h` key / button
  and event-log colours.

## Out of scope

- Distinct animals / species — a hunt yields materials directly, no creature
  model.
- Meat spoilage, cooking, or any use for meat / hide / bone / fur yet (no
  recipes consume them).
- A skill or weapon tier affecting the odds; the bow is pass/fail.
- Save-format change — the new items serialise by name into the existing
  inventory map; `hunt_items` / `last_hunt_time` are derived, never stored.
