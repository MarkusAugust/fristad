# Plan: raskere og feilfri rekonstruksjon fra forbilder

Bestilt av eieren 11. oktober 2026. Metoden i `tools/reconstruct` (se `docs/rekonstruksjon.md`) virker, men hver figur tar rundt 5 minutter per forsøk, og feilene oppdages først når et menneske ser på mellomvinkelen. Målet er at verktøyet selv finner og retter feilene, at teksturen alltid havner riktig, og at én figur bygges ferdig med skjelett på under to minutter uten runder.

## Feilene vi har sett, og årsaken

| Feil | Figur | Årsak |
|---|---|---|
| To figurer oppå hverandre | Pilegrimen, Vaktmannen, Bonden | Visningene er rettet inn bare etter topp, bunn og midten av silhuetten. Står figuren litt skjevt eller holder en stav, blir profilen forskjøvet sideveis, og dybden fra to visninger smelter sammen til to kropper. |
| Dobbelt ansikt | Fiskerkona, Vertinnen | Profilene i hodearket er tegnet i en annen høyde eller dreining enn visningen forfra, og innrettingen tar ikke høyde for det. |
| Redskap i begge hender | Vertinnen, Gorvek | Den manglende profilen speiles, og alt i den følger med. |
| Løse flak og skår | Gorvek, Vertinnen | Rester etter dybde som ikke stemmer mellom visningene, og etter utskjæring av redskapet. |
| Mørk søm over kinnet | Vertinnen | Teksturen prosjiseres fra en visning som ikke stemmer med formen der. |
| Hånda er en klump | alle | Forbildet har 100 px over hånda. Formen kan ikke leses derfra. |
| Strukket topp | alle | Ingen visning ser ovenfra. |
| Strekk ved håndledd og forkle i gange | smeden | Automatiske vekter på et nett uten kjente ledd. |

Fellesnevneren: **vi stoler på at forbildene stemmer med hverandre, og vi bygger alt fra bunnen for hver figur.** Det gjør hver figur til et nytt eksperiment.

## Tre endringer som løser det

### 1. Mal i bunnen: én felles kropp som tilpasses forbildet

I stedet for å smelte visningene til en helt ny form, **tilpasses en felles grunnkropp med kjent topologi, uv-kart, skjelett og vekter** til forbildet (non-rigid ICP mot silhuettene og dybden, med glatthetsstraff). Det er slik spill gjør det: alle figurene deler ett nett.

- **Kjent uv-kart:** teksturen bakes alltid til samme kart. Ansiktet har alltid samme plass og tetthet, og det blir ingen Smart UV og ingen sømmer på nye steder for hver figur.
- **Kjent skjelett og vekter:** riggen følger med malen. Ingen automatiske vekter, ingen strekk i forkle eller håndledd, og hendene har ferdige fingre som kan gripe.
- **Kjente deler:** hodet, halsen, hendene, føttene og klær som skall (kjortel, forkle, skjørt, hette, pels) er egne lag på malen. Da vet verktøyet hva det ser i forbildet, og «redskap i hånda» er en egen kategori i stedet for noe som må gjettes fra fargen.
- **Hull i silhuetten** (mellom arm og kropp) og **ting som henger fritt** er ikke lenger et problem: malen har allerede atskilte armer.
- Dybdeestimatet brukes fortsatt, men som mål å tilpasse seg mot, ikke som form. Da slipper vi flak og skår.

Malen lages av det vi har: MakeHuman-kroppen (rigget, god topologi) forenklet til rundt 8 000 trekanter, med formmål for kjønn, alder, vekt og høyde, og skallene fra `kit`/`people.py` for klærne. Hodet beholder tettere nett (rundt 3 000) så ansiktet fra hodearket får plass.

### 2. Innretting som kan måles og rettes automatisk

Visningene rettes inn mot malen, ikke mot hverandre, med en **målbar feil per visning**:

- **Silhuett-IoU** per visning mellom malens silhuett (i gjeldende positur og tilpasning) og forbildets.
- **Fotometrisk feil** i mellomvinkel: forbildet projisert fra én visning sammenlignes med renderen fra naboen. Doble ansikter og «to figurer» gir høy feil og oppdages uten et menneske.
- **Per-visning frihetsgrader:** sideveis forskyvning, skala, dreining om loddrett akse (±15°) og vipp. Disse optimaliseres mot IoU, slik at en profil som er tegnet litt dreid, blir modellert som dreid i stedet for å ødelegge formen.
- **Speiling med maske:** redskap, poser og alt usymmetrisk i profilen merkes (farge + at det ikke finnes forfra på den siden) og tas aldri med over. Verktøyet rapporterer hva det maskerte, så det kan sjekkes.

Innrettingen kjører uten Blender, i numpy, og tar sekunder. Først når IoU og den fotometriske feilen er under terskel, bakes teksturen.

### 3. Teksturen bakes direkte i malens uv, med sømfri blanding

- **Projeksjon per teksel i numpy** (rasterisering av malens uv → posisjon og normal), uten Cycles-baking. Det tar sekunder i stedet for minutter, og gir samme resultat hver gang.
- **Synlighet per visning** med z-buffer fra malen, og vekt = normal·kamera · avstand fra silhuettkanten · (1 − fotometrisk feil lokalt). Visninger som ikke stemmer et sted, mister vekt der.
- **Poisson-blanding i uv** i stedet for to frekvensbånd: gradientene fra den beste visningen beholdes, fargen justeres mykt så det aldri blir sømmer, heller ikke der visningene har litt ulikt lys.
- **Toppen og undersiden** fylles med malens standardtekstur for hår, hud og klær (flisene fra `assets/textures/people/tiles/`) i stedet for å strekke nærmeste visning.
- **Lyset i forbildet dempes:** et grovt lysestimat (lav frekvens av luminansen mot normalens vinkel) trekkes fra, så spillets lys ikke dobles. Detaljlyset i penselstrøkene beholdes.

## Automatisk kontroll (ingen venter på et menneske)

Hver bygging skriver `rapport.json` med:

- IoU per visning, fotometrisk feil i to mellomvinkler, antall løse komponenter, antall hull, flaket areal, trekanter, teksel per cm på ansiktet, og om redskapet ble funnet og lagt i riktig hånd.
- **Terskler** i `sheets.json` per oppsett. Under terskel: verktøyet prøver automatisk de vanlige rettelsene i rekkefølge (bredere maske rundt redskapet, drei profilen, bruk bare én profil, slakk dybden) og tar den beste. Går ikke det, feiler byggingen med et bilde av akkurat det som er galt, i stedet for å levere en dobbel figur.
- **Mellomvinkel-arket** lages alltid, men bare som dokumentasjon.
- **Regresjonstest:** `tools/reconstruct/tests/` kjører smeden, Vertinnen og Pilegrimen fra bufrede dybdekart (ingen nettverk, ingen Blender) og sjekker at alle tallene holder seg over terskel. Den kjøres før hver endring i verktøyet, så en rettelse for én figur ikke ødelegger en annen (slik `depth_margin` gjorde for Gorvek).

## Fart

| Nå | Mål |
|---|---|
| Dybde: 40 s per ark | bufres per bilde; 0 s ved gjenbygging |
| Smelting: 30–60 s | tilpasning av mal: 10–20 s |
| Smart UV + Cycles-bake: 2–3 min | numpy-projeksjon i fast uv: 10 s |
| Rigg + vekter: 40 s | følger malen: 0 s |
| Render av ark + GIF: 60 s | bare på forespørsel |
| **Rundt 5 min, og 2–4 runder med et menneske** | **Under 2 min, uten runder** |

Alle 13 figurene kan da bygges på nytt på under en halvtime når noe i verktøyet forbedres, i stedet for en natt.

## Rekkefølge

1. **Målinger først** (½ dag): `rapport.json`, IoU, fotometrisk feil, komponent- og hulltelling, og regresjonstesten på de tre figurene med dagens metode. Da ser vi tallene for dagens feil og kan måle forbedringene.
2. **Innretting med frihetsgrader og maskert speiling** (1 dag): retter «to figurer», dobbelt ansikt og redskap i begge hender. Dette bruker dagens form og tekstur, så gevinsten kommer raskt.
3. **Numpy-projeksjon og Poisson-blanding** (1 dag): sømmene og tempoet i teksturen.
4. **Malen** (2–3 dager): grunnkroppen med formmål, lag for klær, hodet, hendene, skjelettet og vektene; tilpasningen mot silhuett og dybde; alt rigget uten automatiske vekter. Dette er det største steget og det som gir hender med fingre og ren bevegelse.
5. **Lysdemping og fylling av topp og underside** (½ dag).
6. **Bygg alle 13 på nytt** med det nye verktøyet, og sammenlign rapportene mot dagens.

Steg 1–3 gjør dagens løype trygg og rask. Steg 4 er det som løfter kvaliteten til nivået eieren vil ha, med ekte hender og bevegelse uten strekk.

## Det som ikke løses her

- Forbilder som mangler en visning (ovenfra) gir fortsatt gjettede flater der, men nå fylt med flisene i stedet for strekk. Et forbilde ovenfra per figur (én ekstra Gemini-rute) ville løst det helt.
- Ansiktsuttrykk og mimikk krever formmål i ansiktet på malen. Det er mulig senere, siden malen har kjent topologi.
