//! The event log prepared for display.

use std::time::Duration;

use crate::game::{EventCategory, Game};

/// One event-log line ready to show: a compact session timestamp, the category
/// (for colouring) and the text.
pub struct RecentEvent<'a> {
    pub timestamp: String,
    pub category: EventCategory,
    pub text: &'a str,
}

/// The `count` most recent events, newest first.
pub fn recent(game: &Game, count: usize) -> impl Iterator<Item = RecentEvent<'_>> {
    game.events()
        .iter()
        .rev()
        .take(count)
        .map(|event| RecentEvent {
            timestamp: compact(event.elapsed()),
            category: event.category(),
            text: event.text(),
        })
}

/// Session-elapsed time as `mm:ss`, or `h:mm:ss` once past an hour.
fn compact(elapsed: Duration) -> String {
    let total = elapsed.as_secs();
    let (hours, minutes, seconds) = (total / 3600, total % 3600 / 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_newest_first_and_respects_count() {
        let mut game = Game::default();
        game.craft("aaa"); // unknown recipe -> deterministic log line
        game.craft("bbb");

        let recent: Vec<RecentEvent> = recent(&game, 2).collect();
        assert_eq!(recent.len(), 2);
        assert!(recent[0].text.contains("bbb"));
        assert!(recent[1].text.contains("aaa"));
    }

    #[test]
    fn count_larger_than_log_returns_whole_log() {
        let game = Game::default();
        assert_eq!(recent(&game, 100).count(), game.events().len());
    }

    #[test]
    fn first_event_of_a_fresh_game_is_stamped_zero() {
        let game = Game::default();
        assert_eq!(recent(&game, 1).next().unwrap().timestamp, "00:00");
    }

    #[test]
    fn recent_exposes_the_event_category() {
        let mut game = Game::default();
        game.craft("whatever");
        assert_eq!(
            recent(&game, 1).next().unwrap().category,
            EventCategory::Crafting
        );
    }

    #[test]
    fn compact_formats_minutes_and_rolls_over_to_hours() {
        assert_eq!(compact(Duration::ZERO), "00:00");
        assert_eq!(compact(Duration::from_secs(167)), "02:47");
        assert_eq!(compact(Duration::from_secs(3800)), "1:03:20");
    }
}
