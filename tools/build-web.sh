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
cp web/index.html web/main.js web/editor.js web/github.js web/music.js web/score.js web/ambience.js web/sounds.js web/style.css web/docs.json dist/
# Åpningsfilmen: den ferdige videoen (tools/film/cut.py skriver target/film/intro.mp4;
# i Actions bygges den i et eget steg med hurtiglager). Mangler den, starter spillet
# uten film, så bygget stopper ikke.
mkdir -p dist/film/fonts && cp web/film/player.js dist/film/ && cp web/film/fonts/* dist/film/fonts/
[ -f target/film/intro.mp4 ] && cp target/film/intro.mp4 dist/film/intro.mp4 || echo "ADVARSEL: target/film/intro.mp4 mangler, siden får ingen åpningsfilm"
# Bildene til katalogen i redigeringen (`?edit`, tools/qa/thumbs.py).
mkdir -p dist/catalog
cp assets/catalog/thumbs/*.jpg dist/catalog/ 2>/dev/null || true
# Ikonene på utstyrskortene (inventaret og skriptoriet): eierens malte, ellers
# et rendret bilde av modellen (tools/models/gear_icons.py).
mkdir -p dist/icons
cp assets/textures/icons/*.png dist/icons/
# Lydene (opptak fra frie spill, se assets/sounds/CREDITS.md).
mkdir -p dist/sounds
cp assets/sounds/*.mp3 assets/sounds/sounds.json assets/sounds/CREDITS.md dist/sounds/
# Instrumentprøvene til musikken (FluidR3_GM, se assets/music/CREDITS.md).
mkdir -p dist/music
cp -r assets/music/. dist/music/
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
