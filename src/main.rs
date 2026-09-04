mod game;
mod i18n;
mod save;
mod ui;
mod viewmodel;

use std::io;
use std::path::PathBuf;

use clap::Parser;

/// A tiny terminal survival game.
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Resume from a JSON save file instead of starting a new game.
    #[arg(long, value_name = "FILE")]
    load: Option<PathBuf>,
    /// UI language (e.g. "en", "pl"). Defaults to the system locale.
    #[arg(long, value_name = "LANG")]
    lang: Option<String>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let language = i18n::detect(cli.lang.as_deref(), |key| std::env::var(key).ok());

    let game = match cli.load {
        Some(path) => match save::load(&path) {
            Ok(game) => game,
            Err(e) => {
                eprintln!("could not load {}: {e}", path.display());
                std::process::exit(1);
            }
        },
        None => game::Game::default(),
    };

    let mut app = ui::App::with_game(game, language);
    ratatui::run(|terminal| app.run(terminal))?;

    match save::save(app.game()) {
        Ok(path) => println!("Game saved to {}", path.display()),
        Err(e) => eprintln!("Warning: could not save game: {e}"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_flag_is_parsed() {
        let cli = Cli::try_parse_from(["the-bug", "--load", "save.json"]).unwrap();
        assert_eq!(cli.load, Some(PathBuf::from("save.json")));
    }

    #[test]
    fn load_flag_is_optional() {
        let cli = Cli::try_parse_from(["the-bug"]).unwrap();
        assert_eq!(cli.load, None);
    }

    #[test]
    fn load_flag_requires_a_value() {
        assert!(Cli::try_parse_from(["the-bug", "--load"]).is_err());
    }

    #[test]
    fn lang_flag_is_parsed_in_either_form() {
        let cli = Cli::try_parse_from(["the-bug", "--lang", "pl"]).unwrap();
        assert_eq!(cli.lang.as_deref(), Some("pl"));

        let cli = Cli::try_parse_from(["the-bug", "--lang=pl"]).unwrap();
        assert_eq!(cli.lang.as_deref(), Some("pl"));
    }

    #[test]
    fn lang_flag_is_optional() {
        let cli = Cli::try_parse_from(["the-bug"]).unwrap();
        assert_eq!(cli.lang, None);
    }
}
