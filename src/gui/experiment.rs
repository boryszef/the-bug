//! The Experiment tab: pick items from the inventory and try to discover a
//! recipe. Mirrors `tui::experiment`, but mouse-driven — the only state is
//! the running [`ItemSelection`]; there is no cursor.

use eframe::egui::{self, Button, Ui};

use crate::game::Item;
use crate::i18n::{self, Language};
use crate::viewmodel::selection::ItemSelection;

#[derive(Default)]
pub(super) struct Experiment {
    selection: ItemSelection,
}

/// What `App` should do after drawing the panel for one frame.
pub(super) enum Outcome {
    /// Nothing to do.
    Idle,
    /// Run `game.experiment(&items)`.
    Run(Vec<(Item, u32)>),
}

impl Experiment {
    /// Draws the two columns and the run button. `inventory` is the player's
    /// inventory in display order, `(item, owned quantity)`.
    pub(super) fn render(
        &mut self,
        ui: &mut Ui,
        inventory: &[(Item, u32)],
        lang: Language,
    ) -> Outcome {
        ui.heading(i18n::ui("panel-experiment-title", lang));

        // The selection is a snapshot from whenever items were picked; the
        // inventory may have shrunk since (e.g. a craft on another tab spent
        // an item this selection counted on), so it's trimmed to the live
        // stock before `owned - picked` is computed below.
        self.selection.clamp_to(inventory);

        let mut outcome = Outcome::Idle;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.columns(2, |columns| {
                columns[0].strong(i18n::ui("panel-available-title", lang));
                for &(item, owned) in inventory {
                    let addable = owned - self.selection.quantity(item);
                    let label = i18n::item_with_quantity(item, addable, lang);
                    if columns[0]
                        .add_enabled(addable > 0, Button::new(label))
                        .clicked()
                    {
                        self.selection.add(item, owned);
                    }
                }

                columns[1].strong(i18n::ui("panel-selected-title", lang));
                // Cloned rather than borrowed: `decrement_at` below needs
                // `&mut self.selection` while this loop is still iterating.
                for (index, &(item, quantity)) in self.selection.items().to_vec().iter().enumerate()
                {
                    let label = i18n::item_with_quantity(item, quantity, lang);
                    if columns[1].button(label).clicked() {
                        self.selection.decrement_at(index);
                    }
                }
            });

            ui.separator();

            if ui
                .add_enabled(
                    !self.selection.is_empty(),
                    Button::new(i18n::ui("action-experiment", lang)),
                )
                .clicked()
            {
                outcome = Outcome::Run(self.selection.take());
            }
        });

        outcome
    }
}
