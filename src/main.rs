//! Entry point. Native: parse args, load or start a `Game`, run the gui.
//! Web (`wasm32`): start a fresh `Game` on the page canvas. Everything else
//! lives in the `the_bug` library crate.

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::io;
    use std::path::PathBuf;

    use clap::Parser;

    use the_bug::game::Game;
    use the_bug::gui;
    use the_bug::i18n::{self, Language};
    use the_bug::save;

    /// A tiny survival game.
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

    pub fn main() -> io::Result<()> {
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
            None => Game::default(),
        };

        run_frontend(game, language)
    }

    /// Runs the graphical (egui) front end, then returns.
    fn run_frontend(game: Game, language: Language) -> io::Result<()> {
        if let Err(e) = gui::run(game, language) {
            eprintln!("gui error: {e}");
            std::process::exit(1);
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
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    native::main()
}

/// Web entry: `trunk` builds this bin to wasm, wasm-bindgen calls `main`.
/// Starts a fresh game on the `#the_canvas_id` canvas from `index.html`.
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    console_error_panic_hook::set_once();

    // No `$LANG` on the web — falls back to English. Reading
    // `navigator.language` is a follow-up.
    let language = the_bug::i18n::detect(None, |_| None);
    let game = the_bug::game::Game::default();

    let canvas = web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
        .get_element_by_id("the_canvas_id")
        .expect("no #the_canvas_id")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("#the_canvas_id is not a <canvas>");

    wasm_bindgen_futures::spawn_local(async move {
        the_bug::gui::run_web(canvas, game, language)
            .await
            .expect("failed to start eframe");
    });
}
