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

## Trubaduren i Vertshuset (med tekst)

Inne i Vertshuset synger en trubadur legenden om Gorvek (eieren 18. oktober). Sangen bygger på kanon (`docs/kanon.md`: Gorvek delte Rammeverket, forseglingen, Ansgar) og på Gallowmark-kanonen (github.com/MarkusAugust/gallowmark: Gorvek av Bonereach, Askefallet, hoppa uten navn, arret fra Gråkappenes krok, trosbekjennelsen). Ingen moderne ord. Fila er `assets/music/game/tavern.mp3`, og den spilles i stedet for byens spor mens du er inne i Vertshuset.

Slå **av** Instrumental i Suno, lim stilen inn i *Style of Music* og teksten under i *Lyrics*.

| Fil | Title | Lengde |
|---|---|---|
| `tavern.mp3` | Visa om Gorvek | 3–4 min |

**Style:**
> medieval troubadour ballad sung in Norwegian, solo male storyteller voice, warm rough baritone, lute and frame drum, soft hurdy-gurdy drone, tavern ambience with quiet crowd murmur and a few voices joining the chorus, dorian mode, 86 bpm, storytelling folk, intimate and epic, no modern instruments

**Lyrics:**
```
[Intro]
[Solo lute]

[Verse 1]
Hør nå, dere som sitter ved ilden,
om mannen fra Bonereach der vinden er hard.
Han gikk inn i gråmen da Askefallet kom,
og ut kom han siden med jern i sin hand.

[Verse 2]
Han rir på ei hoppe han aldri ga navn,
for navn er for prester, og prester for elva.
Hans venstre arm bærer et merke fra kroken,
og Gråkappene ber om hans hode hver vinter.

[Chorus]
Gorvek, Gorvek, med øksa på skuldra,
han spør ikke om lov, og han ser ikke opp.
Det flyter når han sier flyt,
det stanser når han sier stopp.

[Verse 3]
Det var natta den store oppgraderingen kom,
og alt som var bygd, brakk i ett eneste slag.
Rammeverket reiste seg høyere enn kirka,
og ingen i byen sto opp mot det da.

[Verse 4]
Men én hadde aldri spurt noen om lov,
han kom gjennom regnet med øksa i hand.
Han hogg der leddene satt, hele natta,
og løste hvert segl og hvert lenkebånd.

[Chorus]
Gorvek, Gorvek, med øksa på skuldra,
han spør ikke om lov, og han ser ikke opp.
Det flyter når han sier flyt,
det stanser når han sier stopp.

[Bridge]
[spoken, slow]
De spurte ham siden hva en kriger vil ha.
Han svarte: At det flyter når jeg sier flyt.
At det stanser når jeg sier stopp.
Og at det bærer min vilje til andre bredden,
og ingenting mer.

[Verse 5]
Det kunne ikke dø, men det kunne deles,
og munkene lokket det ned under kirka.
Hans menn bar stykkene, lenket, én etter én,
og mester Ansgar gikk sist inn i mørket.

[Verse 6]
Så red han videre på hoppa uten navn,
og ingen i Fristad har sett ham siden.
Men hører du hamring langt nede i krypten,
er det noe der nede som husker hans hugg.

[Final Chorus]
Gorvek, Gorvek, med øksa på skuldra,
han spør ikke om lov, og han ser ikke opp.
Det flyter når han sier flyt,
det stanser når han sier stopp.

[Outro]
[Solo lute, fading]
```
