# The event log

The Events pane shows recent things that have happened, newest first.

## Line shape

```
[02:47] Experiment: 2 Vine -> Cord (new recipe!)
```

- **Timestamp** — session-elapsed time (since the game started), `mm:ss`,
  rolling to `h:mm:ss` past an hour. Monotonic and dependency-free
  (`std::time::Instant`, the same primitive resource decay uses). Written as a
  bracketed prefix by each front end.
- **Colour** — each front end colours the whole line by its `EventKind`:
  default for `Awoke`, green for a find/hunt haul, magenta for a quest
  accepted/completed, yellow for a craft/hunt/disassemble outcome (including
  failures), cyan for an experiment outcome. There's no stored "category" —
  the mapping from kind to colour lives in each front end (`event_color` in
  `src/gui/mod.rs` and `src/tui/app.rs`), so it can't drift from the kind
  itself.
- Event text uses `->`, never the Unicode `→`: egui's default proportional
  font has no glyph for it, so it would render as a tofu box in the gui's log
  (`i18n::tests::every_event_kind_renders_in_every_language` guards this).

## What gets logged

Not every action produces one — see `docs/event-worthiness.md` for the rule
deciding which do (a player-visible state change, or a costly roll resolved
even to nothing) and where the current variants stand against it.

What happened is captured structurally as an [`EventKind`](../src/game/event.rs)
variant — one per `Game::log(...)` call site in `src/game/mod.rs` — and
rendered into player-facing text entirely by `i18n::event()`
(`src/i18n/mod.rs`), in whichever language is active. Nothing here decides
wording; a variant with no `.ftl` message would compile but render
"Unknown localization", which is exactly what
`every_event_kind_renders_in_every_language` exists to catch.

| Action | `EventKind` | Example (English) |
|---|---|---|
| Waking up | `Awoke` | `You wake up and decide to go for a walk.` |
| Searching a tile | `Found` | `You find a Branch in the Forest.` |
| Hunting a tile | `Hunted` / `HuntMissed` / `HuntUnprepared` | `Hunt: 1 Meat + 1 Hide.` / `Hunt: the quarry got away.` / `You need a Wooden Bow to hunt.` |
| Crafting | `Crafted` / `CraftShortage` / `CraftMissingTool` / `UnknownRecipe` | `You craft a Cord.` / `You don't have enough Vine to craft a Cord.` |
| Experimenting | `Experimented` / `ExperimentFailed` / `ExperimentShortage` | see below |
| Disassembling | `Disassembled` | `You take apart a Stone Axe, recovering 1 Branch + 1 Stone + 1 Cord.` |
| Accepting/completing a quest | `QuestAccepted` / `QuestCompleted` | — |
| **Walking** | — | *not logged* |

### Experiment lines (one per attempt, precise)

```
Experiment: 2 Vine -> Cord (new recipe!)
Experiment: 2 Vine -> Cord
Experiment: 1 Branch + 1 Vine -> nothing
Experiment: 5 Stone -> not enough Stone (have 1, need 5)
```

Inputs are sorted by item and joined with ` + ` (`i18n::describe_items`).
Running an experiment with nothing selected does nothing (no log line).

### Crafting a quest's target item also logs the quest's completion

`Game::craft` and `Game::experiment` log their own outcome (`Crafted` /
`Experimented`) *before* granting the item — so if that item happens to
satisfy the player's open quest, the resulting `QuestCompleted` line is
pushed second and appears above the craft/experiment line in the
newest-first pane, reading as cause-then-effect top-down.

## Code

- `src/game/event.rs` — `EventKind`; `Event { kind, elapsed }`.
- `src/game/mod.rs` — `Game::log(kind)` is the single entry point (private;
  every action method calls it directly).
- `src/i18n/mod.rs` — `event(kind, lang)` renders one `EventKind` into text,
  entirely from `.ftl` messages.
- `src/viewmodel/events.rs` — `recent()` yields
  `RecentEvent { timestamp, kind }`; `compact()` owns the time format. Colour
  is not decided here.
- `src/gui/mod.rs` / `src/tui/app.rs` — `render_events` / `event_color()`
  (one pair per front end, kept in sync by hand).
- `src/save.rs` — `EventState` persists `kind` (the full `EventKind`, not
  pre-rendered text) and `elapsed_secs`. An older save file's `{ category,
  text }` shape is deliberately not migrated — see docs/i18n-plan.md — so it
  fails to load with `io::ErrorKind::InvalidData` rather than being silently
  reinterpreted.

## Out of scope (for now)

- Wall-clock / calendar time.
- A category filter, or category tags in the text.
