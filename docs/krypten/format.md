# Nivåformatet for den nye krypten

Kontrakten mellom Blender-siden (`tools/crypt/`, byggesettet og nivåskriptene) og motoren (`src/domain/world/crypt.rs`). Planene i `tools/crypt/levels/level_NN.py` er kilden. Nivåskriptet bygger modellene etter dem og skriver json-fila under.

## Filene per nivå

- `assets/models/crypt/crypt_NN_<bit>.glb` er geometrien i biter på rundt 16 × 16 m (`a`, `b`, `c` …), i nivåets eget koordinatsystem. Origo er i nivåets origo, og `pieces` sier hvilke biter som finnes. Bitene trengs så rendereren kan kutte dem som ikke synes.
- `assets/models/crypt/crypt_NN.json` er alt annet, beskrevet under.
- Felles modeller (kister, spaker, gitter, dører, hvilestedet, fellene, den falske veggen, fallemmen, stigen) ligger i `assets/models/crypt/kit_*.glb` og settes ut av motoren etter `marks`. Modeller som beveger seg (gitter, spak, dør, fallem, falsk vegg) må være egne modeller, ikke del av bitene.

Koordinater er i meter: x mot øst, y opp og z mot sør, som i spillet og i planene. Motoren legger hvert nivå på sin egen plass under byen. Nivå n står med origo i `(0, -40·n, 0)` + en forskyvning, så nivåene aldri overlapper og aldri når opp til byen.

## crypt_NN.json

```json
{
  "level": 1,
  "name": "Gravkjelleren under kirka",
  "theme": "grav",
  "pieces": ["crypt_01_a", "crypt_01_b"],
  "rooms": [
    {"id": "kapell", "name": "Gravkapellet", "poly": [[x, z], ...], "y": 0.0, "h": 5.0, "secret": false}
  ],
  "colliders": [
    {"box": {"center": [x, y, z], "size": [sx, sy, sz], "yaw": 0.0}},
    {"hull": [[x, y, z], ...]}
  ],
  "nav": {
    "verts": [[x, y, z], ...],
    "polys": [[0, 1, 2, 3], ...],
    "poly_room": ["kapell", ...],
    "poly_gate": [null, "gitter_1", ...]
  },
  "lights": [
    {"pos": [x, y, z], "color": [r, g, b], "range": 8.0, "flicker": true, "room": "kapell"}
  ],
  "marks": [
    {"kind": "start", "pos": [x, y, z], "yaw": 0.0},
    {"kind": "mob", "pos": [x, y, z], "label": "button", "n": 2, "patrol": [[x, y, z], ...]},
    {"kind": "lever", "id": "spak_1", "pos": [x, y, z], "yaw": 0.0, "opens": ["gitter_1"]},
    {"kind": "gate", "id": "gitter_1", "pos": [x, y, z], "yaw": 0.0, "width": 3.0}
  ]
}
```

- **`rooms`** er omrisset av hvert rom og hver gang sett ovenfra (polygon i xz), med gulvhøyde og takhøyde. Rommene brukes til kartet og fog of war (et rom avdekkes når du går inn), til lyset (et lys lyser bare i sitt rom og i rom med åpen forbindelse) og til å finne rommet du står i.
- **`colliders`** er bokser og konvekse omhyllinger for murer, gulv, tak, søyler, trapper og ramper. Trapper får en rampe som kollider. Hullet i en fallem og sjakter har ikke gulv.
- **`nav`** er navigasjonsnettet. Det består av konvekse polygoner på alt gulv, alle trapper og ramper, sydd sammen så naboer deler kanter, med hjørnene felles i `verts`. Polygonene ligger 0,4 m inn fra veggene, så fiendene ikke gnisser mot murene. `poly_room` sier hvilket rom hver polygon hører til. `poly_gate` er id-en til porten, døra, broa eller vannstanden som stenger polygonen, og er `null` om den alltid er åpen.
- **`lights`** er fakler, stearinlys, glør og det blå lyset ved trappa ned. `room` sier hvilket rom lyset står i.
- **`marks`** er alt motoren setter ut eller styrer:

  | `kind` | Hva | Ekstra felt |
  |---|---|---|
  | `start` | der du kommer ned | `yaw` |
  | `up` | døra eller trappa opp (til krypten eller nivået over) | `yaw` |
  | `down` | trappa ned (ikke på nivå 10) | `yaw` |
  | `rest` | hvilestedet (alteret med lysestaken) | `yaw` |
  | `mob` | fiender | `label` (slaget), `n`, `patrol` (valgfri), `ambush` (valgfri) |
  | `elite` | eliten med nøkkelen | `label`, `opens` |
  | `boss` | bossen (hvilket slag som helst; stolpen, brølet, kommer ikke igjen når den er slått) | `label`, `arena` (rom-id) |
  | `chest` | kiste | `yaw`, `loot` (valgfri) |
  | `rare` | sjelden komponent | `label` (komponenten) |
  | `lore` | lapp eller bok | `label`, `text` |
  | `bonus` | varig styrke | `label` (`liv` eller `utholdenhet`) |
  | `lever` | spak | `id`, `yaw`, `opens` (id-er) |
  | `gate` | gitter eller port | `id`, `yaw`, `width`; med `arena` (rom-id) står porten oppe og slår igjen bak deg mens bossen lever, og `label: "exit"` er porten ut (stengt så lenge bossen lever) |
  | `bridge` | bro over en avgrunn, senkes når bryteren åpnes (alle spakene som har den i `opens`) | `id`, `yaw`, `size` (bredde, lengde), `height` (hvor høyt den henger), `top` (festene i taket) |
  | `shortcut` | dør med slå, åpnes fra én side | `id`, `yaw`, `width`, `from` (rom-id der slåen er) |
  | `secret_wall` | falsk vegg som slås ned eller åpnes | `id`, `yaw`, `width`, `height` |
  | `trapdoor` | fallem eller sjakt du faller ned i | `id`, `size`, `to` (rom-id) |
  | `ladder` | stige opp | `top` (punkt), `yaw` |
  | `trap` | felle | `label` (`arrows`, `spikes`, `rocks`, `pendulum`, `gas`, `flood`), `id`, `dir`, `size` |
  | `hint` | hint om en hemmelighet (trekk, lyd, sprekk) | `label` (`draft`, `sound`, `crack`), `for` (id-en det gjelder) |
  | `lift` | heisen (`kit_lift`) med heistårnet og vinsjen (`kit_winch`) over sjakta: står oppe med bremsen på til bryteren `id` åpnes (spaken), så kjører styrestangen på dekket den til den andre enden med deg på, og vinsjen oppe henter den opp igjen; fiendene tar den ikke (ingen navigasjon gjennom sjakta) | `id`, `yaw`, `size` (bredde, dybde; 2,8 × 2,8), `bottom` (punktet nede: så langt den går) |
  | `prop` | pynt som egen modell fra byggesettet (sarkofagen `kit_sarcophagus`); kolliderne står i `colliders` | `label` (modellen), `yaw` |
  | `water` | vannflate som står høyt til bryteren `id` åpnes (sluseratten, en spak med `id` i `opens`), og så synker | `id`, `size` (bredde, dybde), `levels` (høy, lav) |
  | `water_block` | usynlig mur som holder deg ute av vannet der en trapp eller gang går ned i det; borte når vannet har sunket | `id` (vannets), `size` (bredde, høyde, dybde), `yaw` |

## Felles regler

- Hvert nivå har nøyaktig én `start`, én `rest` og én `down` (nivå 10 har ingen `down`). `tools/crypt/plan.py` sjekker dette.
- Taket står minst 2,9 m over gulvet overalt du kan gå.
- Hovedveien skal kunne gås med ekte fysikk fra `start` til `down`. Testen `kryptnivaaene_kan_gas_gjennom` gjør det, med spakene trukket etter tur.
- Hver `mob`, `elite` og `boss` står på en polygon i navigasjonsnettet og har vei til `start` når alle porter er åpne. Testen er `fiendene_finner_fram_i_krypten`.
