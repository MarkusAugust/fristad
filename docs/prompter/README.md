# Promptene (ryddet 19. oktober)

Alt eieren skal lage med bildegenerator, TRELLIS.2 og Veo, samlet her. Det som er laget, og metodene vi har gått bort fra (ark med tre retninger, maling av ark på formen, `sheet2model`), er slettet; git-historikken har dem.

**TRELLIS.2** (https://huggingface.co/spaces/microsoft/TRELLIS.2, Resolution 1536, Decimation Target 500000, Texture Size 4096) tar ETT bilde og dikter selv sidene og ryggen. Vi bruker teksturen den gir som den er. Derfor er hver prompt ett bilde:

- **Folk og tobeinte fiender:** ett kvadratisk bilde forfra i A-stilling, armene rundt 30° ut fra kroppen med luft imellom, beina litt fra hverandre, tomme hender, ingenting foran kroppen.
- **Gjenstander:** ett bilde tre kvart forfra og litt ovenfra, hele tingen med luft rundt.
- **Dyr og fiender uten armer:** ett bilde fra siden, litt skrått, alle bein synlige og fra hverandre, halen ut fra kroppen.
- **Trær og busker:** stammen og greinene fra TRELLIS (nakent tre), løvet som malte bladkort i Blender.
- Alt: lys grå bakgrunn, mykt jevnt lys forfra, ingen skygger, malt stilisert som World of Warcraft (modern) og Fable, ingen svarte konturer, og kraftige former (tynne spyd, antenner og hår blir borte).

Hver prompt sier hva fila skal hete, hvor den havner og hva den erstatter. Send filene her; jeg gjør dem spillklare og viser deg resultatet før det tas i bruk.

## Rekkefølgen (det spillet savner mest først)

1. **`ikoner.md`**: de 19 ikonene til våpnene, skjoldene, rustningene og lykta. Alle er stedfortredere (rendret modell).
2. **`trellis-gjenstander.md`, nr. 1–4: nøkkelen, lampeolja, helsedrikken og boka.** Nøkkelen står i `model::PLANNED`.
3. **`teksturer.md`**: de fem underkjente teksturene (brosteinen, grunnmuren, den røde pussen, kryptpussen, kjernesteinen) og det krypttemaene mangler (slim, cisternestein, gruve- og grottegulv, flisgulv, fanestoff, statuestein, forgylling, hvit plast).
4. **`trellis-monstre.md`, nr. 1–6**: resten av fiendene til nivå 5–10, som i dag låner modellene fra gravkjelleren og kloakken.
5. **`trellis-dyr.md`**: sauen og kua (venter i `model::PLANNED`), så hest, geit, hund, høne og katt.
6. **`natur.md`**: stammene til eik, bjørk, gran, furu og hassel, to steiner, furubaret og to fliser.
7. **`trellis-gjenstander.md`, nr. 5–24**: pynten i krypten og byen.
8. **`trellis-folk.md`**: Torgils, som låner Gorveks figur.
9. **`film.md`**: Veo-klipp for seks innstillinger som står som stillbilde, og to nye malerier.

Musikken er ferdig (alle sporene i `assets/music/game/` og `assets/music/film/`), innbyggerne, Rammeverket, Navbaren, Dropdownen, Tvillingvokteren og bueskytteren er bygd fra TRELLIS; promptene for dem er slettet.
