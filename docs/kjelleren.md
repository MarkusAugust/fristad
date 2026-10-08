# Kjelleren v3: håndbygde rom, hvelvede ganger og vindeltrapper

Den gamle kjelleren var et rutenett av flate bokser: rom og ganger like
brede, flatt tak, vegger som tapet. Det føltes som et kart, ikke et sted.
Den nye bygges slik spill som Dark Souls, The Witcher og Diablo bygger
sine: av **håndlagde rom** (set pieces) og et **modulsett av ganger**, satt
sammen av en generator som følger en plan for flyten.

## Arkitekturen

- **Rom er hele modeller fra Blender** (`tools/models/dungeon.py`), med
  hvelv, søyler, gallerier, trapper, møbler, fakler og lys. Hvert rom
  skrives med en json-fil ved siden av modellen (`assets/models/dungeon/`):
  fotavtrykket i celler à 4 m, gulvhøyden og takhøyden per celle (til
  fiendene og takprøven), sokler for dører (celle og retning), kanter
  inne i rommet som er stengt, kollidere (bokser og skall), lys, og
  steder: fiender, kister, eliten, start, port, trappa opp.
- **Gangene er moduler** på én celle: rett (to varianter), hjørne, T, kryss
  og blindende. Gangen er 2,8 m bred med tønnehvelv (vederlag 2,4 m,
  topp 3,8 m), gurtbuer av hogd stein, sokkelsteiner, heller i gulvet, og
  krysshvelv med diagonalribber der ganger møtes. Massen rundt er fjell,
  så alt ser hogd ut.
- **Ubrukte dører** i et rom mures igjen (`dng_seal`): en tilmurt bue
  med løse steiner foran.
- **Vindeltrapp mellom etasjene**: utgangsrommet har portgitteret foran
  et trappetårn som vinder seg 12 m ned til neste etasje, og der kommer
  du ut i trappefoten (`dng_landing`). Ingen flate sjakter lenger.

## Flyten i en etasje

1. **Startrommet** (første etasje: døra opp til krypta; ellers trappefoten).
2. **Hovedveien**: 5–7 rom mot utgangen, med varierte rom: søylehall med
   krysshvelv, cisterne med vann og gangbaner, galleri i to høyder (du kan
   hoppe ned fra galleriet), kapell med apsis og sarkofager.
3. **Sidegrener** til skattkamre med kister.
4. **En løkke**: en gang som binder et senere rom til et tidligere, så du
   kan gå i ring og fiendene kan komme bakfra.
5. **Utgangen**: eliten med nøkkelen vokter portrommet; nederst
   står Rammeverket i bossarenaen.

## Lys og stemning

Lyset er komponert per rom i Blender (fakler i holdere, glør i fat,
stearinlys i nisjer), ikke spredt jevnt. Gangene får fakler i noen moduler
og er mørke i andre, så rommene trer fram som lyse øyer.

## Tema per etasje

Etasjene deler arkitekturen og skifter materialer, lys og pynt etter
epoken: tørr hogd stein med fakler (2026), kald grå stein og
blålig lys (2019), kloakk med vann og lykter (2014), katakomber med
gravnisjer og stearinlys (2009), og lavadypet (1999).

## Senere

- Egne modulsett per tema (kloakkrenner, gravnisjer, lavagrotter).
- Skygger fra punktlys, så lyset ikke går gjennom veggene.
- Snarveier som låses opp fra innsida (dører med slå).
- Fall i kløfter og broer.
