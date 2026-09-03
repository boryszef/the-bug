# Feature: event-log timestamps

## Why

The Events pane showed plain messages with no indication of when anything
happened during the session.

## Outcome

Each event carries a **session-elapsed** timestamp (time since the game
started), and the Events pane renders it as a bracketed prefix:

```
[00:00] You wake up and decide to have a walk.
[00:12] You walk north and visit Forest.
[00:19] You found a Stick in the Forest.
```

Format is `mm:ss`, rolling to `h:mm:ss` once a session passes an hour.
Session-elapsed (not wall-clock) keeps it monotonic and dependency-free —
`std::time::Instant`, the same primitive already used for resource decay.

## Scope

- `src/game.rs`:
  - `pub struct Event { text, elapsed }` with `text()` / `elapsed()` accessors.
  - `Game.events` is now `Vec<Event>`, **private**, exposed via
    `Game::events() -> &[Event]`.
  - `Game.started: Instant`; private `Game::log(text)` stamps and appends —
    the single place events enter the log.
- `src/viewmodel/events.rs`:
  - `recent()` yields `RecentEvent { timestamp: String, text: &str }`.
  - private `compact(Duration) -> String` owns the `mm:ss` / `h:mm:ss` rule.
- `src/ui/app.rs::render_events`: `format!("[{}] {}", timestamp, text)`.

## Out of scope

- Wall-clock / calendar time (needs a date-time crate for local time).
- Styling the timestamp differently from the message.
- Persisting or exporting the log.
