# Musikk til spillet (Suno)

Spillet spiller egne spor for byen, krypten og kampen (eieren 18. oktober). Sporene lages i Suno og legges i `assets/music/game/` med navnene under. Mangler et spor, spiller spillet det syntetiske partituret som før (`web/score.js`).

**Slik lager du dem i Suno:**
- Velg **Custom** og slå på **Instrumental**.
- Lim inn teksten under «Style» i feltet *Style of Music*, og navnet som *Title*.
- Lag to eller tre versjoner av hvert, og velg den som holder samme stemning hele veien uten store skift. Byen og krypten går i løkke, så et spor uten tydelig slutt er best.
- Last ned som mp3 og send meg fila. Jeg klipper løkkepunktene og normaliserer lyden.

Alle sporene bygger på samme lydbilde som filmen: middelalderske instrumenter, ingen moderne trommer eller synth, og ingen sang med ord (munkekor uten tekst er greit).

## Byen

| Fil | Title | Lengde |
|---|---|---|
| `town_day.mp3` | Fristad, torget | 2–3 min |
| `town_evening.mp3` | Fristad, vertshuset om kvelden | 2–3 min |
| `town_night.mp3` | Fristad, natt | 2–3 min |

**town_day.** Style:
> medieval Nordic market town, 14th century, warm and lively but calm, recorder and shawm melody, lute and hurdy-gurdy, bodhran frame drum soft, bagpipe drone, dorian mode, folk dance feel, moderate tempo 96 bpm, no vocals, seamless loop, no fade out

**town_evening.** Style:
> medieval tavern evening, cozy and warm, solo lute and fiddle, gentle hand drum, soft drone, mixolydian, relaxed swaying 3/4 dance, crackling hearth mood, 84 bpm, no vocals, seamless loop

**town_night.** Style:
> medieval town at night, quiet and slightly lonely, solo recorder over harp arpeggios, distant soft drone, aeolian mode, slow 66 bpm, sparse and spacious, no percussion, no vocals, seamless loop

## Krypten

Ett spor per tema, mørkt og kaldt, med plass til lydene i spillet. Melodien skal ikke ta over; dette er stemning.

| Fil | Title | Tema |
|---|---|---|
| `crypt_grav.mp3` | Gravkjelleren | nivå 1–2 |
| `crypt_sewer.mp3` | Kloakken og cisternen | nivå 3–4 |
| `crypt_mine.mp3` | Gruva og krystallgrotta | nivå 5–6 |
| `crypt_castle.mp3` | Den sunkne borgen | nivå 7–8 |
| `crypt_core.mp3` | Den korrupte kjernen | nivå 9–10 |

**crypt_grav.** Style:
> dark medieval crypt ambient, wordless monk choir humming low, cello and bass drone, distant church bell, phrygian mode, very slow 50 bpm, cold stone reverb, ominous but restrained, no drums, no vocals with words, seamless loop

**crypt_sewer.** Style:
> dark underground water cisterns, medieval, low bowed strings drone, hurdy-gurdy groan, dripping echoes, sparse plucked psaltery notes, locrian mode, slow 56 bpm, damp heavy reverb, uneasy, no drums, no vocals, seamless loop

**crypt_mine.** Style:
> abandoned medieval mine and crystal cave, deep drone, slow metallic hammer hits far away, glass harmonica and plucked harp shimmering like crystals, aeolian mode, 60 bpm, mysterious and cold, no vocals, seamless loop

**crypt_castle.** Style:
> sunken medieval castle underground, mournful fanfare echoes on a distant horn, low strings and timpani rolls very soft, war banner ghosts, dorian minor, slow march 64 bpm, heavy and tragic, no vocals, seamless loop

**crypt_core.** Style:
> corrupted ancient machine heart beneath a medieval crypt, wordless choir and organ drone slowly detuning, bowed metal, pulsing low heartbeat, tense and otherworldly, phrygian dominant, 60 bpm, dark and unsettling, still acoustic and medieval, no synth, no vocals, seamless loop

## Kampen

Kampsporet tones inn når et monster er nær og angriper deg, og tones ut noen sekunder etter at kampen er over. Det må komme raskt i gang, uten lang intro.

| Fil | Title | Når |
|---|---|---|
| `combat.mp3` | Kamp i krypten | vanlige fiender |
| `combat_boss.mp3` | Bossen | bossene nederst i hvert tema |
| `combat_framework.mp3` | Rammeverket | sluttbossen i nivå 10 |

**combat.** Style:
> intense medieval battle music, driving frame drums and taiko-like low drums, fast hurdy-gurdy and fiddle ostinato, shawm stabs, bagpipe drone, dorian minor, 140 bpm, aggressive, starts immediately without intro, no vocals, seamless loop

**combat_boss.** Style:
> epic dark medieval boss battle, pounding war drums, low brass horns, choir chanting wordless syllables, fast bowed strings, hurdy-gurdy, phrygian mode, 150 bpm, menacing and huge, starts immediately, no vocals with words, seamless loop

**combat_framework.** Style:
> final boss battle against a corrupted colossal machine, medieval epic, massive war drums, pipe organ, wordless choir, distorted hurdy-gurdy, bells, phrygian dominant, 160 bpm, overwhelming and dramatic, builds in waves, starts immediately, no synth, no vocals with words, seamless loop

## Rekkefølge

Lag først `town_day`, `crypt_grav` og `combat`. De dekker starten av spillet, og jeg kan teste overgangene med dem før du lager resten.
