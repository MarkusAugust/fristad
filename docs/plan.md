# Plan: Fristad på rett kurs – kvalitet før alt, ren kode, målt ytelse, to filmer

## Context

Eieren har underkjent det meste som er laget de siste dagene: figurene (for små, hode-i-hode, smeltede hetter), brønnen og ambolten, kryptnivå 1 (39 hull, tomme rom, harde overganger), kryptens dør, kirkas stil, tønnerader, svevende gårder. Kartleggingen viser at årsakene sitter i måten vi bygger på, ikke i enkeltting:

- **Ingen kontroll før det dyre steget.** sheet2model måler IoU på vokselfeltet, ikke på det ferdige nettet, og skriver glb + Cycles-ark *før* dommen (brønnen gikk inn med `ok: false`, IoU 0,19). Blender-eksportene kontrollerer ingenting. Ingen teksturkontroll finnes. CI kjører ingen tester.
- **Ingen regler for sammensetning.** Flater møtes uten list/sokkel/gesims; hver vegg starter uv på 0 (søm i hvert hjørne); dørbladene strekkes 2× (`kit.door`, `sacred.arched_leaf`); `kit_up` bygger en steinflate 20 cm foran kapellets pussmur; nisjene i krypten skjæres aldri ut (`dress()` kaster returverdien); innredningen er ~5 500 håndskrevne desimaltall; bondegårdene står på høyeste punkt; `audit` godtar én fot på bakken.
- **Oppbygget kode.** ~25 modellnavn i Rust peker på slettede modeller (fiender, dyr, våpen, `npc`); `tools/reconstruct` og `sheet2model` dupliserer sju moduler; `people.py`, `town.py` og `houses.py` skriver de samme filene; hjelpere som `box`/`prism`/`lathe`/`barrel` finnes i 3 kopier; `session.rs` (2 383 linjer, ~65 felt) gjør ti ting som hører i domenet; arkitekturtesten er et tekstsøk som lett lures, og domenet leser json med `include_str!` og parser `serde_json`.
- **Ytelse uten måling.** 9 fps på eierens Mac (GPU 223 ms) mot 36 på master; vi har ingen måling i nettleseren, lavapipe er ubrukelig som mål, og teksturene er 1024² (0,5 teksel/mm) mot kravet 1 teksel/mm.

Eierens beslutninger (11. okt.): én økt, én maskin, ingen skyøkter; dreieark med 8 visninger; ingen overstyring av kontroller; estetisk vurdering av en uavhengig Claude-vurderer med fast rubrikk mot eierens godkjente ting og en skreven stilguide, som lærer (feil → regler i koden, stilguiden vokser, terskler strammes) og gjelder modeller, teksturer, scener, filmene og nettlaget; åpnings- og sluttfilm (detaljer i del 8).

Målet: ingenting når eieren uten å ha bestått automatiske tekniske og estetiske kontroller; det billige kjøres før det dyre; feil rettes ved roten i felles kode og blir regler.

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
- **Budsjett som kontroll:** `snapshot --stats` i CI på de åtte posisjonene; feiler over 500k trekanter / 300 kall eller over grunnlinjen + 10 %. Eierens bench-tall fra Mac og jobb-PC er fasiten for fps (mål: standard ≥ 60 fps på vanlig maskin, avansert ≥ 60 på god), loggført i `docs/kvalitet/ytelse.md` per bygg.

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
| 0 (½–1 dag) | `?bench`, grafikknivå «enkel» | bench-tekst fra Mac og jobb-PC |
| 1 (2–3 dager) | Arkitektur og rydding (del 2): crates, `session.rs` delt, sletting, `tools/lib`, tester i CI | samme spill, mindre kode; arkitekturtest i CI |
| 2 (4–5 dager) | `tools/qa`: rasterer, rammeverk, P0–P7, P-T, byggesettsportene, skrekk- og gullsett, `promote`/`.qa.json`/Rust-test; stilguide, rubrikker og vurdereren | P0/P-T-svar på eksisterende ark og teksturer; **nye prompter (del 6) og filmmanusene** leveres i starten av fasen, så eieren lager dreieark mens portene bygges; animatikkene når manus er godkjent |
| 3 (2–3 dager) | Ytelsesrettelsene (del 7) i den rekkefølgen bench viser; budsjett i CI | bench nr. 2 og 3 |
| 4 (4–5 dager) | Byggesettets regler (del 5): overganger, dørmodul, utkraging, `group()`, terreng/fundament, krypt-moduler; byen og husene eksportert på nytt gjennom portene; kirka som steinkirke; kryptens kapell som pilot, så resten av nivå 1 | godkjenningsark for byen, tre hus, Vertshusets skjenkestue, kapellet; så nivå 1 |
| 5 | Grunnkroppen (6–8k, fast uv, 18 bein), så Button Wight og Buttonsmeden fra dreieark gjennom alle portene; så resten av monstrene og figurene én og én | godkjenningsark per figur/monster (8 vinkler + gang) |
| 6 | Filmene ferdige når lagene er levert og godkjent | filmene i preview |
| 7 | Nettlaget i Fristil (8b) med Esc-menyen (#40); gjenstander på nytt (#39), armer og 3D-våpen, spillregler (#41), historier og rykter (#42), natur og dyr | etter tur |

Ingenting i fase 4–7 starter før portene i fase 2 er grønne på skrekk- og gullsettet.

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
