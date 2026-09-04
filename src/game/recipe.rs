use super::item::Item;

#[derive(Copy, Clone, Debug)]
pub struct Recipe {
    name: &'static str,
    inputs: &'static [(Item, u32)],
    output: Item,
    reversible: bool,
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
        reversible: false,
    },
    Recipe {
        name: "Wooden Bow",
        inputs: &[(Item::Stick, 1), (Item::Cord, 1)],
        output: Item::WoodenBow,
        reversible: true,
    },
    Recipe {
        name: "Cord",
        inputs: &[(Item::Vine, 2)],
        output: Item::Cord,
        reversible: false,
    },
    Recipe {
        name: "Stone Axe",
        inputs: &[(Item::Stick, 1), (Item::Stone, 1), (Item::Cord, 1)],
        output: Item::StoneAxe,
        reversible: true,
    },
    Recipe {
        name: "Coil",
        inputs: &[(Item::CopperWire, 2), (Item::PlasticBottle, 1)],
        output: Item::Coil,
        reversible: false,
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
        reversible: true,
    },
    Recipe {
        name: "Solar Charger",
        inputs: &[
            (Item::CopperWire, 1),
            (Item::SolarPanel, 1),
            (Item::CircuitBoard, 1),
        ],
        output: Item::SolarCharger,
        reversible: true,
    },
    Recipe {
        name: "Umbrella",
        inputs: &[(Item::Fabric, 1), (Item::Pole, 1)],
        output: Item::Umbrella,
        reversible: true,
    },
];

/// The reversible recipe that produces `output`, if any. Drives disassembly:
/// the player can take such an item apart to recover the recipe's inputs.
pub(crate) fn reversible_recipe_for(output: Item) -> Option<Recipe> {
    RECIPES
        .iter()
        .find(|recipe| recipe.reversible && recipe.output == output)
        .copied()
}

/// The recipe whose inputs are exactly `items` (any order), if any. Drives
/// experimenting: combining items that happen to match a known recipe's
/// inputs discovers (or reuses) it.
pub(super) fn find_matching(items: &[(Item, u32)]) -> Option<&'static Recipe> {
    RECIPES.iter().find(|recipe| {
        recipe.inputs.len() == items.len()
            && recipe.inputs.iter().all(|input| items.contains(input))
    })
}
