mod game;
mod ui;
mod viewmodel;

use std::io;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| ui::App::default().run(terminal))
}
