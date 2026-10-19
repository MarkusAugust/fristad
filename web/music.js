// Musikken: ekte instrumentprøver (FluidR3_GM, assets/music/) spilt etter
// noter av partiturspilleren i `score.js`. Oppe i byen spilles fem
// middelalderstykker etter tur, ett nytt hvert halvannet minutt og så
// forfra (estampie, cantiga, carole, klagesang og saltarello i
// kirketonearter, med skalmeie, blokkfløyte og fele over lutt eller harpe,
// tromme og bordun fra sekkepipe eller cello). Nede i krypten dyp bordun,
// orgel som knapt høres, bruddstykker av Dies irae i koret og fjerne
// klokker. Overgangene tones over på tre sekunder. M slår den av og på.
// Til prøvene er lastet, er det stille.
//
// Egne spor fra filer (eieren 18. oktober, Suno, promptene i
// `docs/prompter/musikk-spill.md`): `music/game/<navn>.mp3` (kopiert fra
// `assets/music/game/` av `tools/build-web.sh`). `town` spiller i hele byen,
// om natta dempet og mørkere gjennom et lavpassfilter; `crypt` i alle
// nivåene (`town_night` om natta, og om dagen så lenge `town` mangler;
// `town` om natta mørknes med et lavpass); `combat` tones inn når en fiende som har sett deg er nær
// (`hud.fight` fra økta: 1 fiende, 2 boss, 3 Rammeverket; bossene og
// Rammeverket bruker `combat_boss` om den finnes) og ut igjen `FIGHT_HOLD`
// sekunder etter at kampen er over. Mangler et spor, spiller partituret som
// før; mangler kampsporet, spiller det som spilte. Trubaduren i Vertshuset
// synger kvadet om Gorvek (`tavern`) bare når noen har kjøpt ham et krus mjød
// (`song(true)` fra `Event::Song`), én gang fra start til slutt; det høres i
// alle etasjene (`ambience.tavern` fra økta: 1 i skjenkestua, litt svakere
// oppe og i kjelleren, dempet utenfor døra). Inne i huset tones byens spor
// ned: helt mens han synger, ellers til `TAVERN_HUSH`.

import { arp, beats, createScore, lengthOf, midi, repeat, shift, tune } from "./score.js";

// Stykkene. `bar` er åttedeler per takt, `step` sekunder per åttedel,
// `gain` jevner ut styrken mellom dem, `roots` grunntonen i hver takt (lutta og harpa spiller kvint og oktav
// over den, som i middelalderen), `drum` slagene i en takt (0 er stille).
const SONGS = [
  {
    name: "Estampie i dorisk",
    gain: 0.78,
    step: 0.19,
    bar: 6,
    lead: "shanai",
    accomp: "lute",
    drone: ["D3", "A3"],
    droneInst: "bagpipe",
    drum: [1, 0, 0.4, 0.7, 0, 0.4],
    roots: "D C F D F A C D A D C A F D C D",
    melody: tune(`
      D5/2 E5/1 F5/2 E5/1 | D5/2 C5/1 D5/3 | F5/2 G5/1 A5/2 G5/1 | F5/2 E5/1 D5/3 |
      A5/2 G5/1 F5/2 G5/1 | A5/2 C6/1 A5/3 | G5/1 F5/1 E5/1 F5/2 E5/1 | D5/6 |
      A5/3 C6/3 | D6/2 C6/1 A5/3 | G5/2 A5/1 G5/2 F5/1 | E5/2 D5/1 E5/3 |
      F5/2 G5/1 A5/2 G5/1 | F5/2 E5/1 D5/2 C5/1 | D5/2 E5/1 C5/2 E5/1 | D5/6`),
  },
  {
    name: "Cantiga",
    step: 0.25,
    bar: 6,
    lead: "recorder",
    accomp: "lute",
    drone: ["D3", "A3"],
    droneInst: "cello",
    drum: [0.8, 0, 0, 0, 0.35, 0],
    roots: "D D D A F C F A F D F C D A C D",
    melody: tune(`
      D5/2 F5/2 G5/2 | A5/4 G5/2 | F5/2 E5/2 D5/2 | E5/6 |
      F5/2 G5/2 A5/2 | C6/2 A5/2 G5/2 | A5/2 G5/2 F5/2 | E5/6 |
      A5/2 A5/2 C6/2 | D6/4 C6/2 | A5/2 G5/2 F5/2 | G5/6 |
      F5/2 E5/2 D5/2 | E5/2 F5/2 E5/2 | D5/2 C5/2 E5/2 | D5/6`),
  },
  {
    name: "Carole i miksolydisk",
    gain: 0.98,
    step: 0.21,
    bar: 8,
    lead: "fiddle",
    accomp: "strum",
    drone: ["G2", "D3"],
    droneInst: "bagpipe",
    drum: [0.9, 0, 0, 0, 0.6, 0, 0.3, 0],
    roots: "G C G D G C F G D C G D G G F G",
    scale: [7, 9, 11, 0, 2, 4, 5],
    melody: tune(`
      G4/2 B4/2 D5/2 B4/2 | C5/2 D5/2 E5/4 | D5/2 C5/2 B4/2 A4/2 | G4/4 A4/4 |
      B4/2 D5/2 G5/2 F5/2 | E5/2 D5/2 C5/4 | B4/2 C5/2 A4/2 F4/2 | G4/8 |
      D5/2 D5/1 E5/1 F5/2 D5/2 | E5/2 C5/2 D5/4 | B4/2 C5/2 D5/2 B4/2 | A4/4 F4/4 |
      G4/2 A4/2 B4/2 C5/2 | D5/2 B4/2 A4/4 | B4/2 A4/2 F4/2 A4/2 | G4/8`),
  },
  {
    name: "Klagesang",
    gain: 1.17,
    step: 0.32,
    bar: 6,
    lead: "recorder",
    accomp: "harp",
    drone: ["A2", "E3"],
    droneInst: "cello",
    drum: null,
    roots: "A G A A A C F E C G G A G G E A",
    scale: [9, 11, 0, 2, 4, 5, 7],
    melody: tune(`
      A4/4 C5/2 | B4/2 A4/2 G4/2 | A4/4 E4/2 | A4/6 |
      C5/4 D5/2 | E5/2 D5/2 C5/2 | B4/2 C5/2 A4/2 | B4/6 |
      E5/4 F5/2 | E5/2 D5/2 C5/2 | D5/2 C5/2 B4/2 | A4/6 |
      G4/2 A4/2 C5/2 | B4/2 A4/2 G4/2 | E4/2 G4/2 B4/2 | A4/6`),
  },
  {
    name: "Saltarello",
    gain: 0.78,
    step: 0.17,
    bar: 6,
    lead: "shanai",
    accomp: "lute",
    drone: ["E3", "B3"],
    droneInst: "bagpipe",
    drum: [1, 0.3, 0.5, 0.8, 0, 0.5],
    roots: "E D E E A G D E D D E E",
    scale: [4, 6, 7, 9, 11, 1, 2],
    melody: tune(`
      E5/1 F#5/1 G5/1 A5/2 G5/1 | F#5/1 E5/1 D5/1 E5/3 | B5/2 A5/1 G5/2 F#5/1 | E5/2 D5/1 E5/3 |
      G5/1 A5/1 B5/1 C#6/2 B5/1 | A5/1 G5/1 F#5/1 G5/3 | A5/2 G5/1 F#5/2 D5/1 | E5/6 |
      D5/1 E5/1 F#5/1 G5/2 E5/1 | F#5/2 D5/1 E5/3 | B4/1 D5/1 E5/1 F#5/2 A5/1 | G5/2 F#5/1 E5/3`),
  },
];
const SONG_SECONDS = 90;

// Akkompagnementet over grunntonene: lutt som plukker grunntone, kvint og
// oktav på hver åttedel; lutt som slår klangen på hvert slag; eller harpe
// som brytes langsommere.
function accomp(s) {
  const len = s.bar;
  const cycle = (pattern) => Array.from({ length: len }, (_, i) => pattern[i % pattern.length]);
  const bass = cycle([-12, ...Array(len - 1).fill(null)]);
  if (s.accomp === "strum") {
    const notes = s.roots.split(" ").flatMap((r) => {
      const root = midi(r + 3);
      return Array.from({ length: len / 2 }, (_, i) => ({ notes: [root, root + 7, root + 12], midi: root, len: 2, vel: i === 0 ? 1 : 0.7 }));
    });
    return [
      { inst: "acoustic_guitar_nylon", notes, vel: 0.5, pan: 0.2 },
      { inst: "acoustic_guitar_nylon", notes: arp(s.roots, bass, 3), vel: 0.6, pan: 0.1 },
    ];
  }
  if (s.accomp === "harp") {
    return [
      { inst: "orchestral_harp", notes: arp(s.roots, cycle([0, null, 19, null, 12, null, 7, null]), 3), vel: 0.55, pan: 0.25 },
      { inst: "orchestral_harp", notes: arp(s.roots, bass, 3), vel: 0.55, pan: 0.15 },
    ];
  }
  return [
    { inst: "acoustic_guitar_nylon", notes: arp(s.roots, cycle([0, 7, 12, 7]), 3), vel: 0.6, pan: 0.2 },
    { inst: "acoustic_guitar_nylon", notes: arp(s.roots, bass, 3), vel: 0.65, pan: 0.1 },
  ];
}

// Ett stykke som cue: melodien (pyntet med forslagsnoter fra andre gang),
// tredje gang dobles den en oktav under av blokkfløyta (eller fela), og under ligger
// akkompagnementet, trommen og borduneten.
function songCue(s) {
  const loop = lengthOf(s.melody);
  const parts = [
    { inst: s.lead, notes: s.melody, vel: s.lead === "recorder" ? 0.85 : 0.8, pan: -0.2, grace: 0.45 },
    ...accomp(s),
    { inst: s.droneInst, notes: [{ notes: s.drone.map(midi), midi: midi(s.drone[0]), len: loop * 2, vel: 1 }], vel: s.droneInst === "bagpipe" ? 0.6 : 0.7, pan: 0.05 },
  ];
  if (s.lead !== "recorder") parts.push({ inst: "recorder", notes: shift(s.melody, -12), vel: 0.4, pan: 0.35, when: (p) => p % 3 === 2 });
  else {
    parts.push({ inst: "fiddle", notes: shift(s.melody, -12), vel: 0.35, pan: 0.35, when: (p) => p % 3 === 2 });
    // Andre gang gjennom slår psalteret melodien med fløyta.
    parts.push({ inst: "dulcimer", notes: s.melody, vel: 0.45, pan: 0.3, when: (p) => p % 3 === 1 });
  }
  if (s.drum) {
    parts.push({ inst: "taiko_drum", notes: beats(s.drum, 1, "C4"), vel: 0.7 });
    parts.push({ inst: "taiko_drum", notes: beats([1, ...Array(s.bar * 2 - 1).fill(0)], 1, "C3"), vel: 0.45 });
  }
  return { step: s.step, gain: s.gain ?? 0.85, wet: 0.3, scale: s.scale, parts };
}

// Krypten: som filmens krypt, men uten hjerteslag (spillet har sitt eget ved
// lavt liv), og lengre mellom bruddstykkene.
const CELLAR = {
  step: 0.36,
  gain: 1.6,
  wet: 0.7,
  parts: [
    { inst: "contrabass", notes: tune("D2/96"), vel: 0.85 },
    { inst: "cello", notes: tune("D2/48 Eb2/48 D2/48 C#2/48"), vel: 0.5, pan: 0.2, lp: 700 },
    { inst: "church_organ", notes: tune("D2+A2/96 D2+Ab2/96"), vel: 0.35, lp: 500 },
    { inst: "choir_aahs", notes: tune("-/40 F4/4 E4/4 F4/4 D4/10 -/34 | -/56 E4/4 C4/4 D4/12 -/36"), vel: 0.5, pan: -0.3, lp: 1400, wet: 0.95, legato: 1 },
    { inst: "choir_aahs", notes: tune("-/40 Bb3/4 A3/4 Bb3/4 G3/10 -/34 | -/56 A3/4 F3/4 G3/12 -/36"), vel: 0.3, pan: 0.3, lp: 1200, wet: 0.95, legato: 1 },
    { inst: "tubular_bells", notes: tune("D3/32 -/40 G#2/32 -/56 A2/32"), vel: 0.45, pan: 0.4, lp: 1800, wet: 0.95 },
    { inst: "timpani", notes: repeat(tune("-/60 D2/2:0.6 D2/2:0.4 -/32"), 1), vel: 0.6, lp: 700, wet: 0.9 },
  ],
};

const CUES = Object.fromEntries([...SONGS.map((s, i) => [`town${i}`, songCue(s)]), ["cellar", CELLAR]]);

// Sporene fra filer, og sporet som står i stedet når ett mangler (`null`:
// partituret, for kampen det som spilte der du står).
const TRACKS = { town: "town_night", town_night: "town", crypt: null, combat: null, combat_boss: "combat" };
// Så lenge etter at kampen er over spiller kampsporet før det tones ut.
// Kort hold etter kampen, og kampsporet tones raskt ut (eieren 18. oktober:
// det hang igjen for lenge).
const FIGHT_HOLD = 2;
const COMBAT_OUT = 1.2;
// Kampsporet tones inn over så lang tid, starter aldri de siste
// `COMBAT_TAIL` sekundene, og fortsetter der det var om en ny kamp kommer
// innen `COMBAT_RESUME` sekunder.
const COMBAT_FADE = 1.5;
const COMBAT_TAIL = 20;
const COMBAT_RESUME = 20;

/** Frasegrensene i et spor: der lydstyrken (i biter på 50 ms) stiger brått
 * mot det som var før, minst fire sekunder fra hverandre. */
export function phraseStarts(audio) {
  const data = audio.getChannelData(0);
  const hop = Math.max(1, Math.round(audio.sampleRate * 0.05));
  const env = [];
  for (let i = 0; i + hop <= data.length; i += hop) {
    let e = 0;
    for (let j = i; j < i + hop; j++) e += data[j] * data[j];
    env.push(Math.sqrt(e / hop));
  }
  // Stigningen mot snittet av sekundet før.
  const rise = env.map((e, i) => {
    let m = 0;
    for (let k = Math.max(0, i - 20); k < i; k++) m += env[k];
    return e - m / Math.max(1, Math.min(20, i));
  });
  const order = rise.map((r, i) => [r, i]).filter(([r]) => r > 0).sort((a, b) => b[0] - a[0]);
  const picked = [];
  for (const [, i] of order) {
    const t = Math.max(0, i * 0.05 - 0.05);
    if (picked.every((p) => Math.abs(p - t) >= 4)) picked.push(t);
    if (picked.length >= 24) break;
  }
  return picked.sort((a, b) => a - b);
}
const VOLUME = 0.55;
// Natta i byen når bare dagsporet finnes: det dempes og mørknes.
const NIGHT_GAIN = 0.6;
const NIGHT_LOWPASS = 1100;

// Trubaduren: hvor dempet han er utenfor Vertshuset (lavpass gjennom
// veggene) og i de andre etasjene (gjennom gulvplankene).
const TAVERN_OUTSIDE_LOWPASS = 900;
const TAVERN_FLOOR_LOWPASS = 2400;
// Fra så sterkt `ambience.tavern` er du inne i Vertshuset (alle etasjene).
const TAVERN_HOUSE = 0.6;
// Byens spor inne i Vertshuset når trubaduren ikke synger.
const TAVERN_HUSH = 0.35;

/** Hvilket spor som skal spille: `where` er "town", "cellar" eller
 * "silence", `fight` kampen (0–3) og `night` om det er natt. */
export function trackFor({ where, fight = 0, night = false }) {
  if (where === "silence") return null;
  if (where === "cellar") return fight >= 2 ? "combat_boss" : fight === 1 ? "combat" : "crypt";
  return night ? "town_night" : "town";
}

export function createMusic(context, base = "music/") {
  let c = null;
  let out = null;
  let score = null;
  let ready = null;
  let mode = null;
  let muted = false;
  let song = -1;
  let songStart = 0;
  let timer = null;
  let night = false;
  // Vertshuset: 1 i skjenkestua, mindre i de andre etasjene og utenfor
  // døra (`ambience.tavern`), og om trubaduren synger (`song`).
  let tavern = 0;
  let singing = false;
  // Kalles når kvadet er spilt til endes (motoren får vite det).
  let onSongEnd = null;
  let fight = 0;
  // Den hardeste kampen siden kampen begynte (bossen holder seg til den er over).
  let lastFight = 0;
  let fightUntil = 0;
  // Filsporene: navn → { el, gain, filter, ok (true, false, eller undefined mens vi ser etter) }.
  const files = {};
  let playing = null;
  try {
    muted = localStorage.getItem("fristad.music") === "av";
  } catch {
    /* ingen lagring */
  }
  const now = () => performance.now() / 1000;
  const setup = () => {
    if (c) return true;
    c = context();
    if (!c) return false;
    out = c.createGain();
    out.gain.value = muted ? 0 : VOLUME;
    out.connect(c.destination);
    score = createScore(c, "music/", { cues: CUES, out });
    ready = score.load();
    return true;
  };
  // Lyden i byen etter døgnet (de andre sporene er uendret).
  const tone = (name, secs) => {
    const f = files[name];
    if (!f?.filter) return;
    if (name === "tavern") return;
    const dark = name === "town" && night;
    const inside = name.startsWith("town") && insideTavern();
    const song = inside && singing && files.tavern?.ok === true;
    f.filter.frequency.setTargetAtTime(inside ? TAVERN_OUTSIDE_LOWPASS * 2 : dark ? NIGHT_LOWPASS : 20000, c.currentTime, secs / 3);
    f.level.gain.setTargetAtTime(song ? 0 : inside ? TAVERN_HUSH : dark ? NIGHT_GAIN : 1, c.currentTime, secs / 3);
  };
  // Et filspor: lastes første gang det trengs, og spiller i løkke gjennom
  // sitt eget filter og sin egen styrke, så det kan tones inn og ut.
  // Sporene som finnes (`game/tracks.json` fra tools/build-web.sh), så et spor
  // som mangler aldri hentes (det ga 404 i konsollen). Mangler lista, prøves alle.
  let known;
  fetch(`${base}game/tracks.json`)
    .then((r) => (r.ok ? r.json() : null))
    .then((list) => (known = Array.isArray(list) ? new Set(list) : null))
    .catch(() => (known = null))
    .then(() => c && apply());
  const file = (name) => {
    if (files[name]) return files[name];
    if (known === undefined) return { ok: undefined };
    if (known && !known.has(name)) return (files[name] = { ok: false });
    const f = { el: null, gain: null, filter: null, level: null, ok: undefined };
    files[name] = f;
    try {
      const el = new Audio();
      // Kvadet spilles én gang; resten går i løkke.
      el.loop = name !== "tavern";
      if (name === "tavern") el.addEventListener("ended", () => endSong(true));
      el.preload = "auto";
      el.crossOrigin = "anonymous";
      el.addEventListener("canplaythrough", () => {
        if (f.ok === undefined) {
          f.ok = true;
          apply();
          mixTavern();
        }
      });
      el.addEventListener("error", () => {
        f.ok = false;
        apply();
        mixTavern();
      });
      el.src = `${base}game/${name}.mp3`;
      f.el = el;
      f.filter = c.createBiquadFilter();
      f.filter.type = "lowpass";
      f.filter.frequency.value = 20000;
      f.level = c.createGain();
      f.gain = c.createGain();
      f.gain.gain.value = 0;
      c.createMediaElementSource(el).connect(f.filter);
      f.filter.connect(f.level);
      f.level.connect(f.gain);
      f.gain.connect(out);
      tone(name, 0.1);
    } catch {
      f.ok = false;
    }
    return f;
  };
  // Kampsporene starter ikke likt hver gang (eieren 18. oktober): et
  // tilfeldig startpunkt ved en frasegrense (der lydstyrken stiger brått,
  // regnet ut én gang når sporet lastes), aldri det samme to ganger på rad
  // og ikke de siste `COMBAT_TAIL` sekundene; kommer en ny kamp innen
  // `COMBAT_RESUME` sekunder, fortsetter sporet der det var.
  const starts = {};
  const lastStart = {};
  const ended = {};
  const findStarts = (name) => {
    if (starts[name]) return;
    starts[name] = [];
    fetch(`${base}game/${name}.mp3`)
      .then((r) => (r.ok ? r.arrayBuffer() : Promise.reject(new Error(`HTTP ${r.status}`))))
      .then((buf) => new OfflineAudioContext(1, 1, 8000).decodeAudioData(buf))
      .then((audio) => (starts[name] = phraseStarts(audio)))
      .catch(() => {});
  };
  const seekCombat = (name) => {
    const el = files[name].el;
    if (now() - (ended[name] ?? -Infinity) < COMBAT_RESUME) return;
    const dur = Number.isFinite(el.duration) ? el.duration : 0;
    let list = (starts[name] ?? []).filter((t) => t < dur - COMBAT_TAIL);
    // Uten utregningen: annenhver takt i 140 slag i minuttet (3,4 s).
    if (list.length < 2 && dur > COMBAT_TAIL + 4) list = Array.from({ length: Math.floor((dur - COMBAT_TAIL) / 3.43) }, (_, i) => i * 3.43);
    list = list.filter((t) => t !== lastStart[name]);
    if (!list.length) return;
    const t = list[Math.floor(Math.random() * list.length)];
    lastStart[name] = t;
    try {
      el.currentTime = t;
    } catch {
      /* ikke klar ennå: begynner fra start */
    }
  };
  const fadeOut = (name, secs) => {
    const f = files[name];
    if (!f?.gain) return;
    if (name.startsWith("combat")) ended[name] = now();
    f.gain.gain.cancelScheduledValues(c.currentTime);
    f.gain.gain.setTargetAtTime(0, c.currentTime, secs / 3);
    setTimeout(() => {
      if (playing !== name) f.el.pause();
    }, secs * 1000 + 200);
  };
  const fadeIn = (name, secs) => {
    const f = files[name];
    if (name.startsWith("combat")) seekCombat(name);
    f.gain.gain.cancelScheduledValues(c.currentTime);
    f.gain.gain.setTargetAtTime(1, c.currentTime, secs / 3);
    f.el.play().catch(() => {});
  };
  const nextSong = () => {
    song = (song + 1) % SONGS.length;
    songStart = c.currentTime;
    score.cue(`town${song}`, 3);
  };
  // Partituret der filene mangler.
  const scoreFor = (where) => {
    if (where === "town") {
      if (song < 0 || score.current !== `town${song}`) nextSong();
    } else if (where === "cellar") score.cue("cellar", 3);
    else score.stop(3);
  };
  // Det som skal spille nå: sporet fra fil om det finnes, ellers det som
  // står i stedet, og til slutt partituret.
  const apply = () => {
    if (!c || !mode) return;
    const home = trackFor({ where: mode, night });
    const want = trackFor({ where: mode, night, fight: fight > 0 || now() < fightUntil ? lastFight : 0 });
    const combat = want !== home;
    // Bosskampen faller tilbake på kampen, og kampen på sporet der du står.
    const chain = [];
    for (let k = want; k && !chain.includes(k); k = TRACKS[k]) chain.push(k);
    if (combat && home) chain.push(home);
    // Kampsporet lastes mens du er nede, så det er klart når det trengs,
    // og startpunktene regnes ut.
    if (mode === "cellar") {
      for (const k of ["combat", "combat_boss"]) if (file(k).ok) findStarts(k);
    }
    let pick = null;
    for (const k of chain) {
      const ok = file(k).ok;
      if (ok === undefined) return; // Ser etter fila; det som spiller, fortsetter.
      if (ok) {
        pick = k;
        break;
      }
    }
    // Kampen kommer fort (men ikke hardt), og alt annet tones over langsomt.
    const secs = combat ? COMBAT_FADE : 3;
    if (pick) {
      if (playing !== pick) {
        if (playing) fadeOut(playing, playing.startsWith("combat") && !combat ? COMBAT_OUT : secs);
        playing = pick;
        tone(pick, secs);
        fadeIn(pick, secs);
      }
      if (score.current) score.stop(secs);
      return;
    }
    if (playing) {
      fadeOut(playing, playing.startsWith("combat") ? COMBAT_OUT : 3);
      playing = null;
    }
    ready.then(() => {
      if (playing) return;
      if (insideTavern() && singing) score.stop(2);
      else scoreFor(mode);
    });
  };
  // Trubaduren i Vertshuset: et eget spor over byens mens han synger, så
  // sterkt som du er nær (skjenkestua er alt, de andre etasjene litt
  // mindre), dempet gjennom gulvet og veggene; byens spor tones ned så
  // lenge du er inne i huset.
  const insideTavern = () => mode === "town" && tavern >= TAVERN_HOUSE;
  // Kvadet er over: fila er ferdig (`fromFile`, motoren får vite det), eller
  // økta sa det.
  const endSong = (fromFile) => {
    if (!singing) return;
    singing = false;
    const f = files.tavern;
    if (f?.gain) f.gain.gain.setTargetAtTime(0, c.currentTime, 0.4);
    if (f?.el && !fromFile) setTimeout(() => singing || f.el.pause(), 1500);
    if (fromFile) onSongEnd?.();
    mixTavern();
  };
  let wasInside = false;
  // Sangens lydstyrke (`voice()`): undefined til den trengs, null mens den hentes.
  let voiceEnv;
  const mixTavern = () => {
    if (!c) return;
    // Kvadet går videre (lydløst) når du er borte, så det er der økta er
    // når du kommer tilbake; det stopper bare når det er over.
    const want = mode === "town" && singing ? tavern : 0;
    const f = singing ? file("tavern") : files.tavern;
    if (f?.gain) {
      f.gain.gain.setTargetAtTime(f.ok === true ? want : 0, c.currentTime, 0.6);
      f.filter.frequency.setTargetAtTime(tavern >= 0.99 ? 20000 : tavern >= TAVERN_HOUSE ? TAVERN_FLOOR_LOWPASS : TAVERN_OUTSIDE_LOWPASS, c.currentTime, 0.4);
      if (singing && f.ok === true && f.el.paused && !f.el.ended) f.el.play().catch(() => {});
    }
    // Inne i Vertshuset tones byens spor (eller partituret) ned.
    const inside = insideTavern();
    if (playing) tone(playing, 2);
    if (inside && singing && score.current) score.stop(2);
    if (wasInside && !inside) apply();
    wasInside = inside;
  };
  return {
    set(where) {
      if (!setup()) return;
      if (where === mode) return;
      mode = where;
      ready.then(apply);
      mixTavern();
      // Nytt stykke i byen når tiden er ute, og kampen som tones ut.
      timer ??= setInterval(() => {
        if (mode === "town" && !playing && score.current && c.currentTime - songStart > SONG_SECONDS) nextSong();
        if (fight === 0 && lastFight > 0 && now() >= fightUntil) {
          lastFight = 0;
          apply();
        }
      }, 1000);
    },
    /** Det økta sier (`hud`): kampen nede i krypten. */
    hud(e) {
      if (!c) return;
      const f = e.cellar ? e.fight || 0 : 0;
      if (f > 0) {
        lastFight = Math.max(lastFight, f);
        fightUntil = now() + FIGHT_HOLD;
      }
      if (f === fight) return;
      fight = f;
      apply();
    },
    /** Natt (mørkt tema): nattsporet, eller byens spor dempet og mørknet. */
    night(dark) {
      if (dark === night) return;
      night = dark;
      if (c) {
        tone("town", 4);
        apply();
      }
    },
    /** Lydbildet fra økta (`ambience`): hvor nær Vertshuset du er. */
    ambience(a) {
      const t = a.cellar ? 0 : a.tavern || 0;
      const crossed = (k) => t >= k !== tavern >= k;
      if (Math.abs(t - tavern) < 0.02 && !crossed(0.99) && !crossed(TAVERN_HOUSE)) return;
      tavern = t;
      mixTavern();
    },
    /** Trubaduren begynner kvadet fra start (`on`, noen kjøpte mjød), eller
     * det er over (`Event::Song`). `ended` kalles når fila er spilt ferdig. */
    song(on, ended) {
      if (ended) onSongEnd = ended;
      if (!setup()) return;
      if (!on) return endSong(false);
      singing = true;
      const f = file("tavern");
      if (f.el) {
        try {
          f.el.currentTime = 0;
        } catch {
          /* ikke klar ennå: begynner fra start likevel */
        }
        if (f.ok === true) f.el.play().catch(() => {});
      }
      mixTavern();
    },
    toggle() {
      if (!setup()) return;
      muted = !muted;
      out.gain.setTargetAtTime(muted ? 0 : VOLUME, c.currentTime, 0.3);
      try {
        localStorage.setItem("fristad.music", muted ? "av" : "på");
      } catch {
        /* ingen lagring */
      }
      return !muted;
    },
    /** Hvor sterkt trubaduren synger der sangen er nå (0..1), fra
     * `game/tavern_voice.json` (`tools/music/voice.py`, 20 verdier i
     * sekundet); 0 når sangen ikke spiller. Motoren åpner munnen etter det. */
    voice() {
      const f = files.tavern;
      if (!f?.el || f.el.paused || f.ok !== true) return 0;
      if (voiceEnv === undefined) {
        voiceEnv = null;
        fetch(`${base}game/tavern_voice.json`)
          .then((r) => (r.ok ? r.json() : null))
          .then((j) => (voiceEnv = j))
          .catch(() => {});
      }
      if (!voiceEnv?.values?.length) return 0;
      const i = Math.floor(f.el.currentTime * voiceEnv.rate);
      return (voiceEnv.values[Math.min(i, voiceEnv.values.length - 1)] ?? 0) / 100;
    },
    // Til testing: hvilket stykke som spiller.
    get song() {
      if (playing) return playing;
      return mode === "town" && song >= 0 ? SONGS[song].name : null;
    },
  };
}

export { CUES, SONGS, tune };
