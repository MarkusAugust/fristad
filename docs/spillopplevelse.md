# Spillopplevelsen: kartlegging og forbedringer (17. oktober)

Gjennomgang av hva en ny spiller møter, uten nye bilder, ark, stemmer eller
kanonendringer. Prioritert: øverst det som stopper eller forvirrer
spilleren mest.

## Kartlegging

| # | Område | Hva spilleren møter | Alvor |
|---|--------|---------------------|-------|
| 1 | Hva gjør jeg nå? | Etter filmen står du ved sørporten uten noe mål. Ingenting sier at krypten er på kirkegården utenfor nordporten, at trappa ned er det blå lyset, at bossen åpner veien, eller at det du bærer bare er trygt i byen. | Høy |
| 2 | Lappene | Lappene og bøkene i krypten vises én gang i en boble og er borte. Det finnes ikke noe sted å lese dem igjen, og det du har lest huskes ikke fra en lagring. | Høy |
| 3 | Lagring | Nettleseren lagret bare når lageret endret seg. Hvile ved et alter, et nytt nivå eller en lest lapp ble ikke lagret, og det dypeste nivået (portalens mål) ble glemt mellom øktene. | Høy |
| 4 | Byfolkets replikker | Flere replikker peker til «borgen», «borghaugen» og «lavaen», som ikke finnes lenger: borgruinen er fjernet, og krypten ligger under kirka. Torgils sa «Gorvek» om seg selv i konstanten, men kanon sier at Gorvek aldri er i byen. Ingen replikk sa hvor krypten er. | Middels |
| 5 | Kampkjensle | Hitstop, blink, rusk, skadetall, kameraspark, lyd etter materiale og balanse som brister finnes allerede. Det som mangler mest er at spilleren lærer parade og unnamanøver; tastelinja i kamp sier det, og Torgils sier det nå også. | Lav |
| 6 | Balanse | Tallene i `mob_kind.rs` er ikke målt mot en ekte spiller ennå. Uten spilltesting fra eieren er det gjetting å endre dem, så de står. | Venter |
| 7 | Lappenes innhold | Tekstene i lappene står i nivåutleggene i `tools/crypt/layouts/`, som en annen hjelper eier i natt. Om de bygger på kanon (munkene, Ansgar, forseglingen, hjertet), må sjekkes der. | Venter |

## Gjort

1. **Målet på skjermen** (`application/goals.rs`, `Event::Goal`): én linje
   øverst i panelet til venstre. I byen før du har vært nede: «Gå ut
   nordporten til kirkegården. Døra i krypten der fører ned.» Nede: nivået
   og navnet, og om det er bossen, trappa ned (det blå lyset) eller bunnen
   som gjenstår, pluss en påminnelse når lykta brenner ned eller du bærer
   noe (trygt først i byen, H). Oppe igjen: hvor mange komponenter du har i
   lageret og det dypeste nivået. Sendes bare når teksten endres.
2. **Journalen (J)**: målet og alle lappene og bøkene du har lest, med
   tittel og tekst, i rekkefølgen de står i krypten (`Session::journal`,
   `web/index.html` `#journal`). J står i tastelista.
3. **Lagring av framgangen**: `SaveData.deepest` (det dypeste nivået, så
   portalen og målet husker det) og `SaveData.lore` (titlene du har lest).
   Nettleseren lagrer når målet eller journalen endres (nytt nivå, ny lapp,
   bossen slått) og ved hvert hvilested, ikke bare når lageret endres.
4. **Byfolkets replikker etter kanon** (`world/cast.rs`): ingen borg eller
   lava; krypten er under kirka, ut nordporten; vakta og byfolket sier
   veien; Torgils lærer bort blokk og unnamanøver og nevner Gorvek bare som
   legenden; byfolket forteller om Gorvek som hogg beistet i stykker over
   byen i regnet og ikke ble sett siden; skriveren nevner mester Ansgar som
   gikk ned med de siste sidene; apotekeren minner om lampeolja og
   hvilestedene.

Testen `malet_og_journalen_viser_veien_og_huskes` (src/tests.rs) dekker
målet i byen og nede, at det bare sendes ved endring, at en lest lapp havner
i journalen, og at nivået og lappene overlever en lagring. Replikkene
sjekkes av `kanon_ingen_moderne_ord_i_verden`.

## Står igjen

- Balansen (punkt 6): trenger eierens spilltesting.
- Lappenes tekst (punkt 7): ligger i kryptens filer, som en annen hjelper
  eier; sjekkes mot kanon når krypten er ferdig for natten.
- En markør i verden for målet (pil eller lys) krever grafikk og er ikke
  gjort; målet er tekst.
