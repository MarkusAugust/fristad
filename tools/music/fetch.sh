#!/bin/sh
# Henter instrumentprøvene til musikken (web/score.js) fra FluidR3_GM, slik
# gleitz/midi-js-soundfonts har gjengitt dem som mp3 (se
# assets/music/CREDITS.md). Bare noen få toner per instrument; resten
# spilles med playbackRate. Kjøres med `sh tools/music/fetch.sh`.
set -eu
BASE=https://raw.githubusercontent.com/gleitz/midi-js-soundfonts/gh-pages/FluidR3_GM
OUT=assets/music
# instrument (mappa i FluidR3_GM) og tonene som hentes. Holdes i takt med
# INSTRUMENTS i web/score.js.
while read -r inst notes; do
  [ -z "$inst" ] && continue
  mkdir -p "$OUT/$inst"
  for n in $notes; do
    [ -s "$OUT/$inst/$n.mp3" ] || curl -fsSL -o "$OUT/$inst/$n.mp3" "$BASE/$inst-mp3/$n.mp3"
  done
done <<'LIST'
recorder Eb4 Gb4 A4 C5 Eb5 Gb5 A5 C6 Eb6
shanai C4 Eb4 Gb4 A4 C5 Eb5 Gb5 A5 C6
fiddle G3 Bb3 Db4 E4 G4 Bb4 Db5 E5 G5 Bb5
acoustic_guitar_nylon C2 E2 Ab2 C3 E3 Ab3 C4 E4 Ab4 C5
orchestral_harp C2 E2 Ab2 C3 E3 Ab3 C4 E4 Ab4 C5 E5 Ab5
dulcimer C4 E4 Ab4 C5 E5 Ab5
choir_aahs C3 Eb3 Gb3 A3 C4 Eb4 Gb4 A4 C5
church_organ C2 E2 Ab2 C3 E3 Ab3 C4 E4 Ab4
cello C2 E2 Ab2 C3 E3 Ab3
contrabass A1 Db2 F2
bagpipe C3 E3 Ab3 C4 E4 Ab4
tubular_bells C4 E4 Ab4
timpani D2 Gb2 Bb2
taiko_drum C3 C4
LIST
du -sh "$OUT"
