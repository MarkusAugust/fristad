#!/bin/sh
# Pakker modellene i assets/models/ med gzip til <ut>/models/<sti>.gz, med
# størrelsene i <ut>/models/sizes.json (framdriftsstolpen i main.js).
# Brukes av tools/build-web.sh og tools/pull-dist.sh.
set -eu
OUT="$1/models"
rm -rf "$OUT"
(cd assets/models && find . -name "*.glb" | sed 's|^\./||') | while read -r f; do
  mkdir -p "$OUT/$(dirname "$f")"
  gzip -9 -n -c "assets/models/$f" > "$OUT/$f.gz"
done
(cd "$OUT" && find . -name "*.gz" | sed 's|^\./||' | sort | while read -r f; do
  printf '"%s":%s\n' "${f%.gz}" "$(wc -c < "$f")"
done | paste -sd, - | sed 's/^/{/; s/$/}/') > "$OUT/sizes.json"
du -sh "$OUT"
