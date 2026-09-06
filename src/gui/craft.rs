//! The Craft tab: one row per known recipe — a button, disabled while the
//! player can't afford its inputs, followed by each input as `have/need`.
//! Mirrors `tui::craft`, mouse-driven — no cursor.

use eframe::egui::{self, Button, RichText, Ui};

use crate::i18n::{self, Language};
use crate::viewmodel::crafting::CraftOption;

/// Draws the recipe list. Returns the recipe name to craft
/// ([`CraftOption::id`]) if an affordable one was clicked.
pub(super) fn render(ui: &mut Ui, options: &[CraftOption], lang: Language) -> Option<&'static str> {
    ui.heading(i18n::ui("panel-craft-title", lang));

    if options.is_empty() {
        ui.label(i18n::ui("craft-empty", lang));
        return None;
    }

    let mut chosen = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for option in options {
            // One row per recipe: the button, then its inputs as `have/need`
            // at the same text size, met ones muted and short ones in red.
            ui.horizontal(|ui| {
                let label = i18n::item(option.output, lang);
                if ui.add_enabled(option.enabled, Button::new(label)).clicked() {
                    chosen = Some(option.id);
                }
                for input in &option.inputs {
                    let text = format!(
                        "{}/{} {}",
                        input.have,
                        input.need,
                        i18n::item(input.item, lang)
                    );
                    let colour = if input.met() {
                        ui.visuals().weak_text_color()
                    } else {
                        ui.visuals().error_fg_color
                    };
                    ui.label(RichText::new(text).color(colour));
                }
            });
        }
    });
    chosen
}
