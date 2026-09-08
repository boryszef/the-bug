mod craft;
mod disassemble;
mod experiment;
mod items;
mod map;
mod quests;

use std::collections::HashMap;

use eframe::egui::{self, Color32, Key, RichText, Ui};

use crate::game::{EventKind, Game};
use crate::i18n::{self, Language};
use crate::save;
use crate::viewmodel;
use crate::viewmodel::panel::Panel;

use experiment::Experiment;
use map::{MapCommand, MapView};

/// Launches the egui/eframe front end on `game`, blocking until the window
/// closes. Saves `game` to disk on close (see [`App::on_exit`]).
pub fn run(game: Game, language: Language) -> eframe::Result<()> {
    eframe::run_native(
        "the-bug",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1280.0, 800.0])
                .with_min_inner_size([800.0, 600.0]),
            ..Default::default()
        },
        Box::new(|_cc| {
            Ok(Box::new(App {
                game,
                language,
                panel: Panel::default(),
                map_view: MapView::default(),
                experiment: Experiment::default(),
            }))
        }),
    )
}

/// The `panel-*-title` message id for `panel`'s tab button label. The tab
/// set itself and its cycling order are [`Panel`] (`viewmodel::panel`) — only
/// this label lookup is the gui's own, see docs/gui-panels.md.
fn title_id(panel: Panel) -> &'static str {
    match panel {
        Panel::Map => "panel-map-title",
        Panel::Experiment => "panel-experiment-title",
        Panel::Craft => "panel-craft-title",
        Panel::Disassemble => "panel-disassemble-title",
        Panel::Items => "panel-items-title",
        Panel::Quests => "panel-quests-title",
    }
}

struct App {
    game: Game,
    language: Language,
    panel: Panel,
    map_view: MapView,
    experiment: Experiment,
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
                    let label = i18n::ui(title_id(panel), self.language);
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

        egui::Panel::bottom("hint_bar").show(ui, |ui| {
            ui.label(hint_text(self.panel, self.language));
        });

        egui::CentralPanel::default().show(ui, |ui| match self.panel {
            Panel::Map => {
                let here = viewmodel::map::tile_at(&self.game.map, self.game.player.coordinates);
                let command = self.map_view.ui(
                    ui,
                    viewmodel::map::tile_views(&self.game.map),
                    self.game.player.coordinates,
                    here,
                    self.language,
                );
                match command {
                    Some(MapCommand::Walk(dir)) => self.game.walk(dir),
                    Some(MapCommand::Search) => self.game.search(),
                    Some(MapCommand::Hunt) => self.game.hunt(),
                    None => {}
                }
            }
            Panel::Experiment => {
                let inventory = viewmodel::inventory::combined_sorted(&self.game.player);
                let at_village = self.game.at_craftable_location();
                let player = &self.game.player;
                let outcome = self.experiment.render(
                    ui,
                    &inventory,
                    at_village,
                    |items| player.experiment_would_discover(items),
                    self.language,
                );
                if let experiment::Outcome::Run(items) = outcome {
                    self.game.experiment(&items);
                }
            }
            Panel::Craft => {
                let options = viewmodel::crafting::options(&self.game.player);
                let at_village = self.game.at_craftable_location();
                if let Some(name) = craft::render(ui, &options, at_village, self.language) {
                    self.game.craft(name);
                }
            }
            Panel::Disassemble => {
                let options = viewmodel::disassembly::options(&self.game.player);
                let at_village = self.game.at_craftable_location();
                if let Some(item) = disassemble::render(ui, &options, at_village, self.language) {
                    self.game.disassemble(item);
                }
            }
            Panel::Items => {
                let overview = viewmodel::items::overview(&self.game.player);
                let at_village = self.game.at_craftable_location();
                match items::render(ui, &overview, at_village, self.language) {
                    items::Outcome::TransferToStorage(item) => {
                        self.game.transfer_to_storage(item, 1);
                    }
                    items::Outcome::TransferToEquipment(item) => {
                        self.game.transfer_to_equipment(item, 1);
                    }
                    items::Outcome::DropFromEquipment(item) => {
                        self.game.drop_from_equipment(item, 1);
                    }
                    items::Outcome::DropFromStorage(item) => {
                        self.game.drop_from_storage(item, 1);
                    }
                    items::Outcome::Idle => {}
                }
            }
            Panel::Quests => {
                let overview = viewmodel::quests::overview(&self.game);
                if let Some(id) = quests::render(ui, &overview, self.language) {
                    // Always Ok: `id` came from `overview.available`, built
                    // from `Game::available_quests()` this frame.
                    let _ = self.game.accept_quest(id);
                }
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

/// The bottom hint bar's text: universal controls always, plus the Map
/// tab's pan/zoom hint when it's active. Collapsed to one line since every
/// other panel is self-evident buttons (`docs/panel-layout.md`).
fn hint_text(panel: Panel, lang: Language) -> String {
    let mut hint = i18n::ui("gui-hint", lang);
    if panel == Panel::Map {
        hint.push_str("   ");
        hint.push_str(&i18n::ui("gui-hint-map", lang));
    }
    hint
}

fn render_player(game: &Game, lang: Language, ui: &mut Ui) {
    ui.heading(i18n::ui("panel-player-title", lang));

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
    let (carried, capacity) = game.player.equipment_progress();
    ui.label(i18n::ui_args(
        "player-equipment",
        lang,
        HashMap::from([("carried", carried.into()), ("capacity", capacity.into())]),
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
/// default text color.
fn event_color(kind: &EventKind) -> Option<Color32> {
    match kind {
        EventKind::Awoke => None,
        EventKind::Found { .. } | EventKind::Hunted { .. } => Some(Color32::GREEN),
        EventKind::QuestAccepted { .. } | EventKind::QuestCompleted { .. } => {
            Some(Color32::MAGENTA)
        }
        EventKind::UnknownRecipe { .. }
        | EventKind::CraftShortage { .. }
        | EventKind::CraftMissingTool { .. }
        | EventKind::Crafted { .. }
        | EventKind::Disassembled { .. }
        | EventKind::HuntMissed
        | EventKind::HuntUnprepared { .. }
        | EventKind::EquipmentFull { .. }
        | EventKind::Dropped { .. } => Some(Color32::YELLOW),
        EventKind::ExperimentShortage { .. }
        | EventKind::ExperimentFailed { .. }
        | EventKind::ExperimentMissingTool { .. }
        | EventKind::Experimented { .. } => Some(Color32::CYAN),
    }
}
