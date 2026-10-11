# Figurene i ny stil (logg)

Oppgave: piloten med smeden, Vertinnen og Gorvek (`tools/models/folk.py`), samme modellnavn som før.

## Steg 1 (14:15): oversikt
- `folk.py` fra forrige hjelper: forbildene måles (`Sheet`, `HeadSheet`), MakeHuman-kroppen rettes inn (`retarget`) og formes etter silhuettene (`fit_rows`), hodet strekkes (`fit_head`). Overlegget `fit2_smith.jpg` viste at kroppen passer godt.
- Gjenstår: hår og skjegg, klær og redskap, teksturen (projeksjon), uv, vekter, klipp og eksport.

## Steg 2 (15:20): smeden bygges hele veien
- Rettet: landemerkene i ansiktet er hjørner i nettet (de var utdaterte etter `align_torso`, så hodet ble strukket 2,2 ganger). `contain_head` trekker skallen innenfor hårsilhuetten, `fit_profile` tilpasser panne, bryn og nese i profil. Venstre profil og bakfra registreres mot speilede silhuetter i hodearket (`mirror_align`), med felles skala.
- Håret og skjegget: det visuelle skroget av de fire hodevisningene (superellipse per snitt, marching cubes, Taubin-glatting). Hår mot hud skilles med GrabCut sådd av geometrien (`hair_masks`). Håret ligger minst 8 mm utenpå huden, og kanten glattes og legges ned mot huden. Desimert til rundt 3 200 trekanter. Lokkene og høylysene kommer fra den malte teksturen og normalkartet.
- Teksturen males av forbildene (`paint`): uv-rasterisering, dybdebuffer per visning, lavt bånd blandet mykt og detaljene fra den beste visningen. Hodearket brukes for hodet og kroppsarket for resten. Det ingen ser, fylles fra nærmeste synlige punkt. Hendene får hudfarge og okklusjon per hjørne.
- Forkleet er et panel (`apron`) etter omrisset forfra og fra siden, vektet mellom hoftene og lårene (`skirt_weights`), aldri på armene.
- Hammeren er modellert og malt med tre og jern. Hånda dreies om håndleddet så knokelinja følger skaftet i forbildet, og skaftet går gjennom neven.
- Desimering i pass: resten 5 200, hodet 3 000 og hendene 1 300. Smeden er nå 14 918 trekanter, med én tekstur på 2048².
- Kjør: `FOLK_TMP=... python3.13 tools/models/folk.py smith --build ut.glb`. Kladdeskript i kladdemappa (`w/cmp.py`, `w/close.py`).
- Neste: øynene og hårkanten på nært hold, så Vertinnen (skaut, skjørt, krus i venstre hånd, nøkler) og Gorvek (langt hår, pels, øks).

## Steg 3 (16:10): Vertinnen og Gorvek
- Vertinnen: skjørt (`skirt_shell`: superellipse per rad fra silhuettene, aldri smalere fra knærne og ned, beina inni fjernet), forkle utenpå skjørtet, krus med jernbånd og hank gjennom venstre neve (`tankard`), og nøkkelknippet på beltet (`keyring`, stivt på hoftene). Skautet med knuten kommer fra skroget av hodearket.
- Gorvek: ulvepelsen er skroget av kroppsarket i skulderbåndet, med grå pels skilt ut etter fargen (`fur_mantle`, åpen foran halsen) og vektet uten armene (`drape_weights`). Det lange håret kommer fra hodearket uten det grå, ligger minst 8 mm utenpå pelsen og er vektet fra hodet til brystet. Skjeggøksa har omrisset av bladet målt i forbildet (`axe`, `blade`).
- Rettet: profilene maler ikke ansiktet (det ga doble øyne på tinningene), redskapet stenges ute av forbildene med håndmålte rektangler (`exclude`), hendene får hudfarge fra kinnene, med myk overgang etter vekten, bunnen av skroget (kuttet hals) fjernes, og normalkartet regnes etter at høyden er fylt ut over kantene på uv-øyene.
- Trekanter: smeden rundt 13 750, Vertinnen 14 500 og Gorvek 13 500.
- Neste: legge modellene i assets/models/npc, bygge spillet, snapshot, kontaktark av gangen og hvile, og nærbilder av hendene.

## Stoppet (16:40): eieren syntes resultatet så «helt forferdelig» ut

Arbeidet er stoppet etter beskjed. De tre gamle modellene i `assets/models/npc/` er lagt tilbake (git checkout), så spillet er uendret. De nye ligger bare i kladdemappa.

### Hva fremgangsmåten var
MakeHuman-kroppen ble rettet inn etter leddene og silhuettene i kroppsarket. Hodet ble skalert etter landemerker og trukket inn i hårsilhuetten. Hår, skjegg, skaut og pels ble laget som et visuelt skrog av silhuettene (fire ortografiske visninger, superellipse per snitt, marching cubes). Hår og hud ble skilt med GrabCut og farge. Teksturen ble projisert fra de malte forbildene med dybdebuffer per visning.

### Grunnleggende begrensninger (kan ikke rettes innenfor denne metoden)
1. **Et visuelt skrog er ikke en skulptur.** Fire silhuetter sier ingenting om formen mellom dem eller om fordypningene. Håret blir en glatt «hjelm» eller ballong med riktig omriss, ikke 10–40 lokker med volum, overlapp og skygger. Lokkene finnes bare som maling og normalkart. Fra skrå vinkler og i lys blir det flatt og «papiraktig», og normalkartet laget av luminansen gir krøllete papir i stedet for lokker.
2. **Projeksjon av 2D-maling på en annen form gir uunngåelige feil.** Forbildene er malt med lys, skygge og perspektivjuks som ikke stemmer med noen 3D-form, og de fire visningene stemmer heller ikke med hverandre (AI-genererte ark). Der visningene møtes, blir det sømmer eller spøkelser (doble øyne, profilnese på kinnet, hammeren malt på låret), og der ingen ser, blir det utsmurt. Det kan dempes, men aldri fjernes helt.
3. **MakeHuman er et realistisk hode og kropp.** Stiliserte proporsjoner (stor nese, tunge bryn, store hender, kraftig kjeve) får man bare delvis ved å strekke akser. Ansiktet blir derfor en realistisk form med en stilisert tegning over: det samme problemet som eieren kalte groteskt sist, bare mykere.
4. **Grensen mellom hår og hud** er en kant der et skall går ned i huden. Med desimering og rutenett blir den fillete, sagtakket eller med sprekker (barten, nakken, pelsen). Glatting hjelper, men en skulptert lokk har en designet kant. Det har ikke et skrog.
5. **Pelsen** som skrog blir harde flak og plater, ikke myke tuster, fordi silhuetten bare gir omrisset.

### Feil som kunne rettes (men ikke løser det grunnleggende)
- Streken over brystet hos Vertinnen (toppen av forkleet sto 1 cm ut fra kroppen).
- Den vannrette streken under skjegget (overgangen mellom hodearket og kroppsarket).
- Sømmen i profil ved kinnet (blandingen mellom forfra og profil).
- Hvite biter i barten og fillete kanter (desimeringen etter at kanten var glattet, og fyllingen av det ingen ser).
- Hudfargen på hendene og den mørke fargen i neven.
- Budsjettet (13 500–14 500 trekanter) og riggen, klippene og grepet (knyttneve rundt skaftet) virket. Det samme gjelder innlastingen i spillet.

### Hva som trolig trengs i stedet
- **Hodet, hendene, håret og pelsen modellert som stiliserte former**, ikke avledet av silhuetter: en egen stilisert hodemal (enkle flater, stor nese og bryn) med rent uv-kart, og lokker som egne, avrundede rør eller bånd langs kurver (10–40 per figur), med flisa fra rot til tupp. Pelstuster som myke, overlappende blader.
- **Maling i uv-rommet etter regler og fliser** (som `docs/prompter-folk.md` sier), og bare midten av ansiktet projisert forfra, mykt blandet. Ingen projeksjon fra profilene eller bakfra.
- Eller en ferdig stilisert grunnmodell med fri lisens (CC0), om det finnes en. MakeHuman bør ikke være utgangspunktet for hodet.
- Det som kan gjenbrukes fra `folk.py`: målingen av forbildene (`Sheet`, `HeadSheet`, ledd og silhuetter), retarget av kroppen, desimering i pass, uv og maleren (`paint`), skjørt, forkle og redskap (`apron`, `skirt_shell`, `tankard`, `axe`), vektene (`skirt_weights`, `drape_weights`), grepet (`grip_frame`, `turn_hand`) og eksporten med klipp.
