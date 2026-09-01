mod game;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use game::{Player, RegionMap};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, canvas::Canvas},
};
use std::io;

#[derive(Debug)]
pub struct App {
    player: Player,
    map: RegionMap,
    exit: bool,
}

impl Default for App {
    fn default() -> App {
        let player = Player::default();
        let map = RegionMap::new(&player);
        App {
            player,
            map,
            exit: false,
        }
    }
}

impl App {
    fn render_map(&self, area: Rect, buf: &mut Buffer) {
        let width = self.map.size.0 as f64;
        let height = self.map.size.1 as f64;

        let canvas = Canvas::default()
            .block(Block::bordered().title(" Map "))
            .x_bounds([
                self.map.boundaries.0 as f64 - 1.0,
                self.map.boundaries.1 as f64 + 1.0,
            ])
            .y_bounds([
                self.map.boundaries.2 as f64 - 1.0,
                self.map.boundaries.3 as f64 + 1.0,
            ])
            .paint(|ctx| {
                for (y, row) in self.map.tiles.iter().enumerate() {
                    for (x, tile) in row.iter().enumerate() {
                        let world_x = self.map.boundaries.0 + x as i32;
                        let world_y = self.map.boundaries.3 - y as i32;
                        ctx.print(world_x as f64, world_y as f64, format!("{:?}", tile));
                    }
                }
                ctx.print(
                    self.player.coordinates.0 as f64,
                    self.player.coordinates.1 as f64,
                    Line::from("🯅").style(Style::default().fg(Color::Yellow)),
                );
            });

        canvas.render(area, buf);
    }

    fn render_player(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(" Player ");

        let text = Text::from(vec![Line::from(format!("{:?}", self.player))]);

        Paragraph::new(text).block(block).render(area, buf);
    }

    fn render_events(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title(" Events ");

        Paragraph::new("You entered the world.")
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
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.walk_west(),
            KeyCode::Right => self.walk_east(),
            KeyCode::Up => self.walk_north(),
            KeyCode::Down => self.walk_south(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn walk_west(&mut self) {
        self.player.walk_west(self.map.boundaries);
    }

    fn walk_east(&mut self) {
        self.player.walk_east(self.map.boundaries);
    }

    fn walk_north(&mut self) {
        self.player.walk_north(self.map.boundaries);
    }

    fn walk_south(&mut self) {
        self.player.walk_south(self.map.boundaries);
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

/*impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Move: ".into(),
            "<Left> ".blue().bold(),
            "<Right> ".blue().bold(),
            "<Up> ".blue().bold(),
            "<Down>".blue().bold(),
            " Quit: ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let player = &self.player;
        let map = &self.map;

        let counter_text = Text::from(vec![
            Line::from(vec!["Value: ".into(), self.counter.to_string().yellow()]),
            Line::from(vec![format!("{player:?}").into()]),
            Line::from(vec![format!("{map:?}").into()]),
        ]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}*/

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
