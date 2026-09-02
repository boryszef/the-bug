mod game;
mod ui;

use std::io;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| ui::App::default().run(terminal))
}
