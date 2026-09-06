//! The Disassemble tab: a button per carried item that can be taken apart.
//! Mirrors `tui::disassemble`, mouse-driven — no cursor.

use eframe::egui::{self, Ui};

use crate::game::Item;
use crate::i18n::{self, Language};

/// Draws the item list. Returns the item to take apart if one was clicked.
pub(super) fn render(ui: &mut Ui, options: &[Item], lang: Language) -> Option<Item> {
    ui.heading(i18n::ui("panel-disassemble-title", lang));

    if options.is_empty() {
        ui.label(i18n::ui("disassemble-empty", lang));
        return None;
    }

    let mut chosen = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for &item in options {
            if ui.button(i18n::item(item, lang)).clicked() {
                chosen = Some(item);
            }
        }
    });
    chosen
}
