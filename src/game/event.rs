use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

use super::{FoundIn, Item, QuestID};

/// What happened, described structurally rather than as rendered text —
/// wording lives entirely in `i18n`. One variant per `Game::log(...)` call
/// site (see `src/game/mod.rs`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    /// The game's opening event.
    Awoke,
    /// Found `item` while searching a tile, `source` says whether it came from
    /// the terrain or the tile's POI.
    Found { item: Item, source: FoundIn },
    /// Accepted `quest` as the open quest.
    QuestAccepted { quest: QuestID },
    /// Completed the open quest.
    QuestCompleted { quest: QuestID },
    /// Tried to craft a recipe by a name the player doesn't know.
    UnknownRecipe { recipe: String },
    /// Knew the recipe for `output` but didn't have enough `needed`.
    CraftShortage { needed: Item, output: Item },
    /// Successfully crafted `output`.
    Crafted { output: Item },
    /// Tried to experiment with `items` but didn't have enough of `missing`
    /// (`available` on hand, `needed` for the attempt).
    ExperimentShortage {
        items: Vec<(Item, u32)>,
        missing: Item,
        available: u32,
        needed: u32,
    },
    /// Experimented with `items`; no recipe matched.
    ExperimentFailed { items: Vec<(Item, u32)> },
    /// Experimented with `items` and produced `output` — `newly_learned` is
    /// set the first time this recipe is discovered.
    Experimented {
        items: Vec<(Item, u32)>,
        output: Item,
        newly_learned: bool,
    },
    /// Took `item` apart, recovering the inputs of the recipe it decomposes into.
    Disassembled { item: Item },
}

/// One entry in the event log: what happened, and how far into the session
/// it happened.
#[derive(Debug)]
pub struct Event {
    kind: EventKind,
    elapsed: Duration,
}

impl Event {
    pub(crate) fn new(kind: EventKind, elapsed: Duration) -> Event {
        Event { kind, elapsed }
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
    }

    /// Time from the start of the session to when this event was logged.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

impl super::SaveState for Event {
    type Saved = crate::save::EventState;

    fn save_state(&self) -> Self::Saved {
        crate::save::EventState {
            kind: self.kind.clone(),
            elapsed_secs: self.elapsed().as_secs_f64(),
        }
    }
}

impl super::RestoreState for Event {
    type Saved = crate::save::EventState;

    fn restore_state(saved: Self::Saved) -> io::Result<Event> {
        Ok(Event::new(
            saved.kind,
            Duration::from_secs_f64(saved.elapsed_secs.max(0.0)),
        ))
    }
}
