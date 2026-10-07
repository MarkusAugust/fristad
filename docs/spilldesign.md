# Fristad som spill – forslag

Eieren (7. oktober 2026): «Det er ikke noe morsomt å utforske dokumentasjonen på denne måten. Komponentene virker klistret tilfeldig rundt. De må være en del av spillmekanikken: nivå 1, løse oppgaver, låse opp komponenter, hus, til slutt et skjema som låser opp temageneratoren. Vær kreativ, kom med de mest ekstreme forslagene.»

Dette er forslagene, fra et samlende premiss til de mest ekstreme. Alt kan bygges i motoren vi har (installasjoner, props, flytur, panel, nettlaget), men flere av dem krever nye systemer: NPC-er, inventar, lagring av framgang, en manusmotor for oppdrag.

## Premisset: byen uten designsystem

Du kommer til Fristad den dagen designsystemet forsvant. Byen står, men alt som var Fristil er borte: skiltene er grå bokser med Times New Roman, knappene er nettleserens stygge standardknapper, lenkene er blå og understreket, skjemaene er rå `<input>` uten luft, fargene er borte (hele byen i gråtoner og «unstyled HTML»-estetikk), og innbyggerne står og peker på ting som ikke virker. Porten er låst fordi knappen som åpner den ikke har noen stil.

Spillet er å **hente Fristil tilbake, komponent for komponent**. Hver gang du gjenoppretter en komponent, forvandles bydelen den hører til: fargene kommer tilbake, typografien, skyggene, radiusene. Sluttilstanden er byen slik den ser ut i dag. Da blir det synlig hvorfor et designsystem finnes, og framgangen er dramatisk og visuell, ikke en poengsum.

Dette er det bærende forslaget. De andre er lag oppå det.

## Spillsløyfa

1. **Finn** – et oppdrag peker deg til en komponent som er ødelagt (et skilt du ikke kan lese, en dør som ikke åpner seg, en innbygger som ikke får svar).
2. **Forstå** – du må se på dokumentasjonen for komponenten i panelet (E) og svare på / gjøre noe som viser at du forsto den: velge riktig variant, riktig tilstand, riktig token.
3. **Bruk** – du bruker komponenten i verden (F): knappen åpner porten, bryteren tenner lysene, progress-stolpen fyller brua, dialogen tar en telefon.
4. **Lås opp** – komponenten er gjenopprettet (bydelen får farge), du får den i verktøybeltet, og noe nytt åpner seg: et hus, en gate, et oppdrag, en innbygger som vil snakke.

Verktøybeltet nederst på skjermen er bygd av de ekte komponentene (badges for det du har, en progress-stolpe for akten, toast når noe låses opp). Spillets eget grensesnitt er dermed også dokumentasjon.

## Fem akter

| Akt | Sted | Komponenter | Nøkkel |
|---|---|---|---|
| 1. Ankomst | Plakaten og porten | button, link, breadcrumbs | Porten åpnes først når du har fikset knappen (velg primary, trykk F). Brødsmulene i brosteinen lærer deg å gå. |
| 2. Torget | Torget, oppslagstavla, kiosken | alert, badge, tag, switch, progress, spinner, dialog, toast | Byen er mørk til du finner bryteren. Telefonkiosken ringer: dialogen gir deg første oppdrag fra ordføreren. |
| 3. Bydelene | Østre, Vestre, Nord, Sør | tabs, card, accordion, table, avatar, divider, tooltip, skeleton, connection-status | Hvert hus har en innbygger som er «sin» komponent (se NPC-ene under). Du får nøkkel til neste hus når du har hjulpet den forrige. |
| 4. Søknadskontoret | Kontoret sør for Østre gate | input, select, textarea, checkbox, radio, fieldset, label, help-text, error-text, file-upload, search, heading | Søknaden er «bossen»: hvert felt krever en komponent du har gjenopprettet, og vedleggene er ting du har samlet (saksnummeret fra kortet, fylket fra tabellen, fødselsattesten fra Gorvek). Valideringen er fienden: feil er ekte `.fs-error-text`. |
| 5. Temageneratoren | Tokenparken | tokens (farger, avstander, radius, skrift) | Innvilget søknad gir deg nøkkelen til verkstedet i parken. Der setter du sammen ditt eget tema av tokenene du har plukket i parken, og **hele byen bytter tema live**. Temaet kan eksporteres som ekte CSS (`[data-theme="mitt"]`), så spillet ender i noe du kan bruke på jobben. |

Etter akt 5 åpner veiene til Linelord, Streamlord og Cobble (se «De tre rikene»).

## Innbyggerne er komponentene (NPC-er)

Komponentene «klistret rundt» blir personer med ærend, og dialogen deres er den ekte komponenten:

- **Fru Alert** står på trappa og roper varsler (neutral/info/success/warning/danger etter humøret). Oppdraget: finn ut hva hun egentlig advarer mot.
- **Spinneren** på masta er mannen som aldri blir ferdig; han stopper først når du fikser forbindelsen (`connection-status`), og da får du vite hva han ventet på.
- **Skjelettet** er spøkelset av et hus som ikke er lastet ennå; byggeplassen er der det skal stå. Når du har levert tegningene (en card med riktig variant), reiser huset seg over natta (bokstavelig: du må sove til neste dag, T).
- **Tooltip-mannen** ved kontoret snakker bare når du står helt inntil (hover), og sier én setning om gangen.
- **Toast** er Gorvek: han dukker opp, sier noe høyt og forsvinner etter fem sekunder. Oppdraget hans er en jakt.
- **Dialogen** er telefonkiosken: ordføreren ringer, og du må svare riktig (bekreft/avbryt) under tidspress.
- **Tabs** er huset med tre dører og tre søsken som alle påstår at døra deres er den riktige.
- **Table** er buss-sjåføren som kjører deg (flytur) dit tabellen sier, men bare om du sorterer rutene riktig.
- **Avatar**-ene er tre venner som ikke klarer å bli enige om hvem som står først i stabelen.

Innbyggerne har tilstander som er komponentens tilstander (disabled = sover, invalid = sint, success = fornøyd), så du lærer tilstandene ved å se dem på folk.

## Tokens som valuta

Design-tokens er bokstavelig talt penger og materialer: fargebrikker (`color-accent-fill`), avstandssteiner (`spacing-4`), radiusskiver, skriftvekter. Du finner dem i Tokenparken, i hus, i skuffer, hos innbyggerne. Gjenoppretting av en komponent koster de tokenene den faktisk bruker i CSS-en (knappen koster `accent-fill`, `spacing-2`, `radius-1`, `font-size-m`). Da lærer du hva komponenten er bygd av, og hvorfor tokens er delt. Temageneratoren til slutt er å bruke tokenene du har spart til å lage ditt eget tema.

## De tre rikene (etter søknaden)

Veiviserne på torget peker alt til Linelord, Streamlord og Cobble. De blir tre ekspansjoner med egne regler:

- **Cobble** – brosteinens rike: rutenettet. Alt er grid og layout. Du legger brostein som er `grid-template-areas`, og gater som ikke går opp i 12 kolonner raser sammen.
- **Linelord** – linjehøydens rike: typografi. Tårnene (skriftstørrelsene) er der. Et land der alt står i linjer; feil linjehøyde og teksten kolliderer fysisk.
- **Streamlord** – flytens rike: flexbox og bevegelse. Ei elv der innholdet flyter, og du må styre `justify`/`align` med slusene.

## De ekstreme

1. **Legacy-kjelleren.** Under torget ligger den gamle CSS-en: en prosedyrisk grotte av tabellayout, `!important`, `z-index: 99999`, inline-stiler og jQuery-spaghetti. Monstrene er disse tingene (en `float: left` som drar deg sidelengs, et `position: fixed`-spøkelse som følger deg, `!important`-golemen som ikke lar seg overstyre). Våpnene er komponentene: knappen er sverdet, alert er skjoldet, spinner fryser tida, toast kaster du. Bossen er Spaghetti-monsteret. Hver etasje du renser gir en bydel tilbake fargene. Dette kan være akt 3 i stedet for «gå rundt og hjelp folk», og det er klart det morsomste å bygge.

2. **Kontraststormen.** Et vær der fargene vaskes ut; bare komponenter som holder WCAG AA-kontrast er synlige. Du må bytte til høykontrast-variantene for å finne veien, og lære hvorfor `forced-colors` finnes. Et annet vær: **skjermleser-tåka**, der verden forsvinner og du navigerer bare med Tab/Enter og det komponentene sier med `aria-label`. Tilgjengelighet som overlevelsesmekanikk.

3. **Byggemodus.** Etter søknaden får du inspektørhatten: du kan sette ut komponenter på flater i byen (en level-editor) og komponere ekte sider på husveggene. En URL til komposisjonen din kan deles. Byen blir en lekegrind for prototyping, og dokumentasjonen blir et verktøy.

4. **Flerspiller (Railway).** Innbyggerne er andre spillere. Søknaden må fylles ut av et lag (hver har sitt felt). Det finnes en «code review»-boss der de andre stemmer over løsningen din. Chat med Fristils `dialog` og `toast`.

5. **Dagsyklusen som framgang.** Hvert nivå er en dag. Om natta kan du ikke se komponentene du ikke har gjenopprettet (de lyser ikke). Du må rekke oppdragene før mørket, og lysbryteren er det første du må fikse for å overleve natta.

6. **Byen husker deg.** Framgang, tema, komposisjoner og hvem du har hjulpet lagres (localStorage først, konto seinere). Byen hilser deg med det du har gjort, og innbyggerne refererer til det. En speedrun-klokke for de som vil.

## Hva som må bygges (grovt)

- **Spilltilstand og manus**: en `Quest`-modell i `domain/` (mål, trinn, betingelser, belønning), lagret i localStorage via nettlaget. Oppdragene skrives som data, ikke kode.
- **Ustylet tilstand** for hver komponent: et ark til per komponent rendret *uten* Fristil-CSS (bare nettleserens standard), og en «grå» palett for bydelen til den er gjenopprettet. Dette er billig med arkpipelinen vi har.
- **NPC-er**: figurer (Blender, som dyrene) med enkel animasjon, dialog som ekte `dialog`/`toast`/`alert`-ark, og en tilstandsmaskin.
- **Verktøybelte og journal** i nettlaget, bygd av Fristils komponenter.
- **Tokens som gjenstander**: props du kan plukke opp (F), inventar, og kostnader per komponent hentet fra CSS-en (vi kan lese hvilke `--fs-*`-variabler hver komponent bruker automatisk).
- **Temageneratoren**: tokens → `[data-theme]`-CSS → paletten i motoren (vi har allerede `Tokens::blend` og palettplassene), eksport som fil.
- Legacy-kjelleren, rikene, flerspiller og byggemodus er hver sin store utvidelse etter kjernen.

## Anbefalt rekkefølge

1. Premisset (byen uten designsystem) + spillsløyfa + akt 1–2. Dette gir umiddelbart et spill med start, mål og synlig framgang.
2. NPC-ene i akt 3 og søknaden som boss i akt 4.
3. Temageneratoren (akt 5) som belønningen som faktisk er nyttig.
4. Legacy-kjelleren som den store, morsomme utvidelsen.
5. Resten etter hva som fenger i spilltestene.

## Retningen (eieren, 7. oktober)

Eieren valgte: fjern alle komponentene som er plassert rundt i byen. Komponentene vinnes tilbake i **Legacy-kjelleren**, et roguelite under torget, og kjøpes i **bodene rundt torget** av NPC-er. Hvert oppdrag bygger et **arsenal** av designsystemkomponenter som settes sammen til kraftigere utstyr, og mer avansert utstyr krever mer avanserte skjemaer.

### Fiendene

- **Trashmobs: de korrupte komponentene.** Komponenter som siden 2026 ble blandet med for mye JavaScript- og forretningslogikk, og har mutert: en Knapp som også validerer, henter data og viser en modal (den slår tre ganger og åpner en dialog du må lukke); et Input som husker alt du har skrevet og hvisker det tilbake; en Tabell som sorterer deg; et Kort med 40 props som bytter form hvert sekund; en Accordion som folder seg rundt deg; en Spinner som aldri stopper og suger tid; en Toast-sverm. De ser ut som komponentene, men feil: varianter blandet, tokens i feil plass, tekst i Comic Sans, `!important`-arr. Du kjenner dem igjen fordi du har sett de riktige i bodene. Når de dør, faller de fra hverandre i det de var bygd av: ren komponent (loot) og en klump forretningslogikk (søppel du kan selge).
- **Epokene** nedover: tabellayout, floats, `!important`, Bootstrap-hydraen, jQuery-spaghettien, div-suppa, og nederst 2026-mutantene.
- **Siste bossen: Rammeverket.** Designsystemet selv, som krever at du skriver React, og vil ta livet av deg for å ha vurdert andre språk. Et tårn av JSX som skyter `<Component prop={...} />`-prosjektiler, kaster deg i en «re-render»-løkke, og påstår at ingenting virker uten det. Du vinner ikke med et annet rammeverk (det gjør den bare sintere), men med det Fristil faktisk er: **frittstående** HTML og CSS. Hver komponent du har vunnet og smidd, brukt som ren markup, tar fra den en lagbit, til bare tokenene står igjen. Da får du nøkkelen til temageneratoren.

### Spillsløyfa (roguelite)

- Byen er navet og varig: bodene, boka (dokumentasjonen), smia (søknadskontoret), temagenerator (parken). Brønnen på torget er nedgangen.
- Kjelleren er prosedyrisk per nedstigning. Dør du, mister du det du bar, men beholder en andel tokens og alt som er skrevet i boka. Komponenter du har vunnet én gang kan kjøpes i boden etterpå.
- Utstyret følger komponentenes betydning (knapp = våpen, alert = skjold, progress = liv, spinner = fryse tid, toast = kast, tooltip = avslør svakhet, tabs = bytte sett, accordion = sekk, breadcrumbs = vei tilbake, link = teleport, divider = vegg, switch = feller, badge/tag = buffer, avatar = følgesvenn, connection-status = varsling).
- Sammensetning er Fristils egne mønstre, og skjemaet er smia: label + input + help-text + error-text = Felt; felt × n + fieldset + button = Skjema; card + avatar + badge = Profil; dialog + knapper = Bekreftelse. Valideringen er om smiinga lykkes.

### Milepæl 1 (ferdig)

Brønnen leder ned til én prosedyrisk etasje, én fiendetype (en korrupt knapp), knappen som våpen, liv (progress), tokens, og død som sender deg opp med tokenene i behold.

## Beslutningene (eieren, 7. oktober, etter milepæl 1)

Eieren: utstyret skal ikke se ut som komponenter, kampen skal være animert, treff skal synes, kjelleren skal være variert. «Spillmekanikkene må være mye mye mye bedre.» Det skal føles som en blanding av **The Witcher 2** og det gamle **Might and Magic** (ikke Heroes): tungt, taktisk nærkampspill i førsteperson, med utforsking av varierte kjellere og en by som nav.

1. **Førsteperson**, med synlige armer og våpen.
2. **Skjelettanimasjon i motoren.** Skinnet glTF med rigger og Blender-actions. Stive deler som instanser er ikke godt nok.
3. **Angrep:** venstre mus lett hugg (kombo på tre), hold for tungt hugg, høyre mus blokk med skjold, parade i vinduet, Shift+retning unnamanøver med usårbarhet, utholdenhetsmåler som styrer alt.
4. **Treff:** ingen frys når *vi* treffes, men kameraskjelv, kameraet dreies bort og tilbake, og rødt flash langs kantene. Når *fienden* treffes: kort frys (80 ms), blink hvit, kastes bakover, biter av krøllparenteser og piksler spruter, skadetall i Plex Mono, helsestolpe på fienden.
5. **Fiendens angrep** varsles med animasjon: den gjør seg klar (reiser seg, gaper) før den stuper. Kan blokkes, pareres eller unngås.
6. **Fiendetyper** først: korrupt knapp (nærkamp), korrupt input (skyter strenger), korrupt modal (sperrer gangen, tank), tooltip-sverm (små, raske), tabellen som miniboss. De skal se **mye mer monsteraktige** ut enn dagens knapp.
7. **Utstyr** bygges av komponenter, men ser ut som ekte våpen og utstyr: sverd, skjold, øks, dolk, bue. Tematisk kobling er ok (divider = klinge, button = grep, card = skjoldplate, badge = skjoldbule, progress = bue, tag = pilspiss, tooltip = dolk).
8. **Skjemaet er verkstedet:** oppskriften er et ekte Fristil-skjema i Skjemakontoret, feltene er plassene, validering er kravene. Avansert utstyr krever fieldset, file-upload og search.
9. **Statistikk:** skade, fart, rekkevidde, blokk, ett spesialtrekk per våpen, sjeldenhetsgrader, ingen tilfeldige rulleverdier.
10. **NPC-ene** snakker med **vanlige snakkebobler**, ikke designsystemkomponenter. Knappesmeden (våpen og reparasjon), Skjemakontoret (bygging), Apoteket (drikker), Gorvek lærer trekk. De bor i husene rundt torget.
11. **Fem etasjer per nedstigning**, hver en epoke (2026 forretningslogikk, 2019 klassekomponenter, 2014 jQuery, 2009 tabellayout, 1999 blink), trapp ned bak en rusten port med nøkkel fra minibossen, Rammeverket nederst. En nedstigning er 20 til 30 minutter.
12. **Generator:** rom og ganger (BSP), to høydenivåer med trapper og avsatser, hemmelige rom, kister, tema per epoke (murstein med mose, kloakk med gjørme, serverrom med lysrør, katakomber med stearinlys, lava i dypet), varierte lys (fakler, lykter, glødende mose, flakkende lysrør, lysskjær fra sprekker).
13. **Nøkler** fra minibosser og låste kister, porten er et rustent gitter som sveives opp, låste dører til valgfrie rom.
14. **Død:** mister det du plukket opp, beholder det du hadde på deg og tre firedeler av tokenene. Vunne komponenter er låst opp for alltid.
15. **Ingen erfaringspoeng.** Framgang er komponenter, oppskrifter, trekk, og faste oppgraderinger for tokens.
16. **Dokumentasjonen** i boka: å lese dokumentasjonen for en komponent gir **liv tilbake**.
17. **Lyd** skal inn (WebAudio).
18. **Lagring** i nettleseren; en nedstigning kan ikke lagres midt i.
19. **Rekkefølge:** skjelettanimasjon i motoren først, så kampfølelsen (armer, sverd, treff, fiendeangrep, lyd), så generatoren med trapper og tema, så butikker og bygging, så flere fiender og bossene.

### Status

Gjort: skjelettanimasjon (1–2), kampen (3–5), det riggede monsteret (6, første versjon), kjelleren med fem etasjer, trapper, avsatser, tema per epoke, porter og nøkler (11–13), død med tapt bytte (14), innbyggerne med snakkebobler (10), lager, kister, oppskrifter som skjema, butikker, trening, boka som gir liv (7–9, 15–16), lyd (17) og lagring (18). Gjenstår: flere fiendetyper og bossene (6), fjerne komponentene som står rundt i byen (eller la dem dukke opp etter hvert som de vinnes), og mer realisme på innbyggere og monstre.
