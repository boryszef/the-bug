use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Item {
    Stick,
    Stone,
    Vine,
    Cord,
    StoneAxe,
    Arrow,
    WoodenBow,
    PlasticBottle,
    CopperWire,
    Coil,
    Pole,
    Microcontroller,
    Speaker,
    MetalDetector,
    Battery,
    SolarPanel,
    SolarCharger,
    CircuitBoard,
    Umbrella,
    Fabric,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Item::Stick => "Stick",
            Item::Stone => "Stone",
            Item::Vine => "Vine",
            Item::Cord => "Cord",
            Item::StoneAxe => "Stone Axe",
            Item::Arrow => "Arrow",
            Item::WoodenBow => "Wooden Bow",
            Item::PlasticBottle => "Plastic Bottle",
            Item::CopperWire => "Copper Wire",
            Item::Coil => "Coil",
            Item::Pole => "Pole",
            Item::Speaker => "Speaker",
            Item::Microcontroller => "Microcontroller",
            Item::MetalDetector => "Metal Detector",
            Item::Battery => "Battery",
            Item::SolarPanel => "Solar Panel",
            Item::SolarCharger => "Solar Charger",
            Item::CircuitBoard => "Circuit Board",
            Item::Umbrella => "Umbrella",
            Item::Fabric => "Fabric",
        };
        write!(f, "{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_display_names() {
        assert_eq!(Item::Stick.to_string(), "Stick");
        assert_eq!(Item::StoneAxe.to_string(), "Stone Axe");
    }
}
