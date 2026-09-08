//! The Disassemble tab: a button per carried item that can be taken apart.
//! Mouse-driven — no cursor.

use eframe::egui::{self, Button, RichText, Ui};

use crate::game::Item;
use crate::i18n::{self, Language};

/// Draws the item list. `at_village` disables every button (with a hint
/// explaining why) when the player isn't standing somewhere disassembly is
/// allowed. Returns the item to take apart if one was clicked.
pub(super) fn render(
    ui: &mut Ui,
    options: &[Item],
    at_village: bool,
    lang: Language,
) -> Option<Item> {
    ui.heading(i18n::ui("panel-disassemble-title", lang));

    if !at_village {
        ui.label(
            RichText::new(i18n::ui("village-required-hint", lang))
                .color(ui.visuals().error_fg_color),
        );
    }

    if options.is_empty() {
        ui.label(i18n::ui("disassemble-empty", lang));
        return None;
    }

    let mut chosen = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for &item in options {
            if ui
                .add_enabled(at_village, Button::new(i18n::item(item, lang)))
                .clicked()
            {
                chosen = Some(item);
            }
        }
    });
    chosen
}
