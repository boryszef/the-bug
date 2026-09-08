# the-bug

A crafting/exploration game (Rust, egui/eframe) set in a post-apocalyptic
future reshaped by alien-derived biological computing.

**▶ Play it in the browser: <https://boryszef.github.io/the-bug/>**

## Run it

**Desktop:**

```sh
cargo run                       # new game
cargo run -- --load save.json   # resume
cargo run -- --lang pl          # force a language (en / pl)
```

The game saves to `the-bug-save-<epoch>.json` on quit (`q` or closing the
window).

**Browser (WebAssembly):**

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk             # if it won't install, use scripts/build-web.sh
trunk serve                     # → http://localhost:8080
trunk build --release           # static bundle in dist/
```

No `trunk`? `cargo install wasm-bindgen-cli --version <see Cargo.lock>` then
`./scripts/build-web.sh` and `(cd dist && python3 -m http.server 8080)`.

The browser build autosaves to `localStorage` and resumes on the next visit
(clear the `the-bug-game` key to start over), and picks up the browser
language (`navigator.language`). See `docs/adr/0005-web-build.md`.

Pushing a `v*` tag (or running the "Deploy web build to GitHub Pages"
workflow by hand) publishes `dist/` to <https://boryszef.github.io/the-bug/>.

## Layout

- `src/game/` — the domain model (pure Rust).
- `src/viewmodel/` — presentation-agnostic helpers.
- `src/gui/` — the egui front end (native and wasm).
- `src/i18n/`, `src/save.rs`, `src/mapgen/` — localisation, persistence,
  terrain generation.
- `tests/` — Gherkin functional tests (cucumber-rs).
- `docs/` — one file per feature/decision; `docs/adr/` for architecture
  decisions; `ARCHITECTURE.md` for the standing rules.

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

`prek install` sets up the pre-commit hook (fmt + clippy).
