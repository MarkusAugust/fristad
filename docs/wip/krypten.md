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

## 15.–16. oktober (natt, hjelper): nivå 2, Gravkongens hall

Bygd etter planen `tools/crypt/levels/level_02.py` med utlegget i `tools/crypt/layouts/layout_02.py` (nytt: utleggene for nivå 2 og videre står i egne moduler, `build_level.load_layout`; nivå 1 står fortsatt i `build_level.py`). Nivå 1 bygges bit for bit likt (flatene og `crypt_01.json` er identiske før og etter; sjekket med en hash av alle flatene).

### Generatoren (`tools/crypt/build_level.py`), felles for alle nivåer
- **Mangekantrom** (`Room.kind == "poly"`, `poly_room`): mur langs hver kant med åpningene (gangene finner kanten de treffer), sokkel, kollidere og et **klosterhvelv** (en hvelvkappe fra hver kant inn mot midten, ribber langs gratene på konsoller, sluttstein).
- **Runde rom** (`round_room`, var `beinhus`): galleri rundt med vindeltrapp langs muren er valgfritt per rom (`ROUND[id]`: `gallery`, `torches`, `shelves`, `brazier`, `wall_cell`, `dome_seg` …); uten galleri er det gulv, mur og kuppel (arenaen). Galleriet og trappa blir egne rom i json med navn fra utlegget. Murkollider under en dør høyere oppe (galleriet).
- **Ganger fra et galleri eller en balkong**: `ya`/`yb` i gangens valg; navigasjonen i gangen trekker fra flaten der oppe (`Level.decks`), ikke gulvet.
- **Flater over gulvet** (`Level.decks`): galleriene, balkongene, broene, toppen av kista; det som står på dem (spaker, kister) holdes fritt.
- **Byggeklosser**: `terrace` (balkong langs muren med front og rekkverk, åpning der broene kommer inn), `core_stair` (vindeltrapp rundt en murt kjerne midt i en sal, ring rundt kjernen oppe), `stone_bridge` (murt bro med rekkverk og flat bue), `shaft` (sjakt under en fallem eller stige), `wall_torch`.
- **Avgrunn**: et rektangelrom med `floor_y` langt under gulvhøyden (murene helt ned, steinur på bunnen) og `nowalk`; broa over er en mekanisme (under).
- **Nisjer hogd ut av muren**: `wall_holes` per side i rektangelrommene (nivå 1 hogger dem ikke ut; der ligger nisjene bak muren og synes ikke, se under).
- Utlegget kan justere et rom fra planen (`override`: sekskanten dreid 30° så en kant vender mot gangen, høyere tak i tårnnisjen).
- Trekantbudsjett: `hall_vault_n` (ledd i hvelvbuen i gangene, 8 i nivå 2), `hall_flat_step` (skivene i hvelvet der gangen er flat, 2 m), `nav_grid` (navigasjonen deles i ruter på 4 m før den deles i konvekse biter, så det ikke blir lange flak rundt runde hindringer). Døråpningene i rommene bruker samme antall ledd i buen som hvelvet i gangen (ellers sto det smale kiler av himmel mellom dem, `--leak-scan`).
- Innholdsfila `assets/layout/crypt_NN.json` skrives i fast form av `build_level.py` selv (`fixed_form`, samme form som `crypt_json` i `src/adapters/data.rs`; sjekket mot nivå 1), så `cargo run --bin layout` (som skriver byen også) ikke trengs.

### Motoren (det som trengtes for nivå 2)
- **Broa** (`bridge`-merket: `size`, `height` (så høyt den henger), `top`): `Crypt.bridges`, `Mechanisms.bridges` (senkes på 2,2 s når bryteren åpnes, med et lite dump), kollider som følger dekket (`Session.bridge_solids`), modellene `kit_bridge` (eikedekke på bjelker, jernøyer) og `kit_chain` (12 m kjetting fra øyet og opp gjennom hvelvet) i `kit_grav.py`. Navigasjonen over broa er stengt til den er nede.
- **To spaker til én bryter**: en bryter som står i `opens` på flere spaker, åpnes først når alle er trukket (`Mechanisms::levers_ready`), med egen beskjed.
- **Bossen i et tema** (`boss`-merket med hvilket slag som helst): `Mob.boss` (stolpen i HUD-en, brølet, de tunge skrittene, slaget i bakken), og når den faller: `run.slain` (kommer ikke igjen), eliteskatten i temaet og «… er beseiret». Rammeverket er som før.
- **Arenaporten** (`gate` med `arena`): står oppe, slår igjen bak deg når du går inn i arenaen mens bossen lever, og går opp når den faller; med `label: "exit"` er det porten ut, stengt så lenge bossen lever (`Mechanisms::arena_gates`, `Session::update_crypt`).
- **Navigasjonen på trapper**: `Nav::nearest` belønnet polygoner litt over føttene (negativ høydeforskjell), så på en vindeltrapp vant trinnet tre trinn opp, og fienden gikk fram og tilbake; nå teller høyden som avstand. Startpunktet flyttes 3 cm inn i polygonen.
- **Fiendene ved indre hjørner** (`mob_walk.rs`, flyttet ut av `mobs.rs`): går helt fram til hjørnet når neste strekk ikke er åpent derfra, går ikke forbi hjørnet i ett steg, sideforsøkene må gi minst en firedel av steget, og ellers glir fienden langs kanten av nettet (`snap`).
- `crypt/parts.rs`: dataene for mekanismene flyttet ut av `crypt/mod.rs` (lengdetaket).
- Testboten (`src/tests.rs`): trappa ned tar deg videre alt 1,2 m fra merket; er boten i nivået under, er den framme.
- `snapshot --level N --shots FIL`: linjene «UT X,Y,Z YAW [PITCH]» i nivåets eget rom.

## Nivå 3, Kloakken (samme natt)

Utlegget `tools/crypt/layouts/layout_03.py`. Vannstanden er det nye:
- **Vannet** (`water` og `water_block` i `format.md`): `Crypt.waters`, `Mechanisms.waters` (synker på 5 s når bryteren åpnes), `kit_water` (flate på 1 × 1 m skalert til kanalen og rotunden), og de usynlige murene der gangene og trappa går ned i vannet (`Session.water_solids`, flyttes bort når vannet har sunket). Sluseratten er en spak som åpner `vann_1`.
- Navigasjonen under vannet (bunnen av kanalen og rotunden, trappa ned i rotunden, stikkrenna, gangene til rottereiret og silkammeret) er stengt av `vann_1` (`Room.gate`, gangvalget `water`, `stair_gate` i galleriet).
- Gangbanene i kanalen er hyller på konsoller (`terrace(..., solid=False)`), og bunnen ligger 3,4 m under dem, så gangene ut fra bunnen går inn under hyllene, og det er takhøyde der.
- Rotunden er et rundt rom med ringgang (galleriet på 0 m) og trapp ned til bunnen (−3,6 m); sluket i bunnen er en sjakt ned til `rorbunn`, videre til underkummen og stigen opp i utløpet.
- Eliten (en select) i silkammeret bærer nøkkelen til porten ned.
- Rettet i felles kode: **skjøten i murene i runde rom** (u = 0) lå midt i en dør mot øst, så døra bare ble halvveis hogd ut, og murkolliderne sto i den (`round_room` legger skjøten bort fra dørene). Det gjaldt også den falske veggen øverst i gravtårnet i nivå 2. Muren under en dør høyere oppe (galleriet) stopper 30 cm under terskelen.
- Testboten (`src/tests.rs`): uten fiender er eliten borte, så boten får nøkkelen i lomma (`Session::give_key`); i `fiendene_finner_fram_i_krypten` står spilleren på gulvet, ikke på en rett linje mellom to hjørner langt fra hverandre (lange ganger med trapp).
- Ikke gjort (står i planen): flodluka og giftgassen (fellene `flood` og `gas` setter bare ut trykkplata; bare piler virker i motoren), select som trekker deg under vann, toast som dukker opp av vannet, grønne lykter som tennes når vannet synker, fossen som spyr vann (bare en vannflate), og egne teksturer for kloakken (nivået bruker gravkjellerens stein).

## Nivå 4, Den store cisternen (samme natt)

Utlegget `tools/crypt/layouts/layout_04.py`. Søylesalen (64 × 42 m, bunnen −10 m, vannet på −8 m til ventilen er stengt, så −9,5 m), nordgalleriet med tre buer ut mot salen (to med rekkverk, den østre uten: der faller du i vannet), trappa langs østmuren ned til gangbroene (−4 m), øya med hvilestedet og glørfatet (landemerket), akvedukten med vannrenna, den nedre gangbroa langs sørmuren (−8,5 m, under vann til ventilen er stengt), søylelunden, filterkammeret, plattformsalen med Tabellen (bossen; arenaportene som i nivå 2) og nedgangen. Hemmelighetene: den hule søylen (sjakt ned til søylefoten med Fieldset og stige opp til trappefoten), den falske veggen i akvedukten (tegningene, + liv) og kanalmunningen (porten går opp med ventilen; kiste og + utholdenhet).

Nytt i generatoren (felles):
- **Rom inne i et annet rom** (`virtual`: øya og den nedre gangbroa står i salen): de bygges av utlegget, navigasjonen er flaten utlegget gir (`floor_walk`), og dørene deres hogges i vertsrommets mur.
- `block` (murt blokk fra bunnen: reposene, øya, gangbroa, plattformene), `wall_stair` (rett trapp langs en mur, murt ned til bunnen, rekkverk på den åpne sida, navigasjon som skråplan), `poly_vault_y` (høyden på klosterhvelvet, så søylene i søylelunden når hvelvet).
- Gangvalg: `id` (flere ganger mellom de samme rommene), `rail_b` (rekkverk tvers over enden, der gangen ender i lufta i salen) og `open_b` (ingen rekkverk; navigasjonen stopper før kanten).
- `cells` per rom (ruter på 2 m i murene, gulvet og hvelvet i den store salen), og en advarsel når trappa ned, en spak eller hvilestedet ikke står ved nettet som kan nås fra start.
- Tetthetssjekken (`leak_check.py`) regner rommet opp til toppen av hvelvet (4,6 m over vederlaget): en stråle som ender oppe i hvelvet i en stor sal etter 60 m, er ikke et hull.

Testene: `kryptnivaaene_kan_gas_gjennom` går alle fire nivåene (spakene, broa, vannet, nøkkelen); `gjennomspilling` (med kamp) går alle fire på rundt 680 s spilltid, slår Modalen og Tabellen. `fiendene_finner_fram_i_krypten` bruker gulvet nærmest høyden på veien (`floor_near`), ikke en rett linje mellom hjørnene.

### Status etter natta
- Nivå 1–4 kan spilles gjennom. Trekanter (flatene før eksport): nivå 2 70 000, nivå 3 64 000, nivå 4 58 000 (budsjettet er rundt 50 000; de er større enn nivå 1).
- Motorens sprekkesjekk (`snapshot --leak-scan`, hvert tredje punkt fra `leak_points.py N 6`): nivå 3 og 2 har enkeltpiksler (1–7 px) langs gulvkanter (T-skjøter i rutene), ingen åpninger. Python-sjekken (`leak_check.py`) finner 0 hull i alle fire.
- Ikke gjort: fellene `flood` og `gas` (bare piler virker), kullpannene som tennes av spakene i nivå 2, kjettingheisen i nivå 4 er en stige (heis finnes ikke i motoren), de flytende plattformene i plattformsalen er faste sokler, Tabellen deler seg ikke, «pagination» finnes ikke som fiende (knapper i stedet), og egne teksturer for kloakken og cisternen.
- Nivå 1 har nisjer i gravgangen som ikke er hogd ut av muren (de ligger bak muren og synes ikke); nivå 2 og 3 hogger dem ut (`wall_holes`). Ikke rettet i nivå 1, siden det skal bygges likt.

## 17. oktober (hjelper): nivå 5, Gruva

Utlegget `tools/crypt/layouts/layout_05.py` etter planen `levels/level_05.py`. Første nivå i temaet «Gruva og grottene», og det første som går ned i etapper: gruveinngangen (trappa opp til cisternen) → lavehuset (hvilestedet, lampetavla) → hovedstollen (tømmerstiver hver andre meter, skinner i gulvet) → malmhallen (stor hall med steinpilarer, lasteplattform med malmhaug, vogner på sidespor) → den gamle stollen (raset, slideren på skinnene, det morkne lokket) → vinsjhuset med **heisen** → sjaktbunnen og heisbunnen 14 m under → løkka heisbunnen → krystallåra (blå krystaller) → smeltehula (rund, med den glødende smelteovnen og formannen) → vognbanen → heisbunnen. Gruveporten (låst; formannen bærer nøkkelen) i sør leder til nedgangen, og snarveien er **luftesjakta**: en 98 m lang, slak trapp (20 m opp) fra gruveporten til lavehuset, med slåen nede. Hemmelighetene: tømmerveggen i hovedstollen (regnskapet, + liv), det morkne lokket i den gamle stollen ned i krystallkammeret (Tag) med en sprekk videre ned til krystallåra, og kruttkammeret bak bordveggen i vognbanen (kiste, + utholdenhet). 62 000 trekanter i 39 biter, 0 hull i tetthetssjekken; motorens sprekkesjekk (`snapshot --leak-scan`, hvert tredje punkt) finner 18 enkeltpiksler (1 px, T-skjøter i gulvrutene i heisbunnen og krystallåra, som i nivå 2 og 3), ingen åpninger. Rommene fra planen er justert (`override`) så gangene treffer rette kanter, og gangene inn i den runde smeltehula går radielt (skrå innganger i en rund mur ga glipper ved døra: `leak_check`).

### Heisen (nytt i motoren)
- Merket `lift` (`format.md`): `pos` er dekket oppe, `bottom` punktet nede, `size` dekket. `Crypt.lifts` (`parts::Lift`), `Mechanisms.lifts` (hvor langt ned, og målet), `tick_lifts` (2,2 m/s, `LIFT_SPEED`), `Moved::Lift`. Bremsen er bryteren: spaken med heisens id i `opens` løsner den (`open_switch`), så kjører styrestangen på dekket (`Use::Lift`, siktemålet følger dekket) heisen til den andre enden, og vinsjen oppe (`Use::Winch`) henter den opp når den står nede. Dekket er en bevegelig kollider (`Session.lift_solids`), og økta flytter deg med dekket når du står på det (kapselen følger ikke en kollider som flyttes). Modellene `kit_lift` (dekket; posert) og `kit_winch` (heistårnet med trinse og vinsj; står fast, stolpene får kollider), kjettingene er `kit_chain` strukket opp til bommen (`LIFT_HEAD`). Fiendene tar ikke heisen: sjakta er ikke med i navigasjonsnettet (nivået henger sammen for dem gjennom snarveien, som `reachable` og `fiendene_finner_fram_i_krypten` krever).
- Testboten (`Bot::ride_lift`): etter spaken går den til den åpne sida av dekket, sveiver heisen opp om den står nede, går om bord uten nettet (`stride`), drar i styrestangen og venter til den står. `lift_state` i `testing.rs`.

### Gruvas byggesett (`tools/crypt/kit_mine.py`, eksporteres av `kit_grav.py`)
`kit_strut` (tømmerstiver), `kit_rail` (skinner, 2 m), `kit_cart` (malmvogn med malm), `kit_lamp` (gruvelampe med glødende glass, `fire_lamp`), `kit_crystal` (klynge av blå krystaller, `magic_crystal`), `kit_lift` og `kit_winch`. De fem første settes ut som `prop`-merker (`PROPS`); kolliderne og nav-hindringene legges i utlegget (`struts`, `rails`, `cart`, `crystal`). Nye materialer i `kit_grav.MATS`: `rock`, `fire_lamp`, `magic_crystal`.

### Byttet ut og ikke gjort
- Planens nye fiender finnes ikke: **spinner → breadcrumb** (kryper fram), **progress → accordion** (tung, lader), **slider → avatar** (ruller som et hjul, passer på skinnene). Formannen er en accordion-elite med nøkkelen.
- **Fellene**: steinraset står som `trap` med `rocks` (bare trykkplata settes ut; raset finnes ikke i motoren), og den løpske vogna er en pilfelle (`arrows`) med den lastede vogna stående ved. Begge varsles ikke med lyd ennå.
- Lasteplattformen i malmhallen er en lav sokkel (0,5 m) med malmhaug, ikke en flate å gå på med rampe.
- Det morkne lokket er en vanlig fallem (`kit_trapdoor`); ingen egen råtten lem.
- Egne teksturer for gruva (berg, tømmer i stollene) mangler: nivået bruker kryptens stein med `wall_dark`.
- `kryptnivaaene_kan_gas_gjennom` tar heisen ned; gjennomspillingen med kamp er ikke kjørt for nivå 5.
- Rettet i felles kode: **tåka i krypten** var bare skalert ned med `cellar_t`, ikke null, og 200 m nede er høydeleddet i tåka flere hundre ganger, så nivå 5 var helt borte i tåka i `snapshot` (som tar bildet før øynene er vant til mørket); nå er tettheten null nede (`view.rs`). `snapshot --level N` trenger `--time 3` for at lyset skal være kommet på plass. Gangene inn i et rundt rom må gå radielt og ende 0,4 m innenfor sirkelen; en skrå inngang i den runde muren gir glipper ved døra. Spillet lar deg ikke sikte på et punkt lenger inn enn 2,5 m fra nettet (`Nav::nearest`), så en nøkkelport må stå minst 1,5 m inn i gangen (ellers tror testboten at punktet bak porten kan nås).

## 17. oktober (hjelper): nivå 6, Krystallgrotta

Utlegget `tools/crypt/layouts/layout_06.py` etter planen `levels/level_06.py`. Bunnen av temaet «Gruva og grottene»: stollmunningen (de siste stiverne fra gruva, trappa opp) → krystallgangen (en bøyd grotte med blå krystaller langs berget) → nordstranden ved **undergrunnssjøen** → steinbroa til **holmen** med den hvite krystallen (landemerket, lyser over hele sjøen) → broa videre til sørstranden → Lysgrotta (hvilestedet under den store rosa krystallen), Ekkogrotta (bakholdet, gruvemesterens tavle) og Dryppsteinshallen (skog av stalagmitter og dryppsteiner) → kløfta ned → **vindeltrappa** hogd i berget rundt krystallsøyla (hylla på −10 m der kløfta kommer inn, gulvet på −16 m) → forkammeret → **det store grottekammeret** med Dropdownen mellom krystallsøylene (arenaportene som i nivå 2 og 4) → nedgangen. Snarveien er sprekken fra forkammeret opp til Lysgrotta (slåen nede, 12 m opp). Hemmelighetene: dykket ved boblene (hullet i sjøbunnen → kildesjakta → den skjulte kilden med lore og + utholdenhet → sprekken ut i vindeltrappa) og de tre resonanskrystallene i Dryppsteinshallen (tre spaker som deler én bryter) som åpner krystallveggen inn til Klangkammeret (Progress). 71 500 trekanter i 52 biter, 698 kollidere, 3 659 nav-polygoner, 98 lys; 0 hull i tetthetssjekken (6 051 punkter), og motorens sprekkesjekk (`snapshot --leak-scan`, alle punktene fra `leak_points.py 6 6`) viser bare enkeltpiksler (høyst 3 px) på T-skjøtene i gulvrutene sett nedover, som i nivå 2, 3 og 5.

### Sjøen (nytt i utlegget, felles kode uendret)
- Sjøen er ett rektangelrom med bunnen 6 m under strendene (`floor_y`, `nowalk`, som avgrunnen i nivå 2) og en fast vannflate (`water`-merket med samme høyde i `levels`, så den aldri synker; ingen spak har id-en). Strendene og holmen er rom inne i sjøen (`virtual`, som øya i cisternen): murte blokker fra bunnen opp til strandhøyden (`block`, inn i berget så det ikke står en spalte mellom blokka og muren), steinbroene (`stone_bridge`) nord–holme–sør, og navigasjonen delt som i nivå 4 (strendene har broa til 0,5 m fra holmen, holmen har broa innenfor 1 m, så polygonene overlapper langs brosidene). Faller du i vannet, står du på bunnen (ingen navigasjon der, fiendene går ikke ut i vannet) og går til stigene på øst- og vestsida av holmen. Hullet med boblene er en sjakt (`trapdoor` med `shaft`) ned til kildesjakta.
- Grottene har berg i murene **og hvelvene**: nytt utleggsvalg `vault_mats` (per rom) i generatoren (`Level.vault_mat`), brukt i rektangel-, mangekant- og runde rom; mangekantrom kan få `floor_mat`. `kit_mine.py` har fått `cone` (stalagmitt/dryppstein) og `spire` (et stort krystallspir med små rundt foten), og `kit_grav.MATS` de glødende `magic_white` og `magic_pink`.
- Vindeltrappa er et rundt rom med galleri (`round_room` med `gallery`): trappa fra 20° til 160° (angitt som 380°–520°, så sokkelen tegnes hele veien rundt), hylla rundt resten, kløfta inn på hylla fra øst (`yb` = −10), forkammeret og sprekken fra kilden inn på gulvet fra vest og nord, utenfor trappesektoren.
- **Skjøten i runde rom**: muren i et rundt rom begynner ved en skjøt, og en dør over skjøten blir bare halvveis hogd ut (murkolliderne står i den). Før lå skjøten på 0 eller snudd til 180°; vindeltrappa har dører ved både 0 (kløfta inn på hylla) og 180° (forkammeret), så boten sto fast i den vestre døra. Nytt utleggsvalg `round_seam_free`: skjøten legges på den første av 0, 180°, 90°, 270°, 45° … som er fri for dører. De eldre nivåene beholder den gamle regelen (ellers måtte bitene deres eksporteres på nytt).
- **Lyset i grottene**: krystallenes lys (blått, `flicker` av) måtte opp i styrke 5 med 12 m rekkevidde, den hvite krystallen 28 med 60 m, og de store rommene fikk fyll-lys på 6–12 for at bildene i det hele tatt skulle vise rommet; en sal på 46 × 34 m er svart med lys under 2. Lykta i beltet lyser bare de nærmeste meterne.
- Lærdom: en krystallklynge (eller annen hindring) rett innenfor en dør kutter navigasjonsstripa gangen deler med rommet, og alt bak døra faller ut av nettet («polygoner uten forbindelse tatt bort» for hele resten av nivået). Hold hindringene minst 1,5 m fra dørene. Redigeringsreglene (`edit_tests`) krever at `prop`-merker står inne i et rom (ikke i gangene eller utenfor mangekanten) og at en port står på gulvet (porten ut av arenaen sto i en trapp, så nedgangen fikk samme gulvhøyde som arenaen), og `alt_du_kan_bruke_kan_siktes_pa` at ingen pynt står rett foran en kiste. Ganger som ender i et rundt rom må gå 0,4 m inn i sirkelen, og en gang må være minst 2 m bred for at taket skal nå 2,9 m (`HEADROOM`).

### Byttet ut og ikke gjort
- Planens fiender: **Spinner → breadcrumb**, **Chip-svermen → tooltip** (små flokker), tabellen, modalen og accordionene som de er; **Dropdownen → select** (bossen, `Mob.boss` med stolpen og arenaportene).
- Fellen «fallende krystaller» i kløfta er to `rocks`-trykkplater uten virkning (raset finnes ikke i motoren); krystallene henger i taket som pynt.
- Resonanskrystallene: de tre spakene deler bryteren til krystallveggen (`levers_ready`), men veggen er en `secret_wall` og kan også slås ned med F eller et slag; tonepuslespillet finnes ikke.
- Hylla (−15 m) langs nordveggen i grottekammeret er ikke bygd; kista står på gulvet i arenaen. Tauheisen fra forkammeret er en slak trapp i en sprekk (heisen i motoren starter alltid oppe, og kan ikke kalles ned fra bunnen).
- Dykket er et fall gjennom hullet i sjøbunnen ned i kildesjakta (7 m); ingen svømming eller luftlomme.
- Ingen egne teksturer for grottene: berget bruker teksturlaget `rock` i murer og hvelv, gulvene kryptens gulv og `rubble` på strendene og i sjøbunnen.
- Gjennomspillingen med kamp er ikke kjørt for nivå 6. `cargo test --release`: 72 bestått.
