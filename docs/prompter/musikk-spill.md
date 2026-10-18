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

Inne i Vertshuset synger en trubadur sagaen om Gorvek (eieren 18. oktober): på engelsk, med spenningskurve fra stille ved ilden til storm i kampen og stille igjen. Gorvek er tegnet etter Robert E. Howards Conan: han ber aldri og spør aldri, tar livet i strupen og krever sin egen skjebne, forakter de siviliserte, lever for øyeblikket, har sin egen æresregel og en melankoli under råskapen. Sangen bygger på kanon (`docs/kanon.md`: oppgraderingen, Rammeverket, at det kunne deles, forseglingen, Ansgar) og Gallowmark (github.com/MarkusAugust/gallowmark: Bonereach, the Ashfall, hoppa uten navn, Gråkappenes krok). Bare kanonordene som vink til webfolk (the great upgrade, the Framework, the Council, the Office). Fila er `assets/music/game/tavern.mp3`.

Slå **av** Instrumental i Suno. Lengde 4–5 min; forleng i Suno om den slutter før [Outro]. Én trubadur, ingen kor (eieren: Suno lagde en power ballad hele veien; stilen beskriver nå én mann med lutt og stor forskjell mellom stille og sterkt, og taggene sier hvor lavt hver del er).

| Fil | Title | Lengde |
|---|---|---|
| `tavern.mp3` | The Saga of Gorvek | 4–5 min |

**Style:**
> solo medieval troubadour, one male singer alone with a lute, intimate tavern storytelling, deep weathered baritone, mostly quiet and sparse, half-spoken verses, dramatic dynamics from near-whisper to raw shouting, a single frame drum only in the climax, rubato, dorian minor, acoustic, no choir, no backing vocals, no band, not a power ballad, English lyrics

**Exclude styles** (om Suno-versjonen har feltet):
> choir, backing vocals, orchestra, drum kit, electric guitar, synth, power ballad, pop

**Lyrics:**
```
[Intro]
[Solo lute, slow, quiet tavern murmur]

[Verse 1]
[Soft, half-spoken, close to the fire]
Pour me one more and I'll sing you a man,
Born in the Bonereach where the cold winds ran.
When the Ashfall came, he walked into the grey,
And he walked back out with iron, and none asked what he ate.

[Verse 2]
[Soft, lute picking]
He rides a grey mare that he never has named,
For names are for priests and the rivers they've tamed.
The Greycloaks marked him with a hook and a scar,
They raise his price each winter, and they watch him from afar.

[Pre-Chorus]
[A little stronger]
He drinks when he's thirsty, he fights when he's free,
He sleeps where he falls, and he bends no knee.

[Chorus]
[Sung, medium, steady strumming]
Never asked, never begged,
never knelt to a crown,
He takes life by the throat
and he won't put it down.
No god in his mouth and no lord at his gate,
Gorvek of Bonereach claims his own fate.

[Verse 3]
[Harder strumming, voice rising, scornful]
The Council sat soft in their silk and their gold,
The Office wrote rules for the young and the old.
They never held iron, they never held a quill,
He laughed at their law, and he's laughing still.

[Verse 4]
[Drop to quiet, slow, sad]
He keeps every word, though he gives but a few,
He won't raise his axe to the weak or the true.
But some nights by the fire he stares at the flame,
At the grey that took all, and he won't speak its name.

[Chorus]
[Quiet, almost whispered]
Never asked, never begged,
never knelt to a crown,
He takes life by the throat
and he won't put it down.
No god in his mouth and no lord at his gate,
Gorvek of Bonereach claims his own fate.

[Instrumental Break]
[Lute alone, tension building, faster]

[Bridge]
[Building, frame drum enters, faster]
Then came the night of the great upgrade,
And all that was built broke apart and decayed.
The Framework rose up, taller than the spire,
With the King's seal burning like a crown of fire.
The town fell to its knees and it begged for leave,
But one never asked, and one did not grieve.

[Climax]
[Loud, raw, shouted, hard strumming and frame drum]
He came through the rain with his axe on his back,
He never looked up at the sky turning black.
He struck where the joints were, the whole night long,
Till the grey of the morning, till the night was gone.
It could not die!
[Sudden silence]
[Whispered]
But it could be split.

[Chorus]
[Loud, defiant, solo voice]
Never asked, never begged,
never knelt to a crown,
He takes life by the throat
and he won't put it down.
No god in his mouth and no lord at his gate,
Gorvek of Bonereach claims his own fate.

[Breakdown]
[Lute only, very quiet, slow]
They offered him gold and a seat in the hall,
He turned and he spat, and he took none at all.
His men bore the pieces in chains down below,
And Master Ansgar went last, with a lantern's glow.

[Final Verse]
[Soft, melancholic, slower]
He rode on the mare that he never has named,
He never looked back at the town that he saved.
No one in Fristad has seen him since then,
But down in the crypt, when the hammering starts again,
Something remembers the blow.

[Outro]
[Whispered, lute fading]
Never asked, never begged...
Gorvek of Bonereach...
[End]
```
