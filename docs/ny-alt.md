# Alt i den malte stilen

Bestemt med eieren 10. oktober 2026: alle de gamle teksturene og modellene skal bort, og alt lages på nytt i den malte stilen (`docs/ny-stil.md`). Promptene står i `docs/prompter-3.md` (inventar, krypten, våpen, skjold, rustninger, skrot, trær, busker og dyr), `docs/prompter-folk.md` (menneskene) og `docs/prompter-krypten.md` (gravkjelleren).

## Status

| Område | Status | Hvor |
|---|---|---|
| Husene, Vertshuset, spillerens hus, småbyggene | ny stil | `kit.py`, `houses.py`, `tavern.py`, `home.py`, `yard.py` |
| Smia, Apoteket, Skriptoriet, stavkirka, krypten | ny stil | `workshops.py`, `sacred.py` |
| Bymuren, tårnene, porthusene | ny stil | `fortress.py` |
| Gatelivet (brønner, boder, kors, kjerre, lykter, skilt, kister, dukker) | ny stil | `streetlife.py` |
| Flettverksgjerdet | ny stil | `street.py` (`wattle_fence`) |
| Krypten, gravkjelleren | under arbeid | `tools/crypt/` |
| Krypten, tema 2–5 | planlagt | `docs/krypten-ny.md` |
| Menneskene | under arbeid, pilot med tre figurer | `docs/prompter-folk.md` |
| Inventaret i husene | delvis (kit-møbler), resten etter arkene | `docs/prompter-3.md` |
| Våpen, skjold, rustninger, armene | gammel stil | `arms.py`, `fp_arms.py` |
| Fiendene og bossene | gammel stil | `mobs.py`, egen runde |
| Dyrene | gammel stil | `animals.py` |
| Trær, busker, gress, blomster | gammel stil | `trees.py`, `grass.py` |
| Steiner, steingjerder, gravsteiner | gammel stil | `street.py`, `castle.py` |
| Portalen | gammel stil | `portal.py` |

## Det som slettes

**Gjort 10. oktober (grenen `rydd/gamle`):** alle de gamle modellene og teksturene er slettet (se CLAUDE.md, «Rydding»), og lagene er byttet til de malte (`oak`→`planks`/`timber`, `metal`→`iron`, `grass`→`meadow`, `plywood`→`forest_floor`, `straw`→`straw_floor`, `pebbles`→`gravel`, søla heter `mud`, `brick`→`ashlar`, `door_oak`/`door_dark`→`door`/`door_b`; nye `fur`, `chainmail`, `felt`, `gambeson`, `snow` og `oak_planed` til bordplatene). `npc.py` og `people.py` står til de nye figurene er ferdige (`folk.py` bruker `people`); `town.py` står fordi byggesettet bruker hjelperne der. Mangler til de nye modellene kommer: innbyggerne og byfolket, fiendene og bossen, armene og våpnene, dyrene, trærne, gresset og blomstene, gravene, portalringen og kjelleren.

**Modellene:**
- `castle_ruin`, `crypt_stair` og `portcullis` fra `castle.py`. Kjellerens dør og gitter lages på nytt i kryptens byggesett.
- `assets/models/dungeon/` og `tools/models/dungeon.py`. Dette erstattes av `tools/crypt/`.
- `assets/models/gorvek/` (`chest`, `keg`, `skull`, `torch`).
- `assets/models/house/door_lamp.glb`.
- De gamle skriptene, når det som lages på nytt er ferdig:
  - `walls.py`
  - `buildings.py`
  - `town.py`
  - `house.py`
  - `props.py`
  - de gamle delene av `street.py`
  - `npc.py`
  - `gorvek.py`
  - `people.py`
- `assets/textures/faces/` og de gamle flisene i `assets/textures/people/` (fotoene).

**Teksturene:** lagene uten malt utgave forsvinner:
- `bark`, `brick`, `concrete`, `cryptwall`, `door_dark`, `door_oak`, `floor`, `grass`, `leather`, `linen`, `metal`, `oak`, `pebbles`, `plywood`, `snow`, `soot`, `straw` og `wool`
- `assets/textures/weapons/`

Nye malte lag fra `docs/prompter-3.md` tar over der de trengs:
- `oak_planed`, `wool`, `linen`, `leather`, `fur`, `straw`, `snow`
- `bark_*`, `iron_worked`, `pottery`, `wicker`, `rope`, `parchment`

Et lag slettes først når ingen modell bruker det lenger. Testen `alt_er_ekte_modeller_med_teksturer` passer på at ingenting blir flatt.

## Rekkefølgen

1. **Menneskene:** piloten først (Buttonsmeden, Vertinnen og Gorvek), så de andre ti. Bevegelsene, utseendet og hvordan de holder tingene sine skal være på AAA-nivå før noe godkjennes. Eieren ser bilder og bevegelse først.
2. **Krypten:** motoren og nivå 1, så tema for tema.
3. **Inventaret i husene og skrotet:** møbler, kjøkken, verksted, apotek og skriptorium, tekstiler og skrot. Rommene i husene man går inn i fylles etter arkene.
4. **Våpen, skjold, rustninger og armene i førsteperson.** De synes hele tiden, så de må være skarpe tett på. Hvert våpen og skjold får sitt eget malte ark.
5. **Trær, busker, gress og blomster:** stammer med greiner og barkflis, løvklynger fra atlasene og forbildet for formen. Store, tydelige klynger som i WoW, ikke en sky av små blader.
6. **Dyrene.**
7. **Fiendene og bossene** i en egen runde med prompter: én per Fristil-komponent i `docs/krypten-ny.md`.
8. **Rydding:** slett de gamle modellene, teksturene og skriptene og koden som bare de brukte (CLAUDE.md oppdateres).

## Hvordan arkene brukes

- **Fliser** blir teksturlag (`assets/textures/painted/<navn>/albedo.png`, sydd med `tools/textures/quilt_seam.py` og avledet med `derive.py`). Grensen er 64 lag. Blir det trangt, slås lag som ligner sammen, og fargen kommer fra materialtonen.
- **Ark med ting (3 × 3)** er forbilder for formene i Blender. Fargen og detaljene kommer fra flisene, så tingene deler teksturer og kan tegnes billig.
- **Ark for én ting** (våpen og skjold) projiseres rett på modellen og bakes til en egen tekstur, siden de synes tett på.
- **Løvatlasene** skjæres ut mot den hvite bakgrunnen (`tools/textures/leaf_atlas.py` gjør det samme i dag) og blir klynger med alfa.
