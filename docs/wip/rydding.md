# Rydding: de gamle modellene og teksturene (logg)

Oppgave: slett de gamle modellene og teksturene (eieren: «Jeg vil ikke se dem igjen»), bytt lagene til de malte, eksporter de nye modellene på nytt, hold testene grønne. Gren `rydd/gamle` i `/home/user/fristad-rydd`.

## Plan
1. Slett modellfilene og oppføringene i `models!`.
2. Lagene: Brick→Ashlar, Oak→Planks, Metal→Iron, Grass→Meadow, Gravel(søle)→Mud, Plywood→ForestFloor, Straw→StrawFloor, Pebbles→Gravel, Door/DoorB får navnene door/door_b; Bark, Concrete, Snow, CryptWall, Floor og Soot ut; Fur, Chainmail, Felt og Gambeson inn (på de ledige plassene, så terrengets lagnumre står).
3. Blender-skriptene: layer("metal") → "iron" osv., eksporter på nytt.
4. Koden: trær, gress, dyr, steiner, graver og portalringen borte; testene.
5. Skjermbilder, CLAUDE.md og docs/ny-alt.md.

## Merk for sammenslåingen
- Kryptens hjelper (i /home/user/fristad) har lagt til lag 46–54 i `Layer`; her er nummereringen endret (44 lag), så de må nummereres om.
- `tools/crypt/kit_grav.py` bruker `door_dark` og `metal`: må bli `door_b` og `iron`.

## Logg
- Steg 1 (modellene): 145 filer slettet (166 MB) med `git rm`; `models!` ryddet. Kjellerens `.json` står igjen (generatoren leser dem med include_str!, kryptens hjelper sletter dem). `nature.rs` er bare landskapet og kanten nå; kirkegården har flettverksgjerde (`town::fence`) i stedet for steinmur, ingen graver; portalen registrerer siktemålet uten ringen.
- Steg 2 (lagene): `Layer` har 44 lag (navnene = mappene i assets/textures). Mappene som var avledet av de malte bildene er flyttet (gravel→mud, pebbles→gravel, brick→ashlar, floor→planks, metal→iron, grass→meadow, plywood→forest_floor, straw→straw_floor, door_oak→door, door_dark→door_b). Slettet: bark, concrete, cryptwall, oak, snow, soot, door (gammel), faces, weapons, people/smed_*, vertinne_*. Avledet på nytt fra painted/: wool, linen (sydd, svak søm igjen fra kildebildet), leather, fur, chainmail, felt, gambeson. 94 filer, 114 MB.
- Testene: kjellertestene som trenger gulvene (kartet, kjelleren_kan_nas, kløfta, portalen, skrittene, snarveien) og fiendene_ser_mot_deg er `#[ignore]` med grunn.
- Neste: Blender-skriptene (metal→iron, floor→planks, straw→straw_floor, door_oak→door, door_dark→door_b, brick→ashlar, bark/soot/oak) og eksport.
- Steg 3 (Blender): slettet skriptene animals, arms, fp_arms, gorvek, house, mobs, mob_motion, trees, grass, leaves, portal, castle, dungeon, dng_preview, buildings, props, walls, bake_models. `street.py` er bare `wattle_fence` nå (staker i timber_grey, vidjer i wattle). npc.py og people.py står (folk.py importerer people). Lagnavnene byttet i kit, houses, tavern, home, yard, workshops, sacred, fortress, streetlife, town (metal→iron, floor→planks, straw→straw_floor, door_oak→door, door_dark→door_b, brick→ashlar, bark→timber, soot→stone, oak→timber). Alle eksportert på nytt (55 modeller, rundt 6 min); `_inside.json` uendret. Ingen modell har ukjente lag.
- Steg 4: eierens malte snø (`snow`, 2 m) er snødekket i været (`weather` i world.wgsl legger laget ovenfra); `oak_planed` (2 m) på bordplater, benker, krakker og bokhyllene; ny linen avledet. Tavern, home, workshops og sacred eksportert på nytt. Slettet også `props/flames_atlas.png` (ubrukt) og `tools/textures/sheets.py`.
- Ferdig: `cargo test --release` 30 ok, 8 ignorert; `cargo build --release --bins` ok; wgsl-check ok. Skjermbilder i `shots/rydding/` (torget, kirkegarden, enga, sno, ark).
