//! The six terrain kinds and their save-file letter codes.
//
// This table mirrors `the-bug`'s `src/save.rs::terrain_code` / `terrain_from_code`,
// which are `pub(crate)` and unreachable from here. ADR 0002 rejected a lib/bin
// split, so the mapping is duplicated on purpose — if the game's codes change,
// change both.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Terrain {
    Meadow,
    Forest,
    Cave,
    Ruins,
    Village,
    Deadland,
}

impl Terrain {
    /// The single-character code this terrain is written as in a save file.
    pub fn code(self) -> char {
        match self {
            Terrain::Meadow => 'M',
            Terrain::Forest => 'F',
            Terrain::Cave => 'C',
            Terrain::Ruins => 'R',
            Terrain::Village => 'V',
            Terrain::Deadland => '.',
        }
    }

    /// Parses a terrain name, case-insensitively (`"forest"`, `"Forest"`, ...).
    pub fn parse_name(name: &str) -> Option<Terrain> {
        match name.trim().to_ascii_lowercase().as_str() {
            "meadow" => Some(Terrain::Meadow),
            "forest" => Some(Terrain::Forest),
            "cave" => Some(Terrain::Cave),
            "ruins" => Some(Terrain::Ruins),
            "village" => Some(Terrain::Village),
            "deadland" => Some(Terrain::Deadland),
            _ => None,
        }
    }

    /// Terrains that form clusters: everything the affinity mechanic applies to.
    pub fn is_clustering(self) -> bool {
        matches!(self, Terrain::Meadow | Terrain::Forest | Terrain::Deadland)
    }

    /// Terrains that are only ever scattered, never clustered.
    pub fn is_scatter(self) -> bool {
        matches!(self, Terrain::Cave | Terrain::Ruins)
    }
}

impl fmt::Display for Terrain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Terrain::Meadow => "Meadow",
            Terrain::Forest => "Forest",
            Terrain::Cave => "Cave",
            Terrain::Ruins => "Ruins",
            Terrain::Village => "Village",
            Terrain::Deadland => "Deadland",
        })
    }
}
