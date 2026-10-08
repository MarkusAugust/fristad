// Lydene i byen, laget i nettleseren: folk som prater og ler på torget,
// hamring og blåsebelg i smia, kirkeklokka som ringer med jevne mellomrom,
// sauer og kuer på enga, fugler om dagen og sirisser om natta, og vinden.
// Spillet sender hvor nær du er hver kilde (`set`), og lagene blandes etter
// det. I kjelleren er byen stille.

export function createAmbience(context) {
  let c = null;
  let out = null;
  let state = { square: 0, forge: 0, church: 0, meadow: 0, night: false, cellar: true };
  const bus = {};
  let noise = null;
  let timer = null;

  const setup = () => {
    if (c) return true;
    c = context();
    if (!c) return false;
    out = c.createGain();
    out.gain.value = 0.55;
    out.connect(c.destination);
    for (const name of ["square", "forge", "church", "meadow", "birds", "night", "wind"]) {
      const g = c.createGain();
      g.gain.value = 0;
      g.connect(out);
      bus[name] = g;
    }
    const len = c.sampleRate * 2;
    noise = c.createBuffer(1, len, c.sampleRate);
    const d = noise.getChannelData(0);
    for (let i = 0; i < len; i++) d[i] = Math.random() * 2 - 1;
    wind();
    timer = setInterval(tick, 100);
    return true;
  };

  const rnd = (a, b) => a + Math.random() * (b - a);
  const noiseSrc = (at, dur) => {
    const s = c.createBufferSource();
    s.buffer = noise;
    s.start(at, Math.random() * 1.5, dur + 0.05);
    return s;
  };
  const env = (g, at, peak, attack, dur) => {
    g.gain.setValueAtTime(0.0001, at);
    g.gain.linearRampToValueAtTime(peak, at + attack);
    g.gain.exponentialRampToValueAtTime(0.0001, at + dur);
  };

  // --- Torget: en stemme er pustende støy gjennom to formanter som glir,
  // med en grunntone som går opp og ned som i en setning.
  const VOWELS = [[730, 1090], [530, 1840], [270, 2290], [570, 840], [440, 1020], [300, 870]];
  const voice = (at, female) => {
    const dur = rnd(0.5, 1.6);
    const f0 = female ? rnd(180, 240) : rnd(95, 140);
    const o = c.createOscillator();
    o.type = "sawtooth";
    o.frequency.setValueAtTime(f0, at);
    const syl = Math.max(2, Math.round(dur / 0.18));
    for (let k = 1; k <= syl; k++) o.frequency.linearRampToValueAtTime(f0 * rnd(0.85, 1.2), at + (dur * k) / syl);
    const g = c.createGain();
    // Stavelser: styrken pulserer.
    g.gain.setValueAtTime(0.0001, at);
    for (let k = 0; k < syl; k++) {
      const t = at + (dur * k) / syl;
      g.gain.linearRampToValueAtTime(rnd(0.4, 1), t + 0.04);
      g.gain.linearRampToValueAtTime(0.15, t + (dur / syl) * 0.9);
    }
    g.gain.linearRampToValueAtTime(0.0001, at + dur + 0.05);
    const mix = c.createGain();
    mix.gain.value = 0.012;
    for (const k of [0, 1]) {
      const bp = c.createBiquadFilter();
      bp.type = "bandpass";
      bp.Q.value = 6;
      const v0 = VOWELS[Math.floor(Math.random() * VOWELS.length)];
      bp.frequency.setValueAtTime(v0[k] * (female ? 1.15 : 1), at);
      for (let s = 1; s <= syl; s++) {
        const v = VOWELS[Math.floor(Math.random() * VOWELS.length)];
        bp.frequency.linearRampToValueAtTime(v[k] * (female ? 1.15 : 1), at + (dur * s) / syl);
      }
      o.connect(bp).connect(g);
    }
    // Litt pust over.
    const n = noiseSrc(at, dur);
    const hp = c.createBiquadFilter();
    hp.type = "highpass";
    hp.frequency.value = 2500;
    const ng = c.createGain();
    ng.gain.value = 0.08;
    n.connect(hp).connect(ng).connect(g);
    // Lenger unna er stemmene mørkere.
    const lp = c.createBiquadFilter();
    lp.type = "lowpass";
    lp.frequency.value = rnd(1400, 3500);
    const pan = c.createStereoPanner();
    pan.pan.value = rnd(-0.8, 0.8);
    g.connect(lp).connect(mix).connect(pan).connect(bus.square);
    o.start(at);
    o.stop(at + dur + 0.1);
  };
  const laugh = (at) => {
    const n = Math.floor(rnd(3, 6));
    const female = Math.random() < 0.5;
    for (let k = 0; k < n; k++) {
      const t = at + k * rnd(0.11, 0.15);
      const o = c.createOscillator();
      o.type = "sawtooth";
      const f = (female ? 260 : 150) * (1 - k * 0.04);
      o.frequency.setValueAtTime(f * 1.1, t);
      o.frequency.linearRampToValueAtTime(f, t + 0.08);
      const bp = c.createBiquadFilter();
      bp.type = "bandpass";
      bp.frequency.value = 800;
      bp.Q.value = 3;
      const g = c.createGain();
      env(g, t, 0.02, 0.01, 0.1);
      o.connect(bp).connect(g).connect(bus.square);
      o.start(t);
      o.stop(t + 0.12);
    }
  };
  // Trehjul på brosteinen og en kjerre som skrangler forbi.
  const cart = (at) => {
    for (let k = 0; k < 14; k++) {
      const t = at + k * rnd(0.08, 0.14);
      const n = noiseSrc(t, 0.05);
      const bp = c.createBiquadFilter();
      bp.type = "bandpass";
      bp.frequency.value = rnd(300, 700);
      bp.Q.value = 2;
      const g = c.createGain();
      env(g, t, 0.05 * Math.sin((k / 14) * Math.PI), 0.003, 0.06);
      n.connect(bp).connect(g).connect(bus.square);
    }
  };

  // --- Smia: hammeren mot ambolten (uharmoniske deltoner som dør fort),
  // i grupper på tre-fire slag, og blåsebelgen innimellom.
  const anvil = (at, peak) => {
    for (const [f, a, d] of [[1250, 1, 0.9], [2860, 0.6, 0.5], [4300, 0.35, 0.3], [6550, 0.2, 0.18]]) {
      const o = c.createOscillator();
      o.frequency.value = f * rnd(0.99, 1.01);
      const g = c.createGain();
      env(g, at, peak * a * 0.05, 0.001, d);
      o.connect(g).connect(bus.forge);
      o.start(at);
      o.stop(at + d + 0.05);
    }
    const n = noiseSrc(at, 0.03);
    const g = c.createGain();
    env(g, at, peak * 0.08, 0.001, 0.03);
    n.connect(g).connect(bus.forge);
  };
  const bellows = (at) => {
    for (let k = 0; k < 2; k++) {
      const t = at + k * 1.4;
      const n = noiseSrc(t, 1.2);
      const lp = c.createBiquadFilter();
      lp.type = "lowpass";
      lp.frequency.value = 500;
      const g = c.createGain();
      g.gain.setValueAtTime(0.0001, t);
      g.gain.linearRampToValueAtTime(0.06, t + 0.5);
      g.gain.linearRampToValueAtTime(0.0001, t + 1.2);
      n.connect(lp).connect(g).connect(bus.forge);
    }
  };

  // --- Kirkeklokka: tunge, uharmoniske deltoner (hum, grunntone, ters,
  // kvint, oktav) som klinger lenge.
  const bell = (at) => {
    const f = 196;
    for (const [mul, a, d] of [[0.5, 0.5, 9], [1, 1, 7], [1.183, 0.5, 5], [1.506, 0.35, 4], [2, 0.3, 3.5], [2.74, 0.15, 2]]) {
      const o = c.createOscillator();
      o.frequency.value = f * mul;
      const g = c.createGain();
      env(g, at, a * 0.05, 0.004, d);
      o.connect(g).connect(bus.church);
      o.start(at);
      o.stop(at + d + 0.1);
    }
  };
  const ring = (at, n) => {
    for (let k = 0; k < n; k++) bell(at + k * 2.6);
  };

  // --- Enga: sauer og kuer.
  const sheep = (at) => {
    const o = c.createOscillator();
    o.type = "sawtooth";
    const f = rnd(330, 420);
    o.frequency.setValueAtTime(f, at);
    o.frequency.linearRampToValueAtTime(f * 0.92, at + 0.7);
    // Bræket: rask skjelving i tonen.
    const v = c.createOscillator();
    v.frequency.value = rnd(16, 22);
    const vg = c.createGain();
    vg.gain.value = f * 0.06;
    v.connect(vg).connect(o.frequency);
    const bp = c.createBiquadFilter();
    bp.type = "bandpass";
    bp.frequency.value = 1100;
    bp.Q.value = 2;
    const g = c.createGain();
    g.gain.setValueAtTime(0.0001, at);
    g.gain.linearRampToValueAtTime(0.03, at + 0.08);
    g.gain.linearRampToValueAtTime(0.0001, at + 0.75);
    const pan = c.createStereoPanner();
    pan.pan.value = rnd(-0.7, 0.7);
    o.connect(bp).connect(g).connect(pan).connect(bus.meadow);
    o.start(at);
    v.start(at);
    o.stop(at + 0.8);
    v.stop(at + 0.8);
  };
  const cow = (at) => {
    const o = c.createOscillator();
    o.type = "sawtooth";
    const f = rnd(95, 120);
    o.frequency.setValueAtTime(f * 0.9, at);
    o.frequency.linearRampToValueAtTime(f, at + 0.4);
    o.frequency.linearRampToValueAtTime(f * 0.8, at + 1.8);
    const bp = c.createBiquadFilter();
    bp.type = "bandpass";
    bp.Q.value = 3;
    bp.frequency.setValueAtTime(300, at);
    bp.frequency.linearRampToValueAtTime(700, at + 0.6);
    bp.frequency.linearRampToValueAtTime(400, at + 1.8);
    const g = c.createGain();
    g.gain.setValueAtTime(0.0001, at);
    g.gain.linearRampToValueAtTime(0.06, at + 0.3);
    g.gain.linearRampToValueAtTime(0.0001, at + 1.9);
    const pan = c.createStereoPanner();
    pan.pan.value = rnd(-0.7, 0.7);
    o.connect(bp).connect(g).connect(pan).connect(bus.meadow);
    o.start(at);
    o.stop(at + 2);
  };

  // --- Fugler om dagen og sirisser om natta, over hele byen.
  const bird = (at) => {
    const n = Math.floor(rnd(2, 6));
    const base = rnd(2600, 4200);
    for (let k = 0; k < n; k++) {
      const t = at + k * rnd(0.08, 0.16);
      const o = c.createOscillator();
      o.frequency.setValueAtTime(base * rnd(0.9, 1.15), t);
      o.frequency.exponentialRampToValueAtTime(base * rnd(0.7, 1.4), t + 0.07);
      const g = c.createGain();
      env(g, t, 0.012, 0.005, 0.08);
      const pan = c.createStereoPanner();
      pan.pan.value = rnd(-1, 1);
      o.connect(g).connect(pan).connect(bus.birds);
      o.start(t);
      o.stop(t + 0.1);
    }
  };
  const cricket = (at) => {
    for (let k = 0; k < 3; k++) {
      const t = at + k * 0.09;
      const o = c.createOscillator();
      o.frequency.value = rnd(4300, 4700);
      const am = c.createOscillator();
      am.frequency.value = 45;
      const ag = c.createGain();
      ag.gain.value = 0.5;
      const g = c.createGain();
      env(g, t, 0.006, 0.005, 0.06);
      am.connect(ag).connect(g.gain);
      const pan = c.createStereoPanner();
      pan.pan.value = rnd(-1, 1);
      o.connect(g).connect(pan).connect(bus.night);
      o.start(t);
      am.start(t);
      o.stop(t + 0.07);
      am.stop(t + 0.07);
    }
  };
  const wind = () => {
    const s = c.createBufferSource();
    s.buffer = noise;
    s.loop = true;
    const bp = c.createBiquadFilter();
    bp.type = "bandpass";
    bp.frequency.value = 350;
    bp.Q.value = 0.8;
    const lfo = c.createOscillator();
    lfo.frequency.value = 0.08;
    const lg = c.createGain();
    lg.gain.value = 200;
    lfo.connect(lg).connect(bp.frequency);
    const g = c.createGain();
    g.gain.value = 0.03;
    s.connect(bp).connect(g).connect(bus.wind);
    s.start();
    lfo.start();
  };

  // --- Klokka: planlegger lydene tilfeldig, tettere jo nærmere du er.
  let nextBell = 0;
  let forgeBeat = 0;
  const tick = () => {
    const t = c.currentTime + 0.05;
    const s = state;
    const town = !s.cellar;
    const day = town && !s.night;
    const people = s.square * (s.night ? 0.25 : 1);
    if (people > 0.02 && Math.random() < 0.06 + people * 0.4) voice(t + rnd(0, 0.1), Math.random() < 0.45);
    if (people > 0.1 && Math.random() < 0.004) laugh(t);
    if (people > 0.2 && day && Math.random() < 0.0015) cart(t);
    if (day && s.forge > 0.01) {
      // Tre-fire slag, så en pause.
      forgeBeat = (forgeBeat + 1) % 14;
      if (forgeBeat < 4 && forgeBeat % 1 === 0 && Math.random() < 0.9) anvil(t, forgeBeat === 3 ? 0.6 : 1);
      if (forgeBeat === 8 && Math.random() < 0.15) bellows(t);
    }
    if (town && s.meadow > 0.02) {
      if (Math.random() < 0.012 * s.meadow) sheep(t);
      if (Math.random() < 0.004 * s.meadow) cow(t);
    }
    if (day && Math.random() < 0.03) bird(t);
    if (town && s.night && Math.random() < 0.12) cricket(t);
    // Kirkeklokka: et par minutter mellom hver gang, i hele byen.
    if (town && c.currentTime > nextBell) {
      if (nextBell > 0) ring(t, s.night ? 3 : 6);
      nextBell = c.currentTime + rnd(150, 240);
    }
  };

  const level = (name, v) => {
    const g = bus[name].gain;
    g.setTargetAtTime(v, c.currentTime, 0.4);
  };
  return {
    set(next) {
      if (!setup()) return;
      state = next;
      const town = !next.cellar;
      level("square", town ? 0.15 + next.square * 0.85 : 0);
      level("forge", town ? next.forge : 0);
      // Klokka høres over hele byen, sterkere ved kirka.
      level("church", town ? 0.3 + next.church * 0.7 : 0);
      level("meadow", town ? Math.min(1, next.meadow * 1.2) : 0);
      level("birds", town && !next.night ? 0.8 : 0);
      level("night", town && next.night ? 0.9 : 0);
      level("wind", town ? 1 : 0);
    },
  };
}
