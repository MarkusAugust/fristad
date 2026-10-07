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
echo "dist-web/ er klar ($(git log -1 --format=%s origin/web-dist))"
