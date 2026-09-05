use std::collections::HashMap;

use eframe::egui::{self, Color32, RichText, Ui};

use crate::game::{EventKind, Game};
use crate::i18n::{self, Language};
use crate::save;
use crate::viewmodel;

/// Launches the egui/eframe front end on `game`, blocking until the window
/// closes. Saves `game` to disk on close, mirroring `tui::App`'s
/// save-on-exit in `main`.
pub fn run(game: Game, language: Language) -> eframe::Result<()> {
    eframe::run_native(
        "the-bug",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App { game, language }))),
    )
}

struct App {
    game: Game,
    language: Language,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("player_and_events")
            .resizable(false)
            .exact_size(320.0)
            .show(ui, |ui| {
                render_player(&self.game, self.language, ui);
                ui.separator();
                render_events(&self.game, self.language, ui);
            });

        // Placeholder for the map/craft/disassemble/experiment/quests
        // panels — see docs/gui-frontend.md.
        egui::CentralPanel::default().show(ui, |_ui| {});
    }

    fn on_exit(&mut self) {
        match save::save(&self.game) {
            Ok(path) => println!("Game saved to {}", path.display()),
            Err(e) => eprintln!("Warning: could not save game: {e}"),
        }
    }
}

fn render_player(game: &Game, lang: Language, ui: &mut Ui) {
    ui.heading(i18n::ui("panel-player-title", lang));

    let inventory = viewmodel::inventory::sorted(&game.player);
    let inventory = if inventory.is_empty() {
        i18n::ui("player-inventory-empty", lang)
    } else {
        inventory
            .iter()
            .map(|&(item, quantity)| i18n::item_with_quantity(item, quantity, lang))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let (known_recipes, total_recipes) = game.player.recipe_progress();
    let (completed_quests, total_quests) = game.quest_progress();
    let active_quest = match game.player.open_quest() {
        Some(id) => i18n::quest_name(id, lang),
        None => i18n::ui("player-quest-none", lang),
    };

    ui.label(i18n::ui_args(
        "player-level",
        lang,
        HashMap::from([("level", game.player.level.into())]),
    ));
    ui.label(i18n::ui_args(
        "player-xp",
        lang,
        HashMap::from([("xp", game.player.experience.into())]),
    ));
    ui.label(i18n::ui_args(
        "player-recipes",
        lang,
        HashMap::from([
            ("known", (known_recipes as u32).into()),
            ("total", (total_recipes as u32).into()),
        ]),
    ));
    ui.label(i18n::ui_args(
        "player-quests",
        lang,
        HashMap::from([
            ("completed", (completed_quests as u32).into()),
            ("total", (total_quests as u32).into()),
            ("active", active_quest.into()),
        ]),
    ));
    ui.label(i18n::ui_args(
        "player-inventory",
        lang,
        HashMap::from([("items", inventory.into())]),
    ));
}

fn render_events(game: &Game, lang: Language, ui: &mut Ui) {
    ui.heading(i18n::ui("panel-events-title", lang));

    for event in viewmodel::events::recent(game, 10) {
        let text = format!("[{}] {}", event.timestamp, i18n::event(event.kind, lang));
        match event_color(event.kind) {
            Some(color) => ui.label(RichText::new(text).color(color)),
            None => ui.label(text),
        };
    }
}

/// The colour an event-log line gets based on its kind; `None` keeps the
/// default text color. Mirrors `tui::app::event_color`.
fn event_color(kind: &EventKind) -> Option<Color32> {
    match kind {
        EventKind::Awoke
        | EventKind::Found { .. }
        | EventKind::QuestAccepted { .. }
        | EventKind::QuestCompleted { .. } => None,
        EventKind::UnknownRecipe { .. }
        | EventKind::CraftShortage { .. }
        | EventKind::Crafted { .. }
        | EventKind::Disassembled { .. } => Some(Color32::YELLOW),
        EventKind::ExperimentShortage { .. }
        | EventKind::ExperimentFailed { .. }
        | EventKind::Experimented { .. } => Some(Color32::CYAN),
    }
}
