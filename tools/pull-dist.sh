#!/bin/sh
# Henter den ferdige nettsiden fra grenen web-dist til dist-web/, så den kan
# kjøres lokalt eller i Chromium (node tools/browser-test.mjs dist-web).
set -e
cd "$(dirname "$0")/.."
git fetch -q origin web-dist
rm -rf dist-web
mkdir -p dist-web
git archive origin/web-dist | tar -x -C dist-web
if [ -f assets/external/manifest.json ]; then
  mkdir -p dist-web/assets
  cp -r assets/external/. dist-web/assets/
fi
# Modellene ligger ikke på grenen; de pakkes fra repoet.
sh tools/pack-models.sh dist-web
# Manifestet for modellene som hentes etter start (som i build-web.sh).
cargo run -q --bin manifest -- dist-web/models/manifest.json
gzip -9 -n -f dist-web/models/manifest.json
echo "dist-web/ er klar ($(git log -1 --format=%s origin/web-dist))"
