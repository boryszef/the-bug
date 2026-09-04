use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Widget, Wrap},
};

use crate::game::QuestID;
use crate::viewmodel::quests::Overview;

/// The quests panel: a scrollable list of quests available to accept, plus
/// the active quest's progress and a list of completed quests. Owns only the
/// cursor; quest data comes from [`crate::viewmodel::quests::overview`].
#[derive(Default)]
pub(super) struct Quests {
    cursor: usize,
}

/// What [`App`](super::App) should do after the panel consumed one key.
#[derive(PartialEq, Eq, Debug)]
pub(super) enum Outcome {
    Stay,
    /// Run `game.accept_quest(id)`.
    Accept(QuestID),
}

impl Quests {
    /// Feeds one key to the panel. Navigation/accepting only applies to the
    /// "available to accept" list, which is only actionable while no quest
    /// is already open.
    pub(super) fn handle_key(&mut self, key: KeyCode, overview: &Overview) -> Outcome {
        if overview.active.is_some() {
            return Outcome::Stay;
        }
        match key {
            KeyCode::Up => {
                self.cursor = self.cursor.saturating_sub(1);
                Outcome::Stay
            }
            KeyCode::Down => {
                if self.cursor + 1 < overview.available.len() {
                    self.cursor += 1;
                }
                Outcome::Stay
            }
            KeyCode::Enter => match overview.available.get(self.cursor) {
                Some(quest) => Outcome::Accept(quest.id),
                None => Outcome::Stay,
            },
            _ => Outcome::Stay,
        }
    }

    /// Draws the panel: the active quest's progress (if any) or the
    /// available-to-accept list, plus completed quests.
    pub(super) fn render(&self, area: Rect, buf: &mut Buffer, overview: &Overview) {
        let inner = super::panel_frame(area, " Quests ", buf);
        let rows = Layout::vertical([
            Constraint::Length(10),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(inner);

        match &overview.active {
            Some(active) => {
                let text = format!(
                    "{}\n{}/{} — {}",
                    active.quest.name,
                    active.progress,
                    active.quest.goal(),
                    active.quest.description
                );
                Paragraph::new(text)
                    .block(Block::bordered().title(" Active "))
                    .wrap(Wrap { trim: true })
                    .render(rows[0], buf);
            }
            None => {
                Paragraph::new("No quest accepted.")
                    .block(Block::bordered().title(" Active "))
                    .render(rows[0], buf);
            }
        }

        if overview.active.is_some() {
            Paragraph::new("Complete your active quest to accept another.")
                .block(Block::bordered().title(" Available "))
                .wrap(Wrap { trim: true })
                .render(rows[1], buf);
        } else if overview.available.is_empty() {
            Paragraph::new("No quests available right now.")
                .block(Block::bordered().title(" Available "))
                .render(rows[1], buf);
        } else {
            let items = overview.available.iter().enumerate().map(|(index, quest)| {
                let style = if index == self.cursor {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(quest.name).style(style)
            });
            let list = List::new(items).block(Block::bordered().title(" Available "));
            Widget::render(list, rows[1], buf);
        }

        let completed = if overview.completed.is_empty() {
            "Completed: (none yet)".to_string()
        } else {
            let names: Vec<&str> = overview.completed.iter().map(|q| q.name).collect();
            format!("Completed: {}", names.join(", "))
        };
        Paragraph::new(Line::from(completed)).render(rows[2], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Game, QuestID};
    use crate::viewmodel::quests;

    #[test]
    fn cursor_up_saturates_at_zero() {
        let game = Game::default();
        let overview = quests::overview(&game);
        let mut panel = Quests::default();

        panel.handle_key(KeyCode::Up, &overview);

        assert_eq!(panel.cursor, 0);
    }

    #[test]
    fn cursor_down_stops_at_last_available_quest() {
        let game = Game::default();
        let overview = quests::overview(&game);
        let mut panel = Quests::default();

        for _ in 0..10 {
            panel.handle_key(KeyCode::Down, &overview);
        }

        assert_eq!(panel.cursor, overview.available.len() - 1);
    }

    #[test]
    fn enter_on_available_quest_accepts_it() {
        let game = Game::default();
        let overview = quests::overview(&game);
        let mut panel = Quests::default();
        let expected = overview.available[0].id;

        assert_eq!(
            panel.handle_key(KeyCode::Enter, &overview),
            Outcome::Accept(expected)
        );
    }

    #[test]
    fn enter_on_empty_available_list_stays() {
        let mut game = Game::default();
        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftArrows, QuestID::ExploreRuins]);
        let overview = quests::overview(&game);
        let mut panel = Quests::default();

        assert_eq!(panel.handle_key(KeyCode::Enter, &overview), Outcome::Stay);
    }

    #[test]
    fn keys_are_ignored_while_a_quest_is_active() {
        let mut game = Game::default();
        game.accept_quest(QuestID::CraftArrows).unwrap();
        let overview = quests::overview(&game);
        let mut panel = Quests::default();

        assert_eq!(panel.handle_key(KeyCode::Down, &overview), Outcome::Stay);
        assert_eq!(panel.cursor, 0);
        assert_eq!(panel.handle_key(KeyCode::Enter, &overview), Outcome::Stay);
    }
}
