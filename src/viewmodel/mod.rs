//! UI-agnostic helpers that sit between [`crate::game`] and the front-end
//! module [`crate::gui`].
//!
//! No front-end module is permanent. Anything a different front-end (e.g. a
//! web UI) would also need — deriving a sorted inventory, taking the most
//! recent events, walking the map in world coordinates, tracking an item
//! selection — lives here rather than in a front-end module, which keeps
//! only rendering and input handling.

pub mod crafting;
pub mod disassembly;
pub mod events;
pub mod inventory;
pub mod items;
pub mod map;
pub mod panel;
pub mod quests;
pub mod selection;
