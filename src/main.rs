mod game;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use game::{Direction, Game};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, canvas::Canvas},
};
use std::io;

// Empirical zoom factors so the map roughly fills its pane.
const MAP_X_SCALE: f64 = 6.2;
const MAP_Y_SCALE: f64 = 4.08;

#[derive(Debug, Default)]
pub struct App {
    game: Game,
    exit: bool,
}

impl App {
    fn render_map(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(" Map ");
        let inner = block.inner(area);
        let half = self.game.map.half;
        let player_pos = self.game.player.coordinates;

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
                for (y, row) in self.game.map.tiles.iter().enumerate() {
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

    fn render_player(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(" Player ");

        let text = Text::from(vec![
            Line::from(format!("Level: {}", self.game.player.level)),
            Line::from(format!("Inventory: {:?}", self.game.player.inventory)),
        ]);

        Paragraph::new(text).block(block).render(area, buf);
    }

    fn render_events(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(" Events ");

        let lines: Vec<Line> = self
            .game
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
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('s') => self.game.search(),
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

        self.render_map(columns[1], buf);
        self.render_player(menu[0], buf);
        self.render_events(menu[1], buf);
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
