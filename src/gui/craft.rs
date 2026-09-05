//! The Craft tab: a button per known recipe, disabled while the player
//! can't afford its inputs. Mirrors `tui::craft`, mouse-driven — no cursor.

use eframe::egui::{self, Button, Ui};

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
            let label = i18n::item(option.output, lang);
            if ui.add_enabled(option.enabled, Button::new(label)).clicked() {
                chosen = Some(option.id);
            }
        }
    });
    chosen
}
