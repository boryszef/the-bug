use std::collections::HashMap;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Wrap, canvas::Canvas},
};
use std::io;

use crate::game::{Direction, EventKind, Game};
use crate::i18n::{self, Language};
use crate::viewmodel;

use super::craft::{self, Craft};
use super::disassemble::{self, Disassemble};
use super::experiment::{self, Experiment};
use super::quests::{self, Quests};

// Empirical zoom factors so the map roughly fills its pane.
const MAP_X_SCALE: f64 = 6.2;
const MAP_Y_SCALE: f64 = 4.08;

/// Which panel occupies the right side of the screen.
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
}

#[derive(Default)]
pub struct App {
    game: Game,
    language: Language,
    exit: bool,
    panel: Panel,
    experiment: Experiment,
    craft: Craft,
    disassemble: Disassemble,
    quests: Quests,
}

impl App {
    /// Starts the UI on an existing game (e.g. one loaded from a save file),
    /// rendering in `language`.
    pub fn with_game(game: Game, language: Language) -> Self {
        Self {
            game,
            language,
            ..Default::default()
        }
    }

    /// The game being played, for persisting on exit.
    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key_event) = event::read()?
            && key_event.kind == KeyEventKind::Press
        {
            self.handle_key_event(key_event);
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => {
                self.exit = true;
                return;
            }
            KeyCode::Char('[') => {
                self.panel = self.panel.prev();
                return;
            }
            KeyCode::Char(']') => {
                self.panel = self.panel.next();
                return;
            }
            _ => {}
        }

        match self.panel {
            Panel::Map => self.handle_map_key(key_event.code),
            Panel::Experiment => {
                let inventory = viewmodel::inventory::sorted(&self.game.player);
                if let experiment::Outcome::Run(items) =
                    self.experiment.handle_key(key_event.code, &inventory)
                {
                    self.game.experiment(&items);
                }
            }
            Panel::Craft => {
                let options = viewmodel::crafting::options(&self.game.player);
                if let craft::Outcome::Craft(name) = self.craft.handle_key(key_event.code, &options)
                {
                    self.game.craft(name);
                }
            }
            Panel::Disassemble => {
                let options = viewmodel::disassembly::options(&self.game.player);
                if let disassemble::Outcome::Disassemble(item) =
                    self.disassemble.handle_key(key_event.code, &options)
                {
                    self.game.disassemble(item);
                }
            }
            Panel::Quests => {
                let overview = viewmodel::quests::overview(&self.game);
                if let quests::Outcome::Accept(id) =
                    self.quests.handle_key(key_event.code, &overview)
                {
                    // Always Ok: `id` came from `overview.available`, built
                    // from `Game::available_quests()` moments ago.
                    let _ = self.game.accept_quest(id);
                }
            }
        }
    }

    fn handle_map_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('s') => self.game.search(),
            KeyCode::Char('h') => self.game.hunt(),
            KeyCode::Left => self.game.walk(Direction::West),
            KeyCode::Right => self.game.walk(Direction::East),
            KeyCode::Up => self.game.walk(Direction::North),
            KeyCode::Down => self.game.walk(Direction::South),
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
        let main = rows[0];
        let footer = rows[1];

        let columns =
            Layout::horizontal([Constraint::Min(70), Constraint::Percentage(100)]).split(main);
        let menu = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(columns[0]);

        render_player(&self.game, menu[0], buf, self.language);
        render_events(&self.game, menu[1], buf, self.language);

        match self.panel {
            Panel::Map => render_map(&self.game, columns[1], buf, self.language),
            Panel::Experiment => self.experiment.render(
                columns[1],
                buf,
                &viewmodel::inventory::sorted(&self.game.player),
                self.language,
            ),
            Panel::Craft => self.craft.render(
                columns[1],
                buf,
                &viewmodel::crafting::options(&self.game.player),
                self.language,
            ),
            Panel::Disassemble => self.disassemble.render(
                columns[1],
                buf,
                &viewmodel::disassembly::options(&self.game.player),
                self.language,
            ),
            Panel::Quests => self.quests.render(
                columns[1],
                buf,
                &viewmodel::quests::overview(&self.game),
                self.language,
            ),
        }

        render_footer(self.panel, footer, buf, self.language);
    }
}

fn render_map(game: &Game, area: Rect, buf: &mut Buffer, lang: Language) {
    let inner = super::panel_frame(area, &i18n::ui("panel-map-title", lang), buf);
    let player_pos = game.player.coordinates;

    let canvas = Canvas::default()
        .x_bounds([
            inner.width as f64 / -MAP_X_SCALE,
            inner.width as f64 / MAP_X_SCALE,
        ])
        .y_bounds([
            inner.height as f64 / -MAP_Y_SCALE,
            inner.height as f64 / MAP_Y_SCALE,
        ])
        .paint(|ctx| {
            for ((world_x, world_y), tile) in viewmodel::map::world_tiles(&game.map) {
                ctx.print(world_x as f64, world_y as f64, tile.to_string());
            }
            ctx.print(
                player_pos.0 as f64,
                player_pos.1 as f64,
                Line::from("🯅").style(Style::default().fg(Color::Yellow)),
            );
        });

    canvas.render(inner, buf);
}

fn render_player(game: &Game, area: Rect, buf: &mut Buffer, lang: Language) {
    let block = Block::bordered().title(i18n::ui("panel-player-title", lang));

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
    let text = Text::from(vec![
        Line::from(i18n::ui_args(
            "player-level",
            lang,
            HashMap::from([("level", game.player.level.into())]),
        )),
        Line::from(i18n::ui_args(
            "player-xp",
            lang,
            HashMap::from([("xp", game.player.experience.into())]),
        )),
        Line::from(i18n::ui_args(
            "player-recipes",
            lang,
            HashMap::from([
                ("known", (known_recipes as u32).into()),
                ("total", (total_recipes as u32).into()),
            ]),
        )),
        Line::from(i18n::ui_args(
            "player-quests",
            lang,
            HashMap::from([
                ("completed", (completed_quests as u32).into()),
                ("total", (total_quests as u32).into()),
                ("active", active_quest.into()),
            ]),
        )),
        Line::from(i18n::ui_args(
            "player-inventory",
            lang,
            HashMap::from([("items", inventory.into())]),
        )),
    ]);

    Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .render(area, buf);
}

fn render_events(game: &Game, area: Rect, buf: &mut Buffer, lang: Language) {
    let block = Block::bordered().title(i18n::ui("panel-events-title", lang));

    let lines: Vec<Line> = viewmodel::events::recent(game, 10)
        .map(|event| {
            let line = Line::from(format!(
                "[{}] {}",
                event.timestamp,
                i18n::event(event.kind, lang)
            ));
            match event_color(event.kind) {
                Some(color) => line.style(Style::default().fg(color)),
                None => line,
            }
        })
        .collect();

    Paragraph::new(Text::from(lines))
        .block(block)
        .render(area, buf);
}

/// The colour an event-log line gets based on its kind; `None` keeps the
/// default foreground.
fn event_color(kind: &EventKind) -> Option<Color> {
    match kind {
        EventKind::Awoke => None,
        EventKind::Found { .. } | EventKind::Hunted { .. } => Some(Color::Green),
        EventKind::QuestAccepted { .. } | EventKind::QuestCompleted { .. } => Some(Color::Magenta),
        EventKind::UnknownRecipe { .. }
        | EventKind::CraftShortage { .. }
        | EventKind::CraftMissingTool { .. }
        | EventKind::Crafted { .. }
        | EventKind::Disassembled { .. }
        | EventKind::HuntMissed
        | EventKind::HuntUnprepared { .. } => Some(Color::Yellow),
        EventKind::ExperimentShortage { .. }
        | EventKind::ExperimentFailed { .. }
        | EventKind::Experimented { .. } => Some(Color::Cyan),
    }
}

/// The key hints for whichever panel is active — this is the only "help"
/// there is now that popups are gone.
fn render_footer(panel: Panel, area: Rect, buf: &mut Buffer, lang: Language) {
    let id = match panel {
        Panel::Map => "footer-map",
        Panel::Experiment => "footer-experiment",
        Panel::Craft => "footer-craft",
        Panel::Disassemble => "footer-disassemble",
        Panel::Quests => "footer-quests",
    };

    Paragraph::new(i18n::ui(id, lang))
        .alignment(Alignment::Center)
        .render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Panel; 5] = [
        Panel::Map,
        Panel::Experiment,
        Panel::Craft,
        Panel::Disassemble,
        Panel::Quests,
    ];

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
        for panel in ALL {
            assert_eq!(panel.next().prev(), panel);
            assert_eq!(panel.prev().next(), panel);
        }
    }
}
