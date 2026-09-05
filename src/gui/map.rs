//! The map tab's tile grid, drawn with an [`egui::Painter`].
//!
//! Holds only transient view state (pan/zoom); every tile comes in as a
//! [`viewmodel::map::TileView`](crate::viewmodel::map::TileView) each frame.
//! See `docs/gui-map.md`.

use std::ops::RangeInclusive;

use eframe::egui::{Color32, Key, Pos2, Rect, RichText, Sense, Ui, Vec2};

use crate::game::Direction;
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
            let (r, g, b) = terrain_rgb(tile.terrain);
            painter.rect_filled(
                tile_rect(wx, wy, viewport, self.center, self.tile_px),
                0.0,
                Color32::from_rgb(r, g, b),
            );
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
}
