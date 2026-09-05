//! `mapgen` — a dev tool that generates candidate terrain maps in the game's
//! save-file letter codes, with affinity-controlled clustering.
//!
//! Not shipped with the game and not part of the `the-bug` crate. See
//! `docs/mapgen.md`.

mod generator;
mod terrain;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::generator::{Spec, generate};
use crate::terrain::Terrain;

/// Generate a candidate terrain map (save-file letter codes) with
/// affinity-controlled clustering. Prints the grid to stdout.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Clustering terrain as NAME=PERCENT (repeatable): Meadow, Forest or
    /// Deadland. The percentages must sum to 100.
    #[arg(
        short = 't',
        long = "terrain",
        value_name = "NAME=PCT",
        required = true,
        value_parser = parse_cluster
    )]
    terrain: Vec<(Terrain, f64)>,

    /// Scattered terrain as NAME=COUNT (repeatable): Cave or Ruins. An absolute
    /// tile count, placed uniformly at random on top of the 100%.
    #[arg(short = 's', long = "scatter", value_name = "NAME=COUNT", value_parser = parse_scatter)]
    scatter: Vec<(Terrain, u32)>,

    /// Cluster affinity: 0.0 fully random .. 1.0 one contiguous blob per terrain.
    #[arg(short = 'a', long, default_value_t = 0.6, value_parser = parse_affinity)]
    affinity: f64,

    /// Map edge length. Must be odd. Game maps are 21 + 2*level (21, 23, 25, ...).
    #[arg(long, default_value_t = 23)]
    size: usize,

    /// RNG seed, for reproducible output. Defaults to OS entropy.
    #[arg(long)]
    seed: Option<u64>,

    /// Also write a minimal `the-bug --load`-able JSON save wrapping the grid.
    #[arg(long, value_name = "FILE")]
    json: Option<PathBuf>,
}

fn parse_cluster(spec: &str) -> Result<(Terrain, f64), String> {
    let (name, value) = spec.split_once('=').ok_or("expected NAME=PCT")?;
    let terrain = Terrain::parse_name(name).ok_or_else(|| format!("unknown terrain {name:?}"))?;
    let pct: f64 = value
        .trim()
        .parse()
        .map_err(|_| format!("invalid percentage {value:?}"))?;
    Ok((terrain, pct))
}

fn parse_scatter(spec: &str) -> Result<(Terrain, u32), String> {
    let (name, value) = spec.split_once('=').ok_or("expected NAME=COUNT")?;
    let terrain = Terrain::parse_name(name).ok_or_else(|| format!("unknown terrain {name:?}"))?;
    let count: u32 = value
        .trim()
        .parse()
        .map_err(|_| format!("invalid count {value:?}"))?;
    Ok((terrain, count))
}

fn parse_affinity(value: &str) -> Result<f64, String> {
    let affinity: f64 = value
        .parse()
        .map_err(|_| format!("invalid number {value:?}"))?;
    if (0.0..=1.0).contains(&affinity) {
        Ok(affinity)
    } else {
        Err("must be between 0.0 and 1.0".into())
    }
}

/// The game level a map of this size belongs to (`size = 21 + 2 * level`), if it
/// is a valid game map size.
fn game_level(size: usize) -> Option<usize> {
    (size >= 21 && (size - 21).is_multiple_of(2)).then(|| (size - 21) / 2)
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut rng = match cli.seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_rng(&mut rand::rng()),
    };

    let spec = Spec {
        size: cli.size,
        clusters: cli.terrain,
        scatter: cli.scatter,
        affinity: cli.affinity,
    };

    let grid = match generate(&spec, &mut rng) {
        Ok(grid) => grid,
        Err(e) => {
            eprintln!("mapgen: {e}");
            return ExitCode::FAILURE;
        }
    };

    if game_level(cli.size).is_none() {
        eprintln!(
            "mapgen: warning: size {} is not a game map size (21 + 2*level); \
             the save will load but won't match any level",
            cli.size
        );
    }

    let rows: Vec<String> = grid
        .iter()
        .map(|row| row.iter().map(|t| t.code()).collect())
        .collect();
    for row in &rows {
        println!("{row}");
    }

    if let Some(path) = &cli.json {
        let level = game_level(cli.size).unwrap_or(1).max(1);
        let json = save_json(level, &rows);
        if let Err(e) = std::fs::write(path, json) {
            eprintln!("mapgen: could not write {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        eprintln!("mapgen: wrote {}", path.display());
    }

    ExitCode::SUCCESS
}

/// A minimal save file the game will load: the other `PlayerState` fields are
/// `#[serde(default)]`, and an absent `version` produces no mismatch warning.
/// The grid letters are `[A-Z.]`, so no JSON escaping is needed.
fn save_json(level: usize, rows: &[String]) -> String {
    let terrain = rows
        .iter()
        .map(|row| format!("\"{row}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"player\":{{\"level\":{level},\"coordinates\":[0,0],\"inventory\":{{}},\"recipes\":[]}},\
         \"map\":{{\"terrain\":[{terrain}]}},\"events\":[]}}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_cluster_spec() {
        assert_eq!(parse_cluster("Forest=70").unwrap(), (Terrain::Forest, 70.0));
        assert_eq!(
            parse_cluster("meadow = 30").unwrap(),
            (Terrain::Meadow, 30.0)
        );
        assert!(parse_cluster("Swamp=10").is_err());
        assert!(parse_cluster("Forest").is_err());
    }

    #[test]
    fn parses_a_scatter_spec() {
        assert_eq!(parse_scatter("Cave=6").unwrap(), (Terrain::Cave, 6));
        assert!(parse_scatter("Cave=1.5").is_err());
    }

    #[test]
    fn rejects_out_of_range_affinity() {
        assert_eq!(parse_affinity("0.5").unwrap(), 0.5);
        assert!(parse_affinity("1.5").is_err());
        assert!(parse_affinity("-0.1").is_err());
    }

    #[test]
    fn game_level_matches_map_sizes() {
        assert_eq!(game_level(21), Some(0));
        assert_eq!(game_level(23), Some(1));
        assert_eq!(game_level(25), Some(2));
        assert_eq!(game_level(24), None);
        assert_eq!(game_level(19), None);
    }

    #[test]
    fn save_json_has_the_fields_the_loader_needs() {
        let json = save_json(
            1,
            &["VMF".to_string(), "MMM".to_string(), "FFF".to_string()],
        );
        for needle in [
            "\"player\"",
            "\"level\":1",
            "\"coordinates\":[0,0]",
            "\"inventory\":{}",
            "\"recipes\":[]",
            "\"map\":{\"terrain\":[\"VMF\",\"MMM\",\"FFF\"]}",
            "\"events\":[]",
        ] {
            assert!(json.contains(needle), "missing {needle} in {json}");
        }
    }
}
