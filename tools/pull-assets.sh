#!/bin/sh
# Henter ressursene Actions la på grenen external-assets til assets/external/,
# for lokale skjermbilder der Poly Haven ikke kan nås (Claude sitt skymiljø).
set -eu
git fetch origin external-assets
rm -rf assets/external
mkdir -p assets/external
git archive origin/external-assets | tar -x -C assets/external
echo "assets/external: $(du -sh assets/external | cut -f1)"
