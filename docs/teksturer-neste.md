# Teksturer og fotoer til neste versjon

Modellene under bruker i dag de felles teksturlagene (`assets/textures/<lag>/`,
farget per materiale) eller bakte farger. De ville tjent mest på egne
teksturer fra eieren (bildegenerator, fotoskann eller Blender).

## Figurer

- **Byfolkene** (bonden `npc_farmer`, gutten `npc_boy`, vaktmannen
  `npc_guard`, kremmeren `npc_merchant`, pilegrimen `npc_pilgrim`): har egne
  figurer, men bare fotoet forfra er deres eget. Hetta, brynjehetta og
  hattene dekker sidene og bakhodet, som lånes fra Apotekeren, Smeden og
  Skriveren.
  - Bonden bruker `ung_mann.jpg` og gutten `gutt.jpg`. Side- og bakfoto ville
    latt dem ta av seg hetta.
  - Vaktmannen, kremmeren og pilegrimen låner ansiktet fra Smeden,
    Apotekeren og Skriveren (`smed.jpg`, `gammel.jpg`, `munk.jpg`). De
    trenger egne ansikter forfra, så ingen i byen ser ut som tvillinger.
  - Kona på torget (`npc_woman`) bruker `kvinne.jpg` forfra og Vertinnens
    side- og bakfoto. Egne fotoer ville gitt et riktigere bakhode.
- **Klærne** til alle figurene (ull, lin, lær er felles lag): egne tøyteksturer
  med vev, sømmer og flekker per figur.

## Monstrene (`tools/models/mobs.py`)

- `mutant_button`, `mutant_tooltip`, `mutant_input`, `mutant_modal`,
  `mutant_table`, `framework`: kroppene er flate farger og felles lag.
  - Skall og hud trenger egne teksturer: korrodert plast, glitchy piksler og
    lysende sømmer.
  - Komponentdelene (knappen, tabellen, modalen) trenger UI-flatene fra
    Fristil som tekstur.

## Kjelleren

- **Pynten** (`dngp_*`: hyller, sekker, bannere, mose, røtter, hodeskaller,
  knokkelhauger, lava, glør): egne teksturer, særlig bannerne (våpenskjold),
  mosen og lavaen.
- **Temagangene**:
  - `dng_hall_sewer`: vann, slim og renneutløp.
  - `dng_hall_crypt`: knokler og voks.
  - `dng_hall_lava`: glødende sprekk.
- **Kløfta** (`dng_chasm`): bergvegger i kløfta, plankebro, tau.
- **Snarveidøra** (`dng_shortcut`, `dng_shortcut_leaf`): eikedør med
  jernbånd, og slå.
- **Kista** (`chest`, `chest_lid`) og portgitteret (`portcullis`).

## Byen

- **Brønnen** (`well`, `well_winch`, `well_rope`, `well_bucket`),
  flettverksgjerdene (`wattle_fence`), vedstablene (`woodpile`).
- **Markedsbodene** (`market_stall`, `market_stall_b`): stoffet i takene og
  varene.
- **Veiskiltene** (`signpost`): brent skrift i planken.
- **Dyrene** (`sheep_*`, `cow_*`): pels og hud.
- **Gravsteinene** (`gravestone`) og torgkorset (`market_cross`): hogd stein
  med innskrift.

## Utenfor spillet: skillelinja i Fristil

I Fristil-dokumentasjonen er `<hr class="fs-divider">` usynlig: den får
nettleserens `margin-inline: auto` i flex-stabelen på docs-siden og blir 0 px
bred. Rettingen hører hjemme i Fristil: `margin-inline: 0` i `divider.css`
(eller i `Preview.astro`). Spillet viser den riktig med en egen regel
(`.flate > .fs-divider { margin-inline: 0 }` i boka).
