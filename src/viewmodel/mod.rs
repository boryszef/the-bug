//! UI-agnostic helpers that sit between [`crate::game`] and a front-end
//! module such as [`crate::tui`].
//!
//! No front-end module is permanent. Anything a different front-end (e.g. a
//! graphical UI) would also need — deriving a sorted inventory, taking the
//! most recent events, walking the map in world coordinates, tracking an
//! item selection — lives here rather than in a front-end module, which
//! keeps only rendering and key handling.

pub mod crafting;
pub mod disassembly;
pub mod events;
pub mod inventory;
pub mod map;
pub mod quests;
pub mod selection;
