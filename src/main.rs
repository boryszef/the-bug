use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::prelude::IndexedRandom;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::fmt;
use std::io;

const MAP_MIN_SIZE: u32 = 5;
const MAP_PER_LEVEL_INCREMENT: u32 = 2;

#[derive(Debug)]
struct Player {
    level: u32,
    coordinates: (u32, u32),
    resources: Vec<Resource>,
}

impl Default for Player {
    fn default() -> Player {
        Player {
            level: 1,
            coordinates: (0, 0),
            resources: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum Resource {
    Wood,
}

#[derive(Clone, Copy, Debug)]
enum TerrainType {
    Meadow,
    Forest,
    Village,
    Deadland,
}

impl fmt::Display for TerrainType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let symbol = match self {
            TerrainType::Meadow => '𖧧',
            TerrainType::Forest => '𖠰',
            TerrainType::Village => '🛖',
            TerrainType::Deadland => ' ',
        };
        write!(f, "{symbol}")
    }
}

const PLAYER: char = '𐦂';

const TERRAIN_TYPES: &[TerrainType] = &[
    TerrainType::Meadow,
    TerrainType::Forest,
    TerrainType::Deadland,
];

#[derive(Debug)]
struct MapTile {
    terrain_type: TerrainType,
    resources: Vec<Resource>,
}

impl fmt::Display for MapTile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let symbol = self.terrain_type.to_string();
        write!(f, "{symbol}")
    }
}

impl MapTile {
    fn new() -> MapTile {
        let mut rng = rand::rng();
        let terrain_type = *TERRAIN_TYPES.choose(&mut rng).unwrap();

        MapTile {
            terrain_type,
            resources: Vec::new(),
        }
    }

    fn generate(terrain_type: TerrainType) -> MapTile {
        MapTile {
            terrain_type,
            resources: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct RegionMap {
    tiles: Vec<Vec<MapTile>>,
    size: (u32, u32),
}

impl fmt::Display for RegionMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.tiles {
            write!(f, "|")?;
            for tile in row {
                write!(f, "{tile}")?;
            }
            writeln!(f, "|")?;
        }

        Ok(())
    }
}

impl RegionMap {
    fn new(player: &Player) -> RegionMap {
        let size = MAP_MIN_SIZE + player.level * MAP_PER_LEVEL_INCREMENT;
        let mut map = Vec::new();
        let middle = (size / 2, size / 2);
        for x in 0..size {
            let mut row = Vec::new();
            for y in 0..size {
                let tile = match (x, y) {
                    pos if pos == middle => MapTile::generate(TerrainType::Village),
                    _ => MapTile::new(),
                };
                row.push(tile);
            }
            map.push(row);
        }
        RegionMap {
            tiles: map,
            size: (size, size),
        }
    }
}

#[derive(Debug, Default)]
pub struct App {
    player: Player,
    counter: u8,
    exit: bool,
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
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let player = &self.player;

        let counter_text = Text::from(vec![
            Line::from(vec!["Value: ".into(), self.counter.to_string().yellow()]),
            Line::from(vec![format!("{player:?}").into()]),
        ]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let player = Player::default();
    let map = RegionMap::new(&player);
    let output = String::from("{player:?}");
    frame.render_widget(output, frame.area());
}
