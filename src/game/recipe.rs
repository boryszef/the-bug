use super::item::Item;

/// Which way a recipe runs: whether its `consumables` can be assembled into the
/// `output`, taken back apart from it, or both.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecipeFlow {
    /// Build the output from the consumables; it can't be taken back apart.
    CraftOnly,
    /// Build it, and also take it apart to recover the consumables.
    Both,
    /// Only take it apart — a scavenged object whose parts don't reassemble.
    DisassembleOnly,
}

#[derive(Copy, Clone, Debug)]
pub struct Recipe {
    name: &'static str,
    consumables: &'static [(Item, u32)],
    tools: &'static [Item],
    output: Item,
    flow: RecipeFlow,
}

impl Recipe {
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The items consumed when the recipe runs.
    pub fn consumables(&self) -> &'static [(Item, u32)] {
        self.consumables
    }

    /// Items the player must be holding to run the recipe — one of each
    /// suffices, and none are consumed. A future release may let an entry be a
    /// category ("any axe") rather than a specific item.
    pub fn tools(&self) -> &'static [Item] {
        self.tools
    }

    pub fn output(&self) -> Item {
        self.output
    }

    /// Whether the output can be assembled from the consumables (drives crafting
    /// and experiment discovery).
    pub fn craftable(&self) -> bool {
        matches!(self.flow, RecipeFlow::CraftOnly | RecipeFlow::Both)
    }

    /// Whether the output can be taken apart to recover the consumables (drives
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
        consumables: &[(Item::Branch, 1)],
        tools: &[Item::StoneAxe],
        output: Item::Arrow,
        flow: RecipeFlow::CraftOnly,
    },
    Recipe {
        name: "Wooden Bow",
        consumables: &[(Item::Branch, 1), (Item::Cord, 1)],
        tools: &[Item::StoneAxe],
        output: Item::WoodenBow,
        flow: RecipeFlow::Both,
    },
    Recipe {
        name: "Cord",
        consumables: &[(Item::Vine, 2)],
        tools: &[],
        output: Item::Cord,
        flow: RecipeFlow::CraftOnly,
    },
    Recipe {
        name: "Stone Axe",
        consumables: &[(Item::Branch, 1), (Item::Stone, 1), (Item::Cord, 1)],
        tools: &[],
        output: Item::StoneAxe,
        flow: RecipeFlow::Both,
    },
    Recipe {
        name: "Coil",
        consumables: &[(Item::CopperWire, 2), (Item::PlasticBottle, 1)],
        tools: &[],
        output: Item::Coil,
        flow: RecipeFlow::CraftOnly,
    },
    Recipe {
        name: "Metal Detector",
        consumables: &[
            (Item::Coil, 1),
            (Item::Pole, 1),
            (Item::Speaker, 1),
            (Item::Microcontroller, 1),
        ],
        tools: &[],
        output: Item::MetalDetector,
        flow: RecipeFlow::Both,
    },
    Recipe {
        name: "Solar Charger",
        consumables: &[
            (Item::CopperWire, 1),
            (Item::SolarPanel, 1),
            (Item::CircuitBoard, 1),
        ],
        tools: &[],
        output: Item::SolarCharger,
        flow: RecipeFlow::Both,
    },
    Recipe {
        name: "Umbrella",
        consumables: &[(Item::Fabric, 1), (Item::Pole, 1)],
        tools: &[],
        output: Item::Umbrella,
        flow: RecipeFlow::DisassembleOnly,
    },
    Recipe {
        name: "Electronic Toy",
        consumables: &[(Item::Battery, 1), (Item::Speaker, 1)],
        tools: &[],
        output: Item::ElectronicToy,
        flow: RecipeFlow::DisassembleOnly,
    },
    Recipe {
        name: "Bone Needle",
        consumables: &[(Item::Bone, 1)],
        tools: &[],
        output: Item::BoneNeedle,
        flow: RecipeFlow::CraftOnly,
    },
    Recipe {
        name: "Satchel",
        consumables: &[(Item::Hide, 1), (Item::Cord, 1)],
        tools: &[Item::BoneNeedle],
        output: Item::Satchel,
        flow: RecipeFlow::Both,
    },
];

/// The recipe that produces `output` and can be taken apart, if any. Drives
/// disassembly: the player recovers that recipe's consumables.
pub(crate) fn disassembly_for(output: Item) -> Option<Recipe> {
    RECIPES
        .iter()
        .find(|recipe| recipe.disassemblable() && recipe.output == output)
        .copied()
}

/// The craftable recipe whose consumables are exactly `items` (any order), if
/// any. Drives experimenting: combining items that happen to match a recipe's
/// consumables discovers (or reuses) it. Disassemble-only recipes are skipped —
/// their outputs are scavenged, not built.
pub(super) fn find_matching(items: &[(Item, u32)]) -> Option<&'static Recipe> {
    RECIPES.iter().find(|recipe| {
        recipe.craftable()
            && recipe.consumables.len() == items.len()
            && recipe.consumables.iter().all(|input| items.contains(input))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recipe(flow: RecipeFlow) -> Recipe {
        Recipe {
            name: "x",
            consumables: &[],
            tools: &[],
            output: Item::Branch,
            flow,
        }
    }

    fn recipe_named(name: &str) -> &'static Recipe {
        RECIPES.iter().find(|r| r.name() == name).unwrap()
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

    #[test]
    fn tools_lists_the_recipes_required_but_unconsumed_items() {
        assert_eq!(recipe_named("Wooden Bow").tools(), &[Item::StoneAxe]);
        assert_eq!(recipe_named("Cord").tools(), &[] as &[Item]);
    }
}
