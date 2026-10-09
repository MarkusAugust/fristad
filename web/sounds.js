// Lydene i spillet: opptak fra frie spill (assets/sounds, se CREDITS.md)
// og syntetiske lyder laget med WebAudio. Spillet sender navnet, hvor lyden
// kommer fra sett fra deg og hvor sterk den er; opptakene velges blant
// variantene (aldri den samme to ganger på rad) med litt ulik tonehøyde og
// styrke, og lyder fra et sted går gjennom en PannerNode (HRTF) så du hører
// retning og avstand. Etterklangen følger rommet: lang og våt i kjelleren,
// kort innendørs, nesten ingen ute. Løkkene (regn, vind, ild, vann, lava,
// drypp, torget, smia, fugler) blandes etter lydbildet spillet sender.
// Mangler et opptak (ikke lastet ennå), spilles den syntetiske lyden.

const BASE = "sounds/";
// Løkkene som er ute (dempes og blir mørkere innendørs).
const OUTDOOR = new Set(["loop_rain", "loop_storm", "loop_wind", "loop_market", "loop_birds", "loop_owl"]);

export function createSounds() {
  let ctx = null;
  let master = null;
  let reverb = null;
  let wet = null;
  let outdoor = null;
  let target = null;
  let manifest = null;
  const raw = new Map();
  const buffers = new Map();
  const last = new Map();
  const loops = new Map();
  let space = "";

  const get = () => {
    if (!ctx) {
      try {
        ctx = new (window.AudioContext || window.webkitAudioContext)();
      } catch {
        return null;
      }
      master = ctx.createGain();
      master.gain.value = 0.9;
      master.connect(ctx.destination);
      reverb = ctx.createConvolver();
      wet = ctx.createGain();
      wet.gain.value = 0.05;
      reverb.connect(wet).connect(master);
      outdoor = ctx.createBiquadFilter();
      outdoor.type = "lowpass";
      outdoor.frequency.value = 18000;
      outdoor.connect(master);
      setSpace("outdoor");
      decodeAll();
    }
    if (ctx.state === "suspended") ctx.resume().catch(() => {});
    return ctx;
  };

  // Etterklang: støy som dør ut eksponentielt, i stereo, med de høye
  // frekvensene borte først (mørkere hale).
  const impulse = (secs, decay) => {
    const n = Math.floor(ctx.sampleRate * secs);
    const b = ctx.createBuffer(2, n, ctx.sampleRate);
    for (let ch = 0; ch < 2; ch++) {
      const d = b.getChannelData(ch);
      let lp = 0;
      for (let i = 0; i < n; i++) {
        const t = i / n;
        lp += (Math.random() * 2 - 1 - lp) * (0.9 - t * 0.75);
        d[i] = lp * Math.pow(1 - t, decay);
      }
    }
    return b;
  };
  const setSpace = (s) => {
    if (s === space || !ctx) return;
    space = s;
    const [secs, decay, mix] = s === "cellar" ? [2.8, 2.2, 0.42] : s === "indoor" ? [0.8, 3.0, 0.2] : [0.5, 4.0, 0.05];
    reverb.buffer = impulse(secs, decay);
    wet.gain.setTargetAtTime(mix, ctx.currentTime, 0.3);
  };

  // Opptakene hentes i bakgrunnen og pakkes ut når lydkonteksten finnes.
  const preload = async () => {
    try {
      manifest = await (await fetch(BASE + "sounds.json")).json();
    } catch {
      return;
    }
    const files = [...new Set(Object.values(manifest).flatMap((e) => e.files))];
    // Løkkene sist: de er størst.
    files.sort((a, b) => Number(a.startsWith("loop_")) - Number(b.startsWith("loop_")));
    let i = 0;
    const worker = async () => {
      while (i < files.length) {
        const f = files[i++];
        try {
          const r = await fetch(BASE + f);
          if (r.ok) raw.set(f, await r.arrayBuffer());
        } catch {
          /* uten opptak blir det den syntetiske lyden */
        }
        decodeAll();
      }
    };
    await Promise.all([worker(), worker(), worker(), worker()]);
  };
  const decodeAll = () => {
    if (!ctx) return;
    for (const [f, data] of raw) {
      raw.delete(f);
      ctx.decodeAudioData(data).then((b) => buffers.set(f, b), () => {});
    }
  };

  const pick = (name, files) => {
    const ready = files.filter((f) => buffers.has(f));
    if (!ready.length) return null;
    let k = Math.floor(Math.random() * ready.length);
    if (ready.length > 1 && ready[k] === last.get(name)) k = (k + 1) % ready.length;
    last.set(name, ready[k]);
    return buffers.get(ready[k]);
  };

  // Veien ut for én lyd: styrken, så panneren om den har et sted, og så
  // tørt til hovedvolumet og vått til etterklangen.
  const route = (c, gain, at) => {
    const g = c.createGain();
    g.gain.value = gain;
    let tail = g;
    if (at) {
      const p = c.createPanner();
      p.panningModel = "HRTF";
      p.distanceModel = "inverse";
      p.refDistance = 2;
      p.rolloffFactor = 1.1;
      p.maxDistance = 80;
      if (p.positionX) {
        p.positionX.value = at[0];
        p.positionY.value = at[1];
        p.positionZ.value = at[2];
      } else {
        p.setPosition(at[0], at[1], at[2]);
      }
      g.connect(p);
      tail = p;
    }
    tail.connect(master);
    tail.connect(reverb);
    return g;
  };

  const noise = (c, dur) => {
    const b = c.createBuffer(1, Math.ceil(c.sampleRate * dur), c.sampleRate);
    const d = b.getChannelData(0);
    for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1;
    const s = c.createBufferSource();
    s.buffer = b;
    return s;
  };
  const env = (c, peak, attack, decay, at = 0) => {
    const g = c.createGain();
    const t = c.currentTime + at;
    g.gain.setValueAtTime(0.0001, t);
    g.gain.exponentialRampToValueAtTime(peak, t + attack);
    g.gain.exponentialRampToValueAtTime(0.0001, t + attack + decay);
    g.connect(target || c.destination);
    return g;
  };
  const tone = (c, type, f0, f1, peak, attack, decay, at = 0) => {
    const o = c.createOscillator();
    o.type = type;
    const t = c.currentTime + at;
    o.frequency.setValueAtTime(f0, t);
    o.frequency.exponentialRampToValueAtTime(Math.max(1, f1), t + attack + decay);
    o.connect(env(c, peak, attack, decay, at));
    o.start(t);
    o.stop(t + attack + decay + 0.05);
  };
  const whoosh = (c, f0, f1, peak, dur, at = 0) => {
    const s = noise(c, dur + 0.1);
    const f = c.createBiquadFilter();
    f.type = "bandpass";
    f.Q.value = 1.4;
    const t = c.currentTime + at;
    f.frequency.setValueAtTime(f0, t);
    f.frequency.exponentialRampToValueAtTime(f1, t + dur * 0.4);
    f.frequency.exponentialRampToValueAtTime(f0 * 0.7, t + dur);
    s.connect(f).connect(env(c, peak, dur * 0.25, dur * 0.75, at));
    s.start(t);
  };
  const crunch = (c, peak, dur, cutoff, at = 0) => {
    const s = noise(c, dur + 0.05);
    const f = c.createBiquadFilter();
    f.type = "lowpass";
    f.frequency.value = cutoff;
    s.connect(f).connect(env(c, peak, 0.005, dur, at));
    s.start(c.currentTime + at);
  };
  // En stemme: sagtann gjennom to formantfiltre (vokalen), med fallende
  // tone, litt vibrato og pust. Til stønnet når du blir truffet.
  const voice = (c, f0, f1, formants, peak, dur, at = 0) => {
    const t = c.currentTime + at;
    const o = c.createOscillator();
    o.type = "sawtooth";
    o.frequency.setValueAtTime(f0, t);
    o.frequency.exponentialRampToValueAtTime(f1, t + dur);
    const v = c.createOscillator();
    v.frequency.value = 5.5;
    const vg = c.createGain();
    vg.gain.value = f0 * 0.025;
    v.connect(vg).connect(o.frequency);
    const out = env(c, peak, dur * 0.12, dur * 0.88, at);
    for (const [f, q, g] of formants) {
      const bp = c.createBiquadFilter();
      bp.type = "bandpass";
      bp.frequency.value = f;
      bp.Q.value = q;
      const gg = c.createGain();
      gg.gain.value = g;
      o.connect(bp).connect(gg).connect(out);
    }
    const br = noise(c, dur);
    const bf = c.createBiquadFilter();
    bf.type = "bandpass";
    bf.frequency.value = 1400;
    bf.Q.value = 0.8;
    br.connect(bf).connect(env(c, peak * 0.35, dur * 0.1, dur * 0.9, at));
    o.start(t);
    v.start(t);
    br.start(t);
    o.stop(t + dur + 0.05);
    v.stop(t + dur + 0.05);
  };
  // Glitch: firkantbølge som hopper tilfeldig i tone, stammer av og på og
  // knuses i biter, som et digitalt vesen som går i stykker.
  const glitch = (c, peak, dur, base = 220, at = 0) => {
    const t = c.currentTime + at;
    const o = c.createOscillator();
    o.type = "square";
    const g = c.createGain();
    g.gain.setValueAtTime(0.0001, t);
    const steps = Math.max(4, Math.round(dur / 0.025));
    for (let k = 0; k < steps; k++) {
      const tk = t + (k * dur) / steps;
      o.frequency.setValueAtTime(base * Math.pow(2, (Math.random() * 3 - 1.2)), tk);
      g.gain.setValueAtTime(Math.random() < 0.7 ? peak * (1 - k / steps) : 0.0001, tk);
    }
    g.gain.setValueAtTime(0.0001, t + dur);
    const crush = c.createWaveShaper();
    const curve = new Float32Array(256);
    for (let i = 0; i < 256; i++) curve[i] = Math.round(((i / 255) * 2 - 1) * 4) / 4;
    crush.curve = curve;
    const f = c.createBiquadFilter();
    f.type = "highpass";
    f.frequency.value = 160;
    o.connect(crush).connect(f).connect(g).connect(target || c.destination);
    o.start(t);
    o.stop(t + dur + 0.05);
  };
  let lastGroan = 0;
  const synth = {
    // Hogget: et lagdelt sus gjennom lufta og en kort, lys klang fra stålet.
    swing: (c) => {
      whoosh(c, 500, 2600, 0.35, 0.28);
      whoosh(c, 260, 900, 0.18, 0.32, 0.02);
      tone(c, "sine", 3100, 2900, 0.03, 0.01, 0.2, 0.06);
    },
    swing_heavy: (c) => {
      whoosh(c, 220, 1400, 0.5, 0.5);
      whoosh(c, 120, 500, 0.3, 0.55, 0.04);
      tone(c, "sine", 2400, 2200, 0.035, 0.01, 0.3, 0.12);
    },
    // Treff på et monster: slaget, og monsteret som skjærer seg digitalt.
    hit: (c) => {
      tone(c, "sine", 140, 45, 0.7, 0.005, 0.18);
      crunch(c, 0.45, 0.12, 2500);
      glitch(c, 0.14, 0.22, 260, 0.03);
    },
    hit_heavy: (c) => {
      tone(c, "sine", 110, 35, 0.9, 0.005, 0.3);
      crunch(c, 0.6, 0.2, 1800);
      tone(c, "square", 400, 90, 0.08, 0.005, 0.1);
      glitch(c, 0.18, 0.35, 180, 0.03);
    },
    kill: (c) => {
      tone(c, "sawtooth", 180, 30, 0.5, 0.01, 0.6);
      crunch(c, 0.6, 0.45, 1200);
      tone(c, "square", 900, 200, 0.12, 0.01, 0.35, 0.05);
      glitch(c, 0.2, 0.7, 140, 0.05);
      glitch(c, 0.1, 0.5, 520, 0.2);
    },
    // Kista: lokket knirker opp på gamle hengsler.
    chest: (c) => {
      const t = c.currentTime;
      const o = c.createOscillator();
      o.type = "sawtooth";
      o.frequency.setValueAtTime(95, t);
      for (let k = 1; k <= 14; k++) o.frequency.linearRampToValueAtTime(95 + 70 * Math.random() + k * 6, t + k * 0.06);
      const bp = c.createBiquadFilter();
      bp.type = "bandpass";
      bp.frequency.value = 1100;
      bp.Q.value = 6;
      o.connect(bp).connect(env(c, 0.22, 0.08, 0.85));
      o.start(t);
      o.stop(t + 1.0);
      crunch(c, 0.25, 0.15, 700, 0.85);
      tone(c, "sine", 880, 880, 0.12, 0.005, 0.15, 0.95);
      tone(c, "sine", 1320, 1320, 0.12, 0.005, 0.3, 1.03);
    },
    block: (c) => {
      tone(c, "square", 700, 500, 0.25, 0.003, 0.09);
      crunch(c, 0.3, 0.08, 4000);
    },
    parry: (c) => {
      tone(c, "sine", 1900, 1700, 0.35, 0.003, 0.5);
      tone(c, "sine", 2850, 2600, 0.2, 0.003, 0.4);
      crunch(c, 0.25, 0.05, 6000);
    },
    // Du blir truffet: slaget, og et stønn (ikke oftere enn hvert 0,7 s).
    hurt: (c) => {
      tone(c, "sine", 90, 40, 0.8, 0.005, 0.25);
      crunch(c, 0.35, 0.15, 900);
      if (c.currentTime - lastGroan > 0.7) {
        lastGroan = c.currentTime;
        const vowels = [[[650, 6, 1.0], [1080, 7, 0.6]], [[520, 6, 1.0], [880, 7, 0.5]], [[700, 5, 1.0], [1220, 8, 0.45]]];
        const f0 = 115 + Math.random() * 30;
        voice(c, f0, f0 * 0.72, vowels[Math.floor(Math.random() * vowels.length)], 0.5, 0.42 + Math.random() * 0.2, 0.03);
      }
    },
    dodge: (c) => whoosh(c, 180, 700, 0.3, 0.3),
    // Slaget i kroppen når opptaket står for stemmen.
    thump: (c) => {
      tone(c, "sine", 90, 40, 0.8, 0.005, 0.25);
      crunch(c, 0.35, 0.15, 900);
    },
    // Landingen: et tungt dunk i beina.
    land: (c) => {
      tone(c, "sine", 70, 30, 0.6, 0.004, 0.22);
      crunch(c, 0.2, 0.1, 400);
    },
    // Tungt hugg lades: en stigende spenning i stålet og et pust inn.
    charge: (c) => {
      whoosh(c, 300, 1200, 0.12, 0.6);
      tone(c, "sine", 520, 780, 0.03, 0.25, 0.4);
    },
    // Full ladning: et kort, lyst klang.
    charged: (c) => {
      tone(c, "sine", 2200, 2150, 0.12, 0.004, 0.5);
      tone(c, "sine", 3300, 3250, 0.06, 0.004, 0.35);
    },
    // Balansen brister: noe digitalt som knekker.
    stagger: (c) => {
      glitch(c, 0.16, 0.4, 110);
      crunch(c, 0.35, 0.25, 1500, 0.02);
      tone(c, "square", 300, 60, 0.08, 0.005, 0.35);
    },
    // Knirk i tre og tau: friksjon som hakker.
    creak: (c) => {
      const t = c.currentTime;
      const o = c.createOscillator();
      o.type = "sawtooth";
      const f = 120 + Math.random() * 90;
      o.frequency.setValueAtTime(f, t);
      for (let k = 1; k <= 8; k++) o.frequency.linearRampToValueAtTime(f + Math.random() * 60 + k * 4, t + k * 0.05);
      const bp = c.createBiquadFilter();
      bp.type = "bandpass";
      bp.frequency.value = 900 + Math.random() * 500;
      bp.Q.value = 7;
      o.connect(bp).connect(env(c, 0.12, 0.05, 0.4));
      o.start(t);
      o.stop(t + 0.5);
    },
    // Rotter: pip og små klør som kraver over steinen.
    rats: (c) => {
      for (let k = 0; k < 3 + Math.floor(Math.random() * 3); k++) {
        const at = k * (0.08 + Math.random() * 0.2);
        tone(c, "sine", 3800 + Math.random() * 1500, 4200 + Math.random() * 1500, 0.04, 0.005, 0.06, at);
      }
      for (let k = 0; k < 10; k++) crunch(c, 0.05, 0.02, 5000, 0.3 + k * 0.045);
    },
    // Kirkeklokka ringer inn skumringen eller morgenen.
    bell: (c) => {
      for (let k = 0; k < 3; k++) {
        for (const [mul, a, d] of [[0.5, 0.5, 6], [1, 1, 5], [1.183, 0.5, 3.5], [1.506, 0.35, 3], [2, 0.3, 2.5]]) {
          tone(c, "sine", 196 * mul, 196 * mul, a * 0.08, 0.004, d, k * 2.4);
        }
      }
    },
    // Pause og tilbake: to myke toner ned og opp.
    pause: (c) => {
      tone(c, "triangle", 660, 660, 0.06, 0.01, 0.25);
      tone(c, "triangle", 440, 440, 0.06, 0.01, 0.35, 0.12);
    },
    resume: (c) => {
      tone(c, "triangle", 440, 440, 0.06, 0.01, 0.2);
      tone(c, "triangle", 660, 660, 0.06, 0.01, 0.3, 0.1);
    },
    // Tom for pust: et hest, langt utpust og et kortere innpust.
    breath: (c) => {
      whoosh(c, 900, 1400, 0.12, 0.7);
      whoosh(c, 1200, 1800, 0.08, 0.4, 0.8);
    },
    clang: (c) => {
      tone(c, "square", 1200, 900, 0.2, 0.003, 0.12);
      tone(c, "sine", 2400, 2300, 0.12, 0.003, 0.25);
      crunch(c, 0.2, 0.06, 5000);
    },
    spit: (c) => {
      whoosh(c, 900, 3200, 0.3, 0.22);
      tone(c, "sawtooth", 600, 1400, 0.12, 0.01, 0.15);
    },
    slam: (c) => {
      tone(c, "sine", 60, 25, 1.0, 0.005, 0.7);
      crunch(c, 0.7, 0.5, 600);
      crunch(c, 0.3, 0.9, 200, 0.05);
    },
    roar: (c) => {
      const o = c.createOscillator();
      o.type = "sawtooth";
      const t = c.currentTime;
      o.frequency.setValueAtTime(45, t);
      o.frequency.linearRampToValueAtTime(70, t + 0.5);
      o.frequency.linearRampToValueAtTime(40, t + 1.3);
      const v = c.createOscillator();
      v.frequency.value = 7;
      const vg = c.createGain();
      vg.gain.value = 12;
      v.connect(vg).connect(o.frequency);
      const f = c.createBiquadFilter();
      f.type = "lowpass";
      f.frequency.value = 500;
      o.connect(f).connect(env(c, 0.5, 0.1, 1.3));
      o.start(t);
      v.start(t);
      o.stop(t + 1.5);
      v.stop(t + 1.5);
      crunch(c, 0.3, 1.0, 400, 0.1);
    },
    // Monsteret slår: en digital snerring som bygger seg opp.
    growl: (c) => {
      glitch(c, 0.12, 0.45, 90);
      whoosh(c, 300, 1800, 0.18, 0.4, 0.15);
      const o = c.createOscillator();
      o.type = "sawtooth";
      const t = c.currentTime;
      o.frequency.setValueAtTime(70, t);
      o.frequency.linearRampToValueAtTime(95, t + 0.4);
      const v = c.createOscillator();
      v.frequency.value = 11;
      const vg = c.createGain();
      vg.gain.value = 9;
      v.connect(vg).connect(o.frequency);
      const f = c.createBiquadFilter();
      f.type = "lowpass";
      f.frequency.value = 700;
      o.connect(f).connect(env(c, 0.3, 0.05, 0.5));
      o.start(t);
      v.start(t);
      o.stop(t + 0.6);
      v.stop(t + 0.6);
    },
    talk: (c) => {
      // Et lite mumlende «hm-hm».
      tone(c, "triangle", 220, 180, 0.12, 0.02, 0.12);
      tone(c, "triangle", 260, 200, 0.1, 0.02, 0.14, 0.14);
    },
    pickup: (c) => {
      tone(c, "sine", 880, 880, 0.2, 0.005, 0.12);
      tone(c, "sine", 1320, 1320, 0.2, 0.005, 0.25, 0.08);
    },
    death: (c) => {
      tone(c, "sawtooth", 160, 30, 0.5, 0.02, 1.4);
      crunch(c, 0.4, 0.8, 600);
    },
    door: (c) => {
      // Tung tredør med jernbeslag: knirk og et dunk.
      tone(c, "sawtooth", 140, 210, 0.08, 0.05, 0.5);
      crunch(c, 0.5, 0.25, 500, 0.45);
      tone(c, "sine", 70, 40, 0.5, 0.005, 0.3, 0.45);
    },
    well: (c) => {
      // Brønnen: valsen surrer når bøtta slippes, plask i vannet, og så
      // knirker sveiven i rykk mens bøtta heises opp.
      tone(c, "triangle", 320, 120, 0.05, 0.02, 0.95);
      crunch(c, 0.35, 0.45, 900, 0.95);
      tone(c, "sine", 180, 90, 0.12, 0.005, 0.25, 0.97);
      for (let k = 0; k < 10; k++) {
        const at = 1.45 + k * 0.24;
        tone(c, "sawtooth", 115 + (k % 2) * 35, 95, 0.045, 0.01, 0.12, at);
        crunch(c, 0.06, 0.05, 1800, at + 0.06);
      }
    },
  };

  const play = (name, opts = {}) => {
    const c = get();
    if (!c) return;
    const gain = opts.gain ?? 1;
    const at = opts.at || null;
    const e = manifest?.[name];
    try {
      const out = route(c, gain, at);
      const b = e && e.files.length ? pick(name, e.files) : null;
      if (b) {
        const s = c.createBufferSource();
        s.buffer = b;
        s.playbackRate.value = (e.rate ?? 1) * (1 + (Math.random() * 2 - 1) * (e.pitch ?? 0));
        const g = c.createGain();
        g.gain.value = e.gain * (0.88 + Math.random() * 0.17);
        s.connect(g).connect(out);
        s.start();
      }
      // Den syntetiske lyden: i tillegg når manifestet sier det, ellers
      // som reserve til opptaket er lastet.
      const sy = e ? e.synth ?? (b ? null : name) : name;
      if (sy && synth[sy]) {
        target = out;
        synth[sy](c);
      }
    } catch {
      /* lyden er pynt */
    } finally {
      target = null;
    }
  };

  // En løkke som går så lenge den høres: startes når den trengs og
  // stoppes når den har vært stille en stund.
  const loop = (name, level) => {
    const e = manifest?.[name];
    const b = e && buffers.get(e.files[0]);
    let l = loops.get(name);
    if (!l) {
      if (!b || level < 0.005) return;
      const s = ctx.createBufferSource();
      s.buffer = b;
      s.loop = true;
      // Hopp over stillheten mp3-koderen legger i endene.
      s.loopStart = 0.03;
      s.loopEnd = Math.max(0.1, b.duration - 0.03);
      const g = ctx.createGain();
      g.gain.value = 0;
      s.connect(g).connect(OUTDOOR.has(name) ? outdoor : master);
      if (!OUTDOOR.has(name)) g.connect(reverb);
      s.start(0, Math.random() * b.duration * 0.9);
      l = { s, g, quiet: 0 };
      loops.set(name, l);
    }
    l.g.gain.setTargetAtTime(level * e.gain, ctx.currentTime, 0.6);
    l.quiet = level < 0.005 ? l.quiet + 1 : 0;
    if (l.quiet > 40) {
      l.s.stop(ctx.currentTime + 1);
      loops.delete(name);
    }
  };

  // Hjerteslaget når livet er lavt: raskere jo lavere.
  let heartTimer = null;
  let heartRate = 0;
  const heart = (hp) => {
    heartRate = hp > 0 && hp < 35 ? 1 - hp / 35 : 0;
    if (heartRate > 0 && !heartTimer) {
      const beat = () => {
        if (heartRate <= 0) {
          heartTimer = null;
          return;
        }
        play("heartbeat", { gain: 0.5 + heartRate * 0.5 });
        heartTimer = setTimeout(beat, 1500 - heartRate * 650);
      };
      beat();
    }
  };

  return {
    context: get,
    preload,
    play,
    heart,
    // Lydbildet fra spillet: rommet (etterklangen), og nivået på løkkene.
    ambience(a) {
      const c = get();
      if (!c || !manifest) return;
      setSpace(a.cellar ? "cellar" : a.indoor ? "indoor" : "outdoor");
      outdoor.frequency.setTargetAtTime(a.indoor ? 700 : 18000, c.currentTime, 0.3);
      const town = !a.cellar;
      const day = town && !a.night;
      const muffle = a.indoor ? 0.5 : 1;
      loop("loop_rain", town ? a.rain * muffle : 0);
      loop("loop_storm", town ? Math.max(0, a.wind - 0.6) * 2 * muffle : 0);
      loop("loop_wind", town ? (0.15 + a.wind * 0.6) * muffle : 0);
      loop("loop_market", day ? a.square * muffle : 0);
      loop("loop_birds", day ? (1 - a.square * 0.6) * (1 - a.rain) * 0.8 * muffle : 0);
      loop("loop_owl", town && a.night ? 0.5 * muffle : 0);
      loop("loop_forge", day ? a.forge : 0);
      loop("loop_fire", a.fire);
      loop("loop_drips", a.cellar ? 0.6 : 0);
      loop("loop_cave", a.cellar ? 0.5 : 0);
      loop("loop_water", a.water);
      loop("loop_lava", a.lava);
      loop("loop_chasm", a.chasm ?? 0);
      loop("loop_portal", a.portal ?? 0);
    },
    // Om opptaket finnes (lydene i byen bruker opptak når de kan).
    has(name) {
      const e = manifest?.[name];
      return !!e && e.files.some((f) => buffers.has(f));
    },
  };
}
