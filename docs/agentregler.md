# Regler for hjelpere som jobber parallelt i skymaskinen

Maskinen har 4 kjerner, 15 GB minne og rundt 12 GB ledig disk. Flere hjelpere deler den. Disse reglene hindrer at de krasjer eller stopper hverandre.

## Tunge kommandoer

- **Cargo bare én om gangen, med lås og to jobber:**

  `flock /tmp/fristad-cargo.lock nice -n 10 cargo build --release --bins -j 2`

  Det samme gjelder `cargo test --release -j 2`. Kjør aldri cargo uten låsen.
- **Snapshot og Blender-render med lås:**

  `flock /tmp/fristad-render.lock …`

  Render høyst 1280 × 720 i spillet. I Blender bruker du Workbench eller EEVEE, eller Cycles med få samples (32) og høyst 1024 px.
- **Tidsgrense på alt som kan henge:** `timeout 900 …`. Ingen kommando skal gå lenger enn 15 minutter. Del opp heller.
- **Ikke start flere Blender-prosesser samtidig.**

## Konteksten din

- **Se aldri på store bilder direkte.** Skaler ned til høyst 1024 px på den lengste siden før du åpner dem (`python3 -c "from PIL import Image; …"`). Se på høyst fire bilder per steg, og helst ett kontaktark i stedet for mange enkeltbilder.
- **Les ikke hele store filer.** Bruk `grep` og les bare de linjene du trenger. `session.rs`, `town.rs`, `people.py` og `dungeon.py` er på 1 000–2 000 linjer.
- **Skriv ikke ut store mengder i terminalen.** Bruk `| tail -20` eller `| head`, og `grep -E "^error|warning: unused|test result|FAILED|panicked"` på cargo.

## Lagre underveis

- **Skriv en logg i `docs/wip/<oppgave>.md`** etter hvert steg: hva som er gjort, hva som virker, hva som gjenstår og neste steg. Om du blir stoppet, kan en ny hjelper fortsette derfra.
- **Ta sikkerhetskopi av arbeidet ditt etter hvert ferdige steg** til grenen `wip/<oppgave>`, uten å røre den du står på. Push aldri til `claude/detaljnivaa` eller `master`.

  ```
  export GIT_INDEX_FILE=$(mktemp -u)
  git read-tree HEAD
  git add <dine filer>
  T=$(git write-tree)
  C=$(git commit-tree $T -p HEAD -m "WIP: …")
  rm -f $GIT_INDEX_FILE
  unset GIT_INDEX_FILE
  git push -q origin $C:refs/heads/wip/<oppgave>
  ```

- **Disk:** slett midlertidige render og eksporter du ikke trenger. Legg skjermbilder i `shots/<oppgave>/`, i lav oppløsning når det holder.

## Andres filer

Hver hjelper får beskjed om hvilke filer som er dens. Rør ikke andres filer. Trenger du en endring i en felles fil (`gltf.rs`, `geometry.rs`, `CLAUDE.md`), gjør den liten og legg den til nederst eller på ett sted, så den ikke kolliderer.
