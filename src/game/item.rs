use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Item {
    Branch,
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
    ElectricMotor,
    SteelBolt,
    RustyMetal,
    MetalKnife,
    ElectronicToy,
    Meat,
    Bone,
    Hide,
    Fur,
    Satchel,
    BoneNeedle,
}

impl Item {
    /// Every variant, in declaration order. A test-only helper — used by a
    /// completeness check (e.g. "every item renders in every language" in
    /// `i18n`'s tests) that walks the whole enum, so a new variant can't
    /// silently fall out of a hand-maintained sample list.
    #[cfg(test)]
    pub const ALL: [Item; 31] = [
        Item::Branch,
        Item::Stone,
        Item::Vine,
        Item::Cord,
        Item::StoneAxe,
        Item::Arrow,
        Item::WoodenBow,
        Item::PlasticBottle,
        Item::CopperWire,
        Item::Coil,
        Item::Pole,
        Item::Microcontroller,
        Item::Speaker,
        Item::MetalDetector,
        Item::Battery,
        Item::SolarPanel,
        Item::SolarCharger,
        Item::CircuitBoard,
        Item::Umbrella,
        Item::Fabric,
        Item::ElectricMotor,
        Item::SteelBolt,
        Item::RustyMetal,
        Item::MetalKnife,
        Item::ElectronicToy,
        Item::Meat,
        Item::Bone,
        Item::Hide,
        Item::Fur,
        Item::Satchel,
        Item::BoneNeedle,
    ];
}
