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

    // The active/available/completed transitions are covered end to end by
    // `tests/features/quests.feature`; this is the in-crate smoke check that
    // `overview` shapes a fresh game right.
    #[test]
    fn fresh_game_offers_only_the_dependency_free_quest() {
        let game = Game::default();
        let overview = overview(&game);

        assert!(overview.active.is_none());
        assert!(overview.completed.is_empty());
        let available: Vec<QuestID> = overview.available.iter().map(|q| q.id).collect();
        assert_eq!(available, [QuestID::ExploreRuins]);
    }
}
