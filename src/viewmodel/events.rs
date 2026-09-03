//! The event log prepared for display.

use crate::game::Game;

/// The `count` most recent events, newest first.
pub fn recent(game: &Game, count: usize) -> impl Iterator<Item = &str> {
    game.events.iter().rev().take(count).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Direction;

    #[test]
    fn returns_newest_first_and_respects_count() {
        let mut game = Game::default();
        game.walk(Direction::East);
        game.walk(Direction::North);

        let recent: Vec<&str> = recent(&game, 2).collect();
        assert_eq!(recent.len(), 2);
        assert!(recent[0].starts_with("You walk north"));
        assert!(recent[1].starts_with("You walk east"));
    }

    #[test]
    fn count_larger_than_log_returns_whole_log() {
        let game = Game::default();
        assert_eq!(recent(&game, 100).count(), game.events.len());
    }
}
