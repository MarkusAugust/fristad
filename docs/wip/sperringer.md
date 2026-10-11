# Sperringene, Gorveks plass, kistene og portalen (11. oktober)

Fire feil fra eieren, gren `claude/detaljnivaa` (HEAD 8bc4d83), sikkerhetskopi på `wip/sperringer`.

1. Gorveks øvingsplass var sperret: `enclose` satte den usynlige veggen langs Storgata også forbi plassen
   (`plan.free` med radius 5 · 0,6 = 3 m nådde aldri ut til gatelinja ved x ≈ 2,8).
2. Gjerdene var korte stykker (annenhver meter) langs en usynlig vegg som gikk hele veien.
3. Kista (`chest`/`chest_lid` fra sheet2model) var massiv.
4. Portalmodellene var slettet i ryddingen.

## Logg

- 14:20 Lest reglene og koden. Oversiktsbilde før: `shots/sperringer/over_before.png`.
- 14:50 Steg 1 og 2 gjort i `town.rs`: `enclose` bygger sperrelinjene som sammenhengende gjerder (`fence`, moduler
  på 2 m stokket så de når veggen, kollider `FENCE_SOLID_H` 1,1 m høy akkurat der gjerdet står, tverrgjerde fra
  linja inn til husveggen der huset står et stykke bak), variant per linje (`fence_kind`: stakitt, steinmur, litt
  flettverk), skigard rundt Gorveks plass med 3 m åpning mot gata (`YARD_GAP`), steinmur rundt kirkegården,
  gårdene flettverk/skigard annenhver. Nye modeller i `street.py`: `rail_fence`, `picket_fence`, `stone_fence`.
  Testene `man_kan_ga_til_gorvek` og `sperringene_er_synlige` grønne. Bilder: `shots/sperringer/over_after.png`,
  `yard_after.png`, `fence_row.png`.
- 14:50 Steg 3: `shapes.py` `chest`-prioren er hul (`shell`: bmesh solidify innover, `opt.wall` 3 cm); bygd med
  `python3.13 -m sheet2model chest` (IoU 0,79 i qbr som før) og kopiert til `assets/models/town/`.
- 14:50 Steg 4: `tools/models/portal.py` ny (malt stil, `layer_wall_stone`/`ruin`/`flagstone`, `magic…`-materialer),
  modellene bygd og lagt i `models!`. Gjenstår: `snapshot --portal`, `snapshot --chest`, hele `cargo test`.
- 15:00 `snapshot --portal --time 5 --eye -2.5,+1.7,5.5 --yaw 90` viser portalen (`shots/sperringer/portal.png`), `--chest`
  den åpne kista (`chest.png`, mørk innside). `tools/stats.sh` (`shots/sperringer/stats`): Sørporten 631 k trekanter /
  145 kall, Storgata 685 k / 135, Torget 638 k / 141; gjerdene er 54 moduler på 89 m, så det meste er husene.
  Stakittet slanket 2 200 → 660 trekanter (ingen fas på bordene), steinmuren 1 664 → 768.
- Hele `cargo test --release`: alt grønt unntatt `alt_er_ekte_modeller_med_teksturer`, som feiler på
  `apothecary_interior` (288 hjørner uten albedo) – den modellen er endret av den andre hjelperen (`workshops.py`),
  ikke av denne jobben.
- 15:25 Dekkhellene på steinmuren i `wall_stone` (så de ikke så ut som planker). Etter slankingen: Sørporten 609 k / 145,
  Storgata 637 k / 135, Torget 590 k / 141, Vertshuset 475 k / 115. Hele `cargo test --release`: 47 grønne, 4 ignorert.
  `cargo build --release --bins` grønn. Ikke committet på `claude/detaljnivaa`; alt ligger på `wip/sperringer`.

## Gjenstår / merknader

- Lyktestolpen på Storgata står midt i åpningen til Gorveks plass (ikke i veien, åpningen er 3 m, men den kunne flyttes).
- CLAUDE.md-avsnittene «Byen» (enclose/gjerdene, kirkegårdens steinmur) og «Ned og opp» (portalen er tilbake) bør
  oppdateres av den som slår sammen.
- Kista: innsiden får plankeflisen fra `tiles: "auto"` (ingen rute ser den), tonet mørk; vil man ha lær, settes
  `opt.tiles` i `chest.json`.
