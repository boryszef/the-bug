//! The Craft tab: one row per known recipe — a button, disabled while the
//! player can't afford its consumables, followed by each consumable as
//! `have/need`. Mouse-driven — no cursor.

use eframe::egui::{self, Button, RichText, Ui};

use crate::i18n::{self, Language};
use crate::viewmodel::crafting::CraftOption;

/// Draws the recipe list. `at_village` disables every recipe's button (with
/// a hint explaining why) when the player isn't standing somewhere crafting
/// is allowed. Returns the recipe name to craft ([`CraftOption::id`]) if an
/// affordable one was clicked.
pub(super) fn render(
    ui: &mut Ui,
    options: &[CraftOption],
    at_village: bool,
    lang: Language,
) -> Option<&'static str> {
    ui.heading(i18n::ui("panel-craft-title", lang));

    if !at_village {
        ui.label(
            RichText::new(i18n::ui("village-required-hint", lang))
                .color(ui.visuals().error_fg_color),
        );
    }

    if options.is_empty() {
        ui.label(i18n::ui("craft-empty", lang));
        return None;
    }

    let mut chosen = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for option in options {
            // One row per recipe: the button, then its consumables as
            // `have/need` and its tools as bare names, all at the same text
            // size — met/held ones muted, missing ones in red.
            ui.horizontal(|ui| {
                let label = i18n::item(option.output, lang);
                if ui
                    .add_enabled(option.enabled && at_village, Button::new(label))
                    .clicked()
                {
                    chosen = Some(option.id);
                }
                for consumable in &option.consumables {
                    let text = format!(
                        "{}/{} {}",
                        consumable.have,
                        consumable.need,
                        i18n::item(consumable.item, lang)
                    );
                    let colour = if consumable.met() {
                        ui.visuals().weak_text_color()
                    } else {
                        ui.visuals().error_fg_color
                    };
                    ui.label(RichText::new(text).color(colour));
                }
                // Tools: no quantity — one is enough and it isn't consumed.
                for tool in &option.tools {
                    let colour = if tool.present {
                        ui.visuals().weak_text_color()
                    } else {
                        ui.visuals().error_fg_color
                    };
                    ui.label(RichText::new(i18n::item(tool.item, lang)).color(colour));
                }
            });
        }
    });
    chosen
}
