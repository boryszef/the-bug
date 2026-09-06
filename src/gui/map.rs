//! The map tab's tile grid, drawn with an [`egui::Painter`].
//!
//! Holds only transient view state (pan/zoom); every tile comes in as a
//! [`viewmodel::map::TileView`](crate::viewmodel::map::TileView) each frame.
//! See `docs/gui-map.md`.

use std::ops::RangeInclusive;

use eframe::egui::{Color32, Key, Painter, Pos2, Rect, RichText, Sense, Shape, Stroke, Ui, Vec2};

use crate::game::{Direction, TerrainType};
use crate::i18n::{self, Language};
use crate::viewmodel::map::{TileView, terrain_rgb};

/// Pixel size of one tile at the default zoom.
const DEFAULT_TILE_PX: f32 = 24.0;
/// Zoom clamp: the smallest and largest a tile may be drawn.
const MIN_TILE_PX: f32 = 6.0;
const MAX_TILE_PX: f32 = 96.0;
/// How fast a scroll tick zooms, as a fraction of the current tile size per
/// scroll unit.
const SCROLL_ZOOM_RATE: f32 = 0.002;
/// Player marker radius, as a fraction of the tile size.
const PLAYER_MARKER_RATIO: f32 = 0.3;
const PLAYER_MARKER_COLOR: Color32 = Color32::from_rgb(0xff, 0xd0, 0x2f);

/// One dark ink for the point-of-interest icons (Cave, Ruins), legible on both
/// the cave grey and the ruins tan.
const POI_ICON_COLOR: Color32 = Color32::from_rgb(0x24, 0x20, 0x1c);
/// POI icon extent as a fraction of the tile.
const POI_ICON_RATIO: f32 = 0.6;
/// Below this tile size the POI icon is skipped — it would just be noise.
const POI_ICON_MIN_PX: f32 = 12.0;

/// Wavy-border "trickle": how many teeth of the neighbour's colour reach in
/// along a shared edge, where along the edge they sit (fraction), how deep
/// they go (fraction of the tile), and how wide each is (fraction of the tile).
const TRICKLE_TEETH: usize = 2;
const TRICKLE_OFFSETS: [f32; TRICKLE_TEETH] = [0.32, 0.68];
const TRICKLE_DEPTHS: [f32; TRICKLE_TEETH] = [0.22, 0.12];
const TRICKLE_TOOTH_W: f32 = 0.26;

/// What the Map tab wants `App` to do to `game` after one frame — from an
/// on-screen button or its keyboard accelerator. Mirrors how the other gui
/// panels return their action.
#[derive(Clone, Copy, Debug)]
pub(super) enum MapCommand {
    Walk(Direction),
    Search,
}

/// Transient pan/zoom state for the map tab.
pub struct MapView {
    /// World coordinate under the centre of the viewport.
    center: Vec2,
    /// Pixel size of one tile.
    tile_px: f32,
}

impl Default for MapView {
    fn default() -> Self {
        MapView {
            center: Vec2::ZERO,
            tile_px: DEFAULT_TILE_PX,
        }
    }
}

impl MapView {
    /// Draws the movement/search controls and, below them, the tile grid:
    /// a filled square per visible tile, then the player marker. Consumes
    /// drag (pan) and scroll or pinch (zoom) over the grid. Returns the
    /// player's requested action, from a button or its keyboard accelerator
    /// (arrow keys / `s`).
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        tiles: impl Iterator<Item = TileView>,
        player: (i32, i32),
        lang: Language,
    ) -> Option<MapCommand> {
        let mut command = None;
        ui.horizontal(|ui| {
            // Arrow glyphs via the monospace font: egui's default
            // proportional font has no arrow coverage, but bundled Hack
            // (the Monospace family) does.
            for (glyph, dir) in [
                ("←", Direction::West),
                ("↑", Direction::North),
                ("↓", Direction::South),
                ("→", Direction::East),
            ] {
                if ui.button(RichText::new(glyph).monospace()).clicked() {
                    command = Some(MapCommand::Walk(dir));
                }
            }
            ui.separator();
            if ui.button(i18n::ui("action-search", lang)).clicked() {
                command = Some(MapCommand::Search);
            }
        });

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let viewport = response.rect;

        if response.dragged() {
            let drag = response.drag_delta();
            self.center.x -= drag.x / self.tile_px;
            self.center.y += drag.y / self.tile_px; // screen y is inverted
        }

        if response.hovered() {
            let (pinch, scroll) = ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta.y));
            if pinch != 1.0 {
                self.tile_px = clamp_tile_px(self.tile_px * pinch);
            }
            if scroll != 0.0 {
                self.tile_px = clamp_tile_px(self.tile_px * (1.0 + scroll * SCROLL_ZOOM_RATE));
            }
        }

        let (xs, ys) = visible_tiles(viewport, self.center, self.tile_px);
        for tile in tiles {
            let (wx, wy) = tile.world;
            if !xs.contains(&wx) || !ys.contains(&wy) {
                continue;
            }
            let rect = tile_rect(wx, wy, viewport, self.center, self.tile_px);
            let (r, g, b) = terrain_rgb(tile.terrain);
            painter.rect_filled(rect, 0.0, Color32::from_rgb(r, g, b));

            if is_field(tile.terrain) {
                draw_edge_trickle(
                    &painter,
                    rect.center(),
                    self.tile_px,
                    tile.terrain,
                    &tile.neighbours,
                );
            }

            if self.tile_px >= POI_ICON_MIN_PX {
                match tile.terrain {
                    TerrainType::Cave => draw_cave_icon(&painter, rect.center(), self.tile_px),
                    TerrainType::Ruins => draw_ruins_icon(&painter, rect.center(), self.tile_px),
                    _ => {}
                }
            }
        }

        let marker = world_to_screen(
            Vec2::new(player.0 as f32, player.1 as f32),
            viewport,
            self.center,
            self.tile_px,
        );
        painter.circle_filled(
            marker,
            self.tile_px * PLAYER_MARKER_RATIO,
            PLAYER_MARKER_COLOR,
        );

        command.or_else(|| read_map_keys(ui))
    }
}

/// Arrow keys walk, `s` searches — the keyboard accelerators for the Map
/// tab's on-screen buttons. Only read while the Map tab is drawn (the only
/// caller).
fn read_map_keys(ui: &Ui) -> Option<MapCommand> {
    ui.input(|i| {
        for (key, dir) in [
            (Key::ArrowUp, Direction::North),
            (Key::ArrowDown, Direction::South),
            (Key::ArrowLeft, Direction::West),
            (Key::ArrowRight, Direction::East),
        ] {
            if i.key_pressed(key) {
                return Some(MapCommand::Walk(dir));
            }
        }
        i.key_pressed(Key::S).then_some(MapCommand::Search)
    })
}

/// World point → screen pixel. World Y increases northward
/// (`Direction::North => (0, 1)`) while screen Y increases downward, hence
/// the flip.
fn world_to_screen(world: Vec2, viewport: Rect, center: Vec2, tile_px: f32) -> Pos2 {
    viewport.center()
        + Vec2::new(
            (world.x - center.x) * tile_px,
            (center.y - world.y) * tile_px,
        )
}

/// The screen rectangle covering the tile centred on world `(wx, wy)`.
fn tile_rect(wx: i32, wy: i32, viewport: Rect, center: Vec2, tile_px: f32) -> Rect {
    let middle = world_to_screen(Vec2::new(wx as f32, wy as f32), viewport, center, tile_px);
    Rect::from_center_size(middle, Vec2::splat(tile_px))
}

/// A cave: a small upward wedge (a hill / a cave mouth).
fn draw_cave_icon(painter: &Painter, center: Pos2, tile_px: f32) {
    let tri = cave_triangle(center, tile_px * POI_ICON_RATIO);
    painter.add(Shape::convex_polygon(
        tri.to_vec(),
        POI_ICON_COLOR,
        Stroke::NONE,
    ));
}

/// Ruins: a broken skyline of uneven bars.
fn draw_ruins_icon(painter: &Painter, center: Pos2, tile_px: f32) {
    for bar in ruins_bars(center, tile_px * POI_ICON_RATIO) {
        painter.rect_filled(bar, 0.0, POI_ICON_COLOR);
    }
}

/// The terrains the wavy-border trickle applies to. Mirrors `mapgen`'s private
/// `is_clustering`; promote to a `TerrainType` method if a third caller appears.
fn is_field(terrain: TerrainType) -> bool {
    matches!(
        terrain,
        TerrainType::Meadow | TerrainType::Forest | TerrainType::Deadland
    )
}

/// For each edge whose neighbour is a *different* field terrain, paints a few
/// teeth of that neighbour's colour reaching in from the edge, so the border
/// reads as ragged rather than a straight line. `neighbours` is `[N, E, S, W]`.
fn draw_edge_trickle(
    painter: &Painter,
    center: Pos2,
    tile_px: f32,
    own: TerrainType,
    neighbours: &[Option<TerrainType>; 4],
) {
    for (edge, neighbour) in neighbours.iter().enumerate() {
        let Some(nt) = *neighbour else { continue };
        if nt == own || !is_field(nt) {
            continue;
        }
        let (r, g, b) = terrain_rgb(nt);
        for tooth in edge_teeth(center, tile_px, edge) {
            painter.rect_filled(tooth, 0.0, Color32::from_rgb(r, g, b));
        }
    }
}

/// The teeth reaching in from one edge (`0 = N, 1 = E, 2 = S, 3 = W`) of the
/// tile centred on `center`. Fixed pattern — just enough to break the line.
fn edge_teeth(center: Pos2, tile_px: f32, edge: usize) -> [Rect; TRICKLE_TEETH] {
    let h = tile_px / 2.0;
    let half_w = tile_px * TRICKLE_TOOTH_W / 2.0;
    std::array::from_fn(|i| {
        let along = (TRICKLE_OFFSETS[i] - 0.5) * tile_px;
        let depth = tile_px * TRICKLE_DEPTHS[i];
        match edge {
            0 => Rect::from_min_max(
                Pos2::new(center.x + along - half_w, center.y - h),
                Pos2::new(center.x + along + half_w, center.y - h + depth),
            ),
            1 => Rect::from_min_max(
                Pos2::new(center.x + h - depth, center.y + along - half_w),
                Pos2::new(center.x + h, center.y + along + half_w),
            ),
            2 => Rect::from_min_max(
                Pos2::new(center.x + along - half_w, center.y + h - depth),
                Pos2::new(center.x + along + half_w, center.y + h),
            ),
            _ => Rect::from_min_max(
                Pos2::new(center.x - h, center.y + along - half_w),
                Pos2::new(center.x - h + depth, center.y + along + half_w),
            ),
        }
    })
}

/// The three corners of the cave wedge — apex up, base along the bottom — for
/// an icon box `size` on a side centred on `center`.
fn cave_triangle(center: Pos2, size: f32) -> [Pos2; 3] {
    let h = size / 2.0;
    [
        Pos2::new(center.x, center.y - h),
        Pos2::new(center.x - h, center.y + h),
        Pos2::new(center.x + h, center.y + h),
    ]
}

/// Four skyline bars — uneven heights, bottoms on a common baseline — for an
/// icon box `size` on a side centred on `center`.
fn ruins_bars(center: Pos2, size: f32) -> [Rect; 4] {
    const HEIGHTS: [f32; 4] = [0.5, 0.95, 0.68, 0.82];
    let h = size / 2.0;
    let baseline = center.y + h;
    let left = center.x - h;
    let slot = size / 4.0;
    let bar_w = slot * 0.7;
    std::array::from_fn(|i| {
        let x0 = left + slot * i as f32 + (slot - bar_w) / 2.0;
        Rect::from_min_max(
            Pos2::new(x0, baseline - size * HEIGHTS[i]),
            Pos2::new(x0 + bar_w, baseline),
        )
    })
}

/// The inclusive ranges of world tile coordinates (x, then y) that can be at
/// least partly visible in `viewport`. Generous by up to a tile on each
/// edge — culling only needs to never drop a visible tile.
fn visible_tiles(
    viewport: Rect,
    center: Vec2,
    tile_px: f32,
) -> (RangeInclusive<i32>, RangeInclusive<i32>) {
    let half_w = viewport.width() / (2.0 * tile_px);
    let half_h = viewport.height() / (2.0 * tile_px);
    (
        (center.x - half_w).floor() as i32..=(center.x + half_w).ceil() as i32,
        (center.y - half_h).floor() as i32..=(center.y + half_h).ceil() as i32,
    )
}

fn clamp_tile_px(value: f32) -> f32 {
    value.clamp(MIN_TILE_PX, MAX_TILE_PX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> Rect {
        Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(800.0, 600.0))
    }

    #[test]
    fn world_to_screen_scales_by_tile_px_and_flips_y() {
        let vp = viewport();
        let origin = world_to_screen(Vec2::ZERO, vp, Vec2::ZERO, 20.0);
        let east = world_to_screen(Vec2::new(1.0, 0.0), vp, Vec2::ZERO, 20.0);
        let north = world_to_screen(Vec2::new(0.0, 1.0), vp, Vec2::ZERO, 20.0);

        assert!((origin - vp.center()).length() < 1e-3);
        assert!((east.x - origin.x - 20.0).abs() < 1e-3);
        assert!((east.y - origin.y).abs() < 1e-3);
        // North is "up" the screen: smaller y.
        assert!((north.y - (origin.y - 20.0)).abs() < 1e-3);
        assert!((north.x - origin.x).abs() < 1e-3);
    }

    #[test]
    fn world_to_screen_puts_the_camera_target_at_the_viewport_centre() {
        let vp = viewport();
        let screen = world_to_screen(Vec2::new(5.0, -7.0), vp, Vec2::new(5.0, -7.0), 30.0);
        assert!((screen - vp.center()).length() < 1e-3);
    }

    #[test]
    fn visible_tiles_covers_the_camera_centre() {
        let (xs, ys) = visible_tiles(viewport(), Vec2::new(5.0, -3.0), 24.0);
        assert!(xs.contains(&5));
        assert!(ys.contains(&-3));
    }

    #[test]
    fn visible_tiles_excludes_far_off_tiles_but_keeps_near_ones() {
        let (xs, ys) = visible_tiles(viewport(), Vec2::ZERO, 24.0);
        assert!(xs.contains(&10));
        assert!(!xs.contains(&100));
        assert!(!ys.contains(&-100));
    }

    #[test]
    fn visible_tiles_follows_the_camera() {
        let (xs, _) = visible_tiles(viewport(), Vec2::new(1000.0, 0.0), 24.0);
        assert!(xs.contains(&1000));
        assert!(!xs.contains(&0));
    }

    #[test]
    fn clamp_tile_px_bounds_zoom_both_ways() {
        assert_eq!(clamp_tile_px(0.5), MIN_TILE_PX);
        assert_eq!(clamp_tile_px(10_000.0), MAX_TILE_PX);
        assert_eq!(clamp_tile_px(24.0), 24.0);
    }

    #[test]
    fn cave_triangle_is_an_upward_wedge_within_the_icon_box() {
        let c = Pos2::new(100.0, 50.0);
        let size = 20.0;
        let [apex, bl, br] = cave_triangle(c, size);

        assert!(apex.y < c.y, "apex above centre");
        assert!(bl.y > c.y && br.y > c.y, "base below centre");
        assert!(bl.x < br.x, "base runs left to right");
        for p in [apex, bl, br] {
            assert!((p.x - c.x).abs() <= size / 2.0 + 1e-3);
            assert!((p.y - c.y).abs() <= size / 2.0 + 1e-3);
        }
    }

    #[test]
    fn edge_teeth_reach_in_from_the_named_edge_without_crossing_the_centre() {
        let c = Pos2::new(0.0, 0.0);
        let px = 40.0;
        let h = px / 2.0;

        let north = edge_teeth(c, px, 0);
        for t in &north {
            assert!(
                (t.min.y - (c.y - h)).abs() < 1e-3,
                "north teeth sit on the top edge"
            );
            assert!(
                t.max.y > t.min.y && t.max.y < c.y,
                "reach inward, not past centre"
            );
            assert!(t.min.x >= c.x - h - 1e-3 && t.max.x <= c.x + h + 1e-3);
        }
        for w in north.windows(2) {
            assert!(
                w[0].max.x <= w[1].min.x + 1e-3,
                "teeth don't overlap along the edge"
            );
        }

        let east = edge_teeth(c, px, 1);
        for t in &east {
            assert!(
                (t.max.x - (c.x + h)).abs() < 1e-3,
                "east teeth sit on the right edge"
            );
            assert!(
                t.min.x < t.max.x && t.min.x > c.x,
                "reach inward from the right"
            );
        }

        let west = edge_teeth(c, px, 3);
        for t in &west {
            assert!(
                (t.min.x - (c.x - h)).abs() < 1e-3,
                "west teeth sit on the left edge"
            );
            assert!(t.max.x < c.x, "reach inward from the left");
        }
    }

    #[test]
    fn is_field_is_the_three_clustering_terrains() {
        for t in [
            TerrainType::Meadow,
            TerrainType::Forest,
            TerrainType::Deadland,
        ] {
            assert!(is_field(t));
        }
        for t in [TerrainType::Cave, TerrainType::Ruins, TerrainType::Village] {
            assert!(!is_field(t));
        }
    }

    #[test]
    fn ruins_bars_are_four_distinct_bars_on_a_common_baseline() {
        let c = Pos2::new(0.0, 0.0);
        let size = 24.0;
        let bars = ruins_bars(c, size);

        let baseline = bars[0].max.y;
        for b in &bars {
            assert!((b.max.y - baseline).abs() < 1e-3, "bars share a baseline");
            assert!(b.min.x >= c.x - size / 2.0 - 1e-3 && b.max.x <= c.x + size / 2.0 + 1e-3);
            assert!(
                b.min.y >= c.y - size / 2.0 - 1e-3,
                "bar stays in the icon box"
            );
        }
        for w in bars.windows(2) {
            assert!(
                w[0].max.x <= w[1].min.x + 1e-3,
                "bars left to right, no overlap"
            );
        }
        let heights: Vec<f32> = bars.iter().map(Rect::height).collect();
        for i in 0..heights.len() {
            for j in (i + 1)..heights.len() {
                assert!((heights[i] - heights[j]).abs() > 1e-3, "bar heights differ");
            }
        }
    }
}
