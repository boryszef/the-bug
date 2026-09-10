# Feature: accessibility — theme and font size

## Why

The gui had no visual-accessibility controls at all: no theme switch, no
font-size control. Requested directly (not from `TODO.md`): a dark/light
mode switch, and a way to pick a base font size.

## Behaviour

Both controls live in the top toolbar (`egui::Panel::top("tabs_and_quit")`
in `App::ui`, `src/gui/mod.rs`), alongside the tab buttons and Quit — the
one area drawn on every panel, so the controls are visible everywhere, not
tucked into a settings screen.

- **Theme.** Two `selectable_label`s, `Light` / `Dark` — the same idiom the
  tab buttons already use in this toolbar. Clicking sets `App.theme:
  egui::Theme`; `App::ui` calls `ctx.set_theme(self.theme)` every frame.
  egui 0.36's `Theme`/`set_theme` do the actual `Visuals` swap; no manual
  `Visuals` construction needed. Default: `Theme::Dark`, matching egui's own
  default look, so a fresh launch is visually unchanged.
- **Font size.** A `ComboBox` with four presets — `Small` / `Medium` /
  `Large` / `Extra Large` (`FontSize` enum, `src/gui/mod.rs`) — each a
  multiplier (`0.85` / `1.0` / `1.2` / `1.45`) applied to egui's own default
  `TextStyle` sizes (`Small=9.0, Body=13.0, Button=13.0, Heading=18.0,
  Monospace=13.0`) via `apply_font_size`, called every frame with
  `ctx.all_styles_mut(...)` (mutates both the light and dark `Style` at
  once, so a later theme switch doesn't lose the size). Recomputed from the
  fixed base every call rather than scaling whatever's currently installed,
  so repeated calls can't compound drift. Default: `Medium` (egui's own
  sizes, unscaled).

Scales only text (`TextStyle` sizes) — not `set_pixels_per_point`/
`set_zoom_factor`, which would also scale the map's `Painter`-drawn tile
grid and every widget's spacing, not just legibility.

### Event-log colours follow the theme too

`event_color` (`src/gui/mod.rs`) colours each event-log line by its
`EventKind` (green for a find/hunt haul, magenta for a quest, yellow for a
craft/hunt/disassemble outcome, cyan for an experiment outcome —
`docs/event-log.md`). Switching to `Theme::Light` exposed a problem: those
colours were the bright, fully-saturated dark-mode set
(`Color32::GREEN`/`MAGENTA`/`YELLOW`/`CYAN`), which read fine on egui's
dark background but wash out against a light one — yellow in particular is
nearly invisible on white. `event_color` now takes the active `egui::Theme`
and picks between two sets: the original bright set for `Theme::Dark`, and
a darker, more saturated set for `Theme::Light` (e.g. `(0, 110, 0)` green,
`(130, 90, 0)` amber instead of pure yellow) chosen for contrast against a
light background. The semantic mapping (which `EventKind`s share a colour)
is unchanged — only the four colour values themselves are theme-dependent.
`render_events` takes `theme` as a parameter now, threaded from
`App.theme` at its one call site in `App::ui`.

### Not emoji

Both controls use plain i18n text labels, not icon buttons (no 🌙/☀). This
project already decided against emoji in the gui once, for POI marks
(`docs/map-improvements.md`'s "Not emoji" section) — egui's bundled fonts
may not carry a given glyph, and this codebase hit exactly that failure
mode before (`TODO.md`'s DONE log: "arrow glyph in the event text does not
render in egui"). Same reasoning applies here.

## Scope

- `src/gui/mod.rs` — `App.theme`/`App.font_size` fields; `FontSize` enum;
  `theme_label_id`/`apply_font_size`; the toolbar controls in `App::ui`.
  Lives here, not `viewmodel`, since it's pure egui rendering configuration
  (`egui::Theme`/`TextStyle`), not UI-agnostic game logic.
- `src/i18n/locales/{en,pl}/main.ftl` — `action-theme-light`,
  `action-theme-dark`, `font-size-label`, `font-size-{small,medium,large,
  extra-large}`. Looked up at runtime via `i18n::ui(id, lang)`, the same
  plain `loader().get(id)` path every other toolbar/panel-title label
  already uses — not `fl!()`-macro-checked at compile time, so no new
  test-coverage gap relative to existing practice.

## Out of scope

- **Persistence.** Both controls reset to their defaults (`Dark`, `Medium`)
  on every launch — a deliberate cut, not an oversight. `eframe`'s
  `persistence` Cargo feature isn't enabled (`Cargo.toml`), so `cc.storage`
  is always `None` on native (see the comment in `app_creator`,
  `src/gui/mod.rs`); persisting a choice would need a new preferences file
  on native (mirroring `save.rs`'s pattern) plus a `localStorage` key on
  web (mirroring `GAME_KEY`, the existing autosaved-game key) — real work,
  not required for what was asked. Worth revisiting if wanted later.
- A `System` theme option (matching the OS). egui's `ThemePreference` has a
  three-way `System`/`Dark`/`Light` variant with a ready-made radio-button
  widget; only the two explicit choices were implemented here.
- Per-panel or per-widget font scaling — one base size applies everywhere.
- High-contrast palettes, colourblind-safe recolouring, or any other
  accessibility axis beyond theme and text size.
