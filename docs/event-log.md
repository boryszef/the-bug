# The event log

The Events pane shows recent things that have happened, newest first.

## Line shape

```
[02:47] Experiment: 2 Vine → Cord (new recipe!)
```

- **Timestamp** — session-elapsed time (since the game started), `mm:ss`,
  rolling to `h:mm:ss` past an hour. Monotonic and dependency-free
  (`std::time::Instant`, the same primitive resource decay uses). Written as a
  bracketed prefix by the UI.
- **Category** — every event has one of `General` / `Experiment` / `Crafting`.
  The UI colours the whole line by it: General = default, Crafting = yellow,
  Experiment = cyan.

## What gets logged

| Action | Category | Example |
|---|---|---|
| Waking up | General | `You wake up and decide to go for a walk.` |
| Searching a tile | General | `You find a Stick in the Forest.` |
| Crafting | Crafting | `You craft a Cord.` / `You don't have enough Vine to craft a Cord.` |
| Experimenting | Experiment | see below |
| **Walking** | — | *not logged* |

### Experiment lines (one per attempt, precise)

```
Experiment: 2 Vine → Cord (new recipe!)
Experiment: 2 Vine → Cord
Experiment: 1 Stick + 1 Vine → nothing
Experiment: 5 Stone → not enough Stone (have 1, need 5)
```

Inputs are sorted by material and joined with ` + `. Running an experiment with
nothing selected does nothing (no log line).

## Code

- `src/game.rs` — `EventCategory`; `Event { category, text, elapsed }`;
  `Game::log(category, text)` is the single entry point; `experiment()` builds
  its line via `describe_inputs()`.
- `src/viewmodel/events.rs` — `recent()` yields
  `RecentEvent { timestamp, category, text }`; `compact()` owns the time format.
  Colour is not decided here.
- `src/ui/app.rs` — `render_events` / `category_color()`.
- `src/save.rs` — `EventState` persists `category` (`#[serde(default)]` →
  `General` for older / hand-made files).

## Out of scope (for now)

- Wall-clock / calendar time.
- A category filter, or category tags in the text.
