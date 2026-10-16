# Åpningsfilmen: manus

Eieren 16. oktober: en **selvstendig introfilm** av malte stillbilder av personer og scener, plassert i dybde i flere lag, så de beveger seg mot hverandre med parallakse når kameraet flyr. Spilleren flyr fra hendelse til hendelse gjennom én sammenhengende malt verden, med tekst som svever stort og tydelig i rommet, og en dramaturgisk oppbygning. **Spillets grafikk er ikke med i filmen.** Bare helt til slutt toner bildet over i spillet, der du står utenfor sørporten.

Avklart med eieren:

- **Én sammenhengende verden, lest som et billedteppe** (Bayeux-teppet): å fly sidelengs (mot høyre) er at tiden går, og å dykke innover er å gå inn i en hendelse. Inn i rom går kameraet gjennom vinduer og dører, og ned i krypten gjennom jordlagene. Dag og natt skifter med årene langs veien.
- **Kameraet:** en blanding av sidelengs glid langs lagene og dykk innover i dybden, valgt etter dramaturgien.
- **Tekstene henger i rommet** mellom lagene, store og tydelige. Kameraet flyr forbi og gjennom dem, og de står lenge nok til å leses.
- **Lite liv i bildene:** personene puster og svaier svakt, lys blafrer, røyk og støv driver. Ellers stillbilder.
- **Lengde og avspilling:** 1,5–2 minutter (rundt 110 s), i sanntid i nettleseren (three.js), hver gang, med «Hopp over».
- **Bildene:** de lagdelte settene fra første runde (`assets/film/opening/`) brukes der de passer. Det som mangler, lager eieren lag for lag: bakgrunn uten personer, personer og ting hver for seg på ensfarget grønn bakgrunn. De fem maleriene fra 16. oktober (personene malt inn) er bare forbilder.
- **Historien:** kongen krevde at alt skulle lages med ett rammeverk og virke likt overalt, inkvisisjonen håndhevet det strengt, og designsystemene i landet ble tvunget inn i rammeverk og blandet med forretningslogikk. Fristad ble tvunget som alle andre. Kongens rammeverk er det samme som Rammeverket nederst i krypten (filmen antyder, spillet avslører). Spilleren er en fremmed uten fortid.
- **Tonen:** alvorlig saga med glimt i øyet.

## Verden og kamerabanen

Verden er et langt bånd langs x (tiden), med lagene i dybden langs z: himmel og fjerne åser lengst bak (beveger seg nesten ikke), åser og byer i midten, scenene og personene foran, og forgrunnslag (gress, gravsteiner, bjelker) helt nær kameraet, som suser forbi. Kameraet flyr mot høyre langs båndet og dykker innover (mot −z) inn i de åtte hendelsene, og ut igjen. Mellom hendelsene går det et mellomlandskap, så det aldri er et kutt.

| Del | Tid (s) | Sted i båndet | Kameraet | Lys |
|---|---|---|---|---|
| 1. Kongeriket | 0–12 | Åsene med byene | Fra himmelen ned og sidelengs over åsene | Morgengry |
| 2. Gullalderen | 12–26 | Skriptoriet | Inn gjennom vinduet, langs pultene | Morgensol |
| 3. Kongens bud | 26–40 | Torget i Fristad | Ut gjennom døra, sidelengs over torget til heroldens podium | Dag |
| 4. Inkvisisjonen | 40–54 | Foran Skriptoriet | Glir langs rekka av inkvisitorer, så nær én komponent i ramme | Ettermiddag, kald |
| 5. Forfallet | 54–66 | Skriptoriet igjen | Inn i det samme rommet, nå i uorden og glitch | Skumring |
| 6. Begravelsen | 66–82 | Kirkegården utenfor byen | Sidelengs langs prosesjonen mot kryptdøra | Natt, måne |
| 7. Krypten i dag | 82–96 | Under kirkegården | Ned gjennom jordlagene, gjennom krypten, til noe som gløder dypt nede | Mørkt, fakler |
| 8. Fristad, anno 1350 | 96–110 | Veien til sørporten | Opp av jorda til morgenen, sidelengs langs veien forbi den fremmede, inn mot porten | Morgen |

## Tekstene (ordrett)

Hver tekst står i rommet på dybden oppgitt i manuset (`texts` i `web/film/opening.json`), og kommer ord for ord. Skrift: overskrifter i UnifrakturMaguntia, fortellingen i Uncial Antiqua.

1. **Fristad** (overskrift)
   *I de gode årene hadde hver by sitt skriptorium, og hvert skriptorium sine egne komponenter.*
2. *Hver av dem gjorde én ting, og gjorde den vel.*
3. *Så kom kongens bud:*
   **Alt skal lages med ett rammeverk.**
   **Alt skal virke likt overalt.**
4. *Inkvisisjonen sørget for at budet ble holdt.*
   *Det som ikke passet, ble tvunget til å passe.*
5. *Forretningslogikken krøp inn i alt.*
   *Knappene glemte hva de var til.*
6. *Om natta bar munkene dem ut av byen.*
   *Det de ikke kunne rette, begravde de.*
7. *Men rammeverket sov ikke.*
   *Det som begraves, blir ikke borte.*
8. **Fristad, anno 1350** (overskrift)
   *En fremmed kommer til porten.*

## Bildene

**Gjenbrukes** (fra `assets/film/opening/`):

| Del | Bilde | Bruk |
|---|---|---|
| 2 | `gullalderen/bg`, `mid` (munkene), `fg` (pulten) | Skriptoriet innenfra |
| 5 | `fallet/bg`, `mid` (symbolene), `fg` (blekket) | Skriptoriet i forfall |
| 6 | `begravelsen/bg`, `fg` (gravsteinene) | Kirkegården (`begravelsen/mid` har prosesjonen malt inn og brukes ikke) |
| 7 | `krypten/bg`, `mid` (fiendene), `fg` (beina) | Krypten |
| 8 | `fristad1350/bg`, `mid` (byfolket), `fg` (boden); `sorporten/bg`, `mid` (den fremmede), `fg` (skiltet) | Byen og veien til porten |

**Nye** (18 bilder): promptene står under, komplette og klare til å limes inn. Alle i 16:9, så store som generatoren tillater (helst 2560 × 1440 eller mer). Bilder med «green background» klippes ut av meg, så bakgrunnen må være helt jevn grønn (#00FF00) uten skygger. Lagre dem i `assets/film/intro/` med navnet i overskriften (`.png` eller `.jpg`).

### 1. Kongeriket

**`himmel_morgen.png`** (bakerst; himmel uten land)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. A vast dawn sky over Northern Europe around 1350: soft layered clouds lit pink and gold from below by the rising sun near the horizon, deep blue sky fading to pale gold, a few birds far away. Only sky, no land, no buildings, no people.
```

**`aaser_fjerne.png`** (fjerne åser med byer, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. A long band of distant rolling hills seen from far away at dawn, misty blue-green, with several tiny walled medieval towns on the hilltops, each with a church tower and a tall scriptorium building with lit windows, thin smoke rising, roads winding between them. The hills fill only the lower half of the image; everything above the hills is a perfectly flat pure green background (#00FF00) with no clouds and no shadows, clean edges along the hilltops.
```

**`aaser_naere.png`** (nærmere åser og veien, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. Nearer green rolling hills in morning light running from left to right, a dirt road winding along them, stone walls, small trees and bushes, meadow flowers, long soft shadows. The hills fill the lower 40 percent of the image; everything above is a perfectly flat pure green background (#00FF00) with no shadows, clean edges along the hilltops and trees.
```

### 2. Gullalderen

**`skriptoriet_fasade.png`** (Skriptoriet utenfra; kameraet flyr inn gjennom det midterste vinduet)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, Northern European medieval town around 1350, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. The front of a tall stone scriptorium seen straight on in warm morning sun: carved stone walls, a big arched wooden door, and in the middle of the facade one large arched window. The glass of the large middle window and the sky around the building are a perfectly flat pure green (#00FF00) with no reflections and no shadows, so the window opening and the outline of the building can be cut out cleanly. Ivy on the walls, a hanging sign with a quill, a few doves on the ledge.
```

### 3. Kongens bud

**`torget_bud.png`** (bakgrunn: torget den dagen budet kom)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, Northern European medieval town around 1350, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. The market square of a small Nordic medieval town on a bright day, half-timbered houses around the square, a stone well, the church tower behind, and long royal banners hanging from every house: deep blue banners with a strict golden emblem of a perfect square grid inside a frame, the same emblem on every banner. A wooden platform with steps stands empty on the right side of the square. No people.
```

**`herold.png`** (heroldens podium, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, cinematic, high detail, sharp, no text, no watermark, no border. A royal herald standing on a small wooden platform reading aloud from a long unrolled parchment decree with a large wax seal showing a perfect square grid inside a frame, wearing a blue and gold tabard with the same grid emblem, feathered hat, stern face, a trumpeter beside him with a banner hanging from the trumpet, seen from the front slightly to the side. Isolated, on a perfectly flat pure green background (#00FF00) with no shadows on the background, nothing else in the image, clean edges.
```

**`folkemengde_bud.png`** (byfolket som lytter, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, warm golden light against cool blue shadows, cinematic, high detail, sharp, no text, no watermark, no border. A crowd of medieval townsfolk seen from behind and from the side, listening to a royal decree: craftsmen, a monk in a brown habit, a woman holding a child, a merchant, an old scribe clutching his books to his chest, some looking worried at each other. Full figures in a row from left to right. Isolated, on a perfectly flat pure green background (#00FF00) with no shadows on the background, nothing else in the image, clean edges.
```

**`kongens_segl.png`** (kongens rammeverk som et syn over torget, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, cinematic, high detail, sharp, no text, no watermark, no border. A huge royal seal floating in the air like a vision: a perfect, rigid golden square grid inside a heavy ornate iron frame, with a crown on top and chains hanging from its corners, glowing coldly, every cell of the grid exactly the same. Isolated, on a perfectly flat pure green background (#00FF00) with no shadows on the background, nothing else in the image, clean edges.
```

### 4. Inkvisisjonen

**`skriptoriet_kveld.png`** (Skriptoriet utenfra i kald ettermiddag; bakgrunn)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, cool grey-blue light, Northern European medieval town around 1350, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. The same tall stone scriptorium as before, seen from the square in cold late afternoon light under a grey sky: the big wooden door broken open, papers blowing out of it, royal blue banners with a golden square-grid emblem nailed over the windows, a cart with iron cages standing in front. No people.
```

**`inkvisitorer.png`** (inkvisitorene, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, cool shadows, cinematic, high detail, sharp, no text, no watermark, no border. A row of five stern inquisitors in long crimson robes and black hoods walking from left to right, each with a golden pendant shaped like a square grid inside a frame, one holding a heavy ledger, one carrying a tall iron measuring rod, one dragging a chain, faces hard and certain, seen from the side. Isolated, on a perfectly flat pure green background (#00FF00) with no shadows on the background, nothing else in the image, clean edges.
```

**`komponent_i_ramme.png`** (en komponent spent fast, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, cinematic, high detail, sharp, no text, no watermark, no border. A small magical user interface component, a glowing round push-button with a friendly light, forced into a heavy rigid iron frame with a square grid, bolted and chained in place, ledger pages with numbers and red stamps nailed to the frame, the button's light flickering and turning sickly. Isolated, on a perfectly flat pure green background (#00FF00) with no shadows on the background, nothing else in the image, clean edges.
```

### 6. Begravelsen

**`prosesjon.png`** (prosesjonen med kistene, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, cool blue moonlight with warm lantern light, cinematic, high detail, sharp, no text, no watermark, no border. A solemn night procession of hooded monks walking from left to right, carrying three long wooden coffins bound with iron chains on their shoulders, faint magenta light leaking through the cracks of the coffins, the first monk holding a lantern high, the last one holding a candle, full figures seen from the side. Isolated, on a perfectly flat pure green background (#00FF00) with no shadows on the background, nothing else in the image, clean edges.
```

### 7. Krypten i dag

**`jordlag.png`** (snitt gjennom jorda under kirkegården; kameraet går ned gjennom det)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. A cross-section through the earth under a medieval churchyard at night, like a cut-away diorama: at the top a thin strip of grass with gravestones, below it layers of dark soil with roots, old bones, buried coffins bound with chains, then stone foundations and the top of a brick vaulted crypt with a faint torchlight inside, and at the very bottom a deep faint magenta glow seeping up through cracks. Everything above the grass line is a perfectly flat pure green background (#00FF00) with no shadows.
```

**`rammeverket_sover.png`** (det som sover dypt nede, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, dark with magenta glow, cinematic, high detail, sharp, no text, no watermark, no border. A colossal dormant creature built from a rigid iron square grid and heavy ornate frames, curled up asleep deep underground, half buried in stone and chains, a faded royal crown fused into its head, magenta light pulsing slowly in the seams of the grid like a heartbeat, one eye barely opening. Isolated, on a perfectly flat pure green background (#00FF00) with no shadows on the background, nothing else in the image, clean edges.
```

### Mellom delene

**`veien_natt.png`** (veien fra byen til kirkegården om natta, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, cool blue moonlight, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. Dark rolling hills at night running from left to right with a dirt road leading from a walled town with lit windows on the left towards a small stone church on a hill on the right, old crooked trees, a stone wall along the road, mist in the hollows. The hills fill the lower half of the image; everything above is a perfectly flat pure green background (#00FF00) with no shadows, clean edges along the hilltops and trees.
```

**`natthimmel.png`** (natthimmel bakerst)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. A deep blue night sky with a large bright moon behind thin drifting clouds, many stars, a faint glow near the horizon. Only sky, no land, no buildings, no people.
```

**`forgrunn_gress.png`** (gress og blomster helt nær kameraet, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, warm golden light, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. Tall meadow grass, wild flowers and a few ferns seen very close, running along the bottom of the image from left to right, about one third of the image high, backlit by low sun. Everything else is a perfectly flat pure green background (#00FF00) with no shadows, clean edges along the grass.
```

**`forgrunn_steiner.png`** (mur og stein helt nær kameraet, til utklipp)

```
Hand-painted stylized fantasy illustration, the same style as painted game concept art: soft painterly brushwork, no black outlines, rich saturated colours, cool moonlight, cinematic, wide 16:9, high detail, sharp, no text, no watermark, no border. An old mossy dry-stone wall with a few leaning gravestones and a wooden cross, seen very close, running along the bottom of the image from left to right, about one third of the image high, lit by cold moonlight. Everything else is a perfectly flat pure green background (#00FF00) with no shadows, clean edges.
```

## Musikken

Partituret i `web/score.js` (ekte instrumentprøver). Cuene følger delene: morgengry (1), skriptoriet (2), kongens bud med pauke og trompetfanfare (3), inkvisisjonen med tromme og dyp bordun (4), forfallet (5), begravelsen med munkekor og klokke (6), krypten (7) og porten (8). Eieren vurderer opptaket før musikken låses.

## Det som går ut

Filmen i motoren (kamerabanen gjennom det ekte Fristad, maleriene projisert på verden, `paintover`, `Frame.paint` og Kuwahara-filteret i filmen, `engine.film`) fjernes når den nye filmen er bygd. Bare overgangen til slutt blir igjen: siste bilde toner til spillet ved sørporten (`engine.spawn_eye`).
