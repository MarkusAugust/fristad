# Musikk til spillet (Suno)

Spillet spiller egne spor for byen, krypten og kampen (eieren 18. oktober). Eieren lager så få som mulig, så det er tre spor, pluss ett valgfritt. De legges i `assets/music/game/` med navnene under. Mangler et spor, spiller spillet det syntetiske partituret som før (`web/score.js`).

**Slik lager du dem i Suno:**
- Velg **Custom** og slå på **Instrumental**.
- Lim inn teksten under «Style» i feltet *Style of Music*, og navnet som *Title*.
- Velg den versjonen som holder samme stemning hele veien. Sporene går i løkke, så et spor uten tydelig slutt er best.
- Last ned som mp3 og send meg fila. Jeg klipper løkkepunktene og normaliserer lyden.

Middelalderske instrumenter som i filmen, ingen moderne trommer eller synth, ingen sang med ord.

| Fil | Title | Brukes | Lengde |
|---|---|---|---|
| `town.mp3` | Fristad | hele byen, dag og natt (dempes og mørknes om natta) | 3–4 min |
| `crypt.mp3` | Krypten | alle ti nivåene | 3–4 min |
| `combat.mp3` | Kamp | alle kamper, også bossene | 2–3 min |
| `combat_boss.mp3` (valgfritt) | Bossen | bossene og Rammeverket; mangler den, brukes `combat.mp3` | 2–3 min |

**town.** Style:
> medieval Nordic market town, 14th century, warm and calm, recorder and shawm melody, lute and hurdy-gurdy, soft frame drum, bagpipe drone, dorian mode, gentle folk dance feel, 90 bpm, quieter middle section, no vocals, seamless loop, no fade out

**crypt.** Style:
> dark medieval crypt ambient, wordless monk choir humming low, cello and bass drone, distant church bell, sparse plucked psaltery notes, slowly rising tension, phrygian mode, very slow 52 bpm, cold stone reverb, ominous but restrained, no drums, no vocals with words, seamless loop

**combat.** Style (eieren 18. oktober: mindre fele, mer bass):
> heavy dark medieval battle music, deep low war drums and booming frame drums, driving cello and double bass ostinato, low brass and horn stabs, deep male choir grunts, bagpipe drone low, no fiddle, no violin, no high strings, dorian minor, 135 bpm, heavy bass, aggressive, starts immediately without intro, no vocals with words, seamless loop

**combat_boss** (valgfritt). Style:
> epic dark medieval boss battle, massive low war drums and timpani, pipe organ pedal bass, low brass, cello and double bass ostinato, deep wordless male choir, no fiddle, no violin, no high strings, phrygian mode, 145 bpm, very heavy bass, menacing and huge, starts immediately, no synth, no vocals with words, seamless loop
