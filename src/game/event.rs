use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

use super::{Item, QuestID, TerrainType};

/// Which part of the game an event belongs to. Used to colour the log.
/// Derived from [`EventKind`] (see [`EventKind::category`]), not stored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    #[default]
    General,
    Experiment,
    Crafting,
}

/// What happened, described structurally rather than as rendered text —
/// wording lives entirely in `i18n`. One variant per `Game::log(...)` call
/// site (see `src/game/mod.rs`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    /// The game's opening event.
    Awoke,
    /// Found `item` while searching `terrain`.
    Found { item: Item, terrain: TerrainType },
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
    /// Took `item` apart, recovering its reversible recipe's inputs.
    Disassembled { item: Item },
}

impl EventKind {
    pub fn category(&self) -> EventCategory {
        match self {
            EventKind::Awoke | EventKind::Found { .. } => EventCategory::General,
            EventKind::QuestAccepted { .. } | EventKind::QuestCompleted { .. } => {
                EventCategory::General
            }
            EventKind::UnknownRecipe { .. }
            | EventKind::CraftShortage { .. }
            | EventKind::Crafted { .. }
            | EventKind::Disassembled { .. } => EventCategory::Crafting,
            EventKind::ExperimentShortage { .. }
            | EventKind::ExperimentFailed { .. }
            | EventKind::Experimented { .. } => EventCategory::Experiment,
        }
    }
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

    pub fn category(&self) -> EventCategory {
        self.kind.category()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_matches_each_variant_s_part_of_the_game() {
        assert_eq!(EventKind::Awoke.category(), EventCategory::General);
        assert_eq!(
            EventKind::Crafted { output: Item::Cord }.category(),
            EventCategory::Crafting
        );
        assert_eq!(
            EventKind::ExperimentFailed { items: vec![] }.category(),
            EventCategory::Experiment
        );
    }
}
