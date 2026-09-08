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
  app_creator).await`). `App::on_exit`'s `save::save` (a save *file*) stays
  `#[cfg(not(wasm))]`.

### Persistence and language on the web

- **`localStorage`** — the web build autosaves the game as the same JSON the
  native build writes to disk, under the key `the-bug-game`, via a
  `#[cfg(wasm)] App::save` (eframe calls it on a 30 s timer, on canvas
  focus-loss, and on page unload). `app_creator` reads it back on start and
  resumes that game; a fresh `Game` is only the fallback. A blob that fails
  to parse (corrupt, or an incompatible older schema) is discarded and a new
  game starts — a stale entry never bricks the page. `save.rs` grew
  `to_json`/`from_json` (filesystem-free) for this; `save`/`load` are now
  thin wrappers on them.
- **Language** — the wasm `main` reads `navigator.language` (e.g. `"pl-PL"`)
  and passes it to the existing `i18n::detect`, which already takes the
  language subtag from a BCP 47 tag. No `$LANG` on the web, so this is the
  only signal; it still falls back to English.

### Deps / config

- `web-time` promoted to a direct dep (already in the tree via eframe).
- `[target.'cfg(target_arch = "wasm32")'.dependencies]`:
  `console_error_panic_hook`, `wasm-bindgen-futures`, `web-sys`
  (`Document`/`Window`/`HtmlCanvasElement`/`Navigator`), and
  `getrandom = { version = "0.4", features = ["wasm_js"] }` — `rand`'s RNG
  (mapgen, search, hunt) needs the browser backend. `localStorage` goes
  through eframe's own `Storage` impl, so no extra feature for it.
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
  C build fails under gcc 16). **`scripts/build-web.sh`** is the local-only
  fallback: `cargo build --release --target wasm32-unknown-unknown --bin
  the-bug` → `wasm-bindgen --target web` → optional `wasm-opt` → a plain
  `dist/index.html`. `(cd dist && python3 -m http.server 8080)` to serve.
- `.ftl` locales and `assets/icons/*.png` are `include_bytes!`'d — no runtime
  fetch, nothing else to serve.

### Publishing

- **GitHub Pages**, via `.github/workflows/deploy.yml` (a standard
  `configure-pages` → `upload-pages-artifact` → `deploy-pages` job). It builds
  with `trunk build --release --public-url "/<repo>/"` — the `--public-url`
  makes the hashed asset URLs resolve under the project-page subpath. `trunk`
  installs fine on the GitHub runner (the gcc-16 issue is local to the dev
  box), so CI uses it rather than `scripts/build-web.sh`.
- Trigger: pushing a `v*` tag, or the Actions "Run workflow" button
  (`workflow_dispatch`) — not every push to `main`. The first publish
  therefore needs a tag or a manual run.
- Live at `https://boryszef.github.io/the-bug/`. The repo is public, so Pages
  is free; `configure-pages` self-enables it (`enablement: true`) on the
  first run, else enable it once under Settings → Pages → Source "GitHub
  Actions".
- Adding the workflow meant `.github/` — until now a git-ignored symlink to
  the dev box's shared agent config — becomes a real tracked directory.

## Consequences

- Verified: the release wasm bundle runs in headless Chrome — the map
  generates (RNG works), every panel renders, the event log and the
  "Equipment: n/n" line show. With `--lang=pl-PL` the UI renders in Polish.
  Gameplay + reload round-trips through `localStorage` (walk east, reload,
  the game resumes at the same tile), and a hand-corrupted blob still boots
  a fresh game.
- Bundle size: ~10 MB wasm unoptimised, ~4 MB after `wasm-opt -Os` + gzip.
  Debug is ~48 MB — use release for anything shared.
- The native build, `cargo test`, `cargo clippy --all-targets`, `cargo run`
  are unchanged.
- `.github/` stops being a per-repo symlink to the dev box's shared agent
  config; it's now a real directory in the repo (only `workflows/`).
- `docs/gui-frontend.md`'s "one front end, no web" framing and ADR 0002's
  "not scoped by this decision" note on wasm are now superseded here.

## Not decided / follow-ups

- No CI gate — `deploy.yml` only builds+publishes; `cargo test` / `clippy`
  aren't run on push. A separate CI workflow is still open.
- No in-browser way to clear the save or start over (clear `localStorage`
  by hand); no export/import of the JSON.
- On wasm, `eprintln!` (the version-mismatch note, the "ignoring stored
  game" line) goes nowhere — wiring `web_sys::console` or eframe's
  `WebLogger` is a follow-up if these ever need to be visible.
- Touch/mobile layout, PWA/offline, a WebGPU renderer.
