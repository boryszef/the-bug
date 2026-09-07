//! The Quests tab: the active quest's progress, quests available to accept,
//! and completed quests — each of the latter two a collapsing section with
//! its description, so completed quests read back as the story so far.
//! Mirrors `tui::quests`'s data, but list-of-collapsing-sections replaces
//! `tui`'s joined "Completed: A, B, C" line; mouse-driven — no cursor.

use eframe::egui::{self, Ui};

use crate::game::QuestID;
use crate::i18n::{self, Language};
use crate::viewmodel::quests::Overview;

/// Draws the panel. Returns the quest to accept if its Accept button was
/// clicked — only possible while no quest is already open.
pub(super) fn render(ui: &mut Ui, overview: &Overview, lang: Language) -> Option<QuestID> {
    ui.heading(i18n::ui("panel-quests-title", lang));

    let mut accept = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.strong(i18n::ui("panel-active-title", lang));
        match &overview.active {
            Some(active) => {
                ui.label(format!(
                    "{} — {}/{}",
                    i18n::quest_name(active.quest.id, lang),
                    active.progress,
                    active.quest.goal(),
                ));
                ui.label(i18n::quest_description(active.quest.id, lang));
            }
            None => {
                ui.label(i18n::ui("quests-active-none", lang));
            }
        }

        ui.separator();
        ui.strong(i18n::ui("panel-available-title", lang));
        if overview.active.is_some() {
            ui.label(i18n::ui("quests-active-blocked", lang));
        } else if overview.available.is_empty() {
            ui.label(i18n::ui("quests-available-empty", lang));
        } else {
            for quest in &overview.available {
                ui.collapsing(i18n::quest_name(quest.id, lang), |ui| {
                    ui.label(i18n::quest_description(quest.id, lang));
                    if ui.button(i18n::ui("action-accept", lang)).clicked() {
                        accept = Some(quest.id);
                    }
                });
            }
        }

        ui.separator();
        ui.strong(i18n::ui("panel-completed-title", lang));
        if overview.completed.is_empty() {
            ui.label(i18n::ui("quests-completed-empty", lang));
        } else {
            for quest in &overview.completed {
                ui.collapsing(i18n::quest_name(quest.id, lang), |ui| {
                    ui.label(i18n::quest_description(quest.id, lang));
                });
            }
        }
    });

    accept
}
