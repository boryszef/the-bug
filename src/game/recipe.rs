use super::item::Item;

/// Which way a recipe runs: whether its `inputs` can be assembled into the
/// `output`, taken back apart from it, or both.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecipeFlow {
    /// Build the output from the inputs; it can't be taken back apart.
    CraftOnly,
    /// Build it, and also take it apart to recover the inputs.
    Both,
    /// Only take it apart — a scavenged object whose parts don't reassemble.
    DisassembleOnly,
}

#[derive(Copy, Clone, Debug)]
pub struct Recipe {
    name: &'static str,
    inputs: &'static [(Item, u32)],
    output: Item,
    flow: RecipeFlow,
}

impl Recipe {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn inputs(&self) -> &'static [(Item, u32)] {
        self.inputs
    }

    pub fn output(&self) -> Item {
        self.output
    }

    /// Whether the output can be assembled from the inputs (drives crafting and
    /// experiment discovery).
    pub fn craftable(&self) -> bool {
        matches!(self.flow, RecipeFlow::CraftOnly | RecipeFlow::Both)
    }

    /// Whether the output can be taken apart to recover the inputs (drives
    /// disassembly).
    pub fn disassemblable(&self) -> bool {
        matches!(self.flow, RecipeFlow::Both | RecipeFlow::DisassembleOnly)
    }
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub(super) const RECIPES: &[Recipe] = &[
    Recipe {
        name: "Arrow",
        inputs: &[(Item::Stick, 1)],
        output: Item::Arrow,
        flow: RecipeFlow::CraftOnly,
    },
    Recipe {
        name: "Wooden Bow",
        inputs: &[(Item::Stick, 1), (Item::Cord, 1)],
        output: Item::WoodenBow,
        flow: RecipeFlow::Both,
    },
    Recipe {
        name: "Cord",
        inputs: &[(Item::Vine, 2)],
        output: Item::Cord,
        flow: RecipeFlow::CraftOnly,
    },
    Recipe {
        name: "Stone Axe",
        inputs: &[(Item::Stick, 1), (Item::Stone, 1), (Item::Cord, 1)],
        output: Item::StoneAxe,
        flow: RecipeFlow::Both,
    },
    Recipe {
        name: "Coil",
        inputs: &[(Item::CopperWire, 2), (Item::PlasticBottle, 1)],
        output: Item::Coil,
        flow: RecipeFlow::CraftOnly,
    },
    Recipe {
        name: "Metal Detector",
        inputs: &[
            (Item::Coil, 1),
            (Item::Pole, 1),
            (Item::Speaker, 1),
            (Item::Microcontroller, 1),
        ],
        output: Item::MetalDetector,
        flow: RecipeFlow::Both,
    },
    Recipe {
        name: "Solar Charger",
        inputs: &[
            (Item::CopperWire, 1),
            (Item::SolarPanel, 1),
            (Item::CircuitBoard, 1),
        ],
        output: Item::SolarCharger,
        flow: RecipeFlow::Both,
    },
    Recipe {
        name: "Umbrella",
        inputs: &[(Item::Fabric, 1), (Item::Pole, 1)],
        output: Item::Umbrella,
        flow: RecipeFlow::DisassembleOnly,
    },
    Recipe {
        name: "Electronic Toy",
        inputs: &[(Item::Battery, 1), (Item::Speaker, 1)],
        output: Item::ElectronicToy,
        flow: RecipeFlow::DisassembleOnly,
    },
];

/// The recipe that produces `output` and can be taken apart, if any. Drives
/// disassembly: the player recovers that recipe's inputs.
pub(crate) fn disassembly_for(output: Item) -> Option<Recipe> {
    RECIPES
        .iter()
        .find(|recipe| recipe.disassemblable() && recipe.output == output)
        .copied()
}

/// The craftable recipe whose inputs are exactly `items` (any order), if any.
/// Drives experimenting: combining items that happen to match a recipe's
/// inputs discovers (or reuses) it. Disassemble-only recipes are skipped —
/// their outputs are scavenged, not built.
pub(super) fn find_matching(items: &[(Item, u32)]) -> Option<&'static Recipe> {
    RECIPES.iter().find(|recipe| {
        recipe.craftable()
            && recipe.inputs.len() == items.len()
            && recipe.inputs.iter().all(|input| items.contains(input))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recipe(flow: RecipeFlow) -> Recipe {
        Recipe {
            name: "x",
            inputs: &[],
            output: Item::Stick,
            flow,
        }
    }

    #[test]
    fn flow_maps_to_the_craftable_and_disassemblable_predicates() {
        let craft_only = recipe(RecipeFlow::CraftOnly);
        assert!(craft_only.craftable() && !craft_only.disassemblable());

        let both = recipe(RecipeFlow::Both);
        assert!(both.craftable() && both.disassemblable());

        let disasm_only = recipe(RecipeFlow::DisassembleOnly);
        assert!(!disasm_only.craftable() && disasm_only.disassemblable());
    }
}
