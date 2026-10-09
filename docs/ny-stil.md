# Ny stil: malt by, skarpt bilde, 60 fps på jobb-PC

Bestemt med eieren 9. oktober 2026. Erstatter retningen «maksimal detalj og realisme».

## Mål

- **Utseende:** stilisert og håndmalt, uten konturer (i retning World of Warcraft og Valheim). Varm, mettet by og en kald, grønnlig og korrupt kjeller der Fristil-komponentene gløder.
- **Bildekvalitet:** klare teksturer i høy oppløsning, rundt 1 teksel per millimeter nær. Ingen uskarphet, spøkelser, utsmøring, støy, korn, taggete kanter eller flimmer.
- **Ytelse:** stabilt 60 fps i 1080p på en jobb-PC med integrert grafikk (Intel Iris Xe) i standard grafikk. Nedlastingen spiller ingen rolle.

## Renderer, standard grafikk

| Del | Valg |
|---|---|
| Kantglatting | MSAA 4×. TAA, bevegelsesvektorer og skarping fjernes. Gress og blad bruker alpha-to-coverage, så de ikke flimrer. |
| Lys | Én dynamisk sol, enkelt himmellys (ovenfra og nedenfra) og lykter om natta. Formene får lys fra normalkartene i teksturene. |
| Skygger | Én skarp kaskade ut til rundt 40 m, som toner bort. Lenger ut gir bakt okklusjon dybden: mørke hjørner, takskjegg og smau, uavhengig av hvor sola står. |
| Effekter som beholdes | Dag og natt med glødende lykter og vinduer, regn, snø og vær, avstandståke og røyk fra pipene, vind i trær og gress. |
| Effekter som fjernes | SSAO, volumetrisk tåke med lysstråler, TAA og våte flater med speiling. |
| Teksturer | Mipmaps og anisotropisk filtrering 16×. Flisbare, malte teksturer og trim-ark (striper med bjelker, kanter og listverk) i stedet for unike bakte atlas. |
| Budsjett per bilde | Rundt 500 000 trekanter for alle pass til sammen, og under 300 tegnekall. |

**Avansert grafikk** (velges av spilleren, aldri av seg selv) slår på:
- lengre sikt
- MSAA 8× og høyere oppløsning
- SSAO uten støy

## Modellene

- **Detaljene males i teksturen.** Stein, planker og takstein males inn i stedet for å være egne flater. Formene som synes i silhuetten beholdes som geometri: bjelker, takskjegg, piper, karmer og trapper.
- **Budsjett:** et vanlig hus får 3 000–10 000 trekanter, og et hus på torget med interiør høyst 25 000. I dag har Vertshuset 220 000.
- **Innredning:** bare de fire husene på torget har innredning, som i dag.
- **Figurene** blir malte i samme stil, med forenklede og litt overdrevne former. Fotoansiktene fjernes.
- **Kjelleren og monstrene** bygges om etter byen, i samme stil.

## Teksturene

- **Hvem lager hva:** eieren lager teksturene med en bildegenerator etter prompts fra Claude, og legger dem i `assets/textures/painted/<navn>/albedo.png`. Normal, ruhet og høyde avledes med `tools/textures/derive.py`.
- **Krav til hvert bilde:**
  - 2048 × 2048
  - flisbart
  - sett rett forfra
  - uten lys og skygge malt inn
  - samme malte stil og palett på alle

## Rekkefølgen

1. **Pilot: Vertshuset og torget.**
   - Ny renderer (MSAA, én skyggekaskade, uten TAA, SSAO og volumetrisk tåke).
   - Vertshuset bygd om med malte teksturer innenfor budsjettet.
   - Ny brostein på torget.
   - Eieren godkjenner utseendet på skjermbilder før noe annet bygges om.
2. **Resten av byen:** husene, muren, porthuset, kirka, borgruinen og torget.
3. **Figurene og dyrene.**
4. **Naturen:** trær, gress og steiner.
5. **Kjelleren og monstrene.**

## Måling

- `sh tools/stats.sh` skriver trekanter, tegnekall og instanser fra faste kameraposisjoner (sørporten, Storgata, torget, Vertshuset, borghaugen, utenfor muren, oversikt og kjelleren).
- Grunnlinjen før ombyggingen (avansert grafikk, alle pass):

| Posisjon | Trekanter, alle pass | Kall | Trekanter, hovedpasset |
|---|---|---|---|
| Sørporten | 14,8 mill. | 694 | 3,5 mill. |
| Storgata | 13,9 mill. | 741 | 2,6 mill. |
| Torget | 12,8 mill. | 717 | 2,0 mill. |
| Vertshuset | 13,3 mill. | 647 | 2,3 mill. |
| Borghaugen | 10,7 mill. | 642 | 3,0 mill. |
| Utenfor muren | 11,2 mill. | 708 | 2,7 mill. |
| Oversikt | 10,9 mill. | 606 | 2,9 mill. |
| Kjelleren | 0,19 mill. | 14 | 0,10 mill. |

Skyggekaskadene står for rundt tre firedeler av trekantene.

## Forkastet underveis

- **Flere detaljnivåer med meshoptimizer:** husene er bygd av tusenvis av små, lukkede biter. Uten lov til å gå over sømmene kom den ikke under halvparten av trekantene, og med lov rev den hull i veggene.
- **Klyngeforenkling i flere nivåer:** den ga bare 50–80 % per nivå. Med malte detaljer og få trekanter trengs ikke detaljnivåer for husene.
