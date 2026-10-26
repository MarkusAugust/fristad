#!/bin/sh
# LO-TEK (før Fristil) i spillet: bygger pakken @lo-tek/interfaces fra en
# klone av github.com/MarkusAugust/lo-tek og legger det spillet bruker i
# vendor/lo-tek/: stilarket (dist/lotek.css), web-komponentene og byggerne
# som ES-moduler (dist/**/*.js, uten kart og typer), tokenene
# (src/tokens/tokens.css) og en package.json, så `bun install` tar den inn
# som `@lo-tek/interfaces` (package.json: "file:./vendor/lo-tek"). Pakken er
# ikke publisert på npm ennå (bare en plassholder, 0.0.0); når den er det,
# byttes avhengigheten tilbake til en versjon derfra og mappa slettes.
#
#   sh tools/vendor-lotek.sh [sti-til-lo-tek-klone]
set -e
cd "$(dirname "$0")/.."
SRC=${1:-../markusaugust/lo-tek}
OUT=vendor/lo-tek
(cd "$SRC" && bun install >/dev/null && cd interfaces && bun run build:browser >/dev/null)
rm -rf "$OUT"
mkdir -p "$OUT/dist" "$OUT/src/tokens"
cp "$SRC/interfaces/dist/lotek.css" "$OUT/dist/"
(cd "$SRC/interfaces/dist" && find . -name "*.js" ! -name "cli.js" ! -name "react.js" ! -path "./jsx/*") | while read -r f; do
  mkdir -p "$OUT/dist/$(dirname "$f")"
  cp "$SRC/interfaces/dist/$f" "$OUT/dist/$f"
done
cp "$SRC/interfaces/src/tokens/tokens.css" "$OUT/src/tokens/"
cp "$SRC/interfaces/LICENSE" "$OUT/"
VERSION=$(sed -n 's/^  "version": "\(.*\)",/\1/p' "$SRC/interfaces/package.json")
COMMIT=$(git -C "$SRC" rev-parse --short HEAD)
cat > "$OUT/package.json" <<JSON
{
  "name": "@lo-tek/interfaces",
  "version": "$VERSION",
  "description": "LO-TEK bygd fra github.com/MarkusAugust/lo-tek $COMMIT (tools/vendor-lotek.sh)",
  "license": "MIT",
  "type": "module",
  "main": "./dist/index.js",
  "exports": {
    ".": "./dist/index.js",
    "./lotek.css": "./dist/lotek.css",
    "./tokens.css": "./src/tokens/tokens.css",
    "./*": "./*"
  }
}
JSON
echo "vendor/lo-tek: @lo-tek/interfaces $VERSION ($COMMIT), $(du -sh "$OUT" | cut -f1)"
