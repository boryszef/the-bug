use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

/// Which part of the game an event belongs to. Used to colour the log.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    #[default]
    General,
    Experiment,
    Crafting,
}

/// A message in the event log, tagged with its category and how far into the
/// session it happened.
#[derive(Debug)]
pub struct Event {
    category: EventCategory,
    text: String,
    elapsed: Duration,
}

impl Event {
    pub(crate) fn new(
        category: EventCategory,
        text: impl Into<String>,
        elapsed: Duration,
    ) -> Event {
        Event {
            category,
            text: text.into(),
            elapsed,
        }
    }

    pub fn category(&self) -> EventCategory {
        self.category
    }

    pub fn text(&self) -> &str {
        &self.text
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
            category: self.category(),
            text: self.text().to_string(),
            elapsed_secs: self.elapsed().as_secs_f64(),
        }
    }
}

impl super::RestoreState for Event {
    type Saved = crate::save::EventState;

    fn restore_state(saved: Self::Saved) -> io::Result<Event> {
        Ok(Event::new(
            saved.category,
            saved.text,
            Duration::from_secs_f64(saved.elapsed_secs.max(0.0)),
        ))
    }
}
