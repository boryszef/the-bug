//! The map generator: exact terrain composition with affinity-controlled
//! clustering. Called by `Map::new` (`src/game/map.rs`), which supplies the
//! tuning (`affinity`, the cluster ratio) as consts.
//!
//! Started life as the standalone `tools/mapgen` dev tool; it was folded into
//! the crate once the game generated its map with it directly, so it now shares
//! `game::TerrainType` instead of duplicating the terrain table. See
//! `docs/mapgen.md`.

mod generator;

#[cfg(test)]
pub(crate) use generator::components;
pub(crate) use generator::{Spec, generate};
