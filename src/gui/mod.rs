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

/// `localStorage` key the web build autosaves the game JSON under (see
/// [`App::save`]). Native persistence is a file on disk, not this.
const GAME_KEY: &str = "the-bug-game";

/// The `eframe` app-creator closure, shared by the native and web runners.
fn app_creator(
    game: Game,
    language: Language,
) -> impl FnOnce(
    &eframe::CreationContext<'_>,
) -> eframe::Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
    move |cc| {
        // On the web, resume the autosaved game if `localStorage` holds one.
        // (`cc.storage` is `None` on native without the `persistence`
        // feature, so this is a no-op there — the game came via `--load`.)
        // A corrupt or version-incompatible blob is discarded, not fatal.
        let game = cc
            .storage
            .and_then(|storage| storage.get_string(GAME_KEY))
            .and_then(|json| match save::from_json(&json) {
                Ok(resumed) => Some(resumed),
                Err(e) => {
                    eprintln!("ignoring stored game: {e}");
                    None
                }
            })
            .unwrap_or(game);

        Ok(Box::new(App {
            game,
            language,
            panel: Panel::default(),
            map_view: MapView::new(&cc.egui_ctx),
            experiment: Experiment::default(),
            theme: egui::Theme::Dark,
            font_size: FontSize::Medium,
        }))
    }
}

/// Launches the egui/eframe front end on `game`, blocking until the window
/// closes. Saves `game` to disk on close (see [`App::on_exit`]).
#[cfg(not(target_arch = "wasm32"))]
pub fn run(game: Game, language: Language) -> eframe::Result<()> {
    eframe::run_native(
        "the-bug",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                // Tall enough that a fresh 23×23 map fits at the default zoom,
                // with the 320-wide side panel alongside it.
                .with_inner_size([1280.0, 1000.0])
                .with_min_inner_size([800.0, 600.0]),
            ..Default::default()
        },
        Box::new(app_creator(game, language)),
    )
}

/// Starts the front end on `game` inside `canvas`, in the browser. There is
/// no save-on-exit on the web (see [`App::on_exit`]).
#[cfg(target_arch = "wasm32")]
pub async fn run_web(
    canvas: web_sys::HtmlCanvasElement,
    game: Game,
    language: Language,
) -> Result<(), eframe::wasm_bindgen::JsValue> {
    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(app_creator(game, language)),
        )
        .await
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

/// The `action-theme-*` message id for `theme`'s toggle-button label. Mirrors
/// [`title_id`]. See `docs/accessibility.md`.
fn theme_label_id(theme: egui::Theme) -> &'static str {
    match theme {
        egui::Theme::Light => "action-theme-light",
        egui::Theme::Dark => "action-theme-dark",
    }
}

/// A base text-size preset, applied to every built-in `egui::TextStyle` via
/// [`apply_font_size`]. Purely an egui rendering concern — no game or
/// viewmodel logic — so it lives here, not in `viewmodel`. See
/// `docs/accessibility.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FontSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl FontSize {
    const ALL: [FontSize; 4] = [
        FontSize::Small,
        FontSize::Medium,
        FontSize::Large,
        FontSize::ExtraLarge,
    ];

    /// Multiplier applied to egui's own default `TextStyle` sizes.
    fn scale(self) -> f32 {
        match self {
            FontSize::Small => 0.85,
            FontSize::Medium => 1.0,
            FontSize::Large => 1.2,
            FontSize::ExtraLarge => 1.45,
        }
    }

    /// The `font-size-*` message id for this preset's dropdown label.
    fn label_id(self) -> &'static str {
        match self {
            FontSize::Small => "font-size-small",
            FontSize::Medium => "font-size-medium",
            FontSize::Large => "font-size-large",
            FontSize::ExtraLarge => "font-size-extra-large",
        }
    }
}

/// Rebuilds every built-in `TextStyle`'s size from egui's own defaults
/// (`Small=9.0, Body=13.0, Button=13.0, Heading=18.0, Monospace=13.0`) times
/// `size.scale()`, for both the light and dark `Style`. Recomputed from the
/// fixed base every call — rebuilding is cheap, and starting fresh each time
/// avoids any compounding drift from scaling an already-scaled style.
fn apply_font_size(ctx: &egui::Context, size: FontSize) {
    let scale = size.scale();
    let base = [
        (egui::TextStyle::Small, 9.0),
        (egui::TextStyle::Body, 13.0),
        (egui::TextStyle::Button, 13.0),
        (egui::TextStyle::Heading, 18.0),
        (egui::TextStyle::Monospace, 13.0),
    ];
    ctx.all_styles_mut(|style| {
        for (text_style, base_size) in &base {
            if let Some(font_id) = style.text_styles.get_mut(text_style) {
                font_id.size = base_size * scale;
            }
        }
    });
}

struct App {
    game: Game,
    language: Language,
    panel: Panel,
    map_view: MapView,
    experiment: Experiment,
    theme: egui::Theme,
    font_size: FontSize,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.set_theme(self.theme);
        apply_font_size(&ctx, self.font_size);

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

                ui.separator();
                for theme in [egui::Theme::Light, egui::Theme::Dark] {
                    let label = i18n::ui(theme_label_id(theme), self.language);
                    if ui.selectable_label(self.theme == theme, label).clicked() {
                        self.theme = theme;
                    }
                }

                ui.separator();
                ui.label(i18n::ui("font-size-label", self.language));
                egui::ComboBox::from_id_salt("font_size")
                    .selected_text(i18n::ui(self.font_size.label_id(), self.language))
                    .show_ui(ui, |ui| {
                        for size in FontSize::ALL {
                            let label = i18n::ui(size.label_id(), self.language);
                            ui.selectable_value(&mut self.font_size, size, label);
                        }
                    });

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
                render_events(&self.game, self.language, self.theme, ui);
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

    /// Web autosave: `eframe` calls this on a 30 s timer, on canvas
    /// focus-loss, and on page unload. The game is stored as the same JSON
    /// the native build writes to disk. Native has no `storage` here (that's
    /// `on_exit` → a save file) so this is web-only.
    #[cfg(target_arch = "wasm32")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        match save::to_json(&self.game) {
            Ok(json) => storage.set_string(GAME_KEY, json),
            Err(e) => eprintln!("could not serialize game for localStorage: {e}"),
        }
    }

    fn on_exit(&mut self) {
        // The web build persists via `save` (above) into `localStorage`.
        #[cfg(not(target_arch = "wasm32"))]
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

fn render_events(game: &Game, lang: Language, theme: egui::Theme, ui: &mut Ui) {
    ui.heading(i18n::ui("panel-events-title", lang));

    for event in viewmodel::events::recent(game, 10) {
        let text = format!("[{}] {}", event.timestamp, i18n::event(event.kind, lang));
        match event_color(event.kind, theme) {
            Some(color) => ui.label(RichText::new(text).color(color)),
            None => ui.label(text),
        };
    }
}

/// The colour an event-log line gets based on its kind and the active
/// theme; `None` keeps the default text color. Each semantic colour has a
/// bright dark-mode variant and a darker, more saturated light-mode
/// variant — the dark-mode set reads fine on egui's dark background as-is,
/// but the same bright colours are washed out and hard to read on a light
/// background, so light mode gets its own, higher-contrast set.
fn event_color(kind: &EventKind, theme: egui::Theme) -> Option<Color32> {
    let (green, magenta, yellow, cyan) = match theme {
        egui::Theme::Dark => (
            Color32::from_rgb(0, 255, 0),
            Color32::from_rgb(255, 0, 255),
            Color32::from_rgb(255, 255, 0),
            Color32::from_rgb(0, 255, 255),
        ),
        egui::Theme::Light => (
            Color32::from_rgb(0, 110, 0),
            Color32::from_rgb(150, 0, 150),
            Color32::from_rgb(130, 90, 0),
            Color32::from_rgb(0, 120, 120),
        ),
    };
    match kind {
        EventKind::Awoke => None,
        EventKind::Found { .. } | EventKind::Hunted { .. } => Some(green),
        EventKind::QuestAccepted { .. } | EventKind::QuestCompleted { .. } => Some(magenta),
        EventKind::Crafted { .. }
        | EventKind::Disassembled { .. }
        | EventKind::HuntMissed
        | EventKind::EquipmentFull { .. }
        | EventKind::Dropped { .. } => Some(yellow),
        EventKind::ExperimentFailed { .. }
        | EventKind::ExperimentMissingTool { .. }
        | EventKind::Experimented { .. } => Some(cyan),
    }
}
