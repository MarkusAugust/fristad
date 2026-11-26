#!/bin/sh
# Bygger godkjenningssiden på maskinen (eieren 26. november): alt med bilder,
# i target/qa/dom/. `node tools/rediger.mjs` viser den på
# http://localhost:1350/dom/, og dommene skrives rett i
# assets/catalog/approvals.json.
#
#   sh tools/dom.sh            bildene av modulene og mekanismene, så siden
#   sh tools/dom.sh --raskt    bare siden (bildene fra forrige gang)
#
# Trenger Blender som Python-modul. Første gang lages target/qa/venv med
# Pythonen i Blender.app (3.13) og `pip install bpy pillow numpy`.
set -e
cd "$(dirname "$0")/.."

if [ ! -x target/qa/venv/bin/python3.13 ]; then
  PY=$(ls /Applications/Blender.app/Contents/Resources/*/python/bin/python3.13 2>/dev/null | head -1)
  [ -n "$PY" ] || PY=$(command -v python3.13) || { echo "Fant ikke Python 3.13 (Blender.app eller python3.13)."; exit 1; }
  mkdir -p target/qa
  "$PY" -m venv target/qa/venv
  target/qa/venv/bin/python -m pip install -q bpy pillow numpy
fi
PATH="$PWD/target/qa/venv/bin:$PATH"
export PATH

if [ "$1" != "--raskt" ]; then
  # Bildene tas av spillet selv, så de følger koden som står nå.
  cargo build --release --bin snapshot
  FRISTAD_UTEN_PALETT=1 python3.13 tools/crypt/module_shots.py
  python3.13 tools/crypt/mechanism_shots.py
fi
python3.13 -m tools.qa approvals lokal
