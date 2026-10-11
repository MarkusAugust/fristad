# hode_x: Buttonsmedens hode, «skulptur med kode»

Kode: `tools/models/hode_x.py` (`python3.13 tools/models/hode_x.py ut.png [--fast] [--blend]`).
Ark: `shots/hode_x/`.

## Fremgangsmåte
1. MakeHuman-grunnmodellen (mann, gammel, muskuløs), hode + hals + skuldertopp skåret ut med uv.
2. Subdivisjon 2, så `sculpt`: myke gaussiske deformasjoner rundt landemerker (brynbue, glabella, overlokk,
   kinnben, kjeve/hake, stor nese, pannefurer, ører).
3. Grunnlag for hår og skjegg: avstandsfelt (huden + tykkelse per sone + ellipsoider for skjeggmassen),
   marching cubes, glattet (`hair_base`).
4. Lokker (`Locks`): flate bånd med flat underside og buet overside, rund tupp, uv rot→tupp; kurvene legges
   med kontrollpunkter og snappes med stråler utenfra mot en kjerne (`Snapper`).
5. Huden: fargelag med soner, sot og krumning × hudflis (`paint_skin`).
6. Øyne: kuler med iris fra eyes.jpeg rad 1.

## Logg
- 16:32 start. 16:58 skulpturen gir brynbue, kinnben, nese, pannefurer. Første hår/skjegg-forsøk kjører.
- 17:25 **runde_1.png**: hele kjeden virker (skulptur, grunnlag, lokker, maling, øyne, ark med forbildet).
  Galt: grunnlaget lager store flak under halskuttet (feil fortegn i avstandsfeltet der kuttet er åpent);
  skuldrene er for brede; hårfargen er blek og rosa (flisen tonet feil); lokkene står som fjær/tiara forfra
  og er for tynne; skjegget er tynt og glissent; huden er blek; lyset er flatt og bakgrunnen blå.
  Neste: klipp feltet under halsen, egen fargerampe for håret (flisen bare som detalj), tykkere lokker,
  mer skjeggmasse, varmere hud, grå bakgrunn som forbildet.
- 18:05 **runde_2.png** (t5): halsen kuttes nå rent med tre plan og lukkes (`close_cut`), håret har egen fargerampe,
  lokkene følger et retningsfelt. Fortsatt galt: små øyne (smale spalter), ansiktet leses som en blek, realistisk
  gammel mann og ikke som en stilisert figur; toppen av håret er en glatt hjelm med for få lokker; skjegget er
  parallelle remmer (kost), ikke store buede klumper; løse lokkstumper stikker ut. Stoppet av koordinatoren:
  «digital leire» var klart nærmere forbildet.

## Lærdom (nyttig videre)
- **Lokk-kurver:** kontrollpunkter i et sfærisk/polart rom rundt en kjerne konvergerer feil (vifte/skjell bak på
  hodet). Et **retningsfelt** som integreres over flaten (`hair_flow` + `flow_lock`: opp fra panna, bak over toppen,
  ned i nakken, projisert på tangentplanet, glattet med forrige retning) gir mye mer naturlig fall. Hvert steg snappes
  med stråle utenfra mot kjernen; stopp lokken når strålen bommer eller punktet hopper (ellers rette stenger i lufta).
  Grunnlaget må ikke ha en bratt kant ved hårfestet, ellers bryter lokkene der.
- **Tverrsnitt:** flat underside + buet overside, kantene trukket *ned* mot flaten (ikke opp, da blir det fjær/bånd).
  Lokkene må være brede (5–6 cm på toppen) og få; smale, tykke lokker blir «pølser».
- **Grunnlag av avstandsfelt** (hud-avstand minus tykkelse per sone, smooth-min med ellipsoider, marching cubes) virker,
  men krever et lukket hode: et åpent halskutt gir feil fortegn og store flak. Python-løkke med `find_nearest` er treg
  (2–3 min ved 4 mm) – bruk et lite rutenett eller begrens avstanden.
- **Hårfarge:** bruk flisen bare som luminans (strå) og legg egen rampe: rødbrun base, mørk rot (v) og kanter (|u−0.5|),
  lys midte. Flisens egne farger ga rosa/oransje.
- **Hud:** hudflisen er for flekkete og mettet; bland ~60 % flat farge inn. Fargesoner som fargelag per hjørne
  (gauss rundt landemerker) + krumning fra Laplace fungerer, men må bakes til tekstur for spillet.
- **Skulptur på MakeHuman:** gaussiske deformasjoner rundt landemerker gir former, men hodet forblir realistisk;
  øyelokkene lukker seg lett når brynet trykkes ned – åpne lokkene eksplisitt. Stiliseringen må være mye kraftigere enn
  man tror for å lese i 512 px.
- EEVEE virker uten GPU her (programvare-EGL), ~15–40 s per visning.
