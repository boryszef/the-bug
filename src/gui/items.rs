//! The Items tab: the player's Equipment (left) and Inventory/storage (right)
//! side by side. Mirrors `gui::experiment`'s two-column layout; unlike
//! Experiment there's no running selection to hold, so this is a plain
//! function like `gui::craft`/`gui::disassemble`, not a struct.

use std::collections::HashMap;

use eframe::egui::{self, Button, RichText, Ui};

use crate::game::Item;
use crate::i18n::{self, Language};
use crate::viewmodel::items::Overview;

/// What the Items tab wants `App` to do to `game` after one frame.
pub(super) enum Outcome {
    /// Nothing to do.
    Idle,
    /// Run `game.transfer_to_storage(item, 1)`.
    TransferToStorage(Item),
    /// Run `game.transfer_to_equipment(item, 1)`.
    TransferToEquipment(Item),
    /// Run `game.drop_from_equipment(item, 1)`.
    DropFromEquipment(Item),
    /// Run `game.drop_from_storage(item, 1)`.
    DropFromStorage(Item),
}

/// Draws the two columns. `at_village` gates transfer (both directions) and
/// dropping from storage; dropping from the equipment is always enabled — the
/// player can lighten their load anywhere, but can only visit the storage
/// (or move things into/out of it) at the Village.
pub(super) fn render(
    ui: &mut Ui,
    overview: &Overview,
    at_village: bool,
    lang: Language,
) -> Outcome {
    ui.heading(i18n::ui("panel-items-title", lang));

    if !at_village {
        ui.label(
            RichText::new(i18n::ui("village-required-hint", lang))
                .color(ui.visuals().error_fg_color),
        );
    }

    let mut outcome = Outcome::Idle;
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.columns(2, |columns| {
            let (carried, capacity) = overview.equipment_progress;
            columns[0].strong(i18n::ui_args(
                "items-equipment-title",
                lang,
                HashMap::from([("count", carried.into()), ("capacity", capacity.into())]),
            ));
            if overview.equipment.is_empty() {
                columns[0].label(i18n::ui("items-equipment-empty", lang));
            }
            for &(item, quantity) in &overview.equipment {
                columns[0].horizontal(|ui| {
                    ui.label(i18n::item_with_quantity(item, quantity, lang));
                    // "→": move one unit to storage. Monospace, like the
                    // Map tab's arrows — egui's default proportional font
                    // has no arrow glyph coverage, but bundled Hack does.
                    if ui
                        .add_enabled(at_village, Button::new(RichText::new("→").monospace()))
                        .clicked()
                    {
                        outcome = Outcome::TransferToStorage(item);
                    }
                    // "x": drop one unit, allowed anywhere.
                    if ui.button("x").clicked() {
                        outcome = Outcome::DropFromEquipment(item);
                    }
                });
            }

            columns[1].strong(i18n::ui("items-storage-title", lang));
            if overview.storage.is_empty() {
                columns[1].label(i18n::ui("items-storage-empty", lang));
            }
            for &(item, quantity) in &overview.storage {
                columns[1].horizontal(|ui| {
                    ui.label(i18n::item_with_quantity(item, quantity, lang));
                    // "←": move one unit to the equipment. Monospace, same reason
                    // as "→" above.
                    if ui
                        .add_enabled(at_village, Button::new(RichText::new("←").monospace()))
                        .clicked()
                    {
                        outcome = Outcome::TransferToEquipment(item);
                    }
                    if ui.add_enabled(at_village, Button::new("x")).clicked() {
                        outcome = Outcome::DropFromStorage(item);
                    }
                });
            }
        });
    });

    outcome
}
