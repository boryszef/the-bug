//! The Quests tab: the active quest's progress, quests available to accept
//! (each a collapsing section with its description and an Accept button),
//! and completed quests. Mirrors `tui::quests`, mouse-driven — no cursor.

use std::collections::HashMap;

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
        ui.label(completed_line(overview, lang));
    });

    accept
}

fn completed_line(overview: &Overview, lang: Language) -> String {
    if overview.completed.is_empty() {
        return i18n::ui("quests-completed-none", lang);
    }
    let names: Vec<String> = overview
        .completed
        .iter()
        .map(|q| i18n::quest_name(q.id, lang))
        .collect();
    i18n::ui_args(
        "quests-completed",
        lang,
        HashMap::from([("names", names.join(", ").into())]),
    )
}
