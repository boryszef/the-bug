//! The set of gameplay features unlocked so far — used to onboard new
//! players a piece at a time instead of showing every tab (map, crafting,
//! experimenting...) from the very first frame. See docs/tutorial-unlocks.md.

use serde::{Deserialize, Serialize};

/// One gate-able front-end feature. Growing this enum is how a future
/// milestone earns its own onboarding beat — see `Quest::unlocks_on_accept`
/// / `unlocks_on_complete` in `quest.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unlocked {
    Map,
    Items,
    Craft,
    Disassemble,
    Experiment,
}
