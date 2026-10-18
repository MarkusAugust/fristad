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
// før; mangler kampsporet, spiller det som spilte. Inne i Vertshuset synger
// trubaduren (`tavern`, `ambience.tavern` fra økta: 1 inne, svakere og
// dempet utenfor døra) i stedet for byens spor.

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
const FIGHT_HOLD = 6;
const VOLUME = 0.55;
// Natta i byen når bare dagsporet finnes: det dempes og mørknes.
const NIGHT_GAIN = 0.6;
const NIGHT_LOWPASS = 1100;

// Trubaduren: hvor dempet han er utenfor Vertshuset (lavpass gjennom veggene).
const TAVERN_OUTSIDE_LOWPASS = 900;

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
  // Vertshuset: 1 inne, mindre utenfor døra (`ambience.tavern`).
  let tavern = 0;
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
    const hushed = name.startsWith("town") && insideTavern();
    f.filter.frequency.setTargetAtTime(dark ? NIGHT_LOWPASS : 20000, c.currentTime, secs / 3);
    f.level.gain.setTargetAtTime(hushed ? 0 : dark ? NIGHT_GAIN : 1, c.currentTime, secs / 3);
  };
  // Et filspor: lastes første gang det trengs, og spiller i løkke gjennom
  // sitt eget filter og sin egen styrke, så det kan tones inn og ut.
  const file = (name) => {
    if (files[name]) return files[name];
    const f = { el: null, gain: null, filter: null, level: null, ok: undefined };
    files[name] = f;
    try {
      const el = new Audio();
      el.loop = true;
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
  const fadeOut = (name, secs) => {
    const f = files[name];
    if (!f?.gain) return;
    f.gain.gain.cancelScheduledValues(c.currentTime);
    f.gain.gain.setTargetAtTime(0, c.currentTime, secs / 3);
    setTimeout(() => {
      if (playing !== name) f.el.pause();
    }, secs * 1000 + 200);
  };
  const fadeIn = (name, secs) => {
    const f = files[name];
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
    // Kampsporet lastes mens du er nede, så det er klart når det trengs.
    if (mode === "cellar") file("combat");
    let pick = null;
    for (const k of chain) {
      const ok = file(k).ok;
      if (ok === undefined) return; // Ser etter fila; det som spiller, fortsetter.
      if (ok) {
        pick = k;
        break;
      }
    }
    // Kampen kommer fort, og alt annet tones over langsomt.
    const secs = combat ? 1.2 : 3;
    if (pick) {
      if (playing !== pick) {
        if (playing) fadeOut(playing, secs);
        playing = pick;
        tone(pick, secs);
        fadeIn(pick, secs);
      }
      if (score.current) score.stop(secs);
      return;
    }
    if (playing) {
      fadeOut(playing, 3);
      playing = null;
    }
    ready.then(() => {
      if (playing) return;
      if (insideTavern()) score.stop(2);
      else scoreFor(mode);
    });
  };
  // Trubaduren i Vertshuset: et eget spor over byens, så sterkt som du er
  // nær (inne er det alt), dempet gjennom veggene utenfor; byens spor tones
  // ned så lenge du er inne.
  const insideTavern = () => mode === "town" && tavern >= 0.99 && files.tavern?.ok === true;
  let wasInside = false;
  const mixTavern = () => {
    if (!c) return;
    const want = mode === "town" ? tavern : 0;
    if (want > 0.01) file("tavern");
    const f = files.tavern;
    const on = f?.ok === true && want > 0.01;
    if (f?.gain) {
      f.gain.gain.setTargetAtTime(on ? want : 0, c.currentTime, 0.6);
      f.filter.frequency.setTargetAtTime(want >= 0.99 ? 20000 : TAVERN_OUTSIDE_LOWPASS, c.currentTime, 0.4);
      if (on && f.el.paused) f.el.play().catch(() => {});
      if (!on && !f.el.paused) setTimeout(() => (tavern > 0.01 && mode === "town") || f.el.pause(), 2500);
    }
    // Inne i Vertshuset tones byens spor (eller partituret) ned.
    const inside = insideTavern();
    if (playing) tone(playing, 2);
    if (inside && score.current) score.stop(2);
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
      if (Math.abs(t - tavern) < 0.02 && !(t >= 0.99 && tavern < 0.99) && !(t < 0.99 && tavern >= 0.99)) return;
      tavern = t;
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
    // Til testing: hvilket stykke som spiller.
    get song() {
      if (playing) return playing;
      return mode === "town" && song >= 0 ? SONGS[song].name : null;
    },
  };
}

export { CUES, SONGS, tune };
