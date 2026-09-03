//! UI-agnostic helpers that sit between [`crate::game`] and [`crate::ui`].
//!
//! The ratatui front-end is temporary. Anything a different front-end (e.g. a
//! web UI) would also need — deriving a sorted inventory, taking the most
//! recent events, walking the map in world coordinates, tracking an item
//! selection — lives here rather than in `ui`, which keeps only rendering and
//! key handling.

pub mod crafting;
pub mod events;
pub mod inventory;
pub mod map;
pub mod selection;
