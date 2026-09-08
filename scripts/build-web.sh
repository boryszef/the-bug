#!/usr/bin/env bash
# Build the wasm web bundle into dist/ using wasm-bindgen-cli directly —
# the fallback for when `trunk` (the preferred tool, see Trunk.toml /
# docs/adr/0005-web-build.md) isn't available.
#
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version <the version in Cargo.lock>
#   ./scripts/build-web.sh          # release
#   ./scripts/build-web.sh debug    # faster build, ~10x bigger .wasm
#
# Then serve it:  (cd dist && python3 -m http.server 8080)

set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE="${1:-release}"
FLAG=$([ "$PROFILE" = release ] && echo --release || echo)

cargo build $FLAG --target wasm32-unknown-unknown --bin the-bug

rm -rf dist
mkdir -p dist
wasm-bindgen --target web --no-typescript --out-dir dist --out-name the-bug \
  "target/wasm32-unknown-unknown/$PROFILE/the-bug.wasm"

if command -v wasm-opt >/dev/null && [ "$PROFILE" = release ]; then
  wasm-opt -Os -o dist/the-bug_bg.wasm dist/the-bug_bg.wasm
fi

cat > dist/index.html <<'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1, user-scalable=no" />
  <title>the-bug</title>
  <style>
    html, body { margin: 0; width: 100%; height: 100%; overflow: hidden; background: #0d0d0d; }
    canvas { position: absolute; inset: 0; width: 100%; height: 100%; }
    #loading { position: absolute; inset: 0; display: flex; align-items: center;
               justify-content: center; color: #888; font: 14px system-ui, sans-serif; }
  </style>
</head>
<body>
  <canvas id="the_canvas_id"></canvas>
  <div id="loading">loading the-bug…</div>
  <script type="module">
    import init from './the-bug.js';
    init()
      .then(() => document.getElementById('loading')?.remove())
      .catch((e) => { document.getElementById('loading').textContent = 'failed: ' + e; console.error(e); });
  </script>
</body>
</html>
HTML

echo "built dist/ ($(du -h dist/the-bug_bg.wasm | cut -f1) wasm)"
