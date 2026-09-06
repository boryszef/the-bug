# Feature: quest system (mechanics)

## Why

From `TODO.md`: "implement the quest system: one quest unlocks other quests,
completing a quest awards XP and items, quests also tell the story of the game
world." This covers the mechanics only — accepting a quest, tracking progress,
completing it, granting rewards, unlocking dependents, and persisting all of
that. No UI/viewmodel work is included; `Game`'s new methods are the surface a
future UI would call.

## Design: how completion is verified

Quests declare a single `QuestCondition { event: EventTypeID, count: u32 }`.
`EventTypeID` names a game action (`CraftItem(Item)`, `VisitTerrain(TerrainType)`,
`VisitPoi(Poi)` today). Instead of replaying a stored event history, `Game` fires the relevant
`EventTypeID` at the moment the action happens and reacts immediately:
`Player.quest_progress: u32` increments when the fired event matches the open
quest's condition, and the quest completes once it reaches `count`. The counter
only exists while a quest is open and is reset to `0` on acceptance, so it's
exactly "matching events since this quest started" without needing to store or
scan a log.

This was chosen over storing `EventTypeID` on the existing display `Event`/
`EventCategory` log (used for the on-screen log and persisted in
`save.rs::EventState`) because:
- Walking is deliberately **not** added to that log today (see
  `docs/event-log.md`), but `VisitTerrain` / `VisitPoi` need a signal on every
  step.
- A scan-based "since start" check needs a stored start marker (index or
  timestamp) that must itself survive save/load correctly, on top of the
  existing `started: Instant` reconstruction `Game::from_saved` already does.
  A plain `u32` counter has none of that risk.

If a future quest needs multiple conditions or a real audit trail, that can be
layered on later — not needed for the two quests that exist today.

## Types (`src/game.rs`)

- `QuestID` (existing scaffold) — gains `Clone, Copy, PartialEq, Eq, Serialize,
  Deserialize` derives (needed to fix a build break: `Player` derives `Debug`
  and contains `Option<QuestID>`, but `QuestID` didn't derive `Debug`).
- `EventTypeID` — `CraftItem(Item)`, `VisitTerrain(TerrainType)`,
  `VisitPoi(Poi)`. Not persisted; only used transiently to route a game action
  to the open quest's condition. ("Explore the ruins" counts a `VisitPoi`.)
- `QuestCondition { event: EventTypeID, count: u32 }`.
- `Quest` (existing scaffold) — gains `condition: QuestCondition`,
  `reward_xp: u32`, `reward_items: &'static [(Item, u32)]`.
- `QuestError { AnotherQuestActive, AlreadyCompleted, DependenciesNotMet }`.

## `Player`

- `open_quest: Option<QuestID>` (existing), `quests_completed: Vec<QuestID>`
  (existing), `quest_progress: u32` (new).
- Read accessors: `open_quest()`, `quest_progress()`, `completed_quests()`.
- `pub(crate) restore_quest_state(...)` for `save.rs` to use on load, mirroring
  the existing `grant_recipe`.

## `Game`

- `accept_quest(&mut self, id: QuestID) -> Result<(), QuestError>` — rejects a
  second concurrently-open quest, an already-completed quest, or unmet
  dependencies. Logs "Quest accepted: {name}" (`EventCategory::General`).
- `available_quests(&self) -> Vec<&'static Quest>` — quests not open, not
  completed, whose dependencies are all in `quests_completed`.
- Internal `note_quest_event`/`complete_open_quest`/`quest_for` drive
  progress and completion. Completion grants `reward_xp`/`reward_items` and
  logs "Quest complete: {name}!".
- `grant_item(&mut self, item, amount)` — shared by `craft()` and
  `experiment()` success paths (both already had the identical inventory-grant
  line); fires `EventTypeID::CraftItem`. `search()` (found in the wild) and
  `disassemble()` (recovered by taking something apart) are **not** routed
  through it — those aren't "crafting" and shouldn't silently satisfy a
  craft-count quest.
- `walk()` fires `EventTypeID::VisitTerrain` for the destination tile after
  moving, then `EventTypeID::VisitPoi` if the tile has a POI. This does not add
  anything to the visible event log — walking stays unlogged, per
  `docs/event-log.md`.

## Persistence (`src/save.rs`)

`PlayerState` gains `open_quest`, `quest_progress`, `quests_completed`
(`#[serde(default)]`, so older save files still load with no active/completed
quests).

Alongside this, `capture`/`restore` are refactored to use two new
`pub(crate)` traits defined in `game.rs`, `SaveState { type Saved; fn
save_state(&self) -> Self::Saved; }` and `RestoreState: Sized { type Saved; fn
restore_state(saved: Self::Saved) -> io::Result<Self>; }`. `Player`, `Map`,
and `Event` each implement both, with the `impl` blocks living right next to
their struct/`impl` definitions in `game.rs` (not bundled into `save.rs`) —
this colocates each type's save/restore mapping with the type itself instead
of one large free function in `save.rs` having to remember every field, which
is exactly the kind of thing that almost left the quest fields unpersisted.
`PlayerState`/`MapState`/`EventState` (the DTOs) and the `terrain_code`/
`parse_terrain` helpers stay in `save.rs` as `pub(crate)`, since the on-disk
shape is still deliberately kept separate from the live structs; `game.rs`'s
impls just reference them via `crate::save::...`. This does mean `game.rs` now
depends on `save.rs` for those DTO/helper types, reversing the previous
one-directional dependency — an accepted trade-off for colocating the impls.
The pre-existing `SaveState` DTO struct (the whole-file shape) is renamed to
`SaveFile` to free up the trait name; the on-disk JSON is unchanged.

## What is *not* built here

- No multi-condition quests — `Quest` has exactly one `QuestCondition`.
- Reward amounts are game content, filled in with placeholder values; not
  decided here. (The dependency chain — `CraftAxe` depends on `ExploreRuins` —
  was set later, in "change initial quests".)

The quests panel (`src/ui/quests.rs`, `src/viewmodel/quests.rs`) is wired up
— see `docs/panel-layout.md`.

## Code

- `src/game.rs` — `QuestID`, `Quest`, `QUESTS`, `EventTypeID`, `QuestCondition`,
  `QuestError`, `Player` quest fields/accessors, `Game::accept_quest` /
  `available_quests` / `grant_item` / `note_quest_event` /
  `complete_open_quest` / `quest_for`, `walk()`/`craft()`/`experiment()`
  trigger sites.
- `src/save.rs` — `SaveState`/`RestoreState` traits and impls, `PlayerState`
  quest fields, `SaveFile` (renamed from `SaveState`).
