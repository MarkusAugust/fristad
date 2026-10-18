#!/bin/sh
# Legger den ferdige åpningsfilmen på grenen `film` (bare fila, uten historikk),
# som Actions henter i stedet for å klippe filmen selv. Grenen skrives over hver
# gang, så repoet vokser ikke med en ny film for hver versjon.
#
#   python3 tools/film/cut.py            # klipper target/film/intro.mp4
#   sh tools/film/publish.sh [FIL]       # legger den på grenen film
set -e
src="${1:-target/film/intro.mp4}"
[ -f "$src" ] || { echo "Finner ikke $src. Klipp filmen først: python3 tools/film/cut.py"; exit 1; }
root="$(git rev-parse --show-toplevel)"
src="$(cd "$(dirname "$src")" && pwd)/$(basename "$src")"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cd "$tmp"
git init -q
cp "$src" intro.mp4
git add intro.mp4
git -c user.name="${GIT_AUTHOR_NAME:-$(git -C "$root" config user.name)}" \
    -c user.email="${GIT_AUTHOR_EMAIL:-$(git -C "$root" config user.email)}" \
    commit -qm "Åpningsfilmen klippet $(date -u +%F) fra $(git -C "$root" rev-parse --short HEAD)"
git push -qf "$(git -C "$root" remote get-url origin)" HEAD:refs/heads/film
echo "Filmen ($(du -h intro.mp4 | cut -f1)) ligger på grenen film."
