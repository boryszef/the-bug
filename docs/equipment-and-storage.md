# Feature: the Equipment (limited) and Storage (unlimited)

## Why

`TODO.md` asked for the pool the player carries to have a limited size — so
when it fills, the player takes their findings back to the village and
deposits them in storage or drops them. Until then `Player.inventory` was a
single unlimited pool everything read and wrote: nothing to carry, nowhere
to run out of room. (This pool was called the "Bag" until it was renamed
**Equipment** for consistency with the Player-panel line; the split and the
mechanics are unchanged.)

`Player.inventory` keeps its exact name and role: the unlimited village
Storage, entirely unchanged in behavior. `Player.equipment` is the
capacity-limited pool the player carries — `EQUIPMENT_BASE_CAPACITY = 50`, a
flat total across every item type combined, not a per-item or per-slot
limit.

## What goes where

`search`/`hunt`'s *yields* go to the equipment (`Player::add_to_equipment`), not
storage — a search or hunt happens out in the field. `hunt`'s *gear
prerequisite* moves too: the Wooden Bow and Arrow must be in the equipment, not
storage, since gear has to be carried to be used away from the village.

Crafting, experimenting, and disassembling read **both**: a consumable (or,
for `disassemble`, the item being taken apart) is drawn from storage first,
the equipment for any remainder (`Player::spend_storage_then_equipment`) — the recipe's
required *tools* stay storage-only, though, and so does every *output*
(a craft/experiment's product, disassembly's recovered components, quest
rewards via `grant_reward`). The asymmetry is deliberate: these three
actions only run at the Village (`docs/village-crafting.md`) in the first
place, so it's natural for them to reach into whatever the player happens to
be carrying *in addition to* the village's stockpile — but a tool is kept at
the workshop where it's used regardless, and a guaranteed reward or a fresh
result shouldn't land somewhere capacity-limited by accident (or risk being
lost to full equipment, in the reward's case). This is what gives Transfer a
real bidirectional purpose beyond just capacity management: gear flows
Storage→Equipment before a hunting trip, loot flows Equipment→Storage on return — and a
material picked up mid-expedition can be spent on the spot once the player's
back at the village, without a manual transfer first.

## The Player API

- `add_to_equipment(item, amount) -> bool` — all-or-nothing: fails (nothing
  added) if the equipment's total would exceed `equipment_capacity()`.
- `equipment_count`/`has_item_in_equipment` and `spend_from_equipment` mirror the existing
  `inventory_count`/`has_item`/`spend` exactly, scoped to the equipment.
- `combined_count`/`has_item_combined` — storage and the equipment summed, what
  craft/experiment/disassemble check against (`first_shortage` now reports
  this combined figure too).
- `spend_storage_then_equipment(item, amount)` / `spend_all_storage_then_equipment(items)`
  — what craft/experiment/disassemble spend with: as much as possible from
  storage, the remainder (if any) from the equipment.
- `transfer_to_storage(item, amount) -> bool` / `transfer_to_equipment(item,
  amount) -> bool` — move between the two pools. `transfer_to_equipment` tries
  `add_to_equipment` (capacity-checked) *before* spending from storage, so a
  transfer that doesn't fit never leaves an item in limbo.
- `equipment_capacity() -> u32` — the equipment's current cap: `EQUIPMENT_BASE_CAPACITY` (50),
  plus `SATCHEL_BONUS` (50) once while a Satchel is carried. See "Enhancer:
  the Satchel" below.
- `equipment_progress() -> (u32, u32)` — `(carried, capacity)`, both summed across
  every item type; mirrors `recipe_progress`'s `(known, total)` shape, for
  the gui's "Equipment: 37/50" header. The capacity half is `equipment_capacity()`, so
  the header reads `/100` once a Satchel is in the equipment.

`Game` adds four thin wrappers for the player-initiated side: `transfer_to_storage`,
`transfer_to_equipment` (both village-gated via `at_craftable_location`, reused from
village-crafting — silent no-op away from it, or without enough stock/room,
same as any other refused village-gated action) and `drop_from_equipment` (no
gate — the equipment can be dropped from anywhere) / `drop_from_storage`
(village-gated).

## Enhancer: the Satchel

The Satchel (craftable: Hide + Cord, needs a Bone Needle) is the first
enhancer to touch the Equipment pool, from `TODO.md`'s "items should become
enhancers" line — same shape as bow→hunt: an item that does something just by
being carried.

- **While a Satchel is in the equipment, `equipment_capacity()` is 100** (base 50 +
  `SATCHEL_BONUS` 50). Flat — a second Satchel adds nothing, so there's no
  incentive to carry a pile of them as free capacity.
- **The Satchel counts as an ordinary item in the pool.** It's in the
  `equipment` map, so `equipment_total()` includes it; net usable gain is
  +49. There's no equipment *slot* — it's carried like anything else, and
  capacity is just `base + bonus`.
- **Removing it while the equipment holds more than 50 is allowed and leaves
  the pool over its base cap.** `add_to_equipment` then refuses everything until
  `equipment_total() <= equipment_capacity()` again — the natural "overloaded, go dump
  some" state. No guard blocks the drop/transfer; a `Player` unit test pins
  this down.
- Nothing is stored: the Satchel serialises by name into the existing `equipment`
  map, and `equipment_capacity()` is derived on every read.

## What's an event and what isn't

**`EquipmentFull { item }`** — logged when a search or hunt roll can't be carried
(the equipment is already full). Per `docs/event-worthiness.md`'s cost-bearing-roll
rule: the player committed to an uncertain roll, and "couldn't carry it" is a
real, felt consequence of that roll, distinct from a deterministic refusal.
A hunt whose catch only partly fits still logs `Hunted` for what made it in
(and still counts toward an open quest) — but only if something was actually
kept; a catch entirely lost to full equipment doesn't complete "Stock Up for
Hard Times" any more than a wasted arrow does.

**`Dropped { item }`** — logged whenever `drop_from_equipment`/`drop_from_storage`
actually removes something. A real, permanent, deliberate loss — squarely a
state change under `docs/event-worthiness.md`'s rule 1.

**Transfers are not logged.** They do change trackable state (two counters
move), which rule 1 would technically allow — but at one unit per click,
logging every transfer would flood the pane with pure bookkeeping the player
already fully intended and knows the outcome of, closer to building an
Experiment selection (also unlogged) than to a craft's result. This is a
judgment call the event-worthiness doc doesn't spell out on its own, made
explicit here rather than silently.

## gui: the Items tab

New `Panel::Items` tab (between Disassemble and Quests), two columns
mirroring `gui::experiment`'s layout: **Equipment** (left, with a running
`carried/capacity` header) and **Inventory** (right). Each row is the item,
its quantity, and:

- Equipment: a "→" button (transfer to storage) and an "x" (drop one).
- Inventory: a "←" button (transfer to equipment) and an "x" (drop one).

The arrow glyphs are drawn in the monospace font (`RichText::monospace`),
the same fix the Map tab's `←↑↓→` buttons already needed — egui's default
proportional font has no arrow coverage, only the bundled Hack (Monospace)
does.

Transfer (both directions) and the Inventory "x" are enabled only at the
Village (`at_village`, same pattern as Craft/Experiment/Disassemble); the
Equipment "x" is always enabled. Both move exactly one unit per click, matching
the existing one-unit-per-click precedent (`ItemSelection::add`) — a
"move the whole stack" control is a reasonable future refinement, not built
now. The Player panel's old single "Inventory: ..." line is gone; in its
place an `Equipment: <carried>/<capacity>` line shows the equipment's fill and its
current (Satchel-aware) cap — just the two numbers from `equipment_progress()`, no
item list, since the Items tab is where contents are managed.

(The frozen `tui` never got Equipment/Inventory management — `search`/`hunt`
worked there but their gains had nowhere to be viewed or emptied. `tui` was
retired entirely in `docs/adr/0004-retire-tui-front-end.md`.)

## Scope

- `src/game/player.rs` — `EQUIPMENT_BASE_CAPACITY` + `SATCHEL_BONUS`, `equipment_capacity`,
  `Player.equipment`, `equipment_total`, `add_to_equipment`, `equipment_count`, `has_item_in_equipment`,
  `spend_from_equipment`, `combined_count`, `has_item_combined`,
  `spend_storage_then_equipment`, `spend_all_storage_then_equipment`, `transfer_to_storage`,
  `transfer_to_equipment`, `equipment_progress`. `spend_all` (the old storage-only loop)
  is gone — its only two callers now use `spend_all_storage_then_equipment`.
- `src/save.rs` — `PlayerState.equipment`, `#[serde(default)]` so an old save
  loads with an empty pool. (The on-disk key was `bag` until the rename; old
  saves with a `bag` key now load empty — no alias.)
- `src/game/mod.rs` — `search`/`hunt` route through the equipment; `hunt`'s gear
  check; `craft`/`experiment`/`disassemble` check and spend the combined
  pool; `Game::transfer_to_storage`/`transfer_to_equipment`/`drop_from_equipment`/`drop_from_storage`.
- `src/game/event.rs` + `src/i18n/mod.rs` + both `.ftl`s — `EquipmentFull`,
  `Dropped`.
- `src/viewmodel/inventory.rs` — `combined_sorted` (new; `sorted` is
  unchanged, still storage-only — the Items tab's storage column needs that,
  not the combined view).
- `src/viewmodel/crafting.rs` — `CraftConsumable.have` is now combined;
  `CraftTool.present` stays storage-only, matching `first_missing_tool`.
- `src/viewmodel/disassembly.rs` — `options` now built from
  `combined_sorted`, so an item held only in the equipment still shows up.
- `src/viewmodel/items.rs` (new, `gui`-only) — `Overview { equipment,
  equipment_progress, storage }`.
- `src/gui/items.rs` (new), `src/gui/mod.rs` — the tab, its wiring, the
  Player panel's `Equipment: <carried>/<capacity>` line (was the removed
  inventory line). The Experiment tab's "Available" list also switched to
  `combined_sorted`.

## Out of scope

- A "transfer/drop the whole stack" control — one unit per click only.
- Per-item weight instead of a flat total-count cap.
- Quest rewards routing through the equipment — they go to storage, deliberately
  (see "What goes where").
- Any other enhancer (metal detector, axe→chop) — the Satchel is the first.
- A "you're overloaded" event when removing the Satchel drops you below
  capacity — the silent refusal of the next `add_to_equipment` is the only signal.
