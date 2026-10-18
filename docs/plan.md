# Plan: Fristad på rett kurs – kvalitet før alt, ren kode, målt ytelse, to filmer

## Context

Eieren har underkjent det meste som er laget de siste dagene: figurene (for små, hode-i-hode, smeltede hetter), brønnen og ambolten, kryptnivå 1 (39 hull, tomme rom, harde overganger), kryptens dør, kirkas stil, tønnerader, svevende gårder. Kartleggingen viser at årsakene sitter i måten vi bygger på, ikke i enkeltting:

- **Ingen kontroll før det dyre steget.** sheet2model måler IoU på vokselfeltet, ikke på det ferdige nettet, og skriver glb + Cycles-ark *før* dommen (brønnen gikk inn med `ok: false`, IoU 0,19). Blender-eksportene kontrollerer ingenting. Ingen teksturkontroll finnes. CI kjører ingen tester.
- **Ingen regler for sammensetning.** Flater møtes uten list/sokkel/gesims; hver vegg starter uv på 0 (søm i hvert hjørne); dørbladene strekkes 2× (`kit.door`, `sacred.arched_leaf`); `kit_up` bygger en steinflate 20 cm foran kapellets pussmur; nisjene i krypten skjæres aldri ut (`dress()` kaster returverdien); innredningen er ~5 500 håndskrevne desimaltall; bondegårdene står på høyeste punkt; `audit` godtar én fot på bakken.
- **Oppbygget kode.** ~25 modellnavn i Rust peker på slettede modeller (fiender, dyr, våpen, `npc`); `tools/reconstruct` og `sheet2model` dupliserer sju moduler; `people.py`, `town.py` og `houses.py` skriver de samme filene; hjelpere som `box`/`prism`/`lathe`/`barrel` finnes i 3 kopier; `session.rs` (2 383 linjer, ~65 felt) gjør ti ting som hører i domenet; arkitekturtesten er et tekstsøk som lett lures, og domenet leser json med `include_str!` og parser `serde_json`.
- **Ytelse uten måling.** 9 fps på eierens Mac (GPU 223 ms) mot 36 på master; vi har ingen måling i nettleseren, lavapipe er ubrukelig som mål, og teksturene er 1024² (0,5 teksel/mm) mot kravet 1 teksel/mm.

Eierens beslutninger (11. okt.): én økt, én maskin, ingen skyøkter; dreieark med 8 visninger; ingen overstyring av kontroller; estetisk vurdering av en uavhengig Claude-vurderer med fast rubrikk mot eierens godkjente ting og en skreven stilguide, som lærer (feil → regler i koden, stilguiden vokser, terskler strammes) og gjelder modeller, teksturer, scener, filmene og nettlaget; åpnings- og sluttfilm (detaljer i del 8).

Målet: ingenting når eieren uten å ha bestått automatiske tekniske og estetiske kontroller; det billige kjøres før det dyre; feil rettes ved roten i felles kode og blir regler.

## 0. Nå: hvorfor MSAA koster ~18 ms på M2 (fase 3 trekkes fram)

**Hva to målinger fra eierens Mac mini M2 (Chrome, WebGPU, ~0,7–1 Mpx) har vist:**
- Uten MSAA: torget 10 ms og 120 fps (skjermtaket). Med MSAA 4×: 45 ms og 46 fps. Enkel: 7–8 ms.
- Flat farge med MSAA: 22 ms, mot 4 ms uten. Kostnaden er altså ikke skyggeleggingen.
- Utelukket: HDR-formatet (`Rg11b10Ufloat` 43 mot 45 ms), tidsmålingen (samme fps uten), lagring av MSAA-målene (`Discard`), flere pass (ett hovedpass), alfakort/`discard` i byen eller modellene (ingen har alfa), skyggepasset (2 ms).
- Delene (verden ≈ etterbehandling i hver rad) er ikke til å stole på; fps og summen er det som teller.

**Hypotesene:** `snapshot --stats` viser bare ~170 000–270 000 trekanter og 61–79 kall i hovedpasset (torget, Storgata, sørporten), så ren geometri forklarer neppe 18 ms. Igjen står det som blandes eller fyller hele bildet per MSAA-prøve: røykdottene (32 piper × 24 store, blandede firkanter), himmelen (fullskjerm bak alt), og selve fyllingen av 4 prøver per piksel.

**Målingsrunde 3 (`?bench`, avgjør før noe bygges om):** hver rad skriver trekanter, kall og lerretets størrelse; kjøringene er som i spillet, uten MSAA, flat, uten røyk/nedbør, uten himmel, uten modeller, flat med bare bymeshen, halv oppløsning (med og uten MSAA), enkel og avansert. `engine.set_bench(json)` med bryterne i `ports::Bench` (`sky`, `particles`, `models` nye), `benchScale` i `fit()`.

**Runde 3 (eierens M2, 11. okt.) avgjorde det:** uten modeller (26k △, 23 kall) 47 ms, uten røyk 41, uten himmel 45, flat med bare bymeshen 24 ms mot 4 uten MSAA, halv oppløsning omtrent halv kostnad. MSAA i seg selv koster ~20 ms per bilde i Chrome på M2, uavhengig av innholdet; uten MSAA er hele bildet 9 ms. **Beslutning:** standard bytter MSAA ut med SMAA 1× (eierens valg); avansert beholder MSAA. Safari på samme maskin: MSAA 81 ms mot 14,5 uten, avansert 12 fps, så det er WebGPU på M2, ikke Chrome. SMAA er laget (`adapters/wgpu/smaa.rs`); `?bench` måler SMAA mot MSAA i begge nivåer.

**Rettelsen etter hva runden viser:**
- *Røyken eller himmelen:* røyken tegnes i et eget pass uten MSAA (halv oppløsning, satt sammen før tonemappingen), himmelen bare der dybden er tom.
- *Geometribundet:* trekantene ned til budsjettet. Først det som ikke synes: innsidemodellene som tegnes inntil 22 m selv om du står ute (bare når du er inne eller i døråpningen), fjerne nett for alt over `fine_within`, og culling av modellinstanser bak hus (grov okklusjon per rute fra bymeshen). Så Vertshuset og de tyngste husene ned mot rammene i `ny-stil.md` (≤10 000 ute, ≤25 000 med innside) i fase 4. Budsjettet i CI (`snapshot --stats`, fase 3) holder det der.
- *Fyllbundet, eller fortsatt for dyrt etter at trekantene er nede:* standard bytter MSAA ut med et kantfilter (SMAA 1×, eierens valg 11. okt.) som eget pass før tonemappingen (`post.wgsl`), rundt 1–2 ms. Avansert beholder MSAA. Kontroll: skjermbilder av gjerder, tau og takkanter med og uten filteret.
- Uansett: standardnivået skal gi ≥ 60 fps på referansemaskinen (eierens Mac mini M2; eieren har ingen jobb-PC), bekreftet med eierens `?bench`, før fase 2 fortsetter.

**Verifisering:** eierens `?bench` før og etter hver rettelse (M2, Chrome og Safari), loggført i `docs/kvalitet/ytelse.md`; `cargo test`, `node tools/wgsl-check.mjs`, skjermbilder av torget og krypten med og uten rettelsen.

## 0b. Mens eieren er borte: resten av fase 1 (navnestyring og døde modellnavn)

**Hvorfor:** `gltf.rs::load()` velger lys, glass, vann, magi, alfa, lakk og teksturlag ut fra materialnavnet (`starts_with`, `.paint`, `.magic`, 15 regler, :322–452), og modellnavnet styrer vær (`kit_`, `crypt_NN`) og fjernnett (`_leaves`, `_trunk`). En skrivefeil i Blender gir stille feil. I tillegg peker 30 modellnavn i Rust på modeller som ikke finnes (fiender, `token`/`key`/`bolt`, `arms`, våpen og skjold, dyrene, de 13 figurene), og bare `place` melder det; `reserve` og `figure()` er stille.

**A. Rollene som data i glb-filene**
- `tools/models/roles.py` (ny): én tabell fra materialnavn til rolle (`kind`, `flags`, `layer`), den samme logikken som `gltf.rs` har i dag, og fra modellnavn til modellroller (`no_weather`, `far: cards|cell`).
- `tools/models/slim.py`: `tag(path)` skriver `materials[i].extras.fristad = {kind, flags, layer}` og `extras.fristad` på roten (modellrollene), alltid, før den tidlige returen for kvantiserte filer. Feiler eksporten på et materialnavn tabellen ikke kjenner (ingen stille `kind::PLAIN`).
- Én gang: `python3 tools/models/slim.py --tag assets/models` over alle 128 glb-filene (ren json-omskriving, uten Blender; rundt 26 MB i git).
- `src/adapters/gltf.rs`: leser rollene fra `extras.fristad` og har ingen navneregler igjen; mangler rollene, er det en feil som nevner fila. Det som ikke er navnestyring (`metallic`, `alphaMode`), står.
- Test `modellene_har_roller` i `src/tests.rs`: hvert materiale i hver modell har roller, og ingen kode i adapterne sammenligner materialnavn (tekstsjekk i `architecture.rs` på `named(`/`starts_with("glow"` …).

**B. Modeller som ventes, eksplisitt**
- `crates/fristad-domain/src/domain/model.rs`: `PLANNED`: navnene systemene venter på, med hvorfor (fiendene, plukkene, armene, våpen og skjold, dyrene, figurene).
- `Builder::reserve` og `town::figure` melder manglende modeller inn i `Builder.missing` som `place` gjør, og loggen skiller «venter» (i `PLANNED`) fra «ukjent».
- Test `alle_modellnavn_finnes_eller_ventes`: et navn som er i bruk, finnes eller står i `PLANNED`; et navn i `PLANNED` som finnes nå, må strykes. `assets/models/props/anvil.glb` (underkjent, i ingen tabell) slettes.

**Verifisering:** `cargo test` (nye tester grønne, bevisst feil: et materiale uten roller og et ukjent modellnavn slår ut), skjermbilder av torget om dagen og natta, krypten og portalen før og etter (pikseldiff ≈ 0), `node tools/wgsl-check.mjs`, push samlet med neste målingsrunde.

## 1. Prinsipper (gjelder alt under)
1. **Kontroll før kostnad.** Rekkefølgen er alltid: spesifikasjon → billige kontroller (numpy, sekunder) → Blender/eksport (minutter) → estetisk vurdering → eieren. Ingen Cycles-render i løypene; godkjenningsark lages med numpy-rastereren og ett `snapshot`.
2. **Ingen overstyring.** En kontroll som feiler stopper løypa. Er terskelen feil, endres den i kode med begrunnelse, og alt godkjent kjøres på nytt.
3. **Rot, ikke lapp.** Hver feil som oppdages (av kontroll, vurderer eller eier) får en årsak og en regel i felles kode + en test, og føres i `docs/kvalitet/laerdom.md`. Ingen særtilfeller per modellnavn.
4. **Én kilde til sannheten** for mål, lag og godkjenninger (`assets/specs.json`, `assets/approvals.json`), lest av både Python og Rust.

## 2. Arkitektur og rydding (først – alt annet bygger på det)
- **Kjernen sikres av kompilatoren, ikke tekstsøk:** Cargo-arbeidsområde med egne crates `fristad-domain`, `fristad-ports`, `fristad-app` og adapterne. `fristad-domain` får bare `glam`, `serde`, `log` som avhengigheter; da kan den ikke bruke wgpu/fs/env uansett hva noen skriver. Json-filene (`*_inside.json`, `crypt_NN.json`) leses i adapterne og gis domenet som data gjennom en port (`WorldSource`), ikke `include_str!`/`serde_json` i domenet. `architecture.rs` erstattes av en test som leser `cargo metadata` og krever avhengighetsretningen; i tillegg grenser: ingen fil over 800 linjer i domain/app, ingen `match` på modellnavn utenfor adapterne.
- **`session.rs` deles:** spillerkroppen (`Stance`, fall/redning), mekanismene (port, brønn, snarvei, spak, portal) som tilstandsmaskiner, kryptreglene (feller, plater, nøkler), død/bank/inventar, handel/smiing (bruker `domain/craft`, ikke en kopi) og kartberegningen flyttes til domenet. `Session` blir koordinator: input inn, domenet tikker, fysikk via porten, hendelser ut. Testbakdørene (`#[cfg(test)]`-metodene) erstattes av domenets egne tester.
- **Navnestyring bort:** materialroller (`layer_`, `.paint`, `.magic`, `_interior`, `kit_`) skrives som eksplisitte felt i glb-ekstras ved eksport og leses av `gltf.rs`, i stedet for prefiks/suffiks-gjetting. Lyd og regler velges etter data (`Shield.sound`, `Weapon.special`), ikke `== "shield_tower"`/`"divider_blade"`.
- **Slett:** alle referanser til modeller som ikke finnes (fiender, dyr, våpen, `token`/`key`/`bolt`, `npc`/`npc_woman`, `kit_chest`) erstattes av eksplisitt «mangler»-tilstand fra `specs.json` (systemene står, men vet at modellen ikke er laget); `tools/reconstruct` (etter at sheet2model ikke leser fra den), `people.py`, `npc.py`, `hode_z.py`, `HOUSES`/`__main__` i `town.py`, `assets/models/props/flames_atlas.png`, utdaterte wip-logger, de 13 underkjente figur-glb-ene; dokumentasjon av slettede skript i CLAUDE.md.
- **Ett felles Python-bibliotek** `tools/lib/` (geometri: `box`, `prism`, `lathe`, `tube`, `ring`, `cylinder`; materialer og lag; eksport med kontroller) som `tools/models/`, `tools/crypt/` og `sheet2model` bruker; skript importerer aldri fra hverandre.
- **Tester i CI:** `cargo test` og `pytest` (uten Blender) i `web.yml` før bygget.

## 3. Tekniske kvalitetsporter (`tools/qa/`, felles for alt)
Rammeverk: ren funksjon per port → liste med feil + bilde; stopper løypa; skriver `gates.json`; `qa promote` er eneste vei til `assets/` og skriver `<modell>.qa.json` (sha av kilde, verktøy og kontrollkode). `cargo test` (`alt_har_gronn_kvalitetsrapport`) feiler for enhver glb eller teksturlag uten gjeldende grønn rapport. Felles **numpy-trekantrasterer** (bruker `geom.to_img` med persp; testet mot Blender-maske IoU ≥ 0,995; ≤ 1 s per visning ved 1024²).

**Modeller fra ark (figurer, monstre, gjenstander):**
P0 arket (oppløsning, bakgrunn, avskåret, like høye visninger, forfra = speilet bakfra, silhuettkonsistens mot skrog av de andre visningene, holdning, armer fri, hode/høyde, tomme hender, flatt lys) → P1 spesifikasjon (`specs.json`: høyde, proporsjoner, mål; skalaen regnes fra målhøyden; hode/kropp-skala innen 5 % ellers stopp) → P2 geometri på det ferdige nettet i hver byggevisning (IoU ≥ 0,92 figur / 0,88 gjenstand, per høydebånd ≥ 0,85, konturavstand p95 ≤ 1 %, manglende/overskytende biter, hender/hode per region) → P3 tekstur (ΔE2000 median ≤ 5, SSIM ≥ 0,75, gradientlikhet, skarphetsforhold) → P4 alle vinkler: **de fire 3/4-visningene i dreiearket holdes utenfor byggingen og er fasit** (IoU ≥ 0,88, ΔE median ≤ 9), strekk per flate (anisotropi ≤ 2 for 95 %), kildestrekk (`ndc`), sømmer, hvor fargen kommer fra (ekte ≥ 85 %, speil ≤ 10 %, flis ≤ 5 %), 24 mellomvinkler → P5 topologi (vindingstall: ingen nestede skall/hode-i-hode, inntrengning, én komponent, hull, orientering, budsjett, mål og pivot; eksportert glb = npz) → P6 rigg (skinning i numpy over alle klipp: kantstrekk, volum, fotgliding, selvgjennomtrengning, vekter) → P7 i spillet (mål mot `specs.json`, figurer 0,7–1,1 × spillerens høyde).
**Skrekksett:** dagens 13 figurer, brønnen, ambolten og nivå 1 med hullene må feile i riktig port. **Gullsett** låses; `qa regress` feiler når en verktøyendring gjør noe dårligere.

**Byggesettets modeller (hus, krypt, gatelivet):** mål og budsjett (P1), tekseltetthet og uv-strekk per flate (inkl. dørblad), flater i samme plan (z-fighting via rasteren), orientering, degenererte, åpne hull i kanter mellom deler, utskjæringer som faktisk er skåret ut, mekanismer flush med muren de står mot, `leak_check` for kryptnivåene, og bakkekontakt over **hele** fotavtrykket mot det tegnede bakkenettet (erstatter dagens `audit` som godtar én fot).

**Teksturlag (P-T):** flisbarhet (forskjell langs kantene mot et forskjøvet bilde), ingen ramme/bård/tekst (kantstatistikk), ingen perspektiv (linjer/gradient), jevnt lys (lavpass-gradient over flisa), skarphet, gjentakelse på avstand (autokorrelasjon), og fargebalanse mot de godkjente lagene i samme tema (palett-avstand i Lab). `derive.py` og `quilt_seam.py` kjøres bare på det som passerte P-T. `size_m` får én kilde (`meta.json`), `TEX_SCALE` fjernes. Oppløsning avgjøres og skrives i `ny-stil.md`: kildene lagres i 2048², standard bruker 1024² per 2 m med 1 m fliser der det sees nært, avansert 2048² når BC7 er på plass. Laget `bark` i `workshops.py` rettes; ubrukte lag og de 27 avledningsløse `painted/`-mappene ryddes eller tas i bruk.

## 4. Estetisk kontroll (etter de tekniske portene, før eieren)
- **Stilguide** `docs/kvalitet/stilguide.md`: hva som kjennetegner WoW/Fable-stilen målbart og beskrivende (overdrevne proporsjoner, lesbar silhuett på avstand, chunky former med avfasing, malt lys og AO, varm/kald kontrast, mettet men harmonisk palett, detalj i tekstur ikke i geometri, ingen tomme flater, ingen rette rader, historie i plasseringen), med bra/dårlig-eksempler fra eierens godkjente og underkjente ting.
- **Rubrikk** per type (modell, tekstur, scene, filmscene, nettlag): 6–8 kriterier, poeng 1–5, terskel per kriterium (≥ 4 på silhuett, proporsjoner, overganger; snitt ≥ 4).
- **Vurdereren:** en egen Claude-hjelper som ikke har laget tingen, får bare godkjenningsarket (fast format), rubrikken, stilguiden og referansebildene, og svarer i strukturert form (poeng, begrunnelse, konkrete feil med plassering i bildet). Under terskel → tilbake til revisjon, aldri til eieren. Etter 3 mislykkede revisjoner av samme ting: stopp og rapporter til eieren med vurderingene.
- **Læring:** hver avvisning (vurderer eller eier) analyseres: blir det en regel i byggesettet/sheet2model/qa med test, eller en terskel som strammes (aldri slakkes), eller et nytt eksempel i stilguiden. Loggføres i `laerdom.md`.
- **Faste scener:** de åtte posisjonene i `tools/stats.sh` + tre i krypten rendres ved hver push (lavapipe), vurderes med scene-rubrikken og pikseldiffes mot godkjent grunnlinje.

## 5. Sammensetning og plassering (felles logikk i byggesettet)
- **Overganger som regel:** et overgangsbibliotek i `kit` der hver kombinasjon av lag har en fast løsning: sokkel (stein mot jord/brostein), bånd/gesims mellom etasjer og mellom stein og bindingsverk, hjørnestein/hjørnebjelke i alle ytterhjørner (lukker også hjørnegliper), takskjegg-undersider, vannbrett, karm. En flate kan ikke grense mot en annen uten at overgangen er definert; eksporten feiler ellers. uv fortsetter rundt hjørnene (ingen søm per vegg).
- **Dørmodulen:** én funksjon for alle dører (hus, Vertshuset, kirka, krypten, `kit_up`, snarveien): dørbladet beholder bildets proporsjon (åpningen tilpasses døra), beslag enten i bildet eller som geometri aldri begge, feltet rundt arver veggens lag, steinomramming bare på steinmurer, og mekanismer flush med murplanet.
- **Utkraging og etasjeskiller:** felles løsning (som `JETTY_T`) i `kit`, brukt av alle hus.
- **Innredning `group()`:** ankre (vegg, hjørne, peis, disk, bord, under trapp), grupper på 2–4 med variasjon i dreining/avstand, dørsoner og ganglinjer holdes fri, ingen rader med likt mellomrom, lys på flater aldri gulv; kontroller som feiler eksporten (overlapp, sone, rad-regel, flyt); test som nekter håndkoordinater utenfor `group()` i innredningsskriptene. Romtyper (skjenkestue, kjøkken, kjeller, sovesal, smie, apotek, skriptorium, hjem; kryptens kapell, gravgang, beinhus …) er sett av grupper.
- **Terreng og fundament:** planering per tomt (polygon + høyde + skråkant) for gårder og alt utenfor muren; grunnmur ned til terrenget under hvert hjørne; småting per modul på terrenget. Kontrollen over hele fotavtrykket (del 3).
- **Kryptens moduler:** sokkel, gesims under hvelvet, pilastre, vederlagsbånd, terskler mellom gang og rom, ekte nisjer skåret ut, trapperegler som i `kit`, AO bakt i hjørnefargen, lysoppskrift (lavt grunnlys, varm kjerne, kald utfylling); rom og ganger mindre og lavere etter `dng_grav`.
- **Fjellene** løftes så de står over åsene fra øyehøyde.

## 6. Nye prompter (levert eieren tidlig, så arbeidet går parallelt)
Hver prompt følger et fast format med mål og referansesilhuett der det finnes, og P0/P-T svarer på sekunder om arket kan brukes (med ny prompt hvis ikke).
- **Figurer (13)** og **monstre (20; 8 mottatte sjekkes, 12 nye)** som dreieark med 8 visninger (forfra, 3/4 forfra V, side V, 3/4 bakfra V, bakfra, 3/4 bakfra H, side H, 3/4 forfra H), A-positur, tomme hender, flatt lys, sålelinje, figur ≥ 900 px, med grunnkroppens silhuett i 8 visninger som referansebilde; ny layout `turn8` i sheet2model og PROMPTS.md.
- **Redskap og hodeplagg** som egne gjenstandsark.
- **Gjenstander** på nytt (brønn med tak som egen del, ambolt med større kubbe, gitteret i 3/4) og de som mangler (spak, alter, gryte), med en rett forfra-visning holdt utenfor som fasit.
- **Teksturer:** kryptpuss (rettet til 2048 px/2 m), kirkas dør og steinomramming (`door_church`, `trim_stone`, `carving`), wool/linen uten søm, overgangslister (sokkel, gesims, bånd, hjørnestein) som trim-ark, kryptmoduler, og de lagene P-T underkjenner.
- **Kirka som steinkirke** i kryptens stil.
- **Filmenes lag** (del 8), etter at animatikken er godkjent.
- Dyrene, trærne og buskene (egne ark per dyr) når de står for tur.

## 7. Ytelse – målt og sikret
- **Måling i nettleseren `?bench`:** går gjennom faste posisjoner, skriver GPU-ms per del (skygge, verden, etterbehandling), fps, oppløsning, dpr, MSAA, nettleser/GPU i en kopierbar tekst, med brytere (`msaa`, `a2c`, `shadow`, `flat` fragmentshader) som skiller rasterisering, skygging, MSAA og skyggepass.
- **Grafikknivåer** enkel / standard / avansert (enkel = WebGL2: uten MSAA, 900p, ingen solskygge, ett lag bakke, 4 lys, ingen glød, halv teksturtetthet), oppløsning i trinn (finnes).
- **Rettelser i rekkefølgen tallene peker:** delt skyggepass (ugjennomsiktig uten fragmentshader), terrenget (støy bakt til flis, høyst 3 lag, triplanar bare på berg), `textureSample` der gradientene er uniforme, lys per rute, himmel uten støy per piksel, sortering forfra og bakover, culling på terreng/krypt, skyggekart 2048².
- **Budsjett som kontroll:** `snapshot --stats` i CI på de åtte posisjonene; feiler over 500k trekanter / 300 kall eller over grunnlinjen + 10 %. Eierens bench-tall fra M2 (Chrome og Safari) er fasiten for fps (mål: standard og avansert ≥ 60 fps der; nådd 11. okt. med SMAA), loggført i `docs/kvalitet/ytelse.md` per bygg.

## 8. Filmene (three.js i nettlaget)
**Felles motor** `web/film/`: three.js (fast versjon), malte lag som teksturerte plan i 3D med ekte perspektiv i én sammenhengende kamerabane gjennom scenene (overgangene går gjennom noe i bildet: port, vindu, sprekk, glitch-hull), tåke og blafrende lys, partikler (støv, glør, snø, gnister), små 2D-deformasjoner i figurlag (pust, svai), glitch-effekter på de korrupte komponentene, tekst i middelalderskrift (fri gotisk/unsial skrift) tonet inn per scene, egen syntetisert musikk + lydbilde per scene (som `music.js`/`ambience.js`), musa dreier kameraet noen grader, Esc/mellomrom/knapp hopper over. Fyller 16:9 og 16:10. Manuset og banen er data (`web/film/opening.json`, `ending.json`).
- **Åpningsfilmen:** ~1 min, 6 scener (Skriptoriet i gullalderen → fallet: forretningslogikk lekker inn → begravelsen under kirka → krypten i dag med Button Wight og Modalen → Fristad 1350, ryktene → veien mot sørporten). Spilleren er en fremmed som kommer til byen; tone: mytisk med glimt i øyet. Starter hver gang med ett klikk («Klikk for å begynne», gir lyd og fanger musa); lastingen av spillet går under filmen; siste scene venter og driver over Fristad med en diskret framdriftslinje til spillet er klart, så synker kameraet til øyehøyde ved sørporten og spillet tar over.
- **Sluttfilmen:** spilles når både Rammeverket er beseiret og alle komponentene henger på veggen (idet det siste av de to skjer); ~1 min, 6 scener (Rammeverket faller → krypten blir stille → galleriveggen full → Skriptoriet skriver Fristil på nytt, én sannhet på serveren → festen på torget → over byen mot fjellene); spilleren bare antydet (silhuett, skygge); så rulletekst og tilbake i byen; kan ses igjen fra Esc-menyen.
- **Arbeidsgang:** jeg skriver manus (scener, tekstlinjer, hva hvert lag viser) → eieren godkjenner → **animatikk** med grå plassholdere, banen, tempoet, overgangene og teksten i preview → eieren godkjenner → prompter per lag med mål, dybde og plass i banen → eieren lager hvert lag for seg → P-F-kontroll av lagene (oppløsning, ren alfa etter utskjæring, palett mot spillet, at lagene dekker bildet i banen fra alle kameravinkler i 16:9/16:10) → estetisk vurdering per scene → eieren.

## 8b. Nettlaget i Fristil
- **Alt brukergrensesnitt er Fristil:** startskjermen, Esc-menyen med innstillinger, tastatur og musefølsomhet (#40), HUD-en (stolper, merker, tooltip ved siktekorset, minikartet), butikken og verkstedet (i dag en egen «overleggsboks, ikke designsystemet» – byttes til `fs-dialog`), inventaret, loot-dialogen, boka, toasts, skadetallene og rulleteksten. Bare Fristils komponenter og tokens (`fs-dialog`, `fs-button`, `fs-tabs`, `fs-card`, `fs-radio`, `fs-progress`, `fs-tooltip`, `fs-badge`, `fs-alert` …), ingen egne farger, fonter eller skygger utenfor tokens.
- **Litt dybde, men gjenkjennelig:** dybden lages med lag av Fristils egne flater og tokens (opphøyde kort over spillbildet, myk skygge og lett gjennomsiktig bakgrunn med uskarphet bak dialogene, tynn lys kant), uten å endre komponentenes form, typografi eller farger. Lyst tema om dagen, mørkt om natta som i dag.
- **Kontroll:** en test i `tools/layout-test.mjs` (eller ny `ui-test.mjs`) som feiler om et synlig, interaktivt element i nettlaget mangler en `fs-`-klasse eller bruker en farge/skygge/font som ikke er et Fristil-token (lest fra `fristil.css`), og skjermbilder av hver meny/dialog i lyst og mørkt tema på faste skjermstørrelser som går gjennom den estetiske vurdereren med nettlag-rubrikken («gjenkjennelig som Fristil», «dybde uten å endre komponentene», lesbarhet over spillbildet).
- Filmens tekst er unntaket (middelalderskrift), men «Hopp over»-knappen og «Klikk for å begynne» er Fristil.

## 9. Rekkefølgen
| Fase | Innhold | Eieren ser / gjør |
|---|---|---|
| 0 (½–1 dag) | `?bench`, grafikknivå «enkel» | bench-tekst fra M2 i Chrome og Safari |
| 1 (2–3 dager) | Arkitektur og rydding (del 2): crates, `session.rs` delt, sletting, `tools/lib`, tester i CI | samme spill, mindre kode; arkitekturtest i CI |
| 2 (4–5 dager) | `tools/qa`: rasterer, rammeverk, P0–P7, P-T, byggesettsportene, skrekk- og gullsett, `promote`/`.qa.json`/Rust-test; stilguide, rubrikker og vurdereren | P0/P-T-svar på eksisterende ark og teksturer; **nye prompter (del 6) og filmmanusene** leveres i starten av fasen, så eieren lager dreieark mens portene bygges; animatikkene når manus er godkjent |
| 3 (2–3 dager) | Ytelsesrettelsene (del 7) i den rekkefølgen bench viser; budsjett i CI | bench nr. 2 og 3 |
| 4 (4–5 dager) | Byggesettets regler (del 5): overganger, dørmodul, utkraging, `group()`, terreng/fundament, krypt-moduler; byen og husene eksportert på nytt gjennom portene; kirka som steinkirke; kryptens kapell som pilot, så resten av nivå 1 | godkjenningsark for byen, tre hus, Vertshusets skjenkestue, kapellet; så nivå 1 |
| 5 | Grunnkroppen (6–8k, fast uv, 18 bein), så Button Wight og Buttonsmeden fra dreieark gjennom alle portene; så resten av monstrene og figurene én og én | godkjenningsark per figur/monster (8 vinkler + gang) |
| 6 | Filmene ferdige når lagene er levert og godkjent | filmene i preview |
| 7 | Nettlaget i Fristil (8b) med Esc-menyen (#40); gjenstander på nytt (#39), armer og 3D-våpen, spillregler (#41), historier og rykter (#42), natur og dyr | etter tur |

Ingenting i fase 4–7 starter før portene i fase 2 er grønne på skrekk- og gullsettet.

## 10. Etter kryptnivåene (planlagt 17. oktober mens nivå 8 ble bygd)

### 10a. Krypten lastes nivå for nivå
**Gjort 17. oktober.** Målt i Chromium før: WebGL2 stoppet med `unreachable` da wasm-haugen nådde 4095 MB, og WebGPU tok 2379 MB. Årsaken var mest teksturene, ikke kryptbitene: modellenes bilder sto utpakket i full størrelse samtidig (rundt 1,2 GB RGBA i haugen, 1,6 GB med mipmaps på kortet), mens hele krypten er 24 MB glb. Nå (1) pakkes hvert bilde ut først når rendereren laster det opp og slippes med en gang (`Image.encoded`, `gltf::pixels`), både modellenes teksturer og de 276 bildene til teksturlagene (1,1 GB utpakket); haugen holder bare de kodede filene, i stedet for `createImageBitmap`, som ville krevd egne veier for WebGPU og WebGL2; (2) får teksturene et tak i nettleseren (`delivery::browser_caps`; rundt 420 MB på kortet med mipmaps); (3) hentes kryptbitene nivå for nivå som beskrevet under (`Event::FetchLevel`, `engine.add_level`, `Renderer::add_models`/`drop_models`, `ModelSource::deferred`). Se CLAUDE.md («Modellene er ikke i wasm-fila») for hvordan det henger sammen.

Problemet: med sju nivåer lastet samtidig krasjer Safari på iPhone («A problem repeatedly occurred», minne), og WebGL2-stien stopper med `unreachable` etter at verden er bygd (trolig tom hukommelse i wasm). Hvert nye nivå gjør det verre. Byen trenger ingen kryptbiter før du går ned trappa.

- **Nettlaget** henter ved start bare byens modeller og nivå 1s json (nivådataene er små); kryptbitene (`crypt_NN_*.glb`) hentes først når økta sender `Event::Descend(n)`/går til nivå n, og nivået over slippes når du er ferdig med det. `tools/pack-models.sh` legger bitene per nivå i `dist/models/crypt/NN/`, og `sizes.json` får et felt per nivå, så framdriftsstolpen stemmer.
- **Portene**: `ModelSource` får `load_level(n)`/`unload_level(n)`, og `Renderer` får `add_models`/`remove_models` for en gruppe (i dag får den `StaticScene` én gang). Kjernen (`world::Data`) har alle nivåenes rom, nav og merker fra før (de er små); bare geometrien og kolliderne lastes sent. Kolliderne står i json, så fysikken trenger ikke glb.
- **Utenom nettleseren** (`Embedded`, `snapshot`, testene) lastes alt som i dag; `nettleseren_far_de_samme_modellene` utvides til å kreve at hvert nivås biter ligger i sin mappe.
- **Minnet i wasm**: teksturene fra glb dekodes i dag i wasm før de lastes opp; sjekk om de kan dekodes med `createImageBitmap` i nettleseren og lastes opp rett fra bildet, så wasm-haugen ikke holder dem. Mål med `performance.memory` i `?bench` før og etter.
- **Test**: `tools/browser-test.mjs` med `?webgl` må komme til «motoren er klar» igjen; og `engine.descend()` etterfulgt av lasting av nivå 1 i `tools/map-test.mjs`.

### 10b. Rammeverket som sluttboss (nivå 10)
Modellen `framework` finnes uten rigg («fienden svever»). Den skal lages på nytt etter filmen, så spilleren kjenner den igjen: jernrammer med stive, rette ledd, gylne/magenta rutenett (`magic_…`), kongens segl og krone på hodet, revne sider i leddene, og **skjøtene etter Gorveks hugg**: grove jernsømmer der armene og beina er satt på igjen (kanon: bossen er hjertet som har samlet delene). Rundt 4 m høy, så den får plass i rotunden.
- Forbilde: eieren lager et ark (forfra, fra siden, bakfra) med prompt fra `docs/film/prompter-innstillinger.md` sin beskrivelse av skapningen, i samme stil som monsterarkene i `assets/textures/monsters/ref/`.
- Bygges med `tools/models/monsters.py` (`project_sheet`), rigg med `common.armature`, klippene fra `mob_motion.py` i stilen «tramper og vagger» pluss `slam` og `roar` som alt er definert for bossen, og rykker digitalt i hvile.
- Treffene er jern (`Outcome.sounds`), og slutten (`docs/kanon.md`, «Åpent») avgjøres av eieren før den bygges: beseires med det mønsterboka er.

### 10c. Nivå 9 og 10 (oppdragene)
- **Nivå 9, Overgangen** (`levels/level_09.py`): Sømporten, Sprekkgangen, Speilsalen som glitcher mellom stein og plast (to sett geometri som byttes med et bryter-merke og `Frame.hidden`, eller én tilstand nå og glitchen senere), brohodet, øya med hvilestedet og den hvite lyssøylen, broene over tomrommet (`bridge`-merker uten senking, eller faste `block`-broer), Plastkammeret, Arkivet, Borgfragmentet, Kuben (vridd rom: bygges som vanlig rom nå, vridningen senere), Tvillingsalen med vokteren (elite med nøkkel), nedgangen. Tomrommet er et `backdrop`-rom med gulv langt nede og nett av sømmer (`nowalk` + `Session.safe` løfter deg). Nye materialer: `plastic` (glatt, matt), `magic_seam` (lysende sømmer), begge i `kit_grav.MATS`; teksturlagene `core_plastic` og `glitch_seam` finnes alt i eierens malte teksturer. Fiender: grid → table, skeleton loader → breadcrumb, bueskytterne → tooltip.
- **Nivå 10, Kjernen** (`levels/level_10.py`): spiralen ned mot rotunden, bossen Rammeverket (`boss`-merket med `Kind::Framework`, arenaporter), ingen `down`. Til modellen fra 10b er ferdig står den gamle `framework`-modellen (svever) eller modalen som stedfortreder.


## 11. Redigeringen videre (planlagt 17. oktober)

Dagens redigering (`docs/redigering.md`) velger, flytter, dreier, kopierer, sletter, setter ut fra katalogen, lager rader og usynlige vegger, angrer, spiller fra stedet (P) og lagrer til grenen `redigering`, også nede i krypten. Det som mangler for at eieren skal kunne bygge byen og nivåene selv uten Claude i løkka, i prioritert rekkefølge:

1. **Egenskaper, ikke bare sted.** Et panel for den valgte tingen med tallene (x, y, z, dreining, farge/tint) som kan skrives inn, og for kryptmerkene det som i dag bare kan settes i json: spakens `opens`, portens `id`/`arena`, lappens tekst, kistens `loot`, patruljen til en fiende (punktene tegnes som en strek og dras), lysets farge, rekkevidde og styrke. Dette er det største hullet: alt som har en mening utover posisjonen må i dag gjøres i fila.
2. **Innbyggernes ruter og replikker.** Rutepunktene som håndtak på bakken (som patruljene), ventetid per punkt, og replikknøkkelen fra en liste; nye innbyggere fra katalogen.
3. **Fest og rutenett.** Snapp til 0,5 m og 15° med Ctrl holdt; «sett inntil» mot nærmeste vegg eller nabo (husene bruker alt `EAVES`/`AIR`-reglene, så samme geometri gir et fest); juster en rad med lik avstand.
4. **Boksvalg og grupper.** Dra en ramme for å velge flere; lagre et utvalg som gruppe med eget navn (et tun, en bod med varer), kopier gruppen; dreiing av gruppen rundt felles midtpunkt finnes alt.
5. **Lys i byen** redigeres som i krypten (`Ref::Light` for lyktene og husenes lamper), med forhåndsvisning om natta (T virker i redigeringen).
6. **Kamera og oversikt**: bokmerker (torget, nordporten, hvert kryptnivå), kart ovenfra med klikk for å fly dit, mål mellom to punkter.
7. **Lagring uten nøkkel**: last ned filene som zip når GitHub-nøkkelen mangler, så eieren kan sende dem; og en «Tilbakestill til fila»-knapp.
8. **Nivåene i krypten helt i redigeringen**: rommene og gangene er fortsatt Python (`layouts/layout_NN.py`). Neste steg er å la rommenes polygoner og gangene redigeres som ting (`Ref::Room`, `Ref::Hall`), lagres i `assets/layout/crypt_NN.json`, og la `build_level.py` lese dem derfra. Da bygges et nivå med musa, og generatoren regner mur, hvelv, kollidere og nett. Dette er det største løftet (uker), og tas etter at nivå 1–10 står.
9. **Regler og dom i panelet**: hver regel som brytes peker på tingen og foreslår rettelsen (sett ned, flytt 0,4 m); «godkjent»-dommen kan settes av eieren rett i panelet og skrives til `approvals.json` ved lagring.

Testene følger mønsteret fra steg 3–5: hver regel og hvert verktøy i `src/edit_tests.rs`, og `tools/edit-test.mjs` i nettleseren.

## 12. Alle modellene i den malte stilen (planlagt 17. oktober)

Status (rettet 17. oktober etter kartleggingen): alle 55 teksturlagene spillet bruker er allerede eierens malte bilder, avledet med `derive.py` fra `assets/textures/painted/<lag>/` (manifestet har bare `egen:`-lag; Poly Haven brukes ikke). Meldingen «Lag X bruker fotoskannet materiale» var bare et gammelt navn på kodestien som laster lagbildene, og heter nå «bruker malt bilde». Ett lag var avledet fra et malt bilde som aldri kom i repoet (`shingles2.png`) i stedet for det eieren godkjente; det er avledet på nytt fra det godkjente, og testen `alle_lagene_er_malte` låser at hvert lag peker på sin malte kilde og at manifestet ikke har noe fotoskannet. 27 av eierens malte bilder er godkjent uten å være lag ennå (kloakken, gruva, grotta, borgen, bark, keramikk …); det er steg 5 under. At byen ikke ser helhetlig malt ut, skyldes altså ikke lagene, men bakte teksturer i eldre props, lyset og fargegraderingen; steg 2 og 3 er der virkningen ligger.

Rekkefølgen, billigst og størst virkning først:

1. ~~Byttet av lag~~ **Gjort 17. oktober**: alle lag er malte; testen `alle_lagene_er_malte` (med lista `WAITING` for lag som venter på eierens bilde, tom nå) stopper fotoskannet materiale for godt.
2. **Modellene med bakte teksturer** (ikke lag): `tools/qa catalog` lister hver modell med hvilke teksturer den baker inn. De som baker fotoskannet materiale (eldre props i `street.py`, brønnen, kjerrene, bodene, skiltene) bakes på nytt mot de malte lagene; samme skript, bare kilden byttes. Dommen i `approvals.json` beholdes der bildet ikke endres synlig, ellers legges før/etter til godkjenning.
3. **Stilkontroll som port** (`tools/qa`, del 4): den estetiske vurdereren får stilguidens punkter som rubrikk (ingen konturer, malt flate uten fotostøy, varm by/kald kjeller, mettede farger), og hver modell og hvert lag får en score som vises i katalogen. Under terskelen stoppes modellen før den tas i bruk.
4. **Figurene og armene**: innbyggerne er malte fra TRELLIS; armene (`arms.py`) bruker lagene `linen`, `leather`, `chainmail`, som byttes i steg 1. Vertinnen mangler glb (eieren).
5. **Krypten**: nivå 1–8 bruker `crypt_wall` (malt) i murene, men kloakken, cisternen, gruva, grottene og borgen har ingen egne teksturer (`sewer_brick`, `sewer_floor`, `mine_rock`, `mine_timber`, `cave_rock`, `crystal`, `keep_ashlar`, `keep_floor` er malt og godkjent, men ikke lag). De tas inn som lag og legges i `kit_grav.MATS` per tema, så hvert tema får sin egen stein; bitene eksporteres på nytt per nivå (`build_level.py N --export`).
6. **Naturen** (eng, berg, veier): lagene `meadow`, `mud`, `gravel`, `rock`, `earth` er malt; byttes i steg 1. Trærne og dyrene kommer som nye modeller etter planen i `docs/ny-alt.md` når alt over står.

Målet for hvert steg er det samme: skjermbilder fra de faste kameraene, før og etter, til eieren, og ingen fotoskannet pixel igjen i spillet.

## 13. Inventaret, Skriptoriet, troféene og angrep på avstand (planlagt 18. oktober)

Eieren så på inventaret og Skriptoriet i spillet og vil ha kvaliteten løftet betydelig. Det som er laget 18. oktober (galleri og side om side), er grunnlaget.

### 13a. Felles utstyrskort og sammenligning (brukes både i inventaret og Skriptoriet)
- **Sjeldenhet i farge** (Fristils tokens): vanlig grå, sjelden blå, episk lilla, legendarisk gull, som kant og tag på kortet. Sjeldenheten tas ut av navnet («Gammelt rundskjold (vanlig)»).
- **Små kort med fast bredde i rutenettet:** ikon, navn og sjeldenhet, og ingenting mer. Tallene vises først når kortet er valgt.
- **Én sammenligningstabell** når et kort er valgt: én rad per egenskap med verdien du har, den nye verdien og forskjellen («+0,45») farget etter hva som er bra, med en liten stolpe (`fs-progress`) per verdi. Egenskaper der lavt er bra, vises slik at høyt er bra («Tåler 75 %» i stedet for «Slipper gjennom 25 %»), eller fargen følger det. Like verdier får ikke noe merke («=» bort).
- **Én linje beskrivelse etter kanon** per ting (`craft.rs`), som fiendene har.
- **Ikonene:** lysere bakgrunn som passer kortet, tettere utsnitt og en glorie i sjeldenhetsfargen til eierens malte ikoner kommer (`docs/prompter/ikoner-utstyr.md`).

### 13b. Inventaret (I)
- **Venstre side:** plassene for det du har på (Våpen, Skjold, Rustning, Lys) med ikon, navn og sjeldenhetsfarge. Lys viser lykta med olje og brenntid. Tomme plasser har en svak silhuett og «Finn eller smi en rustning i Skriptoriet».
- **Plassene er filteret** (eieren): klikk på en plass, så viser rutenettet bare det slaget, og det du har på står allerede i sammenligningen. Klikk igjen eller på «Alt» for alt.
- **Høyre side:** rutenettet og sammenligningen med «Bruk <navn>» rett under tabellen, ikke langt nede. Dobbeltklikk eller Enter tar det på, og piltastene går mellom kortene.
- **Liv, utholdenhet og tokens** står øverst som nå; fanen «Komponenter» viser hva den teller.
- Hele dialogen får plass i høyden uten å rulle.

### 13c. Skriptoriet
- **Eieren 18. oktober (revisjon):** «menyen skal ikke dekke hele skjermen i høyden, selv om det er flere gjenstander i kategorien. Da må bruker scrolle. Hvert item skal kun vise en oppsummering, og mer info hvis man trykker på det. Du skal bygge et bedre interface basert på Fristil-komponenter.» Det samme gjelder inventaret og butikkene: lista ruller inne i dialogen, hver rad er en oppsummering (ikon, navn, sjeldenhet, merke), og detaljene kommer først når raden er valgt.
- **Ikke hele vinduets høyde** (eieren): dialogen får fast høyde (rundt 80 % av vinduet, høyst 760 px) med to kolonner. Til venstre står en kompakt liste eller et rutenett av små kort (ikon, navn, sjeldenhet og et merke for «kan smis nå», «mangler 2» eller «har du»), som ruller for seg. Til høyre står detaljene for den valgte oppskriften: stort ikon, beskrivelse, sammenligning med det du har på, og komponentene.
- **Komponentene som en liste med ikoner** (bildet fra galleriet), med «har 2 av 3» og en stolpe. Hvor den finnes, står kort på én linje («Kister i Gruva · elite i Kloakken · Buttonsmeden 12 tokens»). Den lange teksten gjentas ikke under hver komponent; «Bare i krypten» blir ett merke.
- **Sortering:** de du kan smi nå øverst, så de som mangler minst, og det du har nederst og dempet. Filtrene (Alt, Våpen, Skjold, Rustning, Lys) står som nå, med antall.
- **«Smi»** står fast nederst i detaljene, er deaktivert med grunnen («Mangler Card og Badge») og gir en liten smie-animasjon og lyd når den brukes.
- **Liv-, energi- og lysstolpene** skjules mens en dialog står, så de ikke ligger over den.

**Gjort 18. oktober (13a–13c, uten ikonene):** menyene over er bygd slik (`web/main.js`: `toggleInventory`, `openWorkshop`, `openShop`; `tools/ui-test.mjs`). Én linje per ting står i `craft::about`, og hvor komponentene finnes kort i `Source::brief`. Gjenstår: eierens malte ikoner med lysere bakgrunn, og en smie-animasjon som er mer enn gnister over ikonet.

### 13d. Troféene på galleriveggen (eieren: «alt for lav oppløsning, uklare»)
- Årsak: bildene lages i 512 × 512 (`tools/gallery/pack.py`, `SIZE`), med papirstøy og uskarphet over, og nettleseren tar teksturene ned til 1024 (`delivery::browser_caps`).
- Retting: render komponentene skarpere (`tools/gallery/render.mjs` med `deviceScaleFactor` 3), lag platene i 2048 (`SIZE = 2048`), fjern uskarpheten over skriften og la papiret bare ligge i kantene, bak dem inn på nytt (`home.py`), og gi `gallery_*` taket 2048 i `browser_caps`. Anisotropisk filtrering på teksturene, så de er skarpe også på skrå. Sjekk med `snapshot` på 1 m avstand før og etter.
- **Gjort 18. oktober:** platene er 2048 (rendret i 6–12× etter størrelsen, `scaleFor`), komponenten 15rem bred (skriften en tredjedel større), papiret bare i kanten, jpeg 4:4:4, `gallery_*` har taket 2048 (`teksturene_har_tak_i_nettleseren`), og løvkortenes sampler har også anisotropisk filtrering (de andre hadde 8× fra før). Nedlastingen vokste med 1,8 MB (troféene fra 0,7 til 2,5 MB). Før og etter i `assets/catalog/review/gallery_for_etter.jpg`.

### 13e. Angrep på avstand (eieren: vanskelig å se at Korrupt input skyter)
- Årsak: prosjektilet (`Bolt`, modellen `bolt`) står i `model::PLANNED` og finnes ikke, så spyttet er usynlig.
- **Modellen `bolt`:** en glødende klump i `magic`-materiale (fiolett eller giftgrønn), lyser selv og kaster et lite lys, med en hale av partikler bak seg.
- **Varsel før skuddet:** klippet `attack` for inputen får et tydelig trekk bakover og en glød i munnen, en lyd («spytt-lading») rundt et halvt sekund før skuddet, slik at det går an å unnvike.
- **Treffet:** en sprut der det treffer (vegg, gulv eller deg), en lyd og en skadeindikator i kanten av bildet som viser hvor skuddet kom fra.
- **Infopanelet om fienden** sier «Skyter på avstand» for slagene med `bolt` (inputen, Toast-frosken).
- Test: et skudd fra en input lager en synlig instans hvert bilde til det treffer.
- **Gjort 18. oktober:** modellen `bolt` (`tools/models/bolt.py`, 428 trekanter), ladelyden og gløden i munnen (0,6 s før skuddet), halen og det grønne lyset, spruten og `spit_hit`, `Event::Hurt` med den røde buen, `FoeView.ranged`, testen `markoren_synes_og_treffet_sier_hvor_den_kom_fra` og `snapshot --bolt`. Igjen: et eget trekk bakover i klippet `attack` for inputen (`mob_motion.py`).

### 13f. Siktekortet midt på skjermen (eieren 18. oktober)
- Kortet om det du sikter på (fienden, en innbygger, en ting du kan bruke) er for vanskelig å se ute ved høyre kant. Det flyttes midt på skjermen, et lite stykke over stolpene for liv og utholdenhet, som ett kort (`#target` og `#foe` på samme plass, aldri samtidig). Kolonnen til høyre har bare minikartet og tastene. `tools/layout-test.mjs` sjekker at kortet ikke dekker siktepunktet eller stolpene på åtte skjermstørrelser. **Gjort 18. oktober** (`#aimed` i `web/index.html`; under 1100 px slutter kolonnen til høyre over kortet).

### 13g. Trubaduren i Vertshuset (forarbeid 18. oktober, uten eierens modeller)
- **Gjort:** plassen (krakken ved ildstedet, `seated` i `town.json`), de sittende klippene `sit` og `play` (`folk_motion.py`), kjevebeinet `jaw` i `folk3d.py` (`JAW`, `MOUTH`), sangens lydstyrke som data (`tools/music/voice.py`, `tavern_voice.json`) og munnen i motoren (`Command::Voice`, `Npcs::set_voice`). Prøvd på skriveren bygd som stedfortreder (`snapshot --model`).
- **Gjenstår (eieren):** trubadurens ark og glb fra TRELLIS (`docs/prompter-folk-3d.md`, så `python3.13 tools/models/folk3d.py troubadour`), og lutten (`item3d.py`), som henges på hoftebeinet (den ligger i fanget; hendene står i hoftenes rom i `play`). Munnen har ikke noe innside (TRELLIS lukker leppene), så den åpnes som en strekning av underleppa; et mørkt munnhule-lag kan legges på når modellen er der.

### 13i. Balansen: spillet er for lett (eieren 18. oktober)
- **Gjort:** fiendene har 50 % mer liv (`TOUGHNESS` i `mob_kind.rs`, alle slag, eliten og bossene i samme forhold); kampen ellers som før. Senga koster 10 tokens (`SLEEP_PRICE`, avslag uten råd). Boka gir fullt liv, full utholdenhet og 2 minutter olje, én gang per komponent per tur. Brønnen og døden er som før.
- **Gjort:** første gang ned i krypten åpnes lappen fra mester Ansgar (`crypt::run::ANSGAR_TEXT`): målet, H hjem gjennom virvelen, lykta, døden og hvilestedene. Den står i journalen og kommer ikke igjen (`SaveData.lore`).

## 14. Komponentene: mer å finne, og bytte hos kjøpmannen (eieren 18. oktober, i kø)

Eieren spilte hele spillet gjennom uten å få smidd mer enn første trinn av utstyret: komponentene som trengs, kommer for sjelden.
- **Mer å finne:** gå gjennom hva kistene, eliten, bossene og fiendene gir (`drops.rs`, `craft::source`, kistene i nivåfilene og `RECIPES`), og øk det så en vanlig gjennomspilling gir nok til omtrent hele andre trinn og noe av tredje, og alt med litt leting (hemmelige vegger, sjeldne kister). Regn det ut med gjennomspillingsboten (`gjennomspilling`): hva den har samlet ved bunnen, og hva den kunne smidd. Dobbelt så mange kister kommer også (krypt-hjelperen).
- **Bytte hos kjøpmannen:** Buttonsmeden (eller en egen kjøpmann på torget) bytter «billige» komponenter mot dyrere: for eksempel tre vanlige mot én sjelden av samme tema, med en liten avgift i tokens. Kursene i data (`craft.rs`), en egen fane i butikken med de samme kortene som Skriptoriet, og en test for at byttet virker og ikke kan misbrukes (ingen evig løkke som gir tokens).

## 15. Nivåredigeringen: bygg krypten selv (eieren 18. oktober, etter neste prod-utgivelse)

Eieren synes nivåene er for kjedelige og generiske, og vil bygge dem selv i en egen, levende redigering. Svarene fra grillingen:
- **Byggeklosser på rutenett:** ferdige biter i den malte stilen som smetter i hverandre på et rutenett på 4 × 4 m med høyde i trinn på 1 m: rom i flere størrelser, ganger, hjørner, kryss, T, blindgang, trapper (hele og halve etasjer), dobbel høyde, buer, dører, broer, sjakter, vann. Murene, kolliderne, lysene og navigasjonsnettet følger med bitene, så alt som settes sammen kan gås og ser ferdig ut.
- **En egen side: kart ovenfra med 3D ved siden av.** Rommene legges ut på et 2D-kart (etasje for etasje), og 3D-vinduet viser resultatet med en gang. Én knapp spiller nivået fra starten med fiender og alt, og Esc tar deg tilbake dit du var.
- **Alt innholdet:** rom og ganger; mekanismer med koblinger (spak til port, nøkkel, snarvei, hemmelig vegg, feller, heis — koble med musa); fiender (patrulje, bakhold, elite, boss), kister med valgt innhold, lapper og bonuser; lys og stemning (fakler, lysestaker, tema per rom, tåke, lyder).
- **Reglene underveis:** redigeringen sier fra med en gang om noe ikke henger sammen (dør som ikke går noen vei, spak uten port, rom du ikke når, nivå uten trapp ned) — de samme reglene som testene. Utkast huskes i nettleseren, og nivåene lagres på grenen `redigering` som byen; jeg slår dem inn.
- **Bitene:** jeg lager et første sett per tema (gravkjelleren, kloakken, gruva, borgen, kjernen) i Blender med murene vi har, eieren godkjenner dem på godkjenningssiden, og flere kommer når eieren savner noe.
- **De ti nivåene begynnes helt på nytt:** de gamle står til de nye er klare, så fjernes de, og eieren bygger alle fra blankt.
- **Rekkefølge:** etter neste prod-utgivelse og del 14. Planen i detalj (formatet for nivåene, bitene, sammenføyningen, navigasjonen, lagringen, redigeringens skjermbilde) skrives før arbeidet starter og vises eieren.

## Kritiske filer
- Nye: `tools/qa/` (raster, glb, porter, board, golden, vurderer-oppsett), `tools/lib/`, `assets/specs.json`, `assets/approvals.json`, `docs/kvalitet/{stilguide,rubrikker,laerdom,ytelse}.md`, `web/film/`, crates under `crates/` (domain, ports, app, adapters).
- Endres: `sheet2model/cli.py` (portene før finish/rig, ingen glb ved feil), `sheet2model/{geom,texture,report,config}.py`, `tools/models/kit.py` (overganger, dørmodul, `group()`), `tools/models/{houses,tavern,workshops,home,sacred,fortress,streetlife}.py`, `tools/crypt/{kit_grav,build_level,leak_check}.py`, `tools/textures/derive.py`, `src/application/session.rs` (delt), `src/domain/world/{town,mod}.rs`, `src/adapters/gltf.rs`, `src/adapters/wgpu/{mod.rs,shaders/world.wgsl}`, `web/main.js`, `src/tests.rs`, `src/architecture.rs`, `.github/workflows/web.yml`, `CLAUDE.md`, `docs/ny-stil.md`.

## Verifisering
- `cargo test` og `pytest` grønne lokalt og i Actions; arkitekturtesten feiler om domenet får en forbudt avhengighet (prøves med en bevisst feil).
- `python3 -m tools.qa check` på skrekksettet: hver modell stryker i riktig port med bilde; gullsettet passerer; `qa regress` < 10 min.
- `cargo test` feiler for glb/lag uten grønn `.qa.json` og blir grønn bare via `qa promote`.
- Vurdereren avviser et bevisst dårlig eksempel og godkjenner et av eierens godkjente.
- Ytelse: bench-tekst fra eieren før/etter hver rettelse; `snapshot --stats`-budsjett i CI.
- Filmene: animatikken godkjent før lagene lages; P-F grønt; filmen spiller i Chromium uten skjerm (`browser-test.mjs`) og går over i spillet ved sørporten.
- Hver fase avsluttes med push til preview og ett godkjenningsark i fast format til eieren.
