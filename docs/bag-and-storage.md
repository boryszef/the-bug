# Feature: the Bag (limited) and Storage (unlimited)

## Why

`TODO.md`: "Players 'bag' should have a limited size, when full, the player
should take their findings to the village and deposit them in the storage or
drop." Until now `Player.inventory` was a single unlimited pool everything
read and wrote — there was nothing to carry, nowhere to run out of room.

`Player.inventory` keeps its exact name and role: the unlimited village
Storage, entirely unchanged in behavior. New `Player.bag` is the
capacity-limited pool the player actually carries (`BAG_CAPACITY = 50`, a
flat total across every item type combined — not a per-item or per-slot
limit).

## What goes where

Crafting, experimenting, and disassembling **don't change** — they already
only run at the Village (`docs/village-crafting.md`), so their consumables,
tools, and outputs keep using storage exactly as before: a workshop tool or
a stack of materials sitting at the village isn't something you'd carry in a
limited travel bag. Quest rewards (`grant_reward`) also keep going to
storage — a guaranteed milestone reward shouldn't risk being lost to a full
bag, and no live quest grants items today regardless.

What changes is `search`/`hunt`'s *yields* — a search or hunt happens out in
the field, so what it turns up goes into the bag (`Player::add_to_bag`), not
storage. `hunt`'s *gear prerequisite* moves too: the Wooden Bow and Arrow
must now be in the bag, not storage, since gear has to be carried to be used
away from the village — unlike a craft/experiment tool, which stays at the
workshop where it's used. This is what gives Transfer a real bidirectional
purpose: gear flows Storage→Bag before a hunting trip, loot flows Bag→Storage
on return.

## The Player API

- `add_to_bag(item, amount) -> bool` — all-or-nothing: fails (nothing added)
  if the bag's total would exceed `BAG_CAPACITY`.
- `bag_count`/`has_item_in_bag` and `spend_from_bag` mirror the existing
  `inventory_count`/`has_item`/`spend` exactly, scoped to the bag.
- `transfer_to_storage(item, amount) -> bool` / `transfer_to_bag(item,
  amount) -> bool` — move between the two pools. `transfer_to_bag` tries
  `add_to_bag` (capacity-checked) *before* spending from storage, so a
  transfer that doesn't fit never leaves an item in limbo.
- `bag_progress() -> (u32, u32)` — `(carried, capacity)`, both summed across
  every item type; mirrors `recipe_progress`'s `(known, total)` shape, for
  the gui's "Bag: 37/50" header.

`Game` adds four thin wrappers for the player-initiated side: `transfer_to_storage`,
`transfer_to_bag` (both village-gated via `at_craftable_location`, reused from
village-crafting — silent no-op away from it, or without enough stock/room,
same as any other refused village-gated action) and `drop_from_bag` (no
gate — the bag can be dropped from anywhere) / `drop_from_storage`
(village-gated).

## What's an event and what isn't

**`BagFull { item }`** — logged when a search or hunt roll can't be carried
(the bag is already full). Per `docs/event-worthiness.md`'s cost-bearing-roll
rule: the player committed to an uncertain roll, and "couldn't carry it" is a
real, felt consequence of that roll, distinct from a deterministic refusal.
A hunt whose catch only partly fits still logs `Hunted` for what made it in
(and still counts toward an open quest) — but only if something was actually
kept; a catch entirely lost to a full bag doesn't complete "Stock Up for Hard
Times" any more than a wasted arrow does.

**`Dropped { item }`** — logged whenever `drop_from_bag`/`drop_from_storage`
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
mirroring `gui::experiment`'s layout: **Bag** (left, with a running
`carried/capacity` header) and **Inventory** (right). Each row is the item,
its quantity, and:

- Bag: a "→" button (transfer to storage) and an "x" (drop one).
- Inventory: a "←" button (transfer to bag) and an "x" (drop one).

Transfer (both directions) and the Inventory "x" are enabled only at the
Village (`at_village`, same pattern as Craft/Experiment/Disassemble); the
Bag "x" is always enabled. Both move exactly one unit per click, matching
the existing one-unit-per-click precedent (`ItemSelection::add`) — a
"move the whole stack" control is a reasonable future refinement, not built
now. The Player panel's old single "Inventory: ..." line is removed
(**gui only** — `tui`'s own Player panel is untouched, still showing
storage exactly as it always has, since `player.inventory` hasn't changed
meaning).

`tui` gets no Bag/Inventory management at all, matching the established
"frozen, not developed further" precedent from village-crafting and the
quests-list feature — `search`/`hunt` still work there, but their gains
pile up in a bag `tui` has no way to view or empty. `Panel` is shared
between the two front ends, though, so `tui` still needed three minimal
placeholder arms (a "not available in this view" message) purely to keep
compiling — not a feature build.

## Scope

- `src/game/player.rs` — `BAG_CAPACITY`, `Player.bag`, `bag_total`,
  `add_to_bag`, `bag_count`, `has_item_in_bag`, `spend_from_bag`,
  `transfer_to_storage`, `transfer_to_bag`, `bag_progress`.
- `src/save.rs` — `PlayerState.bag`, `#[serde(default)]` so an old save
  loads with an empty bag.
- `src/game/mod.rs` — `search`/`hunt` route through the bag; `hunt`'s gear
  check; `Game::transfer_to_storage`/`transfer_to_bag`/`drop_from_bag`/`drop_from_storage`.
- `src/game/event.rs` + `src/i18n/mod.rs` + both `.ftl`s — `BagFull`,
  `Dropped`.
- `src/viewmodel/items.rs` (new, `gui`-only) — `Overview { bag,
  bag_progress, storage }`.
- `src/gui/items.rs` (new), `src/gui/mod.rs` — the tab, its wiring, the
  Player panel's inventory-line removal.
- `src/tui/app.rs` — the three placeholder arms.

## Out of scope

- A `tui` Bag/Inventory view.
- A "transfer/drop the whole stack" control — one unit per click only.
- Per-item weight instead of a flat total-count cap.
- Quest rewards routing through the bag — they go to storage, deliberately
  (see "What goes where").
