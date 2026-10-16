// Lydbildet i filmen (docs/plan.md del 8): syntetisert med Web Audio, ett
// lag per scene som tones over i det neste. Musikken (`music.js`) ligger
// over; dette er rommet rundt: vind, sirisser, knitter, drypp, folkemengde,
// fugler, klokka og glitchen.

function noiseBuffer(c, seconds = 4) {
  const b = c.createBuffer(1, Math.ceil(c.sampleRate * seconds), c.sampleRate);
  const d = b.getChannelData(0);
  let pink = 0;
  for (let i = 0; i < d.length; i++) {
    pink = 0.97 * pink + 0.03 * (Math.random() * 2 - 1);
    d[i] = pink * 6;
  }
  return b;
}

export function createSoundscape(context) {
  let c = null;
  let out = null;
  let noise = null;
  let current = null;
  const setup = () => {
    if (c) return true;
    c = context?.();
    if (!c) return false;
    out = c.createGain();
    out.gain.value = 0.9;
    out.connect(c.destination);
    noise = noiseBuffer(c);
    return true;
  };

  // En løkke av støy gjennom et filter: vind, rom, folkemengde, vann.
  const bed = (bus, type, freq, q, level, wobble = 0, rate = 0.1) => {
    const src = c.createBufferSource();
    src.buffer = noise;
    src.loop = true;
    src.playbackRate.value = 0.7 + Math.random() * 0.6;
    const f = c.createBiquadFilter();
    f.type = type;
    f.frequency.value = freq;
    f.Q.value = q;
    const g = c.createGain();
    g.gain.value = level;
    if (wobble) {
      const lfo = c.createOscillator();
      lfo.frequency.value = rate;
      const lg = c.createGain();
      lg.gain.value = level * wobble;
      lfo.connect(lg).connect(g.gain);
      lfo.start();
      bus.stops.push(lfo);
    }
    src.connect(f).connect(g).connect(bus.gain);
    src.start();
    bus.stops.push(src);
  };

  // Små hendelser med tilfeldig mellomrom (knitter, drypp, fugl, bjelle).
  const every = (bus, min, max, fire) => {
    let alive = true;
    const go = () => {
      if (!alive) return;
      fire(c.currentTime + 0.05);
      bus.timers.push(setTimeout(go, (min + Math.random() * (max - min)) * 1000));
    };
    bus.timers.push(setTimeout(go, Math.random() * min * 1000));
    bus.stops.push({ stop: () => (alive = false) });
  };

  const tone = (bus, freq, at, dur, level, type = "sine", glide = 0) => {
    const o = c.createOscillator();
    o.type = type;
    o.frequency.setValueAtTime(freq, at);
    if (glide) o.frequency.exponentialRampToValueAtTime(freq * glide, at + dur);
    const g = c.createGain();
    g.gain.setValueAtTime(0.0001, at);
    g.gain.exponentialRampToValueAtTime(level, at + Math.min(0.01, dur / 4));
    g.gain.exponentialRampToValueAtTime(0.0001, at + dur);
    o.connect(g).connect(bus.gain);
    o.start(at);
    o.stop(at + dur + 0.05);
  };

  const burst = (bus, at, dur, freq, level, type = "bandpass") => {
    const src = c.createBufferSource();
    src.buffer = noise;
    const f = c.createBiquadFilter();
    f.type = type;
    f.frequency.value = freq;
    f.Q.value = 6;
    const g = c.createGain();
    g.gain.setValueAtTime(level, at);
    g.gain.exponentialRampToValueAtTime(0.0001, at + dur);
    src.connect(f).connect(g).connect(bus.gain);
    src.start(at, Math.random() * 3, dur + 0.05);
  };

  // Kirkeklokka: grunntone med uharmoniske overtoner som dør langsomt.
  const bell = (bus, at, f = 196, level = 0.08) => {
    for (const [m, l, d] of [[0.5, 0.6, 7], [1, 1, 5], [1.19, 0.5, 3.5], [1.5, 0.35, 3], [2.0, 0.3, 2.5], [2.74, 0.15, 1.8]]) tone(bus, f * m, at, d, level * l);
  };

  const KINDS = {
    // Skriptoriet i gullalderen: et varmt rom, lys som knitrer, pennen som skraper.
    hall: (b) => {
      bed(b, "lowpass", 260, 0.7, 0.05);
      every(b, 0.15, 0.9, (at) => burst(b, at, 0.03, 3200 + Math.random() * 2000, 0.02, "highpass"));
      every(b, 1.2, 3.5, (at) => {
        for (let k = 0; k < 5; k++) burst(b, at + k * 0.07, 0.06, 5200, 0.012);
      });
    },
    // Fallet: rommet går i stykker: brum, digitale pip og knaster.
    glitch: (b) => {
      bed(b, "lowpass", 140, 2, 0.06, 0.6, 0.3);
      every(b, 0.15, 0.7, (at) => tone(b, 400 + Math.random() * 2400, at, 0.05 + Math.random() * 0.08, 0.035, "square", Math.random() < 0.5 ? 0.5 : 1.8));
      every(b, 0.5, 1.6, (at) => burst(b, at, 0.12, 900 + Math.random() * 3000, 0.08));
    },
    // Begravelsen: nattevind, sirisser, klokka som ringer.
    night: (b) => {
      bed(b, "bandpass", 420, 0.6, 0.06, 0.5, 0.07);
      every(b, 0.08, 0.3, (at) => tone(b, 4300 + Math.random() * 300, at, 0.04, 0.006, "sine"));
      every(b, 4.5, 5.0, (at) => bell(b, at));
    },
    // Krypten: dyp bordun, drypp og en kjetting langt borte.
    crypt: (b) => {
      bed(b, "lowpass", 90, 1.2, 0.09, 0.3, 0.05);
      every(b, 0.6, 2.2, (at) => tone(b, 900 + Math.random() * 700, at, 0.18, 0.03, "sine", 0.55));
      every(b, 5, 9, (at) => {
        for (let k = 0; k < 6; k++) burst(b, at + k * 0.11, 0.08, 2400, 0.02);
      });
    },
    // Torget i 1350: folkemengde, fugler, en hest, en hammer langt borte.
    market: (b) => {
      bed(b, "bandpass", 600, 1.2, 0.07, 0.4, 0.6);
      bed(b, "bandpass", 1200, 2, 0.03, 0.5, 0.9);
      every(b, 0.6, 2.4, (at) => {
        const f = 2600 + Math.random() * 1600;
        for (let k = 0; k < 3; k++) tone(b, f * (1 + k * 0.08), at + k * 0.09, 0.07, 0.015, "sine", 1.3);
      });
      every(b, 1.5, 3.5, (at) => burst(b, at, 0.05, 1800, 0.04));
    },
    // Veien mot sørporten: vind i gresset og fugler.
    road: (b) => {
      bed(b, "highpass", 900, 0.5, 0.025, 0.6, 0.12);
      bed(b, "lowpass", 300, 0.7, 0.03, 0.4, 0.08);
      every(b, 0.8, 2.6, (at) => {
        const f = 3000 + Math.random() * 1500;
        for (let k = 0; k < 4; k++) tone(b, f * (k % 2 ? 1.12 : 1), at + k * 0.11, 0.08, 0.012, "sine", 1.2);
      });
    },
  };

  return {
    set(kind, fade = 1.5) {
      if (!setup()) return;
      if (current?.kind === kind) return;
      const now = c.currentTime;
      if (current) {
        const old = current;
        old.gain.gain.setTargetAtTime(0, now, fade / 3);
        setTimeout(() => {
          old.timers.forEach(clearTimeout);
          old.stops.forEach((s) => {
            try {
              s.stop();
            } catch {
              /* stoppet */
            }
          });
          old.gain.disconnect();
        }, fade * 1000 + 500);
      }
      current = null;
      if (!kind || !KINDS[kind]) return;
      const bus = { kind, gain: c.createGain(), stops: [], timers: [] };
      bus.gain.gain.value = 0;
      bus.gain.connect(out);
      bus.gain.gain.setTargetAtTime(1, now, fade / 3);
      KINDS[kind](bus);
      current = bus;
    },
    stop(fade = 1.5) {
      this.set(null, fade);
    },
  };
}
