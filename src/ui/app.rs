use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, canvas::Canvas},
};
use std::io;

use crate::game::{Direction, Game, Material};

use super::experiment::{Experiment, Outcome};
use super::help;

// Empirical zoom factors so the map roughly fills its pane.
const MAP_X_SCALE: f64 = 6.2;
const MAP_Y_SCALE: f64 = 4.08;

#[derive(Default)]
pub struct App {
    game: Game,
    exit: bool,
    show_help: bool,
    experiment: Option<Experiment>,
}

impl App {
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
        if self.show_help {
            if matches!(
                key_event.code,
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')
            ) {
                self.show_help = false;
            }
            return;
        }

        if let Some(experiment) = &mut self.experiment {
            let inventory = sorted_inventory(&self.game);
            match experiment.handle_key(key_event.code, &inventory) {
                Outcome::Stay => {}
                Outcome::Cancel => self.experiment = None,
                Outcome::Run(materials) => {
                    self.experiment = None;
                    self.game.experiment(&materials);
                }
            }
            return;
        }

        self.handle_game_key(key_event.code);
    }

    fn handle_game_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('s') => self.game.search(),
            KeyCode::Char('c') => self.game.craft("Cord"),
            KeyCode::Char('e') => self.experiment = Some(Experiment::default()),
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
        let columns =
            Layout::horizontal([Constraint::Min(50), Constraint::Percentage(100)]).split(area);
        let menu = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(columns[0]);

        render_map(&self.game, columns[1], buf);
        render_player(&self.game, menu[0], buf);
        render_events(&self.game, menu[1], buf);

        if self.show_help {
            help::render(area, buf);
        } else if let Some(experiment) = &self.experiment {
            experiment.render(area, buf, &sorted_inventory(&self.game));
        }
    }
}

/// The player's inventory as a list sorted by material name, for stable display
/// order and cursor indexing.
fn sorted_inventory(game: &Game) -> Vec<(Material, u32)> {
    let mut materials = game
        .player
        .inventory
        .iter()
        .map(|(material, quantity)| (*material, *quantity))
        .collect::<Vec<_>>();
    materials.sort_by_key(|(material, _)| format!("{material:?}"));
    materials
}

fn render_map(game: &Game, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered().title(" Map ");
    let inner = block.inner(area);
    let half = game.map.half;
    let player_pos = game.player.coordinates;

    let canvas = Canvas::default()
        .block(block)
        .x_bounds([
            inner.width as f64 / -MAP_X_SCALE,
            inner.width as f64 / MAP_X_SCALE,
        ])
        .y_bounds([
            inner.height as f64 / -MAP_Y_SCALE,
            inner.height as f64 / MAP_Y_SCALE,
        ])
        .paint(|ctx| {
            for (y, row) in game.map.tiles.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    let world_x = (x as i32 - half) as f64;
                    let world_y = (y as i32 - half) as f64;
                    ctx.print(world_x, world_y, format!("{tile}"));
                }
            }
            ctx.print(
                player_pos.0 as f64,
                player_pos.1 as f64,
                Line::from("🯅").style(Style::default().fg(Color::Yellow)),
            );
        });

    canvas.render(area, buf);
}

fn render_player(game: &Game, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered().title(" Player ");

    let inventory = sorted_inventory(game)
        .iter()
        .map(|(material, quantity)| format!("{material} {quantity}"))
        .collect::<Vec<_>>()
        .join(", ");

    let text = Text::from(vec![
        Line::from(format!("Level: {}", game.player.level)),
        Line::from(format!("Inventory: {inventory}")),
    ]);

    Paragraph::new(text).block(block).render(area, buf);
}

fn render_events(game: &Game, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered().title(" Events ");

    let lines: Vec<Line> = game
        .events
        .iter()
        .rev()
        .take(10)
        .map(|e| Line::from(e.as_str()))
        .collect();

    Paragraph::new(Text::from(lines))
        .block(block)
        .render(area, buf);
}
