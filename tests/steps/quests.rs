//! Steps for `tests/features/quests.feature`. Quest state is reached through
//! real gameplay (`complete_quest` in `world.rs`), never `restore_quest_state`.
//! The active/available/completed lists are read through
//! `viewmodel::quests::overview`.

use cucumber::{given, then, when};
use the_bug::game::QuestError;
use the_bug::viewmodel;

use crate::steps::world::{GameWorld, quest};

#[given(regex = r#"^the player has accepted "([^"]+)"$"#)]
async fn has_accepted(world: &mut GameWorld, name: String) {
    world
        .game
        .accept_quest(quest(&name))
        .unwrap_or_else(|e| panic!("could not accept {name:?}: {e:?}"));
}

#[when(regex = r#"^the player accepts "([^"]+)"$"#)]
async fn accepts(world: &mut GameWorld, name: String) {
    world.mark();
    world.accept_result = Some(world.game.accept_quest(quest(&name)));
}

#[then(regex = r#"^"([^"]+)" is accepted$"#)]
async fn is_accepted(world: &mut GameWorld, name: String) {
    assert_eq!(world.accept_result, Some(Ok(())), "accept did not succeed");
    assert_eq!(world.game.player.open_quest(), Some(quest(&name)));
}

#[then(regex = r#"^accepting "[^"]+" is refused because another quest is active$"#)]
async fn refused_another_active(world: &mut GameWorld) {
    assert_eq!(
        world.accept_result,
        Some(Err(QuestError::AnotherQuestActive))
    );
}

#[then(regex = r#"^accepting "[^"]+" is refused because its prerequisites aren't met$"#)]
async fn refused_prerequisites(world: &mut GameWorld) {
    assert_eq!(
        world.accept_result,
        Some(Err(QuestError::DependenciesNotMet))
    );
}

#[then(regex = r#"^the active quest is "([^"]+)"$"#)]
async fn active_quest_is(world: &mut GameWorld, name: String) {
    let overview = viewmodel::quests::overview(&world.game);
    assert_eq!(
        overview.active.map(|a| a.quest.id),
        Some(quest(&name)),
        "wrong active quest"
    );
}

#[then("there is no active quest")]
async fn no_active_quest(world: &mut GameWorld) {
    assert!(viewmodel::quests::overview(&world.game).active.is_none());
}

#[then("no quest is available")]
async fn no_quest_available(world: &mut GameWorld) {
    assert!(
        viewmodel::quests::overview(&world.game)
            .available
            .is_empty()
    );
}

#[then(regex = r#"^"([^"]+)" is available$"#)]
async fn quest_is_available(world: &mut GameWorld, name: String) {
    let available = viewmodel::quests::overview(&world.game).available;
    assert!(
        available.iter().any(|q| q.id == quest(&name)),
        "{name:?} is not available"
    );
}

#[then(regex = r#"^"([^"]+)" is not available$"#)]
async fn quest_is_not_available(world: &mut GameWorld, name: String) {
    let available = viewmodel::quests::overview(&world.game).available;
    assert!(
        available.iter().all(|q| q.id != quest(&name)),
        "{name:?} is unexpectedly available"
    );
}

#[then(regex = r#"^"([^"]+)" is completed$"#)]
async fn quest_is_completed(world: &mut GameWorld, name: String) {
    let completed = viewmodel::quests::overview(&world.game).completed;
    assert!(
        completed.iter().any(|q| q.id == quest(&name)),
        "{name:?} is not in the completed list"
    );
}

#[then(regex = r"^the quest progress is (\d+)$")]
async fn quest_progress_is(world: &mut GameWorld, progress: u32) {
    assert_eq!(world.game.player.quest_progress(), progress);
}
