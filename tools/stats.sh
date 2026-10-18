#!/bin/sh
# Trekanter, tegnekall og instanser fra faste kameraposisjoner, til
# før/etter-tabellene når noe i rendereren endres.
#
#   sh tools/stats.sh [snapshot-binær] [mappe for bildene]
#
# Skriver én linje per posisjon: alle pass (trekanter, kall) og hovedpasset
# (trekanter, kall, instanser). Bildene havner i mappa, så nærbildene kan
# sammenlignes. Skyen tegner med programvare, så tiden sier ingenting.
BIN=${1:-./target/release/snapshot}
OUT=${2:-shots/stats}
mkdir -p "$OUT"
printf "%-12s %12s %8s %12s %7s %9s\n" posisjon "trek. alle" "kall" "trek. hoved" "kall" instanser
while read -r name args; do
    [ -z "$name" ] && continue
    # shellcheck disable=SC2086
    line=$("$BIN" --out "$OUT/$name.png" --size 960x540 --frames 3 --stats $args 2>/dev/null | grep '^STATS')
    set -- $line
    printf "%-12s %12s %8s %12s %7s %9s\n" "$name" "$2" "$4" "$3" "$5" "$6"
done <<EOF
sorporten   --eye 4.7,+1.7,50 --yaw 0
storgata    --eye 4.7,+1.7,28 --yaw 0
torget      --eye 2,+1.7,9 --yaw 0
vertshuset  --eye -3.5,+1.7,4 --yaw 180
borghaugen  --eye 0,+1.7,-110 --yaw 180 --pitch -6
utenfor     --eye 4,+1.7,78 --yaw 0
oversikt    --eye 0,+50,90 --yaw 0 --pitch -25
kjelleren   --descend
EOF
