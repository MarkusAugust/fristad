#!/bin/sh
# Bygger Fristad for nettleseren til dist/. Kjøres av Netlify ved hver push,
# og kan kjøres lokalt med `sh tools/build-web.sh`.
set -eu

echo "== Rust for wasm32"
if command -v rustup >/dev/null 2>&1; then
  rustup target add wasm32-unknown-unknown
fi

# wasm-bindgen-verktøyet må ha nøyaktig samme versjon som biblioteket i
# Cargo.lock. Den ferdigbygde utgaven fra GitHub er mye raskere enn å bygge.
WB=$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/version = |"/,""); print; exit}' Cargo.lock)
TOOLS="$PWD/target/tools"
mkdir -p "$TOOLS"
if [ ! -x "$TOOLS/wasm-bindgen" ] || [ "$("$TOOLS/wasm-bindgen" --version | awk '{print $2}')" != "$WB" ]; then
  echo "== wasm-bindgen $WB"
  ARCHIVE="wasm-bindgen-$WB-x86_64-unknown-linux-musl"
  if curl -fsSL "https://github.com/rustwasm/wasm-bindgen/releases/download/$WB/$ARCHIVE.tar.gz" | tar -xz -C "$TOOLS"; then
    cp "$TOOLS/$ARCHIVE/wasm-bindgen" "$TOOLS/wasm-bindgen"
  else
    cargo install wasm-bindgen-cli --version "$WB" --root "$TOOLS/cargo"
    cp "$TOOLS/cargo/bin/wasm-bindgen" "$TOOLS/wasm-bindgen"
  fi
fi

echo "== Fristil"
command -v bun >/dev/null 2>&1 || npm install -g bun
bun install
bun tools/export-tokens.ts
# Dokumentasjonen hentes ferskt fra GitHub. Feiler det, brukes den som
# ligger i repoet.
bun tools/export-docs.ts || echo "Bruker web/docs.json fra repoet"

echo "== Teksturlagene (egne bilder og CC0 fra Poly Haven)"
# Feiler nedlastingen, bakes lagene i motoren.
bun tools/fetch-assets.ts || echo "Bruker bakte lag"

echo "== Motoren"
cargo build --release --lib --target wasm32-unknown-unknown

echo "== Siden"
rm -rf dist
mkdir -p dist
"$TOOLS/wasm-bindgen" --target web --no-typescript --out-dir dist/pkg \
  target/wasm32-unknown-unknown/release/fristad.wasm
cp web/index.html web/main.js web/music.js web/ambience.js web/sounds.js web/style.css web/docs.json dist/
# Lydene (opptak fra frie spill, se assets/sounds/CREDITS.md).
mkdir -p dist/sounds
cp assets/sounds/*.mp3 assets/sounds/sounds.json assets/sounds/CREDITS.md dist/sounds/
cp node_modules/@fristil/designsystem/dist/fristil.css dist/
# De frittstående web-komponentene (toast, connection-status) importeres som
# ES-moduler fra pakken, uten kildekart og typer.
SRC=node_modules/@fristil/designsystem/dist
(cd "$SRC" && find . -name "*.js" ! -name "cli.js" ! -name "react.js" ! -name "*.map") | while read -r f; do
  mkdir -p "dist/fristil/$(dirname "$f")"
  cp "$SRC/$f" "dist/fristil/$f"
done
# Modellene ligger ved siden av motoren, gzip-pakket (main.js pakker dem ut).
sh tools/pack-models.sh dist
if [ -f assets/external/manifest.json ]; then
  mkdir -p dist/assets
  cp -r assets/external/. dist/assets/
fi
ls -la dist dist/pkg
du -sh dist/fristil 2>/dev/null || true
