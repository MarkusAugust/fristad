#!/bin/sh
# Pakker modellene i assets/models/ med gzip til <ut>/models/<sti>.gz, med
# størrelsene i <ut>/models/sizes.json (framdriftsstolpen i main.js).
# Bitene til hvert kryptnivå (crypt/crypt_NN_aa.glb) hentes først når økta
# ber om nivået (plan del 10a), og ligger i en egen mappe per nivå,
# <ut>/models/crypt/NN/crypt_NN_aa.glb.gz (`gltf::dist_path`); i sizes.json
# står de under "levels" etter nivået, resten etter stien.
# Brukes av tools/build-web.sh og tools/pull-dist.sh.
set -eu
OUT="$1/models"
rm -rf "$OUT"
dist() {
  case "$1" in
    crypt/crypt_[0-9][0-9]_*) n=$(echo "$1" | cut -c13-14); echo "crypt/$n/${1#crypt/}" ;;
    *) echo "$1" ;;
  esac
}
mkdir -p "$OUT"
LIST="$OUT/.list"
(cd assets/models && find . -name "*.glb" | sed 's|^\./||' | sort) > "$LIST"
while read -r f; do
  d=$(dist "$f")
  mkdir -p "$OUT/$(dirname "$d")"
  gzip -9 -n -c "assets/models/$f" > "$OUT/$d.gz"
done < "$LIST"
{
  printf '{'
  first=1
  while read -r f; do
    case "$f" in crypt/crypt_[0-9][0-9]_*) continue ;; esac
    [ $first = 1 ] || printf ','
    first=0
    printf '"%s":%s' "$f" "$(wc -c < "$OUT/$f.gz")"
  done < "$LIST"
  printf ',"levels":{'
  last=""
  while read -r f; do
    case "$f" in crypt/crypt_[0-9][0-9]_*) ;; *) continue ;; esac
    n=$(echo "$f" | cut -c13-14)
    if [ "$n" != "$last" ]; then
      [ -z "$last" ] || printf '},'
      printf '"%d":{' "$((1$n - 100))"
      last=$n
    else
      printf ','
    fi
    printf '"%s":%s' "$f" "$(wc -c < "$OUT/$(dist "$f").gz")"
  done < "$LIST"
  [ -z "$last" ] || printf '}'
  printf '}}\n'
} > "$OUT/sizes.json"
rm -f "$LIST"
du -sh "$OUT"
