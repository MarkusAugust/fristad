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
