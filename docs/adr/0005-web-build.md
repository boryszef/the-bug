# 5. A WebAssembly (browser) build

## Status

Accepted. Realises the "client-side web build via eframe's wasm target"
that ADR 0001 ("A web build becomes available later via `eframe`'s
`wasm32-unknown-unknown` target") and ADR 0002 flagged as future work.

## Context

`eframe` compiles to `wasm32-unknown-unknown` and runs egui in a browser
canvas with no server. The-bug's gui is the only front end; the game core is
pure Rust. Making it playable in a browser is almost entirely `#[cfg]` plumbing
plus two real changes.

## Decision

The `the-bug` **binary** compiles for `wasm32-unknown-unknown` alongside the
native build; a bundler turns it into a static page.

### Code

- **`web_time::Instant`** (`docs`… actually a prior commit) replaces
  `std::time::Instant` in `game/mod.rs` (session clock) and `game/map.rs`
  (search/hunt decay) — `Instant::now()` aborts on the base wasm target.
  Transparent: `std::time::Instant` verbatim on native.
- **`src/main.rs`** — the whole native path (clap `Cli`, `save::load`,
  `std::process::exit`, its tests) is `#[cfg(not(target_arch = "wasm32"))]`
  in a `mod native`. A `#[cfg(target_arch = "wasm32")] fn main()` installs
  `console_error_panic_hook`, builds a fresh `Game`, finds `#the_canvas_id`,
  and `spawn_local`s `gui::run_web`.
- **`src/gui/mod.rs`** — the `App`-creator closure is factored into
  `app_creator()`, shared by `run` (native, `run_native`) and `run_web`
  (wasm, `eframe::WebRunner::start(canvas, WebOptions::default(),
  app_creator).await`). `App::on_exit`'s `save::save` is
  `#[cfg(not(wasm))]` — **the web build has no persistence** (fresh game each
  visit).

### Deps / config

- `web-time` promoted to a direct dep (already in the tree via eframe).
- `[target.'cfg(target_arch = "wasm32")'.dependencies]`:
  `console_error_panic_hook`, `wasm-bindgen-futures`, `web-sys`
  (`Document`/`Window`/`HtmlCanvasElement`), and
  `getrandom = { version = "0.4", features = ["wasm_js"] }` — `rand`'s RNG
  (mapgen, search, hunt) needs the browser backend.
- `.cargo/config.toml` — `[target.wasm32-unknown-unknown] rustflags = ["--cfg",
  "getrandom_backend=\"wasm_js\""]`.
- `rust-embed` gains `features = ["debug-embed"]` so a **debug** wasm build
  embeds the `.ftl` locales too (release already does; a wasm build has no
  filesystem to read them from — otherwise `i18n::…loader()` panics).
- **Renderer**: eframe 0.36's default is `wgpu`, and `egui-wgpu` enables
  `wgpu/webgl` on wasm — the web build runs on **WebGL2**, works in Firefox.
  No Cargo feature change; native is untouched.

### Bundling

- **`trunk`** is the intended tool: `index.html` (repo root, with the
  `data-trunk` directive) + `Trunk.toml`; `trunk serve` for dev, `trunk build
  --release` for deploy.
- `trunk` currently won't `cargo install` on the dev box (its `libdeflate-sys`
  C build fails under gcc 16). **`scripts/build-web.sh`** is the fallback:
  `cargo build --release --target wasm32-unknown-unknown --bin the-bug` →
  `wasm-bindgen --target web` → optional `wasm-opt` → a plain `dist/index.html`.
  `(cd dist && python3 -m http.server 8080)` to serve.
- `.ftl` locales and `assets/icons/*.png` are `include_bytes!`'d — no runtime
  fetch, nothing else to serve.

## Consequences

- Verified: the release wasm bundle runs in headless Chrome — the map
  generates (RNG works), every panel renders, the event log and the
  "Equipment: n/n" line show, English (no `$LANG` on web).
- Bundle size: ~10 MB wasm unoptimised, ~4 MB after `wasm-opt -Os` + gzip.
  Debug is ~48 MB — use release for anything shared.
- The native build, `cargo test`, `cargo clippy --all-targets`, `cargo run`
  are unchanged.
- `docs/gui-frontend.md`'s "one front end, no web" framing and ADR 0002's
  "not scoped by this decision" note on wasm are now superseded here.

## Not decided / follow-ups

- **Publishing** — no host chosen. GitHub Pages (needs the repo pushed to
  GitHub + a `trunk build` Actions workflow), itch.io (zip `dist/`), or a
  static host. The repo has no remote yet.
- **`localStorage` persistence** — either the existing JSON via `web_sys`
  `local_storage()`, or eframe's `Storage` (different format, touches
  `App::save`/`load`).
- `navigator.language` locale detection (English default for now).
- Touch/mobile layout, PWA/offline, a WebGPU renderer.
