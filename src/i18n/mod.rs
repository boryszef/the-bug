//! Player-facing text, in English and Polish, via Project Fluent
//! (`i18n-embed` + `i18n-embed-fl`). See `docs/i18n-plan.md`.
//!
//! `Language` is chosen once at startup ([`detect`]) — nothing here supports
//! switching languages mid-session.

use std::collections::HashMap;
use std::sync::OnceLock;

use fluent::FluentValue;
use i18n_embed::{
    LanguageLoader,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use i18n_embed_fl::fl;
use rust_embed::RustEmbed;

use crate::game::{EventKind, FoundIn, Item, Poi, QuestID, TerrainType, reversible_recipe_for};

#[derive(RustEmbed)]
#[folder = "src/i18n/locales/"]
struct Localizations;

/// The player-facing language. Startup-only: detected once in `main`, then
/// threaded down as a plain value — no in-game switch, no stored preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Polish,
}

impl Language {
    fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Polish => "pl",
        }
    }

    /// This language's loader, with its own bundle first and the fallback
    /// (English) bundle behind it. Built once per language and reused.
    fn loader(self) -> &'static FluentLanguageLoader {
        static ENGLISH: OnceLock<FluentLanguageLoader> = OnceLock::new();
        static POLISH: OnceLock<FluentLanguageLoader> = OnceLock::new();

        let cell = match self {
            Language::English => &ENGLISH,
            Language::Polish => &POLISH,
        };

        cell.get_or_init(|| {
            let loader = fluent_language_loader!();
            let mut wanted = Vec::new();
            if self != Language::English {
                wanted.push(
                    self.code()
                        .parse()
                        .expect("language code is a valid identifier"),
                );
            }
            wanted.push(loader.fallback_language().clone());
            loader
                .load_languages(&Localizations, &wanted)
                .expect("embedded locale resources must load");
            // Fluent wraps interpolated values in bidi-isolation marks by
            // default; a single-direction terminal UI doesn't need them, and
            // they'd show up as stray characters in the rendered text. Must
            // come after `load_languages`, which is what creates the bundles
            // this configures.
            loader.set_use_isolating(false);
            loader
        })
    }
}

/// Detects the startup language: `cli_lang` (the `--lang` flag, already
/// parsed by `clap`) wins; then `$LC_ALL` / `$LC_MESSAGES` / `$LANG` in that
/// order (skipping unset, empty, `C`, or `POSIX`); otherwise English.
pub fn detect(cli_lang: Option<&str>, env: impl Fn(&str) -> Option<String>) -> Language {
    if let Some(value) = cli_lang
        && let Some(lang) = parse_locale(value)
    {
        return lang;
    }

    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Some(value) = env(key)
            && !value.is_empty()
            && value != "C"
            && value != "POSIX"
            && let Some(lang) = parse_locale(&value)
        {
            return lang;
        }
    }

    Language::default()
}

/// The language subtag of a locale string, e.g. `"pl"` from `"pl_PL.UTF-8"`.
fn parse_locale(value: &str) -> Option<Language> {
    let lang = value
        .split(['_', '-', '.'])
        .next()
        .unwrap_or(value)
        .to_ascii_lowercase();
    match lang.as_str() {
        "en" => Some(Language::English),
        "pl" => Some(Language::Polish),
        _ => None,
    }
}

/// One event-log line.
pub fn event(kind: &EventKind, lang: Language) -> String {
    let loader = lang.loader();
    match kind {
        EventKind::Awoke => fl!(loader, "event-awoke"),
        EventKind::Found { item, source } => {
            // "You find X in Y". Polish "Znajdujesz" takes its object in the
            // accusative (X); the place (Y) has no preposition, so the locative
            // case attribute carries the whole phrase (e.g. "w lesie").
            let item = match lang {
                Language::English => self::item(*item, lang),
                Language::Polish => item_attr(*item, "accusative", lang),
            };
            match source {
                FoundIn::Terrain(terrain) => {
                    let place = match lang {
                        Language::English => terrain_name(*terrain, lang),
                        Language::Polish => terrain_attr(*terrain, "locative", lang),
                    };
                    fl!(loader, "event-found", item = item, terrain = place)
                }
                FoundIn::Poi(poi) => {
                    let place = match lang {
                        Language::English => poi_name(*poi, lang),
                        Language::Polish => poi_attr(*poi, "locative", lang),
                    };
                    fl!(loader, "event-found-poi", item = item, poi = place)
                }
            }
        }
        EventKind::QuestAccepted { quest } => fl!(
            loader,
            "event-quest-accepted",
            quest = quest_name(*quest, lang)
        ),
        EventKind::QuestCompleted { quest } => fl!(
            loader,
            "event-quest-completed",
            quest = quest_name(*quest, lang)
        ),
        EventKind::UnknownRecipe { recipe } => {
            fl!(loader, "event-unknown-recipe", recipe = recipe.as_str())
        }
        EventKind::CraftShortage { needed, output } => {
            // "not enough of X" wants X in the plural genitive case in
            // Polish (a shortage of a countable noun); "to make Y" wants Y
            // in the accusative (direct object of "zrobić"). English has no
            // case to apply, so both stay nominative.
            let needed_arg = match lang {
                Language::English => self::item(*needed, lang),
                Language::Polish => item_attr(*needed, "genitive-plural", lang),
            };
            let output_arg = match lang {
                Language::English => self::item(*output, lang),
                Language::Polish => item_attr(*output, "accusative", lang),
            };
            fl!(
                loader,
                "event-craft-shortage",
                needed = needed_arg,
                output = output_arg
            )
        }
        EventKind::Crafted { output } => {
            // "You craft Y" wants Y in the accusative in Polish (direct
            // object of "Tworzysz").
            let output_arg = match lang {
                Language::English => self::item(*output, lang),
                Language::Polish => item_attr(*output, "accusative", lang),
            };
            fl!(loader, "event-crafted", output = output_arg)
        }
        EventKind::ExperimentShortage {
            items,
            missing,
            available,
            needed,
        } => {
            let available: u32 = *available;
            let needed: u32 = *needed;
            let missing_arg = match lang {
                Language::English => self::item(*missing, lang),
                Language::Polish => item_attr(*missing, "genitive-plural", lang),
            };
            fl!(
                loader,
                "event-experiment-shortage",
                items = describe_items(items, lang),
                missing = missing_arg,
                available = available,
                needed = needed
            )
        }
        EventKind::ExperimentFailed { items } => fl!(
            loader,
            "event-experiment-failed",
            items = describe_items(items, lang)
        ),
        EventKind::Experimented {
            items,
            output,
            newly_learned,
        } => fl!(
            loader,
            "event-experimented",
            items = describe_items(items, lang),
            output = self::item(*output, lang),
            newly_learned = if *newly_learned { "yes" } else { "no" }
        ),
        EventKind::Disassembled { item } => {
            let recovered = reversible_recipe_for(*item)
                .map(|recipe| describe_items(recipe.inputs(), lang))
                .unwrap_or_default();
            fl!(
                loader,
                "event-disassembled",
                item = self::item(*item, lang),
                recovered = recovered
            )
        }
    }
}

fn item_id(item: Item) -> &'static str {
    match item {
        Item::Stick => "item-stick",
        Item::Stone => "item-stone",
        Item::Vine => "item-vine",
        Item::Cord => "item-cord",
        Item::StoneAxe => "item-stone-axe",
        Item::Arrow => "item-arrow",
        Item::WoodenBow => "item-wooden-bow",
        Item::PlasticBottle => "item-plastic-bottle",
        Item::CopperWire => "item-copper-wire",
        Item::Coil => "item-coil",
        Item::Pole => "item-pole",
        Item::Microcontroller => "item-microcontroller",
        Item::Speaker => "item-speaker",
        Item::MetalDetector => "item-metal-detector",
        Item::Battery => "item-battery",
        Item::SolarPanel => "item-solar-panel",
        Item::SolarCharger => "item-solar-charger",
        Item::CircuitBoard => "item-circuit-board",
        Item::Umbrella => "item-umbrella",
        Item::Fabric => "item-fabric",
    }
}

/// An item's nominative name, for lists/menus and as a sentence fragment.
pub fn item(item: Item, lang: Language) -> String {
    lang.loader().get(item_id(item))
}

/// A grammatical-case attribute of an item's name (e.g. `"genitive"`),
/// where the target language's `.ftl` defines one.
fn item_attr(item: Item, attr: &str, lang: Language) -> String {
    lang.loader().get_attr(item_id(item), attr)
}

/// `"<quantity> <item name>"`, e.g. for inventory/selection lists.
pub fn item_with_quantity(item: Item, quantity: u32, lang: Language) -> String {
    format!("{quantity} {}", self::item(item, lang))
}

fn terrain_id(terrain: TerrainType) -> &'static str {
    match terrain {
        TerrainType::Meadow => "terrain-meadow",
        TerrainType::Forest => "terrain-forest",
        TerrainType::Deadland => "terrain-deadland",
    }
}

fn terrain_name(terrain: TerrainType, lang: Language) -> String {
    lang.loader().get(terrain_id(terrain))
}

/// A grammatical-case attribute of a terrain's name (e.g. `"locative"`),
/// where the target language's `.ftl` defines one.
fn terrain_attr(terrain: TerrainType, attr: &str, lang: Language) -> String {
    lang.loader().get_attr(terrain_id(terrain), attr)
}

fn poi_id(poi: Poi) -> &'static str {
    match poi {
        Poi::Cave => "poi-cave",
        Poi::Ruins => "poi-ruins",
        Poi::Village => "poi-village",
    }
}

fn poi_name(poi: Poi, lang: Language) -> String {
    lang.loader().get(poi_id(poi))
}

/// A grammatical-case attribute of a POI's name (e.g. `"locative"`), where the
/// target language's `.ftl` defines one.
fn poi_attr(poi: Poi, attr: &str, lang: Language) -> String {
    lang.loader().get_attr(poi_id(poi), attr)
}

pub fn quest_name(id: QuestID, lang: Language) -> String {
    let key = match id {
        QuestID::CraftArrows => "quest-craft-arrows-name",
        QuestID::ExploreRuins => "quest-explore-ruins-name",
    };
    lang.loader().get(key)
}

pub fn quest_description(id: QuestID, lang: Language) -> String {
    let key = match id {
        QuestID::CraftArrows => "quest-craft-arrows-description",
        QuestID::ExploreRuins => "quest-explore-ruins-description",
    };
    lang.loader().get(key)
}

/// A one-off title/hint/label with no interpolation.
pub fn ui(id: &str, lang: Language) -> String {
    lang.loader().get(id)
}

/// A one-off title/hint/label interpolated with `args`.
pub fn ui_args(id: &str, lang: Language, args: HashMap<&str, FluentValue>) -> String {
    lang.loader().get_args_concrete(id, args)
}

/// A sorted, `" + "`-joined rendering of a set of items with quantities, e.g.
/// `"1 Stick + 1 Stone + 1 Cord"` — used inside event messages that carry a
/// runtime list ([`EventKind::ExperimentShortage`] and friends).
fn describe_items(items: &[(Item, u32)], lang: Language) -> String {
    let mut sorted = items.to_vec();
    sorted.sort_by_key(|&(item, _)| item);
    sorted
        .iter()
        .map(|&(item, quantity)| item_with_quantity(item, quantity, lang))
        .collect::<Vec<_>>()
        .join(" + ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::EventKind;

    #[test]
    fn detect_reads_the_cli_flag() {
        assert_eq!(detect(Some("pl"), |_| None), Language::Polish);
    }

    #[test]
    fn detect_falls_back_to_env_then_english() {
        assert_eq!(
            detect(None, |k| (k == "LANG").then(|| "pl_PL.UTF-8".to_string())),
            Language::Polish
        );
        assert_eq!(detect(None, |_| None), Language::English);
    }

    #[test]
    fn detect_skips_c_and_posix() {
        let env = |k: &str| match k {
            "LC_ALL" => Some("C".to_string()),
            "LANG" => Some("pl_PL".to_string()),
            _ => None,
        };
        assert_eq!(detect(None, env), Language::Polish);
    }

    #[test]
    fn detect_cli_beats_env() {
        let env = |k: &str| (k == "LANG").then(|| "pl_PL".to_string());
        assert_eq!(detect(Some("en"), env), Language::English);
    }

    #[test]
    fn event_renders_found_in_terrain_in_english() {
        let text = event(
            &EventKind::Found {
                item: Item::Stick,
                source: FoundIn::Terrain(TerrainType::Forest),
            },
            Language::English,
        );
        assert_eq!(text, "You find a Stick in the Forest.");
    }

    #[test]
    fn event_renders_found_in_a_poi_in_polish_with_the_locative_case() {
        let text = event(
            &EventKind::Found {
                item: Item::Stone,
                source: FoundIn::Poi(Poi::Cave),
            },
            Language::Polish,
        );
        assert_eq!(text, "Znajdujesz Kamień w jaskini.");
    }

    #[test]
    fn event_renders_found_item_in_the_polish_accusative() {
        let text = event(
            &EventKind::Found {
                item: Item::PlasticBottle,
                source: FoundIn::Poi(Poi::Ruins),
            },
            Language::Polish,
        );
        assert_eq!(text, "Znajdujesz Plastikową Butelkę w ruinach.");
    }

    #[test]
    fn event_renders_craft_shortage_in_polish_with_plural_genitive_and_accusative() {
        let text = event(
            &EventKind::CraftShortage {
                needed: Item::Stick,
                output: Item::Arrow,
            },
            Language::Polish,
        );
        assert_eq!(text, "Masz za mało patyków, aby zrobić Strzałę.");
    }

    #[test]
    fn event_renders_crafted_in_polish_with_the_accusative_case() {
        let text = event(
            &EventKind::Crafted {
                output: Item::Arrow,
            },
            Language::Polish,
        );
        assert_eq!(text, "Tworzysz Strzałę.");
    }

    /// Every `EventKind` variant, in every `Language`, resolves to a
    /// non-empty Fluent message — the safety net standing in for the
    /// `match`-exhaustiveness a hand-written translation layer would get
    /// from the compiler for free (a missing Fluent key doesn't fail to
    /// compile).
    #[test]
    fn every_event_kind_renders_in_every_language() {
        let samples = [
            EventKind::Awoke,
            EventKind::Found {
                item: Item::Stick,
                source: FoundIn::Terrain(TerrainType::Forest),
            },
            EventKind::Found {
                item: Item::Stone,
                source: FoundIn::Poi(Poi::Cave),
            },
            EventKind::QuestAccepted {
                quest: QuestID::CraftArrows,
            },
            EventKind::QuestCompleted {
                quest: QuestID::CraftArrows,
            },
            EventKind::UnknownRecipe {
                recipe: "Widget".to_string(),
            },
            EventKind::CraftShortage {
                needed: Item::Stick,
                output: Item::Arrow,
            },
            EventKind::Crafted { output: Item::Cord },
            EventKind::ExperimentShortage {
                items: vec![(Item::Stone, 5)],
                missing: Item::Stone,
                available: 1,
                needed: 5,
            },
            EventKind::ExperimentFailed {
                items: vec![(Item::Vine, 1), (Item::Stick, 1)],
            },
            EventKind::Experimented {
                items: vec![(Item::Vine, 2)],
                output: Item::Cord,
                newly_learned: true,
            },
            EventKind::Disassembled {
                item: Item::StoneAxe,
            },
        ];

        for kind in &samples {
            for lang in [Language::English, Language::Polish] {
                let text = event(kind, lang);
                assert!(
                    !text.is_empty() && !text.contains("Unknown localization"),
                    "{kind:?} in {lang:?} rendered {text:?}"
                );
                // egui's proportional font has no arrow glyph, so event text
                // must avoid `→` (it shows as a tofu box in the gui log).
                assert!(
                    !text.contains('\u{2192}'),
                    "{kind:?} in {lang:?} has an unrenderable arrow: {text:?}"
                );
            }
        }
    }

    #[test]
    fn event_renders_disassembled_recovered_items() {
        let text = event(
            &EventKind::Disassembled {
                item: Item::StoneAxe,
            },
            Language::English,
        );
        assert_eq!(
            text,
            "You take apart a Stone Axe, recovering 1 Stick + 1 Stone + 1 Cord."
        );
    }

    #[test]
    fn quest_text_renders_in_every_language() {
        for id in [QuestID::CraftArrows, QuestID::ExploreRuins] {
            for lang in [Language::English, Language::Polish] {
                assert!(!quest_name(id, lang).is_empty());
                assert!(!quest_description(id, lang).is_empty());
            }
        }
    }
}
