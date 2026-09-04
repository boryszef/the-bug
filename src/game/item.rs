use serde::{Deserialize, Serialize};

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
