mod map;

use std::collections::HashMap;

use eframe::egui::{self, Color32, Key, RichText, Ui};

use crate::game::{EventKind, Game};
use crate::i18n::{self, Language};
use crate::save;
use crate::viewmodel;

use map::MapView;

/// Launches the egui/eframe front end on `game`, blocking until the window
/// closes. Saves `game` to disk on close, mirroring `tui::App`'s
/// save-on-exit in `main`.
pub fn run(game: Game, language: Language) -> eframe::Result<()> {
    eframe::run_native(
        "the-bug",
        eframe::NativeOptions::default(),
        Box::new(|_cc| {
            Ok(Box::new(App {
                game,
                language,
                panel: Panel::default(),
                map_view: MapView::default(),
            }))
        }),
    )
}

/// Which panel occupies the central area. Mirrors `tui::app::Panel`; only
/// the tab-switching chrome is built here so far, not the panels' own
/// content — see docs/gui-frontend.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Panel {
    #[default]
    Map,
    Experiment,
    Craft,
    Disassemble,
    Quests,
}

impl Panel {
    const ALL: [Panel; 5] = [
        Panel::Map,
        Panel::Experiment,
        Panel::Craft,
        Panel::Disassemble,
        Panel::Quests,
    ];

    fn next(self) -> Panel {
        match self {
            Panel::Map => Panel::Experiment,
            Panel::Experiment => Panel::Craft,
            Panel::Craft => Panel::Disassemble,
            Panel::Disassemble => Panel::Quests,
            Panel::Quests => Panel::Map,
        }
    }

    fn prev(self) -> Panel {
        match self {
            Panel::Map => Panel::Quests,
            Panel::Experiment => Panel::Map,
            Panel::Craft => Panel::Experiment,
            Panel::Disassemble => Panel::Craft,
            Panel::Quests => Panel::Disassemble,
        }
    }

    /// The `panel-*-title` message id for this tab's button label.
    fn title_id(self) -> &'static str {
        match self {
            Panel::Map => "panel-map-title",
            Panel::Experiment => "panel-experiment-title",
            Panel::Craft => "panel-craft-title",
            Panel::Disassemble => "panel-disassemble-title",
            Panel::Quests => "panel-quests-title",
        }
    }
}

struct App {
    game: Game,
    language: Language,
    panel: Panel,
    map_view: MapView,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.key_pressed(Key::OpenBracket)) {
            self.panel = self.panel.prev();
        }
        if ctx.input(|i| i.key_pressed(Key::CloseBracket)) {
            self.panel = self.panel.next();
        }
        if ctx.input(|i| i.key_pressed(Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        egui::Panel::top("tabs_and_quit").show(ui, |ui| {
            ui.horizontal(|ui| {
                for panel in Panel::ALL {
                    let label = i18n::ui(panel.title_id(), self.language);
                    if ui.selectable_label(self.panel == panel, label).clicked() {
                        self.panel = panel;
                    }
                }
                if ui.button(i18n::ui("action-quit", self.language)).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        egui::Panel::left("player_and_events")
            .resizable(false)
            .exact_size(320.0)
            .show(ui, |ui| {
                render_player(&self.game, self.language, ui);
                ui.separator();
                render_events(&self.game, self.language, ui);
            });

        egui::CentralPanel::default().show(ui, |ui| match self.panel {
            Panel::Map => self.map_view.ui(
                ui,
                viewmodel::map::tile_views(&self.game.map),
                self.game.player.coordinates,
            ),
            // Placeholder for the craft/disassemble/experiment/quests
            // panels' own content — see docs/gui-frontend.md.
            _ => {
                ui.heading(i18n::ui(self.panel.title_id(), self.language));
            }
        });
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
        EventKind::Awoke => None,
        EventKind::Found { .. } => Some(Color32::GREEN),
        EventKind::QuestAccepted { .. } | EventKind::QuestCompleted { .. } => {
            Some(Color32::MAGENTA)
        }
        EventKind::UnknownRecipe { .. }
        | EventKind::CraftShortage { .. }
        | EventKind::Crafted { .. }
        | EventKind::Disassembled { .. } => Some(Color32::YELLOW),
        EventKind::ExperimentShortage { .. }
        | EventKind::ExperimentFailed { .. }
        | EventKind::Experimented { .. } => Some(Color32::CYAN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_cycles_through_every_panel_in_order_and_wraps() {
        assert_eq!(Panel::Map.next(), Panel::Experiment);
        assert_eq!(Panel::Experiment.next(), Panel::Craft);
        assert_eq!(Panel::Craft.next(), Panel::Disassemble);
        assert_eq!(Panel::Disassemble.next(), Panel::Quests);
        assert_eq!(Panel::Quests.next(), Panel::Map);
    }

    #[test]
    fn prev_cycles_through_every_panel_in_reverse_and_wraps() {
        assert_eq!(Panel::Map.prev(), Panel::Quests);
        assert_eq!(Panel::Quests.prev(), Panel::Disassemble);
        assert_eq!(Panel::Disassemble.prev(), Panel::Craft);
        assert_eq!(Panel::Craft.prev(), Panel::Experiment);
        assert_eq!(Panel::Experiment.prev(), Panel::Map);
    }

    #[test]
    fn next_and_prev_are_inverses_for_every_panel() {
        for panel in Panel::ALL {
            assert_eq!(panel.next().prev(), panel);
            assert_eq!(panel.prev().next(), panel);
        }
    }
}
