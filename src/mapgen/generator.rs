//! The map generator: exact composition, affinity-controlled clustering.
//!
//! Pipeline (see `docs/mapgen.md` for the rationale):
//!
//! 1. **Quotas** — turn the cluster percentages into exact per-terrain tile
//!    counts over every cell (largest-remainder rounding, so the counts always
//!    sum exactly).
//! 2. **`affinity = 1` layout** — recursive rectilinear bisection slices the
//!    cells into one contiguous block per terrain.
//! 3. **Melt** — swap pairs of cells, always taking a swap that
//!    doesn't worsen clustering and taking a worsening one with flat probability
//!    `(1 - affinity)^2`. This disorders the clean layout toward randomness
//!    without a temperature to tune. Swaps never change counts, so the
//!    composition stays exact. The two endpoints (`0.0` = uniform shuffle,
//!    `1.0` = untouched layout) are handled directly.

use std::fmt;

use rand::RngExt;
use rand::seq::SliceRandom;

use crate::game::TerrainType;

/// Melt iterations, as a multiple of the clustering-cell count. Enough that a
/// fully-disordering melt (`affinity` near 0) reaches uniform.
const MELT_SWEEPS: usize = 80;
/// Distance from an `affinity` endpoint (0 or 1) still treated as that endpoint.
const EPS: f64 = 1e-6;

type Cell = (usize, usize);

/// What to generate.
pub(crate) struct Spec {
    /// Map edge length. Must be odd and at least 5.
    pub(crate) size: usize,
    /// Clustering terrain and its share, as a percentage. Must sum to 100.
    pub(crate) clusters: Vec<(TerrainType, f64)>,
    /// `0.0` fully random .. `1.0` one contiguous blob per clustering terrain.
    pub(crate) affinity: f64,
}

#[derive(Debug, PartialEq)]
pub(crate) enum GenError {
    EmptyClusters,
    NegativePercent(TerrainType, f64),
    PercentSum(f64),
    EvenSize(usize),
    TooSmall(usize),
    DuplicateTerrain(TerrainType),
}

impl fmt::Display for GenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenError::EmptyClusters => write!(f, "clusters must not be empty"),
            GenError::NegativePercent(t, pct) => {
                write!(f, "{t:?}'s share can't be negative (got {pct})")
            }
            GenError::PercentSum(sum) => {
                write!(f, "cluster percentages must sum to 100 (got {sum})")
            }
            GenError::EvenSize(n) => write!(f, "size must be odd (got {n})"),
            GenError::TooSmall(n) => write!(f, "size must be at least 5 (got {n})"),
            GenError::DuplicateTerrain(t) => write!(f, "{t:?} is listed more than once"),
        }
    }
}

impl std::error::Error for GenError {}

impl Spec {
    pub(crate) fn validate(&self) -> Result<(), GenError> {
        if self.clusters.is_empty() {
            return Err(GenError::EmptyClusters);
        }

        let mut seen = Vec::new();
        for &(t, pct) in &self.clusters {
            if seen.contains(&t) {
                return Err(GenError::DuplicateTerrain(t));
            }
            if pct < 0.0 {
                // A negative share can still sum to 100 alongside one over
                // 100 (e.g. `[150.0, -50.0]`), which would otherwise pass
                // the sum check below and then underflow the largest-
                // remainder rounding in `cluster_quotas`.
                return Err(GenError::NegativePercent(t, pct));
            }
            seen.push(t);
        }

        let sum: f64 = self.clusters.iter().map(|&(_, p)| p).sum();
        if (sum - 100.0).abs() > 1e-6 {
            return Err(GenError::PercentSum(sum));
        }

        if self.size < 5 {
            return Err(GenError::TooSmall(self.size));
        }
        if self.size.is_multiple_of(2) {
            return Err(GenError::EvenSize(self.size));
        }

        Ok(())
    }
}

/// Generates a `size x size` grid: every cell filled with a clustering terrain
/// at exactly its quota. (The village and other points of interest are an
/// overlay laid on top by `Map::new` — `docs/map-pois.md`.)
pub(crate) fn generate(
    spec: &Spec,
    rng: &mut impl rand::Rng,
) -> Result<Vec<Vec<TerrainType>>, GenError> {
    spec.validate()?;

    let size = spec.size;
    let quotas = cluster_quotas(&spec.clusters, size * size);

    let mut cells: Vec<Cell> = (0..size)
        .flat_map(|y| (0..size).map(move |x| (x, y)))
        .collect();
    cells.shuffle(rng);

    let mut grid = vec![vec![TerrainType::Deadland; size]; size];

    if spec.affinity <= EPS {
        // `cells` is already a random permutation (shuffled above); pairing
        // it in fixed order with `bag` (grouped by terrain) already assigns
        // each cell a uniform random draw from the terrain multiset, so
        // `bag` itself doesn't need its own shuffle too.
        let mut bag: Vec<TerrainType> = Vec::with_capacity(cells.len());
        for &(terrain, count) in &quotas {
            bag.extend(std::iter::repeat_n(terrain, count));
        }
        for (&(x, y), &terrain) in cells.iter().zip(&bag) {
            grid[y][x] = terrain;
        }
        return Ok(grid);
    }

    // The affinity-1 layout: recursively slice the cells into one contiguous
    // block per terrain, each exactly its quota. Over a full rectangle every
    // slice comes out contiguous, so unlike the old scatter-holed grid there's
    // nothing that can fail here.
    let active: Vec<(TerrainType, usize)> =
        quotas.iter().copied().filter(|&(_, q)| q > 0).collect();
    bisect(&mut cells, &active, rng, &mut grid);

    if spec.affinity < 1.0 - EPS {
        melt(&mut grid, &cells, spec.affinity, rng);
    }

    Ok(grid)
}

/// Largest-remainder rounding: `floor(pct/100 * n)` per terrain, then the
/// leftover cells go to the largest fractional parts (ties broken by list
/// order). The result always sums to exactly `n`.
fn cluster_quotas(clusters: &[(TerrainType, f64)], n: usize) -> Vec<(TerrainType, usize)> {
    let mut rows: Vec<(TerrainType, usize, f64)> = clusters
        .iter()
        .map(|&(t, pct)| {
            let exact = pct / 100.0 * n as f64;
            (t, exact.floor() as usize, exact.fract())
        })
        .collect();

    let assigned: usize = rows.iter().map(|&(_, floor, _)| floor).sum();
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| {
        rows[b]
            .2
            .partial_cmp(&rows[a].2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    for &i in order.iter().take(n - assigned) {
        rows[i].1 += 1;
    }

    rows.into_iter().map(|(t, count, _)| (t, count)).collect()
}

/// Recursive rectilinear bisection ("slice and dice"). Splits `cells` into one
/// block per terrain, each exactly its quota, by repeatedly cutting the current
/// group along an axis at the point that separates the first half of the quota
/// from the second. A prefix of cells sorted by `(x, y)` is a set of whole
/// columns plus a partial one — contiguous — and likewise by `(y, x)`, so every
/// block comes out 4-connected. Alternating the axis by recursion depth keeps
/// the blocks blocky rather than striped.
fn bisect(
    cells: &mut [Cell],
    quotas: &[(TerrainType, usize)],
    rng: &mut impl rand::Rng,
    grid: &mut [Vec<TerrainType>],
) {
    match quotas {
        [] => {}
        [(terrain, _)] => {
            for &(x, y) in cells.iter() {
                grid[y][x] = *terrain;
            }
        }
        _ => {
            let total: usize = quotas.iter().map(|&(_, q)| q).sum();
            let mut acc = 0;
            let mut split = 1;
            for (i, &(_, q)) in quotas.iter().enumerate() {
                acc += q;
                if acc * 2 >= total {
                    split = i + 1;
                    break;
                }
            }
            let left_len: usize = quotas[..split].iter().map(|&(_, q)| q).sum();

            if rng.random::<bool>() {
                cells.sort_unstable_by_key(|&(x, y)| (x, y));
            } else {
                cells.sort_unstable_by_key(|&(x, y)| (y, x));
            }
            if rng.random::<bool>() {
                cells.reverse();
            }

            let (left, right) = cells.split_at_mut(left_len);
            bisect(left, &quotas[..split], rng, grid);
            bisect(right, &quotas[split..], rng, grid);
        }
    }
}

/// Disorders the clean affinity-1 layout toward randomness by repeatedly trying
/// to swap two clustering cells of different terrain. A swap that doesn't raise
/// the unlike-neighbour count is always taken; one that does is taken with a flat
/// probability `(1 - affinity)^2`, independent of how bad it is. So `affinity`
/// near 1 keeps the blocks (only edges soften), and near 0 every swap goes
/// through and the field mixes to uniform. Swaps never change tile counts.
fn melt(grid: &mut [Vec<TerrainType>], cells: &[Cell], affinity: f64, rng: &mut impl rand::Rng) {
    let disorder = (1.0 - affinity).powi(2);
    let iterations = MELT_SWEEPS * cells.len();

    for _ in 0..iterations {
        let p = cells[rng.random_range(0..cells.len())];
        let mut q = cells[rng.random_range(0..cells.len())];
        let mut tries = 0;
        while grid[q.1][q.0] == grid[p.1][p.0] && tries < 8 {
            q = cells[rng.random_range(0..cells.len())];
            tries += 1;
        }
        if grid[q.1][q.0] == grid[p.1][p.0] {
            continue;
        }

        let worsens = boundary_delta(grid, p, q) > 0;
        if !worsens || rng.random::<f64>() < disorder {
            let tmp = grid[p.1][p.0];
            grid[p.1][p.0] = grid[q.1][q.0];
            grid[q.1][q.0] = tmp;
        }
    }
}

/// Change in the number of unlike orthogonally-adjacent pairs if the terrain at
/// `p` and `q` were swapped. Only edges touching `p` or `q` move.
fn boundary_delta(grid: &[Vec<TerrainType>], p: Cell, q: Cell) -> i32 {
    let (tp, tq) = (grid[p.1][p.0], grid[q.1][q.0]);
    let mut delta = 0;

    for nb in orthogonal(p, grid.len()) {
        if nb == q {
            continue;
        }
        let tn = grid[nb.1][nb.0];
        delta += i32::from(tn != tq) - i32::from(tn != tp);
    }
    for nb in orthogonal(q, grid.len()) {
        if nb == p {
            continue;
        }
        let tn = grid[nb.1][nb.0];
        delta += i32::from(tn != tp) - i32::from(tn != tq);
    }

    delta
}

fn orthogonal((x, y): Cell, size: usize) -> Vec<Cell> {
    let mut out = Vec::with_capacity(4);
    if x > 0 {
        out.push((x - 1, y));
    }
    if x + 1 < size {
        out.push((x + 1, y));
    }
    if y > 0 {
        out.push((x, y - 1));
    }
    if y + 1 < size {
        out.push((x, y + 1));
    }
    out
}

/// The number of 4-connected components of `terrain` in `grid` (other terrain
/// is ignored). A test helper.
#[cfg(test)]
pub(crate) fn components(grid: &[Vec<TerrainType>], terrain: TerrainType) -> usize {
    let size = grid.len();
    let mut seen = vec![vec![false; size]; size];
    let mut count = 0;

    for y in 0..size {
        for x in 0..size {
            if grid[y][x] != terrain || seen[y][x] {
                continue;
            }
            count += 1;
            let mut stack = vec![(x, y)];
            seen[y][x] = true;
            while let Some(cell) = stack.pop() {
                for nb in orthogonal(cell, size) {
                    if !seen[nb.1][nb.0] && grid[nb.1][nb.0] == terrain {
                        seen[nb.1][nb.0] = true;
                        stack.push(nb);
                    }
                }
            }
        }
    }

    count
}

/// The fraction of orthogonally-adjacent cell pairs whose two ends differ.
/// Near `0` when clustered, near the mixing probability when random. A test
/// helper — used only by this module's own tests, unlike `components` (also
/// a test helper, but re-exported for `game::map`'s tests too).
#[cfg(test)]
fn boundary_ratio(grid: &[Vec<TerrainType>]) -> f64 {
    let size = grid.len();
    let (mut unlike, mut total) = (0u64, 0u64);

    for y in 0..size {
        for x in 0..size {
            let t = grid[y][x];
            for nb in [(x + 1, y), (x, y + 1)] {
                if nb.0 >= size || nb.1 >= size {
                    continue;
                }
                total += 1;
                if grid[nb.1][nb.0] != t {
                    unlike += 1;
                }
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        unlike as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(0xC0FFEE)
    }

    fn count(grid: &[Vec<TerrainType>], terrain: TerrainType) -> usize {
        grid.iter().flatten().filter(|&&t| t == terrain).count()
    }

    fn spec(size: usize, affinity: f64) -> Spec {
        Spec {
            size,
            clusters: vec![(TerrainType::Forest, 70.0), (TerrainType::Meadow, 30.0)],
            affinity,
        }
    }

    #[test]
    fn composition_is_exact() {
        let s = spec(23, 0.5);
        let quotas = cluster_quotas(&s.clusters, 23 * 23);
        let grid = generate(&s, &mut rng()).unwrap();

        for (terrain, quota) in quotas {
            assert_eq!(count(&grid, terrain), quota, "{terrain:?}");
        }
        let total: usize = grid.iter().map(|r| r.len()).sum();
        assert_eq!(total, 23 * 23);
        assert_eq!(
            count(&grid, TerrainType::Forest) + count(&grid, TerrainType::Meadow),
            23 * 23
        );
    }

    #[test]
    fn affinity_one_is_a_single_component_per_terrain() {
        for seed in 0..6 {
            let grid = generate(&spec(23, 1.0), &mut StdRng::seed_from_u64(seed)).unwrap();
            assert_eq!(
                components(&grid, TerrainType::Forest),
                1,
                "seed {seed} forest"
            );
            assert_eq!(
                components(&grid, TerrainType::Meadow),
                1,
                "seed {seed} meadow"
            );
        }
    }

    #[test]
    fn affinity_zero_is_close_to_a_plain_shuffle() {
        let grid = generate(&spec(23, 0.0), &mut rng()).unwrap();
        let ratio = boundary_ratio(&grid);

        // A uniform 70/30 mix has ~2*0.7*0.3 unlike neighbours.
        let expected = 2.0 * 0.7 * 0.3;
        assert!(
            (ratio - expected).abs() < 0.06,
            "boundary ratio {ratio} not near {expected}"
        );
        assert!(
            components(&grid, TerrainType::Forest) > 5,
            "forest not fragmented"
        );
    }

    #[test]
    fn higher_affinity_clusters_more() {
        let ratio = |a| boundary_ratio(&generate(&spec(23, a), &mut rng()).unwrap());
        let (low, mid, high) = (ratio(0.0), ratio(0.5), ratio(1.0));
        assert!(low > mid, "0.0 ({low}) should be noisier than 0.5 ({mid})");
        assert!(
            mid > high,
            "0.5 ({mid}) should be noisier than 1.0 ({high})"
        );
    }

    #[test]
    fn deterministic_given_a_seed() {
        let a = generate(&spec(23, 0.6), &mut StdRng::seed_from_u64(42)).unwrap();
        let b = generate(&spec(23, 0.6), &mut StdRng::seed_from_u64(42)).unwrap();
        let c = generate(&spec(23, 0.6), &mut StdRng::seed_from_u64(43)).unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn largest_remainder_always_sums_to_n() {
        let cases: &[(&[f64], usize)] = &[
            (&[33.0, 33.0, 34.0], 529),
            (&[98.0, 1.0, 1.0], 400),
            (&[100.0, 0.0], 361),
            (&[50.0, 50.0], 7),
        ];
        for &(pcts, n) in cases {
            let clusters: Vec<(TerrainType, f64)> = pcts
                .iter()
                .zip([
                    TerrainType::Forest,
                    TerrainType::Meadow,
                    TerrainType::Deadland,
                ])
                .map(|(&p, t)| (t, p))
                .collect();
            let quotas = cluster_quotas(&clusters, n);
            let sum: usize = quotas.iter().map(|&(_, q)| q).sum();
            assert_eq!(sum, n, "pcts {pcts:?}, n {n}");
            for (&(_, pct), &(_, q)) in clusters.iter().zip(&quotas) {
                let ideal = pct / 100.0 * n as f64;
                assert!((q as f64 - ideal).abs() < 1.0);
            }
        }
    }

    #[test]
    fn bisect_handles_a_hard_three_way_split() {
        for seed in 0..6 {
            let s = Spec {
                size: 23,
                clusters: vec![
                    (TerrainType::Forest, 34.0),
                    (TerrainType::Meadow, 33.0),
                    (TerrainType::Deadland, 33.0),
                ],
                affinity: 1.0,
            };
            assert!(
                generate(&s, &mut StdRng::seed_from_u64(seed)).is_ok(),
                "seed {seed}"
            );
        }
    }

    #[test]
    fn grid_shape_and_alphabet() {
        let grid = generate(&spec(27, 0.5), &mut rng()).unwrap();
        assert_eq!(grid.len(), 27);
        assert!(grid.iter().all(|r| r.len() == 27));
        assert!(
            grid.iter()
                .flatten()
                .all(|&t| "MF.".contains(crate::save::terrain_code(t)))
        );
    }

    #[test]
    fn rejects_bad_specs() {
        let base = spec(23, 0.5);

        let bad = Spec {
            clusters: vec![(TerrainType::Forest, 70.0), (TerrainType::Meadow, 20.0)],
            ..spec(23, 0.5)
        };
        assert!(matches!(bad.validate(), Err(GenError::PercentSum(_))));

        assert_eq!(
            Spec {
                size: 22,
                ..spec(23, 0.5)
            }
            .validate(),
            Err(GenError::EvenSize(22))
        );
        assert_eq!(
            Spec {
                size: 3,
                ..spec(23, 0.5)
            }
            .validate(),
            Err(GenError::TooSmall(3))
        );

        let dupe = Spec {
            clusters: vec![(TerrainType::Forest, 50.0), (TerrainType::Forest, 50.0)],
            ..spec(23, 0.5)
        };
        assert_eq!(
            dupe.validate(),
            Err(GenError::DuplicateTerrain(TerrainType::Forest))
        );

        // sums to 100, but only because one share is negative
        let negative = Spec {
            clusters: vec![(TerrainType::Forest, 150.0), (TerrainType::Meadow, -50.0)],
            ..spec(23, 0.5)
        };
        assert_eq!(
            negative.validate(),
            Err(GenError::NegativePercent(TerrainType::Meadow, -50.0))
        );

        assert!(base.validate().is_ok());
    }
}
