//! The map tab's tile grid, drawn with an [`egui::Painter`].
//!
//! Holds only transient view state (pan/zoom); every tile comes in as a
//! [`viewmodel::map::TileView`](crate::viewmodel::map::TileView) each frame.
//! See `docs/gui-map.md`.

use std::ops::RangeInclusive;

use eframe::egui::{Color32, Key, Painter, Pos2, Rect, RichText, Sense, Shape, Stroke, Ui, Vec2};

use crate::game::{Direction, Poi, TerrainType};
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

/// The lit ink for the point-of-interest icons — a pale bone that reads on any
/// of the terrain fills a POI can sit on (dark deadland included).
const POI_ICON_COLOR: Color32 = Color32::from_rgb(0xe4, 0xdd, 0xcf);
/// The shadow ink — a cave mouth, a doorway, the far side of a ruin. Drawn on
/// top of the lit shapes, so it only has to read against [`POI_ICON_COLOR`].
const POI_ICON_SHADOW: Color32 = Color32::from_rgb(0x37, 0x2e, 0x24);
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
    Hunt,
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
    /// Draws the movement/search/hunt controls and, below them, the tile grid:
    /// a filled square per visible tile, then the player marker. Consumes
    /// drag (pan) and scroll or pinch (zoom) over the grid. Returns the
    /// player's requested action, from a button or its keyboard accelerator
    /// (arrow keys / `s` / `h`).
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        tiles: impl Iterator<Item = TileView>,
        player: (i32, i32),
        current: Option<TileView>,
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
            if ui.button(i18n::ui("action-hunt", lang)).clicked() {
                command = Some(MapCommand::Hunt);
            }
        });

        // What the player is standing on, in words.
        if let Some(tile) = &current {
            let mut here = i18n::terrain(tile.terrain, lang);
            if let Some(poi) = tile.poi {
                here.push_str("  ·  ");
                here.push_str(&i18n::poi(poi, lang));
            }
            ui.label(here);
        }

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

            // A POI tile stays a clean square so its icon reads clearly;
            // everywhere else the border trickles into its neighbours.
            if tile.poi.is_none() {
                draw_edge_trickle(
                    &painter,
                    rect.center(),
                    self.tile_px,
                    tile.terrain,
                    &tile.neighbours,
                );
            } else if self.tile_px >= POI_ICON_MIN_PX {
                let c = rect.center();
                match tile.poi {
                    Some(Poi::Cave) => draw_cave_icon(&painter, c, self.tile_px),
                    Some(Poi::Ruins) => draw_ruins_icon(&painter, c, self.tile_px),
                    Some(Poi::Village) => draw_village_icon(&painter, c, self.tile_px),
                    None => {}
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
        if i.key_pressed(Key::S) {
            return Some(MapCommand::Search);
        }
        i.key_pressed(Key::H).then_some(MapCommand::Hunt)
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

/// A cave: a bone arch with a darker arched opening cut into it — a cave mouth.
fn draw_cave_icon(painter: &Painter, center: Pos2, tile_px: f32) {
    let size = tile_px * POI_ICON_RATIO;
    let baseline = center.y + size / 2.0;
    painter.add(Shape::convex_polygon(
        arch_points(center.x, baseline, size / 2.0, size * 0.95),
        POI_ICON_COLOR,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        arch_points(center.x, baseline, size * 0.28, size * 0.58),
        POI_ICON_SHADOW,
        Stroke::NONE,
    ));
}

/// Ruins: a standing column and a broken one, a lintel across the top, and a
/// shaded inner face for depth.
fn draw_ruins_icon(painter: &Painter, center: Pos2, tile_px: f32) {
    let ruin = ruins_parts(center, tile_px * POI_ICON_RATIO);
    for column in ruin.columns {
        painter.rect_filled(column, 0.0, POI_ICON_COLOR);
    }
    painter.rect_filled(ruin.lintel, 0.0, POI_ICON_COLOR);
    painter.rect_filled(ruin.shade, 0.0, POI_ICON_SHADOW);
}

/// The village: a little hut — a body under a triangular roof, with a dark
/// doorway.
fn draw_village_icon(painter: &Painter, center: Pos2, tile_px: f32) {
    let (roof, body) = village_hut(center, tile_px * POI_ICON_RATIO);
    painter.rect_filled(body, 0.0, POI_ICON_COLOR);
    painter.add(Shape::convex_polygon(
        roof.to_vec(),
        POI_ICON_COLOR,
        Stroke::NONE,
    ));
    painter.rect_filled(village_door(body), 0.0, POI_ICON_SHADOW);
}

/// For each edge whose neighbour is a *different* terrain, paints a few teeth
/// of that neighbour's colour reaching in from the edge, so the border reads as
/// ragged rather than a straight line. `neighbours` is `[N, E, S, W]`.
fn draw_edge_trickle(
    painter: &Painter,
    center: Pos2,
    tile_px: f32,
    own: TerrainType,
    neighbours: &[Option<TerrainType>; 4],
) {
    for (edge, neighbour) in neighbours.iter().enumerate() {
        let Some(nt) = *neighbour else { continue };
        if nt == own {
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

/// Points tracing the upper half of an ellipse — the outline of a filled arch —
/// from the left foot of the span (`cx - rx`, `baseline_y`) up and over to the
/// right foot (`cx + rx`, `baseline_y`). The polygon's own closing edge along
/// the baseline finishes the silhouette.
fn arch_points(cx: f32, baseline_y: f32, rx: f32, ry: f32) -> Vec<Pos2> {
    const SEGMENTS: usize = 16;
    (0..=SEGMENTS)
        .map(|i| {
            let angle = std::f32::consts::PI * i as f32 / SEGMENTS as f32;
            Pos2::new(cx - rx * angle.cos(), baseline_y - ry * angle.sin())
        })
        .collect()
}

/// The village hut: `(roof triangle, body rect)` for an icon box `size` on a
/// side centred on `center`. The body is the lower ~55%, the roof the upper
/// ~55% (they overlap a little at the eaves).
fn village_hut(center: Pos2, size: f32) -> ([Pos2; 3], Rect) {
    let h = size / 2.0;
    let eaves = center.y - h * 0.1;
    let body = Rect::from_min_max(
        Pos2::new(center.x - h * 0.7, eaves),
        Pos2::new(center.x + h * 0.7, center.y + h),
    );
    let roof = [
        Pos2::new(center.x, center.y - h),
        Pos2::new(center.x - h, eaves),
        Pos2::new(center.x + h, eaves),
    ];
    (roof, body)
}

/// The dark doorway at the foot of the hut `body` — centred, a third of its
/// width, a bit over half its height.
fn village_door(body: Rect) -> Rect {
    let w = body.width() * 0.32;
    let height = body.height() * 0.55;
    Rect::from_min_max(
        Pos2::new(body.center().x - w / 2.0, body.max.y - height),
        Pos2::new(body.center().x + w / 2.0, body.max.y),
    )
}

/// A ruined structure: a full-height column, a broken (shorter) one with a
/// gap between, a lintel resting across the top of the tall column, and a
/// shadow strip down its inner face. For an icon box `size` centred on `center`.
struct RuinsShape {
    /// `[full-height, broken]`.
    columns: [Rect; 2],
    lintel: Rect,
    shade: Rect,
}

fn ruins_parts(center: Pos2, size: f32) -> RuinsShape {
    let h = size / 2.0;
    let baseline = center.y + h;
    let top = center.y - h;
    let column_w = size * 0.24;
    let gap = size * 0.18;
    let left_x = center.x - gap / 2.0 - column_w;
    let right_x = center.x + gap / 2.0;

    let tall = Rect::from_min_max(
        Pos2::new(left_x, top),
        Pos2::new(left_x + column_w, baseline),
    );
    let broken = Rect::from_min_max(
        Pos2::new(right_x, center.y - h * 0.1),
        Pos2::new(right_x + column_w, baseline),
    );
    let lintel = Rect::from_min_max(
        Pos2::new(left_x, top),
        Pos2::new(right_x + column_w * 0.4, top + size * 0.16),
    );
    let shade = Rect::from_min_max(
        Pos2::new(left_x + column_w - size * 0.07, top + size * 0.16),
        Pos2::new(left_x + column_w, baseline),
    );
    RuinsShape {
        columns: [tall, broken],
        lintel,
        shade,
    }
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
    fn arch_points_trace_a_dome_from_one_foot_to_the_other() {
        let (cx, baseline, rx, ry) = (100.0_f32, 60.0_f32, 10.0_f32, 16.0_f32);
        let pts = arch_points(cx, baseline, rx, ry);

        assert!(pts.len() >= 3);
        let first = *pts.first().unwrap();
        let last = *pts.last().unwrap();
        assert!((first.x - (cx - rx)).abs() < 1e-3 && (first.y - baseline).abs() < 1e-3);
        assert!((last.x - (cx + rx)).abs() < 1e-3 && (last.y - baseline).abs() < 1e-3);

        // every point sits within the arch's bounding half-ellipse, above the
        // baseline, and the span runs strictly left to right
        for p in &pts {
            assert!(p.x >= cx - rx - 1e-3 && p.x <= cx + rx + 1e-3);
            assert!(p.y <= baseline + 1e-3 && p.y >= baseline - ry - 1e-3);
        }
        for w in pts.windows(2) {
            assert!(w[0].x < w[1].x + 1e-3, "left to right");
        }
        let apex = pts.iter().min_by(|a, b| a.y.total_cmp(&b.y)).unwrap();
        assert!(
            apex.y < baseline - ry * 0.9,
            "the top of the dome reaches up"
        );
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
    fn ruins_parts_are_two_columns_on_a_baseline_with_a_lintel_on_the_taller() {
        let c = Pos2::new(0.0, 0.0);
        let size = 24.0;
        let half = size / 2.0 + 1e-3;
        let RuinsShape {
            columns: [tall, broken],
            lintel,
            shade,
        } = ruins_parts(c, size);

        assert!((tall.max.y - broken.max.y).abs() < 1e-3, "share a baseline");
        assert!(
            tall.height() > broken.height() + 1e-3,
            "one is broken short"
        );
        assert!(
            tall.max.x <= broken.min.x + 1e-3,
            "tall on the left, a gap between"
        );

        assert!(lintel.min.y <= tall.min.y + 1e-3, "lintel rests on the top");
        assert!(
            lintel.min.x <= tall.min.x + 1e-3 && lintel.max.x > tall.max.x,
            "lintel bridges out from the tall column"
        );
        assert!(
            shade.min.x >= tall.min.x - 1e-3 && shade.max.x <= tall.max.x + 1e-3,
            "shade runs down the tall column's face"
        );

        for r in [tall, broken, lintel, shade] {
            assert!((r.min.x - c.x).abs() <= half && (r.max.x - c.x).abs() <= half);
            assert!((r.min.y - c.y).abs() <= half && (r.max.y - c.y).abs() <= half);
        }
    }

    #[test]
    fn village_hut_has_a_roof_above_a_body_with_a_door_at_its_foot() {
        let c = Pos2::new(0.0, 0.0);
        let size = 20.0;
        let ([apex, rl, rr], body) = village_hut(c, size);
        let door = village_door(body);

        assert!(apex.y < body.min.y, "roof apex above the body");
        assert!(rl.x < c.x && rr.x > c.x, "roof spans the centre");
        assert!(
            body.min.x > rl.x && body.max.x < rr.x,
            "body narrower than the roof"
        );
        assert!(
            (door.max.y - body.max.y).abs() < 1e-3,
            "door sits on the ground"
        );
        assert!(
            door.min.x > body.min.x && door.max.x < body.max.x && door.min.y > body.min.y,
            "door is inside the lower body"
        );
        for p in [apex, rl, rr, body.min, body.max] {
            assert!((p.x - c.x).abs() <= size / 2.0 + 1e-3);
            assert!((p.y - c.y).abs() <= size / 2.0 + 1e-3);
        }
    }
}
