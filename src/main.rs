mod game;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use game::{Direction, Game, Material};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, Widget, canvas::Canvas},
};
use std::io;

// Empirical zoom factors so the map roughly fills its pane.
const MAP_X_SCALE: f64 = 6.2;
const MAP_Y_SCALE: f64 = 4.08;

#[derive(Default)]
pub struct App {
    game: Game,
    exit: bool,
    show_help: bool,
    experiment: Option<ExperimentState>,
}

#[derive(Default)]
struct ExperimentState {
    selected: Vec<(Material, u32)>,
    cursor: usize,
    focus: ExperimentFocus,
}

#[derive(Default)]
enum ExperimentFocus {
    #[default]
    Available,
    Selected,
}

impl ExperimentState {
    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            ExperimentFocus::Available => ExperimentFocus::Selected,
            ExperimentFocus::Selected => ExperimentFocus::Available,
        };
        self.cursor = 0;
    }

    fn cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn cursor_down(&mut self, len: usize) {
        if self.cursor + 1 < len {
            self.cursor += 1;
        }
    }

    fn add_material(&mut self, material: Material) {
        match self.selected.iter_mut().find(|(m, _)| *m == material) {
            Some((_, quantity)) => *quantity += 1,
            None => self.selected.push((material, 1)),
        }
    }

    fn remove_current_material(&mut self) {
        if self.selected.is_empty() {
            return;
        }

        let index = self.cursor.min(self.selected.len() - 1);
        let (_, quantity) = &mut self.selected[index];
        *quantity -= 1;

        if *quantity == 0 {
            self.selected.remove(index);
            self.cursor = self.cursor.min(self.selected.len().saturating_sub(1));
        }
    }

    fn selected_materials(&self) -> Vec<(Material, u32)> {
        self.selected.clone()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

fn render_material_list(
    area: Rect,
    buf: &mut Buffer,
    materials: &[(&Material, &u32)],
    title: &str,
    cursor: Option<usize>,
) {
    let items = materials
        .iter()
        .enumerate()
        .map(|(index, (material, quantity))| {
            let style = if cursor == Some(index) {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            ListItem::new(format!("{material:?}  {quantity}")).style(style)
        })
        .collect::<Vec<_>>();

    let list = List::new(items).block(Block::bordered().title(title));

    Widget::render(list, area, buf);
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

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let popup_area = centered_rect(60, 50, area);

        Clear.render(popup_area, buf);

        let text = vec![
            Line::from("Keyboard shortcuts"),
            Line::from(""),
            Line::from("← → ↑ ↓    Move"),
            Line::from("s          Search"),
            Line::from("c          Craft"),
            Line::from("e          Experiment"),
            Line::from("?          Show help"),
            Line::from("q          Quit"),
        ];

        let popup = Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(" Help ")
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Left);

        popup.render(popup_area, buf);
    }

    fn render_experiment(&self, area: Rect, buf: &mut Buffer) {
        let experiment = self.experiment.as_ref().unwrap();

        let popup = centered_rect(70, 70, area);

        // Paint over whatever is underneath so the popup is opaque.
        Clear.render(popup, buf);

        let block = Block::bordered()
            .title(" Experiment ")
            .border_type(BorderType::Rounded);
        let inner = block.inner(popup);
        block.render(popup, buf);

        let rows = Layout::vertical([Constraint::Min(5), Constraint::Length(1)]).split(inner);

        let lists = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let available_cursor = match experiment.focus {
            ExperimentFocus::Available => Some(experiment.cursor),
            ExperimentFocus::Selected => None,
        };

        let inventory = self.sorted_inventory();
        let available = inventory
            .iter()
            .map(|(material, quantity)| (material, quantity))
            .collect::<Vec<_>>();
        render_material_list(lists[0], buf, &available, " Available ", available_cursor);

        let selected_cursor = match experiment.focus {
            ExperimentFocus::Available => None,
            ExperimentFocus::Selected => Some(experiment.cursor),
        };

        let selected = experiment
            .selected
            .iter()
            .map(|(material, quantity)| (material, quantity))
            .collect::<Vec<_>>();
        render_material_list(lists[1], buf, &selected, " Selected ", selected_cursor);

        Paragraph::new("↑↓ move   ←→ add/remove   Tab switch   e run   Esc cancel")
            .alignment(Alignment::Center)
            .render(rows[1], buf);
    }

    fn sorted_inventory(&self) -> Vec<(Material, u32)> {
        let mut materials = self
            .game
            .player
            .inventory
            .iter()
            .map(|(material, quantity)| (*material, *quantity))
            .collect::<Vec<_>>();
        materials.sort_by_key(|(material, _)| format!("{material:?}"));
        materials
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
        if self.show_help {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                    self.show_help = false;
                }
                _ => {}
            }
            return;
        }

        if self.experiment.is_some() {
            self.handle_experiment_key(key_event);
            return;
        }

        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('s') => self.game.search(),
            KeyCode::Char('c') => self.game.craft("Cord"),
            KeyCode::Char('e') => self.open_experiment(),
            KeyCode::Left => self.game.walk(Direction::West),
            KeyCode::Right => self.game.walk(Direction::East),
            KeyCode::Up => self.game.walk(Direction::North),
            KeyCode::Down => self.game.walk(Direction::South),
            _ => {}
        }
    }

    fn open_experiment(&mut self) {
        self.experiment = Some(ExperimentState::default());
    }

    fn close_experiment(&mut self) {
        self.experiment = None;
    }

    fn handle_experiment_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.close_experiment(),

            KeyCode::Tab => {
                if let Some(experiment) = self.experiment.as_mut() {
                    experiment.toggle_focus();
                }
            }

            KeyCode::Up => {
                if let Some(experiment) = self.experiment.as_mut() {
                    experiment.cursor_up();
                }
            }

            KeyCode::Down => {
                let len = self.experiment_list_len();
                if let Some(experiment) = self.experiment.as_mut() {
                    experiment.cursor_down(len);
                }
            }

            KeyCode::Right | KeyCode::Enter => self.experiment_add_current(),

            KeyCode::Left | KeyCode::Backspace => {
                if let Some(experiment) = self.experiment.as_mut()
                    && matches!(experiment.focus, ExperimentFocus::Selected)
                {
                    experiment.remove_current_material();
                }
            }

            KeyCode::Char('e') => self.finish_experiment(),

            _ => {}
        }
    }

    fn experiment_list_len(&self) -> usize {
        let Some(experiment) = self.experiment.as_ref() else {
            return 0;
        };

        match experiment.focus {
            ExperimentFocus::Available => self.game.player.inventory.len(),
            ExperimentFocus::Selected => experiment.selected.len(),
        }
    }

    fn experiment_add_current(&mut self) {
        let Some(experiment) = self.experiment.as_ref() else {
            return;
        };

        if !matches!(experiment.focus, ExperimentFocus::Available) {
            return;
        }

        let Some((material, _)) = self.sorted_inventory().get(experiment.cursor).copied() else {
            return;
        };

        if let Some(experiment) = self.experiment.as_mut() {
            experiment.add_material(material);
        }
    }

    fn finish_experiment(&mut self) {
        let Some(experiment) = self.experiment.take() else {
            return;
        };

        let materials = experiment.selected_materials();

        self.game.experiment(&materials);
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
        if self.show_help {
            self.render_help(area, buf);
        } else if self.experiment.is_some() {
            self.render_experiment(area, buf);
        }
    }
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}
