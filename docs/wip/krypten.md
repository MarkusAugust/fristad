# Krypten – arbeidslogg (hjelper 3)

Oppgave: steg 1–5 i oppdraget (motor + testnivå grønt, byggesett, nivåskript, nivå 1, docs).

## Status ved start (10. okt)
- `cargo build --release --bins` bygger uten feil eller advarsler med arbeidskopien fra de forrige hjelperne.
- `src/tests.rs` bruker fortsatt den gamle `dungeon`; `src/domain/world/dungeon.rs` ligger igjen, men er ikke modul lenger.
- Navnene i byggesettet (`kit_grav.py`: `kit_lever_handle`, `kit_shortcut_door`, `kit_false_wall`, `kit_gate`) stemmer ikke med motoren (`kit_lever_arm`, `kit_shortcut_leaf`, `kit_secret_wall`, `portcullis`).

## Steg 1 ferdig (motor + testnivå grønt)
- Slettet: `src/domain/world/dungeon.rs`, `tools/models/dungeon.py`, `tools/models/dng_preview.py`, `assets/models/dungeon/` (70 filer), `docs/kjelleren.md`, de gamle generatortestene.
- Nye tester i `src/tests.rs` (seksjonen «Krypten»): `krypten_kan_nas_og_fiendene_biter`, `kryptnivaaene_har_det_de_skal_og_taket_er_hoyt_nok`, `fiendene_finner_fram_i_krypten` (statisk + hver fiende følger deg til start), `fiendene_patruljerer_og_ligger_i_bakhold`, `kryptnivaaene_kan_gas_gjennom` (bot med Rapier, trekker spaker), `gjennomspilling` (ignorert, med kamp og kister), `hvilestedet_vekker_deg_der_du_tente`, `spaken_heiser_gitteret`, `snarveien_apnes_fra_en_side`, `den_falske_veggen_apnes`, `fallemmen_og_stigen`, `kistene_lappene_og_den_varige_styrken`, `pilfella_laerer_deg_forst`, `lyset_folger_rommene`, `kartet_viser_det_du_har_sett` (rom).
- Testhjelpere: `Session::use_exact/feet/lever_pulled`, `Mobs::wake_all/dormant` (cfg(test)).
- `mobs.rs`: oppe i byen står fienden stille og ser fortsatt etter deg (fiendene_ser_mot_deg).
- `web/main.js`: kartet tegner rommene (polygoner) i stedet for cellene; `tools/map-test.mjs` venter «Nivå 1».
- `cargo test --release`: 49 ok, gjennomspilling ok (42 s spilltid). wgsl-check ok.
- Sikkerhetskopi: `wip/krypten` (scratchpad backup.sh: alt unntatt de andre hjelpernes filer).

Neste: steg 2, byggesettet (`kit_grav.py`): navnene må følge motoren (`kit_lever_arm`, `kit_shortcut`/`kit_shortcut_leaf`, `kit_secret_wall`, `kit_trapdoor`, `kit_ladder`, `kit_plate`, `kit_arrow`, `kit_lore`, `kit_bonus`, `kit_crack`, `kit_up`, `kit_down`, `kit_rest`).

## Steg 2 ferdig (byggesettet)
- `tools/crypt/kit_grav.py`: mekanismene har motorens navn og mål: `kit_rest` (flammene med), `kit_lever`/`kit_lever_arm` (dreiepunkt 0,56 m), `kit_gate` (2,8 × 3,0), `kit_shortcut` (karmen, ny) / `kit_shortcut_leaf` (1,6 × 2,3), `kit_secret_wall`, `kit_trapdoor`, `kit_ladder`, `kit_plate`, nye `kit_arrow`, `kit_lore`, `kit_bonus`, `kit_crack`, `kit_up`, `kit_down` (trappa ned med blått lys, lik i alle nivåer). Eksportert til `assets/models/crypt/kit_*.glb` (16 modeller, 2 200 trekanter til sammen).
- Motoren: `PORTCULLIS = "kit_gate"`, `GATE_W 2.8`, `GATE_H 3.0`, `LEVER_PIVOT 0.56`, `REST_FLAME 1.75`; kollidere for `kit_up` (rampe + mur), `kit_down` (brystvern), snarveikarmen, lesepulten og relikvieskrinet.
- Skjermbilder: `shots/krypten/kit_mekanismer.png`, `shots/krypten/kit_moduler.png`.
- Testene grønne (49).

## Steg 3 ferdig, steg 4 påbegynt (stoppet av koordinatoren: figurene prioriteres)
- `tools/crypt/build_level.py 1` (python3 uten Blender: json + `docs/krypten/nav_01.png`; `python3.13 … 1 --export`: glb-bitene). Bygger nivå 1 etter planen med utlegget i `LAYOUTS[1]` (rette ganger, trapper, ekstra rom `sjaktbunn`/`stigebunn` for brønnsjakta og stigen, beinhuset med galleri og trapp langs muren, `galleri` og `beintrapp` som egne rom i json). 47 194 trekanter i 28 biter (`crypt_01_aa` … `crypt_01_bb`), 543 kollidere, 827 nav-polygoner, 44 lys. Nav-polygoner uten forbindelse fjernes (samme kantregel som `overlap` i nav.rs).
- Motoren bruker nå det ekte nivå 1 (build.rs velger `assets/models/crypt/crypt_01.json`; bitene og byggesettet tas inn automatisk av build.rs, ikke via `models!`).
- Motor: fallem med `label: "bolted"` (boltet fra undersida, åpnes av stigen), kollidere for `kit_down` (rampe, bunn, terskel) og `kit_up` (trinn som bokser), fiender går til siden når de møter hverandre (`walk` i mobs.rs).
- Tester: alle 49 grønne på nivå 1, `gjennomspilling` grønn (125 s spilltid, 11 drept, 3 kister). Bot-feil rettet (fremdrift måles fra føttene; tungt hugg slippes).
- Lagnavn: kit_grav.py bruker `iron` og `door_b` (nye navn fra `rydd/gamle`); `gltf.rs` har et midlertidig alias (`new_layer_name`) til `metal`/`door_dark` her. Ved sammenslåingen: kryptlagene må få numre fra 46 og TEX_SCALE utvides; aliaset kan fjernes.
- Første skjermbilde virker: `shots/krypten/start.png` (kapellet, 39 310 trekanter, 6 tegnekall). `shots/krypten/beinhus.png` ok.

## Åpent problem (neste steg)
- `snapshot --level 1 --eye …`: noen kjøringer viser byen i stedet for krypten (stats 450–550 k trekanter), også i kapellet med andre yaw/--frames. Første kjøring (`--eye 1.5,1.7,3.6 --yaw 15 --pitch 2 --frames 4`) virket. Mistanke: noe i de 15 tick-ene (--time 0.5) flytter spilleren (fall/respawn/ascend?), og den siste `Place` setter bare kameraet. Undersøk med en test som kjører `Session` på samme måte (go_to_level(0), place, 15 × update) og skriver posisjonen.

## Neste steg
1. Rett snapshot-problemet over, ta skjermbilder fra start, beinhuset, søylegangen og nedgangen (+ galleriet), mål trekanter/kall (`--stats`).
2. Stemningen: kapellets og alterets kalkmaleri (`fresco`) ser ut som tapet – bytt til `wall` eller bruk det bare i felt; vurder mørkere omgivelseslys nede.
3. Forbedre mekanismene etter eierens forbilder: `assets/textures/props/ref/dng_grav.jpeg` og `dng_mekanikk.jpeg` (hvilestedet, spaken, gitteret, døra).
4. Steg 5: CLAUDE.md (Kjelleren-avsnittet beskriver fortsatt den gamle generatoren) og `docs/ny-alt.md`.
5. Kryptkoden skal tåle at fiendemodellene mangler (rydd/gamle) – sjekk etter sammenslåingen.
