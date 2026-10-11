# Gjenstandene fra prop2-arkene (sheet2model)

Oppgave: de ti heltegjenstandene i `assets/textures/props/prop2/` bygges med `sheet2model` (`layout: prop2`), og de
fem i piloten (kista, tønna, brønnen, ambolten, hvilestedet) inn i spillet med dagens modellnavn.

Mine filer: `sheet2model/` (config, sheet, props, cli, finish, nytt `parts.py`/`split.py`), `sheet2model/examples/fristad/<gjenstand>.json`,
`assets/models/town/{chest,chest_lid,barrel,well}.glb`, `assets/models/crypt/kit_rest.glb`, `assets/models/town/forge_interior.glb`
(ambolten importeres i `tools/models/workshops.py`), `shots/gjenstander/`, denne loggen.

## Funn før start
- Kameraet i `q_front_left` har +x-flaten til høyre i bildet: det tegneren kaller «forsiden» (låsen på kista) er +x i
  sheet2model-verdenen, mens spillet vil ha fronten mot +z. Brønnens vinde ligger langs x i begge (ok). Derfor
  `opt.game_yaw` per oppslag ved eksport.
- Høyre rute er ekte «bakfra høyre» (sveiva på brønnen bytter side), så `prop2`-oppsettet stemmer.
- Dagens mål: kista 1,08 × 0,64 × 0,465 (hengsel `CHEST_HINGE` (0, 0.46, −0.28)), tønna 0,66 × 0,86, brønnen: kum R 0,98,
  stolper ±0,75, aksel 1,55 (`WELL_AXLE`), `kit_rest` 1,5 × 0,8 × 1,0 med flammen 1,75 (`REST_FLAME`), ambolten i
  Smia på (−1,2, −1,15) med kubbe 0,5 høy og toppen 0,88 over gulvet.
- Disk: 1,5 GB ledig. `target/sheet2model/<navn>/texels.npz` og `field.npy` (190 MB per bygg) slettes etter hvert bygg.

## Logg
- 09:30–10:30 `sheet2model`: `parts` i oppslaget (`parts.py`: hoveddelen gir kameraet, de andre låses med `fix_view`, forskyvning
  trianguleres fra nullpunktene, atlas med hylleplassering, `own` for deler som blir egne modeller), `part_box`/`erase` per del i
  `sheet.segment`, priorene `cylinder_x` og `ring`, `lid` og `bulge_range` som oppslagsvalg, `prior_carve` (dybden får bare
  gi 12 px relieff inn i prioren: uten den ble kista en klump), grovere voksel for store ark (`max_voxels` også for gjenstander),
  `split.py` (`game_yaw`, `height`, `ground`, deling om hengslet med fylte snittflater), `fix_view`/`pitch_range`/`persp_range`.
  Kistas prior hadde lokkaksen langs x (låsflaten ble kortsiden): nå langs z, så låsflaten er +x som i rutene.
- Kista v1: IoU 0,85/0,88, foto 0,054, 3 000 tri, men beslagene dobbelt (speilbildet malt oppå den ekte ruta) → `mirror_weight`
  0,02 for speilede visninger på gjenstander. Vippen ble 5° med persp 0,14 → `pitch_range` 8, `persp_range` 0,06.
- `tools/models/workshops.py`: `import_glb` tar inn `assets/models/props/anvil.glb` i stedet for byggesettets ambolt (faller
  tilbake om fila mangler). Smia må eksporteres på nytt (`python3.13 tools/models/workshops.py forge`).
- 10:45 Koordinatoren: geometrien skal VÆRE prioren. Nytt `shapes.py`: parametriske nett (boks med fas, kiste = kasse + halvsylinderlokk
  som to deler, dreiekropper `cylinder`/`barrel`/`revolve` (profil med kontrollradier passet til silhuetten), `ring`, `cylinder_x`),
  Smart UV, tette flatepunkter til z-bufferen. `prior_only` (standard på for gjenstander): fuse passer bare prioren, ingen dybde,
  mesh bygger nettet fra prioren, teksturen projiseres som før. Kista: 112 trekanter, IoU 0,94/0,92, skarpe beslag.
- Feil funnet: fortegnet på `pitch` i `geom.basis` var snudd (positiv vipp = kameraet under), derfor rømte søket mot 5° og
  prioren fikk strukket topp. Rettet; vipp/perspektiv/yaw holdes nå innenfor vinduer (`pitch_range` 8, `persp_range` 0,06,
  `yaw_range`). Hengslet: lik skalering (høyden) og forskyvning i z, ikke ulik skalering.
- Gjenstår på kista: silhuetten alene lar bredde/dybde bytte med yaw; kista ble 1,87× så bred som dyp (tegningen ca. 1,6×).
- 11:30 Koordinatoren: kista godkjent som form/tekstur; målene skal låses til spillets (`size`), ambolten skal være en ekte ambolt i
  deler, lysestaken i deler. Gjort: `opt.size` (meter per priorparameter → låst, bare kameraet søkes, med målestokk `sy` per rute),
  `init`/`fix`/`param_range`, `at` (fast sted, `null` = fri akse), `on`/`under`/`y_of`/`to_ground` (stabling), `lock: "z"|"x"`
  (to motsatte ruter ser bare én vannrett akse), priorene `cone_x` og `gable` (saltak). Byggingen av deler er omstrukturert:
  først prioren for alle delene, så forskyvningene, så nett og tekstur.
- Lærdom: silhuett-IoU alene lar en boks bli en flat plate under vippen (ambolten) og lar bredde/dybde bytte med yaw; derfor
  skal målene komme fra spillet. Sylindere er trygge hoveddeler (vippen fra ellipsen); bokser drar kameraet mot grensene.
- 12:30 Status per gjenstand (alle med spillets mål låst, geometri = prior):
  - Kista: OK. Kasse 1,08 × 0,60 × 0,46 + halvellipselokk 0,18 høyt over hele dybden (`chest`-prioren er nå boks + halv ellipse),
    delt i `chest`/`chest_lid` med origo i hengslet (0, 0.46, −0.28). 112 trekanter, IoU 0,81/0,79, skarphet 526. Installert i
    `assets/models/town/`. Tegningen er mer kubisk (høyde ≈ 0,85 × bredde) enn spillets mål (0,6), så lokket males litt strukket.
  - Tønna: OK. R 0,33, 0,86 høy, bul 0,14 (låst; fri bul ble 0,64 = kule). 624 trekanter, IoU 0,85/0,83. Installert.
  - Hvilestedet (`kit_rest`): sokkel 0,6 × 1,0 (boks, hoveddel) + tre bein, stett, skål og lys (revolve) stablet med `at`; 1,72 høy,
    flammen på 1,75 settes av spillet. 1 520 trekanter (dreiekroppene). Sokkelen og sprekken skarpe. Installert i `assets/models/crypt/`.
  - Ambolten: kubbe (sylinder R 0,22, 0,6 høy, hoveddel) + fot, midje, topp (bokser), horn (kjegle), hæl, toppen 0,88 over gulvet.
    Formen er en ambolt, men teksturen er smurt fordi tegningens ambolt er større i forhold til kubben enn Smias mål tillater.
    Ligger i `assets/models/props/anvil.glb` og tas inn i `forge_interior` av `workshops.py`.
  - Brønnen: kum (ring R 0,9, 1,15 høy, hoveddel: vippen og målestokken fra ellipsen) + saltak (`gable`) på aksen, stolper ±0,75
    til bakken, vinda som egen modell `well_winch` (origo i akselen). Kummen god (IoU 0,92/0,87). Siste bygg (snudde takflater)
    pågår; `well_rope`/`well_bucket` beholdes fra i dag.
- Feil rettet underveis: `lid` og `bulge` ble søkt selv om de var låst; delene fikk luft mellom seg (silhuettsentrene under
  vippen) → stabling; to motsatte ruter kan ikke triangulere langs synslinja → `lock: "z"`/`at`.
- 13:10 Inn i spillet: `assets/models/town/{chest,chest_lid,barrel,well,well_winch}.glb`, `assets/models/crypt/kit_rest.glb`,
  `assets/models/props/anvil.glb` (importert i `forge.glb` av `workshops.py forge`; `forge_inside.json` uendret). `cargo build
  --release --bins` grønn. Skjermbilder: `shots/gjenstander/game_torg.png` (tønna og brønnen på torget), `game_chest.png` (kista
  åpen i krypten, lokket om hengslet), `game_well.png` (brønnen; bøtta henger under kanten fordi kummen er 1,15 høy og
  `WELL_REST` 1,05), `game_smia.png` (ambolten på kubben). Normalkartet er flatt for priornett (tangentrammen på smale endeflater
  ga svarte trekanter på saltaket).
- Klart til spillet: kista, tønna, hvilestedet. Brukbart, men svakt: ambolten (teksturen smurt: tegningen har større ambolt enn
  Smias mål), brønnen (taket er et glatt prisme med strukket tekstur; kummen er god). Ikke begynt: spaken, gitteret, sarkofagen,
  gryta, alteret (oppslag for gitter/sarkofag/gryte finnes, ikke bygd).
- Neste: tak som egen tynnere `gable` med takskjegg og teksturen fra ruta ovenfra; ambolten med `size` nærmere tegningen
  (kubbe 0,75 høy) om Smia tåler det; `WELL_REST` over kummens kant; de fem siste gjenstandene.
- 13:40 `cargo test --release`: 44 ok, 3 ignorert. `alt_er_ekte_modeller_med_teksturer` feilet først fordi `workshops.py` skrev
  `layer_metal` (laget heter nå `iron` i `geometry.rs`, og HEADs glb var alt omdøpt): `layer("metal"` → `layer("iron"` i
  `workshops.py` (6 steder). Smia eksportert på nytt, `game_smia.png` tatt om.
- Sikkerhetskopi: grenen `wip/gjenstander` (tvunget push, egen gren). Ikke committet på `claude/detaljnivaa`.
