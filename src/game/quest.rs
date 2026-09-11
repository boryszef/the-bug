use super::item::Item;
use super::map::{Poi, TerrainType};
use super::unlock::Unlocked;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestID {
    ExploreRuins,
    OldCivilization,
    CraftAxe,
    StockUp,
}

/// Failure reasons for [`super::Game::accept_quest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestError {
    /// Another quest is already open; only one can be active at a time.
    AnotherQuestActive,
    /// This quest is already in `Player::completed_quests()`.
    AlreadyCompleted,
    /// Not every quest in `Quest::dependencies` has been completed yet.
    DependenciesNotMet,
}

/// A game action that can count toward an open quest's [`QuestCondition`].
/// Fired at the moment the action happens (see `Game::grant_item`/`walk`) —
/// not stored or replayed, see docs/quest-system.md for why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EventTypeID {
    FindItem(Item),
    CraftItem(Item),
    VisitTerrain(TerrainType),
    VisitPoi(Poi),
    /// One successful hunt — a hunt that brought at least one thing back.
    Hunt,
}

/// What it takes to complete a quest: `count` occurrences of `event`.
pub(super) struct QuestCondition {
    pub(super) event: EventTypeID,
    pub(super) count: u32,
}

impl QuestCondition {
    /// Whether `event` is the kind of occurrence this condition counts.
    pub(super) fn matches(&self, event: EventTypeID) -> bool {
        self.event == event
    }

    /// Whether `progress` occurrences are enough to satisfy this condition.
    pub(super) fn is_satisfied_by(&self, progress: u32) -> bool {
        progress >= self.count
    }
}

pub struct Quest {
    pub id: QuestID,
    pub dependencies: &'static [QuestID],
    pub(super) condition: QuestCondition,
    pub(super) reward_xp: u32,
    pub(super) reward_items: &'static [(Item, u32)],
    /// Front-end features unlocked the moment this quest is accepted — see
    /// docs/tutorial-unlocks.md.
    pub(super) unlocks_on_accept: &'static [Unlocked],
    /// Front-end features unlocked once this quest is completed.
    pub(super) unlocks_on_complete: &'static [Unlocked],
}

impl Quest {
    /// Occurrences of the condition's event needed to complete this quest —
    /// the progress goal (e.g. the `1` in "craft one stone axe").
    pub fn goal(&self) -> u32 {
        self.condition.count
    }
}

pub(super) const QUESTS: &[Quest] = &[
    Quest {
        id: QuestID::ExploreRuins,
        dependencies: &[],
        condition: QuestCondition {
            event: EventTypeID::VisitPoi(Poi::Ruins),
            count: 1,
        },
        reward_xp: 10,
        reward_items: &[],
        unlocks_on_accept: &[Unlocked::Map],
        unlocks_on_complete: &[],
    },
    Quest {
        id: QuestID::OldCivilization,
        dependencies: &[QuestID::ExploreRuins],
        condition: QuestCondition {
            event: EventTypeID::FindItem(Item::CopperWire),
            count: 1,
        },
        reward_xp: 10,
        reward_items: &[(Item::CircuitBoard, 1)],
        unlocks_on_accept: &[Unlocked::Items],
        unlocks_on_complete: &[],
    },
    Quest {
        id: QuestID::CraftAxe,
        dependencies: &[QuestID::OldCivilization],
        condition: QuestCondition {
            event: EventTypeID::CraftItem(Item::StoneAxe),
            count: 1,
        },
        reward_xp: 20,
        reward_items: &[(Item::SolarPanel, 1)],
        unlocks_on_accept: &[Unlocked::Experiment],
        unlocks_on_complete: &[Unlocked::Craft, Unlocked::Disassemble],
    },
    Quest {
        id: QuestID::StockUp,
        dependencies: &[QuestID::CraftAxe],
        condition: QuestCondition {
            event: EventTypeID::Hunt,
            count: 5,
        },
        reward_xp: 30,
        reward_items: &[(Item::Microcontroller, 1)],
        unlocks_on_accept: &[],
        unlocks_on_complete: &[],
    },
];

/// Looks up a quest by id. Panics if `QUESTS` is missing a variant — a bug in
/// the static table, not a runtime condition.
pub(super) fn quest_for(id: QuestID) -> &'static Quest {
    QUESTS
        .iter()
        .find(|quest| quest.id == id)
        .expect("QUESTS must contain every QuestID")
}

/// Whether every quest in `quest.dependencies` is in `completed`.
pub(super) fn dependencies_met(quest: &Quest, completed: &[QuestID]) -> bool {
    quest.dependencies.iter().all(|dep| completed.contains(dep))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_QUEST: Quest = Quest {
        id: QuestID::CraftAxe,
        dependencies: &[QuestID::ExploreRuins],
        condition: QuestCondition {
            event: EventTypeID::CraftItem(Item::StoneAxe),
            count: 1,
        },
        reward_xp: 7,
        reward_items: &[(Item::Cord, 2)],
        unlocks_on_accept: &[],
        unlocks_on_complete: &[],
    };

    #[test]
    fn dependencies_met_is_false_when_a_dependency_is_missing() {
        assert!(!dependencies_met(&FIXTURE_QUEST, &[]));
    }

    #[test]
    fn dependencies_met_is_true_once_every_dependency_is_completed() {
        assert!(dependencies_met(&FIXTURE_QUEST, &[QuestID::ExploreRuins]));
    }
}
