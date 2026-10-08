// Musikken: laget i nettleseren mens du spiller, ingen filer. Oppe i byen
// spilles fem middelalderstykker etter tur, ett nytt hvert halvannet
// minutt og så forfra (estampie, cantiga, carole, klagesang og
// saltarello i kirketonearter, med skalmeie, blokkfløyte, fele, lutt og
// harpe over en lirekassebordun og tromme). Nede i krypten kalde,
// dissonante bordunlyder, vind og fjerne klokkeklang. Overgangene tones
// over på tre sekunder. M slår den av og på.

const hz = (midi) => 440 * Math.pow(2, (midi - 69) / 12);

// Notene skrives «D5/2»: tonen og varigheten i åttedeler; «-/2» er pause.
const NAMES = { C: 0, D: 2, E: 4, F: 5, G: 7, A: 9, B: 11 };
function midi(name) {
  const m = /^([A-G])(#|b)?(\d)$/.exec(name);
  if (!m) return null;
  return NAMES[m[1]] + (m[2] === "#" ? 1 : m[2] === "b" ? -1 : 0) + (Number(m[3]) + 1) * 12;
}
function tune(text) {
  return text
    .split(/[\s|]+/)
    .filter(Boolean)
    .map((tok) => {
      const [n, d] = tok.split("/");
      return { midi: n === "-" ? null : midi(n), len: Number(d) };
    });
}

// Stykkene. `bar` er åttedeler per takt, `step` sekunder per åttedel,
// `roots` grunntonen i hver takt (lutta og harpa spiller kvint og oktav
// over den, som i middelalderen), `drum` slagene i en takt (0 er stille).
const SONGS = [
  {
    name: "Estampie i dorisk",
    step: 0.19,
    bar: 6,
    lead: "shawm",
    accomp: "lute",
    drone: ["D3", "A3"],
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
    step: 0.21,
    bar: 8,
    lead: "fiddle",
    accomp: "strum",
    drone: ["G2", "D3"],
    drum: [0.9, 0, 0, 0, 0.6, 0, 0.3, 0],
    roots: "G C G D G C F G D C G D G G F G",
    melody: tune(`
      G4/2 B4/2 D5/2 B4/2 | C5/2 D5/2 E5/4 | D5/2 C5/2 B4/2 A4/2 | G4/4 A4/4 |
      B4/2 D5/2 G5/2 F5/2 | E5/2 D5/2 C5/4 | B4/2 C5/2 A4/2 F4/2 | G4/8 |
      D5/2 D5/1 E5/1 F5/2 D5/2 | E5/2 C5/2 D5/4 | B4/2 C5/2 D5/2 B4/2 | A4/4 F4/4 |
      G4/2 A4/2 B4/2 C5/2 | D5/2 B4/2 A4/4 | B4/2 A4/2 F4/2 A4/2 | G4/8`),
  },
  {
    name: "Klagesang",
    step: 0.32,
    bar: 6,
    lead: "recorder",
    accomp: "harp",
    drone: ["A2", "E3"],
    drum: null,
    roots: "A G A A A C F E C G G A G G E A",
    melody: tune(`
      A4/4 C5/2 | B4/2 A4/2 G4/2 | A4/4 E4/2 | A4/6 |
      C5/4 D5/2 | E5/2 D5/2 C5/2 | B4/2 C5/2 A4/2 | B4/6 |
      E5/4 F5/2 | E5/2 D5/2 C5/2 | D5/2 C5/2 B4/2 | A4/6 |
      G4/2 A4/2 C5/2 | B4/2 A4/2 G4/2 | E4/2 G4/2 B4/2 | A4/6`),
  },
  {
    name: "Saltarello",
    step: 0.17,
    bar: 6,
    lead: "shawm",
    accomp: "lute",
    drone: ["E3", "B3"],
    drum: [1, 0.3, 0.5, 0.8, 0, 0.5],
    roots: "E D E E A G D E D D E E",
    melody: tune(`
      E5/1 F#5/1 G5/1 A5/2 G5/1 | F#5/1 E5/1 D5/1 E5/3 | B5/2 A5/1 G5/2 F#5/1 | E5/2 D5/1 E5/3 |
      G5/1 A5/1 B5/1 C#6/2 B5/1 | A5/1 G5/1 F#5/1 G5/3 | A5/2 G5/1 F#5/2 D5/1 | E5/6 |
      D5/1 E5/1 F#5/1 G5/2 E5/1 | F#5/2 D5/1 E5/3 | B4/1 D5/1 E5/1 F#5/2 A5/1 | G5/2 F#5/1 E5/3`),
  },
];
const SONG_SECONDS = 90;

export function createMusic(context) {
  let c = null;
  let master = null;
  let verb = null;
  let mode = null;
  let muted = false;
  let timer = null;
  let next = 0;
  let beat = 0;
  const layers = {};
  try {
    muted = localStorage.getItem("fristad.music") === "av";
  } catch {
    /* ingen lagring */
  }
  const setup = () => {
    if (c) return true;
    c = context();
    if (!c) return false;
    master = c.createGain();
    master.gain.value = muted ? 0 : 0.5;
    master.connect(c.destination);
    // Etterklang: en impulsrespons av støy som dør ut, som et steinrom.
    verb = c.createConvolver();
    const len = c.sampleRate * 3.2;
    const ir = c.createBuffer(2, len, c.sampleRate);
    for (let ch = 0; ch < 2; ch++) {
      const d = ir.getChannelData(ch);
      for (let i = 0; i < len; i++) d[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / len, 3.2);
    }
    verb.buffer = ir;
    const wet = c.createGain();
    wet.gain.value = 0.5;
    verb.connect(wet).connect(master);
    for (const name of ["town", "cellar"]) {
      const g = c.createGain();
      g.gain.value = 0;
      g.connect(master);
      g.connect(verb);
      layers[name] = g;
    }
    return true;
  };

  // --- Instrumentene -----------------------------------------------------

  // En plukket streng (Karplus–Strong): lutt, eller harpe med lengre klang.
  const pluck = (bus, f, at, peak, harp = false) => {
    const n = Math.round(c.sampleRate / f);
    const dur = harp ? 3.5 : 2.2;
    const buf = c.createBuffer(1, Math.ceil(c.sampleRate * dur), c.sampleRate);
    const d = buf.getChannelData(0);
    for (let i = 0; i < n; i++) d[i] = Math.random() * 2 - 1;
    const keep = harp ? 0.4992 : 0.497;
    for (let i = n; i < d.length; i++) d[i] = keep * (d[i - n] + d[i - n + 1]);
    const src = c.createBufferSource();
    src.buffer = buf;
    const lp = c.createBiquadFilter();
    lp.type = "lowpass";
    lp.frequency.value = harp ? 3400 : 2600;
    const g = c.createGain();
    g.gain.value = peak;
    src.connect(lp).connect(g).connect(bus);
    src.start(at);
  };
  // Tonens omhylling: anslag, hold og slipp.
  const envelope = (g, at, dur, peak, attack) => {
    g.gain.setValueAtTime(0.0001, at);
    g.gain.linearRampToValueAtTime(peak, at + attack);
    g.gain.setValueAtTime(peak, at + Math.max(attack, dur - 0.08));
    g.gain.linearRampToValueAtTime(0.0001, at + dur + 0.04);
  };
  const vibrato = (o, f, at, depth, rate, delay) => {
    const v = c.createOscillator();
    v.frequency.value = rate;
    const vg = c.createGain();
    vg.gain.setValueAtTime(0, at);
    vg.gain.linearRampToValueAtTime(f * depth, at + delay);
    v.connect(vg).connect(o.frequency);
    return v;
  };
  // Blokkfløyte: myk grunntone, litt oktav, og pust i anslaget.
  const recorder = (bus, f, at, dur, peak) => {
    const g = c.createGain();
    envelope(g, at, dur, peak, 0.05);
    g.connect(bus);
    const nodes = [];
    for (const [mul, amp, type] of [[1, 1, "sine"], [2, 0.12, "sine"], [3, 0.05, "triangle"]]) {
      const o = c.createOscillator();
      o.type = type;
      o.frequency.value = f * mul;
      const a = c.createGain();
      a.gain.value = amp;
      o.connect(a).connect(g);
      nodes.push(o, vibrato(o, f * mul, at, 0.006, 5, dur * 0.5));
    }
    const noise = c.createBufferSource();
    noise.buffer = noiseBuf();
    const bp = c.createBiquadFilter();
    bp.type = "bandpass";
    bp.frequency.value = f * 2;
    bp.Q.value = 3;
    const ng = c.createGain();
    ng.gain.setValueAtTime(peak * 0.5, at);
    ng.gain.exponentialRampToValueAtTime(0.0001, at + 0.09);
    noise.connect(bp).connect(ng).connect(bus);
    noise.start(at);
    noise.stop(at + 0.1);
    for (const o of nodes) {
      o.start(at);
      o.stop(at + dur + 0.1);
    }
  };
  // Skalmeie: nasal og summende rørbladlyd, som på markedet.
  const shawm = (bus, f, at, dur, peak) => {
    const g = c.createGain();
    envelope(g, at, dur, peak, 0.025);
    const bp = c.createBiquadFilter();
    bp.type = "bandpass";
    bp.frequency.value = 1300;
    bp.Q.value = 0.9;
    const lp = c.createBiquadFilter();
    lp.type = "lowpass";
    lp.frequency.value = 3600;
    bp.connect(lp).connect(g).connect(bus);
    const nodes = [];
    for (const [type, det] of [["sawtooth", -5], ["square", 5]]) {
      const o = c.createOscillator();
      o.type = type;
      o.frequency.value = f;
      o.detune.value = det;
      o.connect(bp);
      nodes.push(o, vibrato(o, f, at, 0.004, 5.5, 0.25));
    }
    for (const o of nodes) {
      o.start(at);
      o.stop(at + dur + 0.1);
    }
  };
  // Fele: strøket streng med mykt anslag og vibrato som kommer etter hvert.
  const fiddle = (bus, f, at, dur, peak) => {
    const g = c.createGain();
    envelope(g, at, dur, peak, 0.09);
    const lp = c.createBiquadFilter();
    lp.type = "lowpass";
    lp.frequency.value = 2400;
    lp.Q.value = 2;
    lp.connect(g).connect(bus);
    const o = c.createOscillator();
    o.type = "sawtooth";
    o.frequency.value = f;
    const v = vibrato(o, f, at, 0.008, 6, Math.min(0.4, dur * 0.6));
    o.connect(lp);
    o.start(at);
    v.start(at);
    o.stop(at + dur + 0.1);
    v.stop(at + dur + 0.1);
  };
  const LEADS = { recorder, shawm, fiddle };
  const PEAK = { recorder: 0.07, shawm: 0.035, fiddle: 0.045 };

  let noise = null;
  const noiseBuf = () => {
    if (noise) return noise;
    const len = c.sampleRate;
    noise = c.createBuffer(1, len, c.sampleRate);
    const d = noise.getChannelData(0);
    for (let i = 0; i < len; i++) d[i] = Math.random() * 2 - 1;
    return noise;
  };
  // Tromme (tabor): dump slag med skinnsnerr.
  const tabor = (bus, at, amp) => {
    const o = c.createOscillator();
    o.frequency.setValueAtTime(110, at);
    o.frequency.exponentialRampToValueAtTime(55, at + 0.18);
    const g = c.createGain();
    g.gain.setValueAtTime(amp * 0.35, at);
    g.gain.exponentialRampToValueAtTime(0.0001, at + 0.3);
    o.connect(g).connect(bus);
    o.start(at);
    o.stop(at + 0.32);
    const n = c.createBufferSource();
    n.buffer = noiseBuf();
    const bp = c.createBiquadFilter();
    bp.type = "bandpass";
    bp.frequency.value = 1800;
    bp.Q.value = 0.8;
    const ng = c.createGain();
    ng.gain.setValueAtTime(amp * 0.09, at);
    ng.gain.exponentialRampToValueAtTime(0.0001, at + 0.12);
    n.connect(bp).connect(ng).connect(bus);
    n.start(at, Math.random() * 0.5);
    n.stop(at + 0.13);
  };
  // Lirekassa: bordun på grunntone og kvint som varer mens stykket spiller,
  // med hjulets summing. Gir en funksjon som toner den ut.
  const hurdy = (bus, notes, at) => {
    const g = c.createGain();
    g.gain.setValueAtTime(0.0001, at);
    g.gain.linearRampToValueAtTime(0.045, at + 2.5);
    const lp = c.createBiquadFilter();
    lp.type = "lowpass";
    lp.frequency.value = 900;
    const body = c.createBiquadFilter();
    body.type = "peaking";
    body.frequency.value = 500;
    body.gain.value = 6;
    lp.connect(body).connect(g).connect(bus);
    const nodes = [];
    for (const n of notes) {
      for (const det of [-6, 6]) {
        const o = c.createOscillator();
        o.type = "sawtooth";
        o.frequency.value = hz(midi(n));
        o.detune.value = det;
        o.connect(lp);
        nodes.push(o);
      }
    }
    // Hjulet: en langsom svingning i styrken.
    const lfo = c.createOscillator();
    lfo.frequency.value = 0.35;
    const lg = c.createGain();
    lg.gain.value = 0.008;
    lfo.connect(lg).connect(g.gain);
    nodes.push(lfo);
    for (const o of nodes) o.start(at);
    return (when) => {
      g.gain.cancelScheduledValues(when);
      g.gain.setValueAtTime(g.gain.value, when);
      g.gain.linearRampToValueAtTime(0.0001, when + 3);
      for (const o of nodes) o.stop(when + 3.1);
    };
  };

  // --- Byen: stykkene etter tur ------------------------------------------

  let song = -1;
  let songBus = null;
  let songStart = 0;
  let songBeat = 0;
  let stopDrone = null;
  let pos = 0;
  let left = 0;
  let pass = 0;
  const nextSong = (t) => {
    // Det gamle stykket tones ut mens det nye begynner.
    if (songBus) {
      const old = songBus;
      old.gain.setValueAtTime(old.gain.value, t);
      old.gain.linearRampToValueAtTime(0.0001, t + 3);
      setTimeout(() => old.disconnect(), 8000);
    }
    stopDrone?.(t);
    song = (song + 1) % SONGS.length;
    songBus = c.createGain();
    songBus.gain.setValueAtTime(0.0001, t);
    songBus.gain.linearRampToValueAtTime(1, t + 2);
    songBus.connect(layers.town);
    stopDrone = hurdy(songBus, SONGS[song].drone, t);
    songStart = t;
    songBeat = 0;
    pos = 0;
    left = 0;
    pass = 0;
  };
  const town = (t) => {
    if (song < 0) nextSong(t);
    let s = SONGS[song];
    // Nytt stykke når tiden er ute, ved starten av en takt.
    if (t - songStart > SONG_SECONDS && songBeat % s.bar === 0) {
      nextSong(t);
      s = SONGS[song];
    }
    const roots = s.roots.split(" ");
    const barNo = Math.floor(songBeat / s.bar);
    const step = songBeat % s.bar;
    const root = midi(roots[barNo % roots.length] + "3");
    const human = () => Math.random() * 0.012;
    // Akkompagnementet: kvint og oktav over grunntonen.
    const voicing = [root, root + 7, root + 12, root + 7];
    if (s.accomp === "lute") {
      pluck(songBus, hz(voicing[step % 4]), t + human(), step === 0 ? 0.42 : 0.26);
      if (step === 0) pluck(songBus, hz(root - 12), t, 0.4);
    } else if (s.accomp === "strum" && step % 2 === 0) {
      for (const [k, n] of [root, root + 7, root + 12].entries()) pluck(songBus, hz(n), t + k * 0.018, step === 0 ? 0.3 : 0.2);
    } else if (s.accomp === "harp" && step % 2 === 0) {
      pluck(songBus, hz(voicing[(step / 2) % 4] + (step === 0 ? 0 : 12)), t + human(), 0.3, true);
      if (step === 0) pluck(songBus, hz(root - 12), t, 0.35, true);
    }
    if (s.drum && s.drum[step] > 0) tabor(songBus, t + human(), s.drum[step] * (pass % 2 ? 1 : 0.8));
    // Melodien: neste tone når den forrige er ferdig. Annenhver gang
    // gjennom pyntes lange toner med en forslagsnote.
    if (left <= 0) {
      const n = s.melody[pos];
      const dur = n.len * s.step;
      if (n.midi !== null) {
        const play = LEADS[s.lead];
        const peak = PEAK[s.lead] * (step === 0 ? 1.1 : 0.9);
        if (pass % 2 === 1 && n.len >= 3 && Math.random() < 0.5) {
          play(songBus, hz(n.midi + 2), t, s.step * 0.45, peak * 0.8);
          play(songBus, hz(n.midi), t + s.step * 0.45, dur - s.step * 0.45, peak);
        } else {
          play(songBus, hz(n.midi), t + human(), dur * 0.95, peak);
        }
        // Tredje gang gjennom dobles melodien en oktav under av blokkfløyta.
        if (pass % 3 === 2 && s.lead !== "recorder") recorder(songBus, hz(n.midi - 12), t, dur * 0.95, 0.04);
      }
      left = n.len;
      pos++;
      if (pos >= s.melody.length) {
        pos = 0;
        pass++;
      }
    }
    left--;
    songBeat++;
  };

  // --- Krypten -----------------------------------------------------------

  const drone = (bus, freqs, type, cutoff, peak) => {
    const lp = c.createBiquadFilter();
    lp.type = "lowpass";
    lp.frequency.value = cutoff;
    const g = c.createGain();
    g.gain.value = peak;
    lp.connect(g).connect(bus);
    const lfo = c.createOscillator();
    lfo.frequency.value = 0.07;
    const lg = c.createGain();
    lg.gain.value = cutoff * 0.4;
    lfo.connect(lg).connect(lp.frequency);
    lfo.start();
    for (const f of freqs) {
      for (const det of [-4, 4]) {
        const o = c.createOscillator();
        o.type = type;
        o.frequency.value = f;
        o.detune.value = det;
        o.connect(lp);
        o.start();
      }
    }
  };
  // Fjerne klokker i frygisk, slag langt borte og vind.
  const bells = [38, 39, 43, 45, 46, 50, 51, 55];
  const cellar = (t) => {
    if (beat % 8 === 0 && Math.random() < 0.55) {
      const f = hz(bells[Math.floor(Math.random() * bells.length)] + 24);
      for (const [mul, amp] of [[1, 0.07], [2.76, 0.03], [5.4, 0.015]]) {
        const o = c.createOscillator();
        o.frequency.value = f * mul;
        const g = c.createGain();
        g.gain.setValueAtTime(amp, t);
        g.gain.exponentialRampToValueAtTime(0.0001, t + 4.5);
        o.connect(g).connect(layers.cellar);
        o.start(t);
        o.stop(t + 4.6);
      }
    }
    if (beat % 16 === 6 && Math.random() < 0.4) {
      const o = c.createOscillator();
      o.frequency.setValueAtTime(52, t);
      o.frequency.exponentialRampToValueAtTime(30, t + 0.8);
      const g = c.createGain();
      g.gain.setValueAtTime(0.25, t);
      g.gain.exponentialRampToValueAtTime(0.0001, t + 1.2);
      o.connect(g).connect(layers.cellar);
      o.start(t);
      o.stop(t + 1.3);
    }
  };
  const wind = () => {
    const src = c.createBufferSource();
    src.buffer = noiseBuf();
    src.loop = true;
    const bp = c.createBiquadFilter();
    bp.type = "bandpass";
    bp.frequency.value = 400;
    bp.Q.value = 2.5;
    const lfo = c.createOscillator();
    lfo.frequency.value = 0.05;
    const lg = c.createGain();
    lg.gain.value = 260;
    lfo.connect(lg).connect(bp.frequency);
    const g = c.createGain();
    g.gain.value = 0.05;
    src.connect(bp).connect(g).connect(layers.cellar);
    src.start();
    lfo.start();
  };

  // --- Klokka ------------------------------------------------------------

  let started = false;
  let nextTown = 0;
  const start = () => {
    if (started) return;
    started = true;
    drone(layers.cellar, [hz(26), hz(32), hz(33)], "sawtooth", 220, 0.09);
    wind();
    next = nextTown = c.currentTime + 0.1;
    // Planlegger tonene litt fram i tid. Byen går i stykkets tempo,
    // krypten i fast puls. Byen spiller bare mens du er der (eller tones ut).
    timer = setInterval(() => {
      const ahead = c.currentTime + 0.25;
      while (next < ahead) {
        if (mode === "cellar") cellar(next);
        next += 60 / 84 / 2;
        beat++;
      }
      while (nextTown < ahead) {
        if (mode === "town") town(nextTown);
        nextTown += song >= 0 ? SONGS[song].step : 0.2;
      }
    }, 60);
  };
  const fade = (name, to) => {
    const g = layers[name].gain;
    g.cancelScheduledValues(c.currentTime);
    g.setValueAtTime(g.value, c.currentTime);
    g.linearRampToValueAtTime(to, c.currentTime + 3);
  };
  return {
    set(where) {
      if (!setup()) return;
      start();
      if (where === mode) return;
      mode = where;
      fade("town", where === "town" ? 1 : 0);
      fade("cellar", where === "cellar" ? 1 : 0);
    },
    toggle() {
      if (!setup()) return;
      muted = !muted;
      master.gain.setTargetAtTime(muted ? 0 : 0.5, c.currentTime, 0.3);
      try {
        localStorage.setItem("fristad.music", muted ? "av" : "på");
      } catch {
        /* ingen lagring */
      }
      return !muted;
    },
    // Til testing: hvilket stykke som spiller.
    get song() {
      return song >= 0 ? SONGS[song].name : null;
    },
  };
}

export { SONGS, tune };
