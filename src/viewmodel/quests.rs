//! The quests panel: the player's active/available/completed quests
//! prepared for display.

use crate::game::{Game, Quest};

/// The player's open quest, with how far along it they are.
pub struct ActiveQuest {
    pub quest: &'static Quest,
    pub progress: u32,
}

pub struct Overview {
    pub active: Option<ActiveQuest>,
    pub available: Vec<&'static Quest>,
    pub completed: Vec<&'static Quest>,
}

pub fn overview(game: &Game) -> Overview {
    Overview {
        active: game.player.open_quest().map(|id| ActiveQuest {
            quest: game.quest(id),
            progress: game.player.quest_progress(),
        }),
        available: game.available_quests(),
        completed: game
            .player
            .completed_quests()
            .iter()
            .map(|&id| game.quest(id))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::QuestID;

    #[test]
    fn fresh_game_offers_only_the_dependency_free_quest() {
        let game = Game::default();
        let overview = overview(&game);

        assert!(overview.active.is_none());
        assert!(overview.completed.is_empty());
        let available: Vec<QuestID> = overview.available.iter().map(|q| q.id).collect();
        assert_eq!(available, [QuestID::ExploreRuins]);
    }

    #[test]
    fn accepted_quest_becomes_active_and_leaves_the_available_list() {
        let mut game = Game::default();
        game.accept_quest(QuestID::ExploreRuins).unwrap();

        let overview = overview(&game);

        let active = overview.active.expect("a quest is open");
        assert_eq!(active.quest.id, QuestID::ExploreRuins);
        assert_eq!(active.progress, 0);
        assert!(
            overview.available.is_empty(),
            "nothing else is unlocked yet"
        );
    }

    #[test]
    fn completed_quest_appears_in_completed_and_clears_active() {
        let mut game = Game::default();
        game.player
            .restore_quest_state(None, 0, vec![QuestID::CraftAxe]);

        let overview = overview(&game);

        assert!(overview.active.is_none());
        assert_eq!(overview.completed.len(), 1);
        assert_eq!(overview.completed[0].id, QuestID::CraftAxe);
    }
}
