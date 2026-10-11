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

## 11. oktober (hjelper, grenen `wip/krypten-2`): tetthet, puss og sarkofag

Eieren gikk nivå 1: himmel gjennom veggen i trappa (lyse kiler over hvert trinn), døråpning og buet glugge rett ut i himmelen i rotunden/korsgangen, søm mellom to biter; «pussen ser ut som et barnerom»; «hvorfor er sarkofagen en steinklump?».

### Tetthetssjekken (`tools/crypt/leak_check.py`)
- Fra 2 489 punkter på navigasjonsnettet (midten av hver polygon og et rutenett på 1 m, i øyehøyde 1,6 m) skytes 128 stråler (Fibonacci-kule, også opp og ned) 60 m. Geometrien er nivåets flater (`MB`) pluss byggesettets mekanismer der merkene står (`kit_up`, `kit_down`, gitteret, snarveidøra, den falske veggen, fallemmen …). Flatene rasteres tett i et voksel-rutenett på 8 cm (utvidet ett voksel, så ingen stråle smetter diagonalt); en stråle som går ut av omhyllingen (eller er utenfor rommene etter 60 m) er en lekkasje, og stedet der den sist var inne i et rom er hullet. Hullene grupperes per meter og skrives til `shots/krypten/leaks_NN.txt`. `--glb` sjekker de eksporterte bitene (egen glb-leser i numpy) i stedet for flatene i minnet. Hull under rundt 10 cm ses ikke.
- `build_level.py` kjører sjekken før eksporten og nekter å eksportere et nivå som lekker (`--no-check` hopper over). Ingen Rust-test: 2 500 × 128 stråler mot 50 000 trekanter uten BVH er for tregt i `cargo test`; sjekken ligger i eksporten.
- **Før: 48 lekkende stråler, 39 hull.** Årsakene (alle i felles kode, `Level.hall` i `build_level.py`):
  1. **Murtoppen var en rampe, hvelvet trappet.** Gangmurene gikk opp til gulvet + SPRING (følger trappa som en rampe), mens tønnehvelvet ligger i skiver på 1 m med vederlaget etter det høyeste gulvet i skiva. Mellom rampa og det trappete vederlaget sto det en kile av himmel over hvert trinn (eierens «lyse kiler»). Nå: `hall_spring` gir vederlaget per skive, og muren prøves tett ved skivegrensene så toppen følger trinnene (+3 cm overlapp).
  2. **Ingen lunetter i hjørnefagene.** Krysshvelvet i hjørnene (`groin_bay`) har ingen vegg under buene, og murene gikk bare opp til vederlaget: en halvsirkel av luft over hver lukket side i hvert hjørne (eierens «buede glugge»). Nå går muren opp til buen S + a·√(1 − t²) i fagene.
  3. **Trinnet mellom krysshvelvet og tønna.** `vault_step` lukket bare trinnene mellom skivene innenfor ett strekk, ikke mellom hjørnefaget og første/siste skive når trappa begynner eller slutter der (den rektangulære «døråpningen»). Nå lukkes de også.
  4. **Trappene tangerte murene.** Trinnene var nøyaktig like brede som gangen; nå går de `STAIR_IN` (10 cm) inn i murblokka på begge sider.
- **Etter: 0 lekkende stråler** (flatene i minnet). De eksporterte bitene sjekkes med `--glb` etter eksporten (se under).
- Eksportert på nytt (`build_level.py 1 --export`, 28 biter, 48 185 trekanter) og sjekket igjen med `--glb`: **0 lekkasjer i de eksporterte bitene også.** Bildet fra trappa der hullene var: `shots/krypten/hall_ned.png` (gangen beinhus → nedgang, fra (30, −1.4, 54) mot sør). Flere skjermbilder i `shots/krypten/`: `kapell.png`, `sidegrav_sarkofag.png`, `soylegang_sarkofag.png`, `alter.png`, `beinhus_*.png`, `galleri*.png`.
- Ikke funnet igjen: rotundens «døråpning og glugge rett ut i himmelen» og sømmen mellom to biter. Åtte skjermbilder fra beinhuset (gulvet, galleriet, beintrappa) viser ingen himmel, og sjekken fra alle nav-punktene finner ingen hull; det eieren så, var trolig gangen beinhus → nedgang (rektangelet = det åpne trinnet mellom krysshvelvet og tønna, buen = lunetten) sett fra rotunden. Sømmen er trolig okklusjonsbakingen per bit (`bake_ao` per modell), ikke et hull.

### Pussen og sarkofagen (gjort før beskjeden om å droppe dem; står fordi testene er grønne)
- `fresco` i `kit_grav.py` bruker `plaster_old` med tonen (0.52, 0.55, 0.60), ruhet 0.62, i stedet for `crypt_plaster` (stjernene). Byggesettet eksportert (`kit_grav.py` hopper nå over `kit_rest`, som er bygd fra arket med sheet2model, om det ikke nevnes uttrykkelig; den første eksporten skrev den over, og den ble hentet tilbake fra HEAD). Ny prompt for ekte kryptpuss i `docs/prompter-krypten.md` («Runde 3»), merket som erstatning for `crypt_plaster`.
- Sarkofagen: `sheet2model/examples/fristad/sarcophagus.json` med fire deler, geometri = prior: sokkel (boks 2,0 × 0,16 × 0,9), kiste (boks 1,8 × 0,6 × 0,76), lokk (boks 2,0 × 0,12 × 0,9) og ridderen som liggende, avsmalnende kjegle (`cone_x`, r 0,21 → 0,14, hodet mot −x) sunket i lokket; 1,1 m høy, 196 trekanter, atlas 2048². Fire bygg: boks-ridderen ble en flat plate (steinklump igjen); `tiles: auto` valgte `weatherboard_grey` til de usette flatene (striper), så `crypt_wall` på 2 m er satt; delboksene ble utvidet. Svakhet: store deler av lokket og sokkelen regnes som «usett» (dekket 0,12 for to av delene) og får flisen; årsaken (silhuettboksen per del eller z-bufferen mellom delene) er ikke funnet. Eksportert til `assets/models/crypt/kit_sarcophagus.glb`.
- Nytt merke `prop` i nivåfila (`format.md`): `label` er modellen, motoren setter den ut (`PROPS` i `crypt/mod.rs`, `b.place`), kolliderne og nav-hindringen står i nivåfila som før. `build_level.py`: `Level.prop(...)` i stedet for `K.sarcophagus` på de tre stedene (sidegraven, dypgraven, søylegangen; 10 stykker).

### Status og neste
- Eieren: nivå 1 bygges om i sin helhet etterpå; derfor ikke mer puss/sarkofag nå.
- Tetthetssjekken bør kjøres på hvert nytt nivå (den ligger i eksporten). Grensen 10 cm: skal mindre glipper fanges, sett `VOXEL` 0.05 (rutenettet blir 4× større).
- `leak_check.py` tar bare med mekanismene i `KIT_AT_MARK`; nye merker som dekker åpninger må legges til der.
