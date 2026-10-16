// Partituret: ekte instrumentprøver (FluidR3_GM, se assets/music/CREDITS.md)
// spilt etter noter skrevet som data. Prøvespilleren henter noen få toner
// per instrument (`assets/music/<instrument>/<tone>.mp3`) og spiller
// mellomtonene med playbackRate; de holdte tonene (fløyte, kor, orgel,
// bordun) får en løkke med innbakt krysstoning, så de kan vare så lenge
// noten sier. Alt går gjennom etterklangen fra et steinrom.
//
// Cuene er filmens deler (dawn, scriptorium, fall, burial, crypt, town,
// gate), og hver går i løkke til neste tones over. Spillets musikk
// (`music.js`) bruker samme spiller med sine egne cuer.
//
//   const score = createScore(context, "music/");
//   await score.load();
//   score.cue("dawn");        // tones inn
//   score.cue("scriptorium"); // krysstones over på 2 sekunder
//   score.stop(3);

// --- Notene ----------------------------------------------------------------

// Notene skrives «D5/2»: tonen og varigheten i åttedeler. «-/2» er pause,
// «D4+A4/4» er en klang, og «D5/2:0.6» gir tonen styrke 0,6.
const NAMES = { C: 0, D: 2, E: 4, F: 5, G: 7, A: 9, B: 11 };
export function midi(name) {
  const m = /^([A-G])(#|b)?(-?\d)$/.exec(name);
  if (!m) return null;
  return NAMES[m[1]] + (m[2] === "#" ? 1 : m[2] === "b" ? -1 : 0) + (Number(m[3]) + 1) * 12;
}
export function tune(text) {
  return text
    .split(/[\s|]+/)
    .filter(Boolean)
    .map((tok) => {
      const [body, vel] = tok.split(":");
      const [n, d] = body.split("/");
      const notes = n === "-" ? [] : n.split("+").map(midi);
      if (notes.some((m) => m === null) || !(Number(d) > 0)) throw new Error(`Ukjent note «${tok}»`);
      return { notes, midi: notes[0] ?? null, len: Number(d), vel: vel ? Number(vel) : 1 };
    });
}
const note = (notes, len, vel = 1) => ({ notes, midi: notes[0] ?? null, len, vel });

// Hjelperne som setter sammen stemmene.
export const cat = (...tunes) => tunes.flat();
export const rest = (len) => [note([], len)];
export const lengthOf = (t) => t.reduce((s, n) => s + n.len, 0);
export const repeat = (t, times) => Array.from({ length: times }, () => t).flat();
// Flytter en stemme `semis` halvtoner (parallelle kvinter og kvarter).
export const shift = (t, semis) => t.map((n) => note(n.notes.map((m) => m + semis), n.len, n.vel));
// Hoketus: annenhver tone i melodien, resten pause (`odd` velger hvilke).
export function hocket(t, odd) {
  let k = 0;
  return t.map((n) => (n.notes.length && k++ % 2 !== (odd ? 1 : 0) ? note([], n.len) : n));
}
// Brutt akkord over grunntonene (en per takt): `pattern` er intervallene
// over grunntonen, én åttedel hver (null er pause), som lutt og harpe
// spilte i middelalderen: grunntone, kvint og oktav.
export function arp(roots, pattern, octave = 3, len = 1) {
  return roots
    .split(/\s+/)
    .filter(Boolean)
    .flatMap((r) => {
      const root = midi(r + octave);
      return pattern.map((iv, i) => note(iv === null ? [] : [root + iv], len, i === 0 ? 1 : 0.72));
    });
}
// Slagene i en takt (styrke per åttedel, 0 er stille), `bars` ganger.
export function beats(pattern, bars = 1, tone = "C4") {
  const m = midi(tone);
  return repeat(pattern.map((v) => (v > 0 ? note([m], 1, v) : note([], 1))), bars);
}

// --- Instrumentene ---------------------------------------------------------

// Tonene som finnes (holdes i takt med tools/music/fetch.sh), hva slags
// instrument det er og hvor sterkt det står i blandingen. `sustain` holder
// tonen så lenge noten varer, `pluck` klinger ut av seg selv (`ring`
// sekunder), `hit` spilles helt ut. `cents` retter prøver som ikke er stemt
// (ett tall for alle, eller per tone).
export const INSTRUMENTS = {
  recorder: { notes: "Eb4 Gb4 A4 C5 Eb5 Gb5 A5 C6 Eb6", kind: "sustain", gain: 0.42, attack: 0.03, release: 0.12 },
  shanai: { notes: "C4 Eb4 Gb4 A4 C5 Eb5 Gb5 A5 C6", kind: "sustain", gain: 0.3, attack: 0.02, release: 0.08 },
  fiddle: { notes: "G3 Bb3 Db4 E4 G4 Bb4 Db5 E5 G5 Bb5", kind: "sustain", gain: 0.4, attack: 0.07, release: 0.18 },
  acoustic_guitar_nylon: { notes: "C2 E2 Ab2 C3 E3 Ab3 C4 E4 Ab4 C5", kind: "pluck", gain: 0.5, ring: 0.9 },
  orchestral_harp: { notes: "C2 E2 Ab2 C3 E3 Ab3 C4 E4 Ab4 C5 E5 Ab5", kind: "pluck", gain: 0.5, ring: 3.2 },
  dulcimer: { notes: "C4 E4 Ab4 C5 E5 Ab5", kind: "pluck", gain: 0.35, ring: 2.2 },
  // De dype korprøvene ligger lavt i FluidR3 (målt med autokorrelasjon), orgelet litt høyt.
  choir_aahs: {
    notes: "C3 Eb3 Gb3 A3 C4 Eb4 Gb4 A4 C5",
    kind: "sustain",
    gain: 0.5,
    attack: 0.28,
    release: 0.6,
    cents: { C3: 19, Eb3: 38, Gb3: 26, A3: 32, C4: 13, Eb4: 21, Gb4: 12, A4: -8 },
  },
  church_organ: { notes: "C2 E2 Ab2 C3 E3 Ab3 C4 E4 Ab4", kind: "sustain", gain: 0.24, attack: 0.12, release: 0.5, cents: -6 },
  cello: { notes: "C2 E2 Ab2 C3 E3 Ab3", kind: "sustain", gain: 0.36, attack: 1.2, release: 2.0 },
  contrabass: { notes: "A1 Db2 F2", kind: "sustain", gain: 0.4, attack: 1.5, release: 2.5 },
  bagpipe: { notes: "C3 E3 Ab3 C4 E4 Ab4", kind: "sustain", gain: 0.16, attack: 0.6, release: 0.8 },
  tubular_bells: { notes: "C4 E4 Ab4", kind: "hit", gain: 0.42 },
  timpani: { notes: "D2 Gb2 Bb2", kind: "hit", gain: 0.85 },
  taiko_drum: { notes: "C3 C4", kind: "hit", gain: 0.55 },
};

// Kirketonearter som tonesett (til forslagsnotene).
const DORIAN_D = [2, 4, 5, 7, 9, 11, 0];

// --- Cuene -----------------------------------------------------------------

// Hver cue: `step` sekunder per åttedel, `gain`, `wet` (etterklang), `scale`
// (tonearten forslagsnotene holder seg i) og stemmene (`parts`). En stemme
// går i løkke over sin egen lengde. Valg per stemme: `inst`, `notes`, `vel`,
// `pan`, `wait` (åttedeler før den begynner), `when(gjennomgang)` (spilles
// bare da), `grace` (sjanse for forslagsnote på lange toner fra andre
// gjennomgang), `attack`, `release`, `legato` (brøk av noten som holdes),
// `bend` ([fra, til] cent over hver tone), `lp` (lavpass i Hz) og `wet`.

// Morgenlys over åsene: bordun, harpe og blokkfløyte i dorisk, rolig og vidt.
const DAWN_MELODY = tune(`
  -/12 |
  A4/3 D5/3 | E5/2 F5/1 E5/3 | D5/6 | -/3 C5/3 |
  D5/3 F5/3 | G5/2 A5/1 G5/3 | F5/2 E5/1 D5/3 | E5/6 |
  A5/3 C6/3 | D6/2 C6/1 A5/3 | G5/2 A5/1 F5/2 E5/1 | D5/6 |
  -/12`);
const DAWN_ROOTS = "D D D C D C D C F A A D C D D D";

// Gullalderen: en sang i dorisk med resitasjonstonen på A, sunget med
// organum (parallell kvint under, B molle der kvinten faller på F).
const CHANT = tune(`
  D4/2 F4/2 G4/2 A4/2 | A4/4 A4/2 G4/2 | A4/2 C5/2 B4/2 A4/2 | G4/4 A4/4 |
  A4/2 A4/2 G4/2 F4/2 | G4/2 A4/2 G4/2 F4/2 | E4/2 F4/2 D4/4 | D4/8 |
  F4/2 G4/2 A4/2 C5/2 | D5/4 C5/2 A4/2 | C5/2 B4/2 A4/2 G4/2 | A4/8 |
  G4/2 A4/2 F4/2 G4/2 | E4/2 D4/2 F4/2 E4/2 | C4/2 E4/2 D4/4 | D4/8`);
const CHANT_ROOTS = "D D C G D C C D F D C A G A C D";

// Forfallet: den samme sangen, men tonene glir av, faller ut og skjærer.
const BROKEN = tune(`
  D4/2 F4/2 G4/2 Ab4/2 | A4/4 -/2 G4/2 | A4/2 C5/2 B4/1 Bb4/1 A4/2 | G4/4 -/4 |
  A4/2 A4/2 G#4/2 F4/2 | G4/1 -/1 A4/2 G4/2 Eb4/2 | E4/2 F4/2 D4/4 | C#4/8 |
  F4/2 G4/2 Ab4/2 C5/2 | Db5/4 C5/2 A4/2 | C5/1 B4/1 Bb4/2 A4/2 G4/2 | Ab4/8 |
  G4/2 -/2 F4/2 Gb4/2 | E4/2 Eb4/2 F4/2 E4/2 | C4/2 Eb4/2 D4/4 | C#4/8`);

// Dies irae (sekvensen fra 1200-tallet, første modus): første og andre
// verselinje, og så en linje i samme stil.
const DIES_IRAE = tune(`
  F4/2 E4/2 F4/2 D4/2 | E4/2 C4/2 D4/4 |
  F4/2 F4/2 G4/2 F4/2 | E4/2 D4/2 C4/2 E4/2 | F4/2 E4/2 D4/4 |
  A4/2 A4/2 G4/2 A4/2 | F4/2 G4/2 E4/2 D4/2 | C4/2 E4/2 D4/4 | -/16`);

// Estampie i dorisk (samme som byens første stykke), med lutt og tromme.
const ESTAMPIE = tune(`
  D5/2 E5/1 F5/2 E5/1 | D5/2 C5/1 D5/3 | F5/2 G5/1 A5/2 G5/1 | F5/2 E5/1 D5/3 |
  A5/2 G5/1 F5/2 G5/1 | A5/2 C6/1 A5/3 | G5/1 F5/1 E5/1 F5/2 E5/1 | D5/6 |
  A5/3 C6/3 | D6/2 C6/1 A5/3 | G5/2 A5/1 G5/2 F5/1 | E5/2 D5/1 E5/3 |
  F5/2 G5/1 A5/2 G5/1 | F5/2 E5/1 D5/2 C5/1 | D5/2 E5/1 C5/2 E5/1 | D5/6`);
const ESTAMPIE_ROOTS = "D C F D F A C D A D C A F D C D";
const TABOR = [1, 0, 0.4, 0.7, 0, 0.4];

// Mot sørporten: morgenmelodien igjen på fela, som ender i en kadens med
// dobbel ledetone (E–C# over G# til D–A–D).
const GATE_MELODY = tune(`
  A4/3 D5/3 | E5/2 F5/1 E5/3 | D5/3 C5/3 | D5/6 |
  D5/3 F5/3 | G5/2 A5/1 G5/3 | A5/2 G5/1 F5/2 E5/1 | D5/6 |
  F5/2 G5/1 A5/3 | G5/2 F5/1 E5/3 | E5/3 C#5/3 | D5/6`);

export const CUES = {
  dawn: {
    step: 0.3,
    gain: 0.9,
    wet: 0.45,
    parts: [
      { inst: "cello", notes: tune("D2+A2/96"), vel: 0.8, pan: -0.15 },
      { inst: "contrabass", notes: tune("D2/96"), vel: 0.5, pan: 0.1 },
      { inst: "orchestral_harp", notes: arp(DAWN_ROOTS, [0, 7, 12, 14, 19, 12], 3), vel: 0.6, pan: 0.3 },
      { inst: "orchestral_harp", notes: arp(DAWN_ROOTS, [-12, null, null, null, null, null], 3), vel: 0.55, pan: 0.2 },
      { inst: "recorder", notes: DAWN_MELODY, vel: 0.85, pan: -0.25, grace: 0.3 },
      // Andre gang gjennom svarer en fløyte en kvart under (organum).
      { inst: "recorder", notes: shift(DAWN_MELODY, -5), vel: 0.45, pan: 0.35, when: (p) => p % 2 === 1 },
    ],
  },
  scriptorium: {
    step: 0.28,
    gain: 1.0,
    wet: 0.55,
    parts: [
      { inst: "church_organ", notes: tune("D3+A3/64 D3+A3/64"), vel: 0.75, pan: -0.1, lp: 1800 },
      { inst: "choir_aahs", notes: CHANT, vel: 0.9, pan: -0.15, legato: 1 },
      { inst: "choir_aahs", notes: shift(CHANT, -7), vel: 0.6, pan: 0.2, legato: 1 },
      { inst: "orchestral_harp", notes: arp(CHANT_ROOTS, [0, 7, 12, 7, 14, 12, 19, 12], 3), vel: 0.42, pan: 0.35 },
      { inst: "orchestral_harp", notes: arp(CHANT_ROOTS, [-12, null, null, null, null, null, null, null], 3), vel: 0.5, pan: 0.25 },
    ],
  },
  fall: {
    step: 0.26,
    gain: 1.05,
    wet: 0.5,
    parts: [
      // Borduneten synker en halvtone for hver fjerdedel av løkka.
      { inst: "cello", notes: tune("D2/32 C#2/32 C2/32 B1/32"), vel: 0.9, bend: [0, -100], attack: 0.4, release: 0.6, legato: 1.02 },
      { inst: "contrabass", notes: tune("D2/32 C#2/32 C2/32 B1/32"), vel: 0.6, bend: [0, -100], attack: 0.4, release: 0.6, legato: 1.02 },
      { inst: "shanai", notes: BROKEN, vel: 0.8, pan: -0.25 },
      // Fela spiller den hele sangen en oktav over; der de to er uenige, skjærer det.
      { inst: "fiddle", notes: shift(CHANT, 12), vel: 0.45, pan: 0.3 },
      { inst: "taiko_drum", notes: beats([1, 0, 0, 0.6, 0, 0, 0.75, 0, 0.9, 0, 0.5, 0, 0, 0.65, 0.3, 0], 1, "C3"), vel: 0.8, pan: 0.05 },
      { inst: "taiko_drum", notes: beats([0, 0, 0.3, 0, 0, 0.45, 0, 0.25, 0, 0.35, 0, 0, 0.4, 0, 0, 0.5, 0, 0, 0.3, 0, 0.45], 1, "C4"), vel: 0.5, pan: -0.3 },
      { inst: "choir_aahs", notes: tune("-/32 D4+Ab4/16 -/16 C#4+G4/16 -/48"), vel: 0.4, pan: 0.15, bend: [0, -60] },
    ],
  },
  burial: {
    step: 0.4,
    gain: 1.12,
    wet: 0.6,
    parts: [
      { inst: "contrabass", notes: tune("D2/80"), vel: 0.8 },
      { inst: "church_organ", notes: tune("D2+A2/80"), vel: 0.55, lp: 900 },
      { inst: "choir_aahs", notes: DIES_IRAE, vel: 0.9, pan: -0.15, legato: 1 },
      { inst: "choir_aahs", notes: shift(DIES_IRAE, -12), vel: 0.7, pan: 0.15, legato: 1 },
      // Likeklokka slår hver takt, dypt og langsomt.
      { inst: "tubular_bells", notes: repeat(tune("D3/8:1 D3/8:0.8"), 5), vel: 1.1, pan: 0.35, lp: 4000 },
    ],
  },
  crypt: {
    step: 0.27,
    gain: 1.3,
    wet: 0.65,
    parts: [
      { inst: "contrabass", notes: tune("D2/128"), vel: 0.9 },
      { inst: "cello", notes: tune("D2/64 Eb2/64"), vel: 0.55, pan: 0.2, lp: 700 },
      { inst: "church_organ", notes: tune("D2+A2/128"), vel: 0.4, lp: 500 },
      // Hjertet: et dumpt dobbeltslag på pauken.
      { inst: "timpani", notes: beats([1, 0.5, 0, 0, 0, 0, 0, 0], 16, "D2"), vel: 0.8, lp: 900 },
      // Bruddstykker av Dies irae, langt borte.
      {
        inst: "choir_aahs",
        notes: tune("-/32 F4/4 E4/4 F4/4 D4/8 -/12 | -/24 E4/4 C4/4 D4/12 -/20 | -/16"),
        vel: 0.55,
        pan: -0.3,
        lp: 1400,
        wet: 0.9,
        legato: 1,
      },
      { inst: "choir_aahs", notes: tune("-/32 Bb3/4 A3/4 Bb3/4 G3/8 -/12 | -/24 A3/4 F3/4 G3/12 -/20 | -/16"), vel: 0.35, pan: 0.3, lp: 1200, wet: 0.9, legato: 1 },
      { inst: "tubular_bells", notes: tune("D3/48 -/16 G#2/48 -/16"), vel: 0.5, pan: 0.4, lp: 1800, wet: 0.9 },
    ],
  },
  town: {
    step: 0.17,
    gain: 0.8,
    wet: 0.3,
    parts: [
      { inst: "bagpipe", notes: tune("D3+A3/96"), vel: 0.7, pan: 0.1, attack: 0.3 },
      // Først skalmeie med fela en kvart under, så hoketus mellom dem.
      { inst: "shanai", notes: cat(ESTAMPIE, hocket(ESTAMPIE, 0)), vel: 0.8, pan: -0.25, grace: 0.35 },
      { inst: "fiddle", notes: cat(shift(ESTAMPIE, -5), hocket(ESTAMPIE, 1)), vel: 0.7, pan: 0.3, grace: 0.2 },
      { inst: "acoustic_guitar_nylon", notes: arp(ESTAMPIE_ROOTS, [0, 7, 12, 7, 12, 7], 3), vel: 0.65, pan: 0.15 },
      { inst: "acoustic_guitar_nylon", notes: arp(ESTAMPIE_ROOTS, [-12, null, null, null, null, null], 3), vel: 0.7, pan: 0.1 },
      { inst: "taiko_drum", notes: beats(TABOR, 1, "C4"), vel: 0.75, pan: -0.05 },
      { inst: "taiko_drum", notes: beats([1, 0, 0, 0, 0, 0, 0.6, 0, 0, 0, 0, 0], 1, "C3"), vel: 0.6, pan: 0.05 },
    ],
  },
  gate: {
    step: 0.3,
    gain: 0.95,
    wet: 0.45,
    parts: [
      { inst: "cello", notes: tune("D2+A2/72"), vel: 0.75, pan: -0.1 },
      { inst: "fiddle", notes: GATE_MELODY, vel: 0.8, pan: -0.2, grace: 0.25 },
      { inst: "recorder", notes: shift(GATE_MELODY, -5), vel: 0.42, pan: 0.35, when: (p) => p > 0 },
      {
        inst: "orchestral_harp",
        notes: cat(arp("D C D D D C F D F C", [0, 7, 12, 14, 19, 12], 3), tune("E3+B3+G#4/3 E3+B3+G#4/3 | D3+A3+D4+A4/6")),
        vel: 0.6,
        pan: 0.3,
      },
      { inst: "orchestral_harp", notes: arp("D C D D D C F D F C E D", [-12, null, null, null, null, null], 3), vel: 0.55, pan: 0.2 },
    ],
  },
};

// --- Etterklangen ----------------------------------------------------------

// Et lite pseudotilfeldig tall (samme hver gang, så testene gir samme lyd).
function rng(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

// Impulsresponsen til et steinrom: tidlige refleksjoner fra murene, og så
// en hale som dør ut over `rt` sekunder og blir mørkere jo lenger den varer.
function stoneRoom(c, seconds = 3.6, rt = 2.8) {
  const sr = c.sampleRate;
  const len = Math.floor(sr * seconds);
  const ir = c.createBuffer(2, len, sr);
  const rand = rng(1350);
  for (let ch = 0; ch < 2; ch++) {
    const d = ir.getChannelData(ch);
    const pre = Math.floor(sr * 0.014);
    let y = 0;
    for (let i = pre; i < len; i++) {
      const t = (i - pre) / sr;
      const a = 0.85 * Math.exp(-t * 1.6) + 0.08;
      y += a * (rand() * 2 - 1 - y);
      d[i] = y * Math.exp((-6.9 * t) / rt) * Math.min(1, t / 0.03);
    }
    for (let k = 0; k < 9; k++) {
      const at = pre + Math.floor(sr * (0.006 + rand() * 0.07));
      d[at] += (rand() < 0.5 ? -1 : 1) * (0.6 - k * 0.05);
    }
  }
  return ir;
}

// --- Spilleren -------------------------------------------------------------

export function createScore(context, base = "music/", opts = {}) {
  const c = context;
  const cues = opts.cues ?? CUES;
  const offline = typeof OfflineAudioContext !== "undefined" && c instanceof OfflineAudioContext;
  const rand = rng(opts.seed ?? 7);

  const master = c.createGain();
  master.gain.value = opts.gain ?? 0.9;
  // Bremsen mot klipping når mange stemmer faller sammen.
  const limit = c.createDynamicsCompressor();
  limit.threshold.value = -8;
  limit.knee.value = 6;
  limit.ratio.value = 6;
  limit.attack.value = 0.005;
  limit.release.value = 0.25;
  // Rumling under bassen (prøvenes støy, paukene) fjernes.
  const floor = c.createBiquadFilter();
  floor.type = "highpass";
  floor.frequency.value = 32;
  master.connect(floor).connect(limit).connect(opts.out ?? c.destination);
  const verb = c.createConvolver();
  verb.buffer = stoneRoom(c);
  const verbOut = c.createGain();
  verbOut.gain.value = 0.7;
  verb.connect(verbOut).connect(master);

  // --- Prøvene ---
  const banks = {};
  const loading = {};
  // Gjør en dekodet prøve klar: stillheten før anslaget kuttes, nivået
  // jevnes ut, og de holdte tonene får en løkke der slutten er krysstonet
  // inn i stykket før løkkestarten, så sømmen ikke høres.
  const prepare = (raw, kind, m, cents) => {
    const sr = raw.sampleRate;
    const ch0 = raw.getChannelData(0);
    let start = 0;
    while (start < ch0.length && Math.abs(ch0[start]) < 0.002) start++;
    start = Math.max(0, start - 32);
    let end = raw.length;
    if (kind === "sustain") end = Math.max(start + 1, raw.length - Math.floor(sr * 0.12));
    const len = end - start;
    const out = c.createBuffer(raw.numberOfChannels, len, sr);
    // Nivået: de holdte etter middelstyrken i den jevne delen, de andre etter toppen.
    let level = 0;
    if (kind === "sustain") {
      let sum = 0;
      let n = 0;
      for (let ch = 0; ch < raw.numberOfChannels; ch++) {
        const d = raw.getChannelData(ch);
        for (let i = start + Math.floor(sr * 0.3); i < Math.min(end, start + sr * 2); i++, n++) sum += d[i] * d[i];
      }
      level = 0.2 / Math.max(1e-4, Math.sqrt(sum / Math.max(1, n)));
    } else {
      let peak = 0;
      for (let ch = 0; ch < raw.numberOfChannels; ch++) {
        const d = raw.getChannelData(ch);
        for (let i = start; i < end; i++) peak = Math.max(peak, Math.abs(d[i]));
      }
      level = 0.8 / Math.max(1e-4, peak);
    }
    // Løkka er et helt antall svingninger lang, og starten finjusteres der
    // bølgen før løkkestarten ligner mest på bølgen før slutten, så
    // krysstoningen ikke slukker tonen der fasene står mot hverandre.
    const fade = Math.floor(sr * 0.3);
    let a = Math.floor(sr * 0.75);
    if (kind === "sustain" && len > a + fade * 2) {
      const period = sr / (440 * Math.pow(2, (m + cents / 100 - 69) / 12));
      a = Math.round(len - Math.round((len - a) / period) * period);
      const src = raw.getChannelData(0);
      const W = Math.min(2048, fade);
      let best = -Infinity;
      let pick = a;
      for (let cand = Math.round(a - period); cand <= Math.round(a + period); cand++) {
        let sum = 0;
        for (let i = 0; i < W; i += 2) sum += src[start + cand - W + i] * src[start + len - W + i];
        if (sum > best) {
          best = sum;
          pick = cand;
        }
      }
      a = pick;
    }
    for (let ch = 0; ch < raw.numberOfChannels; ch++) {
      const src = raw.getChannelData(ch);
      const d = out.getChannelData(ch);
      for (let i = 0; i < len; i++) d[i] = src[start + i] * level;
      if (kind === "sustain" && len > a + fade * 2) {
        for (let i = 0; i < fade; i++) {
          const w = i / fade;
          const j = len - fade + i;
          d[j] = (src[start + j] * (1 - w) + src[start + a - fade + i] * w) * level;
        }
      }
    }
    return { buf: out, loopStart: a / sr, loopEnd: len / sr };
  };
  const loadInstrument = (inst) => {
    if (loading[inst]) return loading[inst];
    const spec = INSTRUMENTS[inst];
    if (!spec) return Promise.resolve();
    const names = spec.notes.split(" ");
    loading[inst] = Promise.all(
      names.map(async (n) => {
        try {
          const res = await fetch(`${base}${inst}/${n}.mp3`);
          if (!res.ok) return null;
          const raw = await c.decodeAudioData(await res.arrayBuffer());
          const cents = typeof spec.cents === "number" ? spec.cents : (spec.cents?.[n] ?? 0);
          return { midi: midi(n), cents, ...prepare(raw, spec.kind, midi(n), cents) };
        } catch {
          return null;
        }
      }),
    ).then((list) => {
      banks[inst] = list.filter(Boolean).sort((x, y) => x.midi - y.midi);
    });
    return loading[inst];
  };
  const instrumentsOf = (names) => [...new Set(names.flatMap((n) => (cues[n]?.parts ?? []).map((p) => p.inst)))];

  // --- Én tone ---
  const voice = (inst, m, at, dur, vel, dest, p) => {
    const bank = banks[inst];
    if (!bank?.length) return;
    let s = bank[0];
    for (const b of bank) if (Math.abs(b.midi - m) < Math.abs(s.midi - m)) s = b;
    const spec = INSTRUMENTS[inst];
    const rate = Math.pow(2, (m - s.midi) / 12);
    const src = c.createBufferSource();
    src.buffer = s.buf;
    src.playbackRate.value = rate;
    src.detune.value = s.cents;
    if (p.bend) {
      src.detune.setValueAtTime(s.cents + p.bend[0], at);
      src.detune.linearRampToValueAtTime(s.cents + p.bend[1], at + dur);
    }
    const g = c.createGain();
    const peak = spec.gain * vel;
    let stop;
    if (spec.kind === "sustain") {
      src.loop = true;
      src.loopStart = s.loopStart;
      src.loopEnd = s.loopEnd;
      const a = Math.min(p.attack ?? spec.attack, dur * 0.5);
      const r = p.release ?? spec.release;
      const off = at + Math.max(a, dur);
      g.gain.setValueAtTime(0, at);
      g.gain.linearRampToValueAtTime(peak, at + a);
      g.gain.setValueAtTime(peak, off);
      g.gain.setTargetAtTime(0, off, r / 3);
      stop = off + r * 2;
    } else if (spec.kind === "pluck") {
      const off = at + Math.max(dur, p.ring ?? spec.ring);
      g.gain.setValueAtTime(peak, at);
      g.gain.setValueAtTime(peak, off);
      g.gain.setTargetAtTime(0, off, 0.12);
      stop = Math.min(off + 0.6, at + s.buf.duration / rate);
    } else {
      g.gain.setValueAtTime(peak, at);
      stop = at + s.buf.duration / rate;
    }
    src.connect(g).connect(dest);
    src.start(at);
    src.stop(stop);
  };

  // --- Cuene i gang ---
  const live = [];
  let current = null;
  let timer = null;
  const begin = (name, at, fade) => {
    const def = cues[name];
    const dry = c.createGain();
    const wet = c.createGain();
    for (const g of [dry, wet]) {
      g.gain.value = 0.0001;
      g.gain.setValueAtTime(0.0001, at);
      g.gain.linearRampToValueAtTime(def.gain ?? 1, at + Math.max(0.05, fade));
    }
    dry.connect(master);
    wet.connect(verb);
    const parts = def.parts.map((p) => {
      let node = c.createGain();
      const input = node;
      if (p.lp) {
        const lp = c.createBiquadFilter();
        lp.type = "lowpass";
        lp.frequency.value = p.lp;
        node.connect(lp);
        node = lp;
      }
      const pan = c.createStereoPanner();
      pan.pan.value = p.pan ?? 0;
      node.connect(pan);
      pan.connect(dry);
      const send = c.createGain();
      send.gain.value = p.wet ?? def.wet ?? 0.4;
      pan.connect(send).connect(wet);
      return { p, input, idx: 0, pass: 0, t: at + (p.wait ?? 0) * def.step };
    });
    const cue = { name, def, dry, wet, parts, start: at, end: Infinity };
    live.push(cue);
    return cue;
  };
  const end = (cue, at, fade) => {
    cue.end = at;
    for (const g of [cue.dry, cue.wet]) {
      if (g.gain.cancelAndHoldAtTime) g.gain.cancelAndHoldAtTime(at);
      else {
        g.gain.cancelScheduledValues(at);
        g.gain.setValueAtTime(cue.def.gain ?? 1, at);
      }
      g.gain.linearRampToValueAtTime(0.0001, at + Math.max(0.05, fade));
    }
    cue.gone = at + fade + 8;
  };
  // Forslagsnoten: nabotonen over i tonearten.
  const neighbour = (m, scale) => (scale.includes((m + 2) % 12) && !scale.includes((m + 1) % 12) ? m + 2 : scale.includes((m + 1) % 12) ? m + 1 : m + 2);
  const schedule = (cue, st, until) => {
    const { def } = cue;
    const p = st.p;
    const step = def.step;
    while (st.t < until && st.t < cue.end) {
      const n = p.notes[st.idx];
      const dur = n.len * step;
      if (n.notes.length && (!p.when || p.when(st.pass)) && banks[p.inst]) {
        const at = Math.max(st.t + (rand() - 0.5) * 0.016, cue.start, c.currentTime);
        const v = (p.vel ?? 1) * n.vel * (0.94 + rand() * 0.12);
        const hold = dur * (p.legato ?? 0.94);
        const grace = p.grace && st.pass > 0 && n.len >= 3 && n.notes.length === 1 && rand() < p.grace;
        for (const m of n.notes) {
          if (grace) {
            const g = step * 0.45;
            voice(p.inst, neighbour(m, def.scale ?? DORIAN_D), at, g, v * 0.75, st.input, p);
            voice(p.inst, m, at + g, hold - g, v, st.input, p);
          } else {
            voice(p.inst, m, at, hold, v, st.input, p);
          }
        }
      }
      st.t += dur;
      st.idx++;
      if (st.idx >= p.notes.length) {
        st.idx = 0;
        st.pass++;
      }
    }
  };
  const advance = (until) => {
    for (const cue of live) for (const st of cue.parts) schedule(cue, st, until);
    // De som er tonet helt ut, kobles fra.
    for (let i = live.length - 1; i >= 0; i--) {
      if (live[i].gone !== undefined && c.currentTime > live[i].gone) {
        live[i].dry.disconnect();
        live[i].wet.disconnect();
        live.splice(i, 1);
      }
    }
  };
  // Tonene planlegges litt fram i tid. Skjult fane kan kjøre tidtakeren
  // bare én gang i sekundet, så forspranget er over det.
  const run = () => {
    if (offline || timer) return;
    timer = setInterval(() => advance(c.currentTime + 1.3), 80);
  };

  return {
    // Henter prøvene til de nevnte cuene (alle om ingen er nevnt).
    load(names = Object.keys(cues)) {
      return Promise.all(instrumentsOf(names).map(loadInstrument)).then(() => undefined);
    },
    // Toner over til en ny cue. `at` er til gjengivelse uten lydkort
    // (OfflineAudioContext): da planlegges alt fram til `at` først.
    cue(name, fade = 2, at = c.currentTime) {
      if (!cues[name]) throw new Error(`Ukjent cue «${name}»`);
      if (current && current.name === name && current.end === Infinity) return;
      advance(at);
      if (current && current.end === Infinity) end(current, at, fade);
      current = begin(name, at, fade);
      this.load([name]);
      run();
    },
    stop(fade = 2, at = c.currentTime) {
      advance(at);
      if (current && current.end === Infinity) end(current, at, fade);
      current = null;
    },
    // Planlegger tonene fram til `until` (gjengivelse uten lydkort).
    advance,
    get current() {
      return current && current.end === Infinity ? current.name : null;
    },
    // Stopper tidtakeren for godt.
    dispose() {
      clearInterval(timer);
      timer = null;
      for (const cue of live) {
        cue.dry.disconnect();
        cue.wet.disconnect();
      }
      live.length = 0;
    },
  };
}
