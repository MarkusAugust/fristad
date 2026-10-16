// Filmene (docs/plan.md del 8): malte lag som teksturerte plan i 3D med
// ekte perspektiv (three.js). Kameraet glir sakte, så lagene forskyves mot
// hverandre og gir dybde. Manuset er data (`opening.json`): scenene, lagene
// og dybden deres, kamerabanen, tåka, partiklene, musikken og teksten i
// middelalderskrift. Esc, mellomrom eller «Hopp over» hopper over; musa
// dreier kameraet litt. Lagene ligger i `film/<film>/<scene>/<lag>.webp`
// (`tools/film/layers.py`, promptene i docs/prompter-film.md).

// three.js 0.160.0 (MIT, web/film/THREE-LICENSE), lagt ved så filmen ikke trenger nett.
import * as THREE from "./three.module.min.js";

const FADE = 0.9; // sekunder til og fra svart mellom scenene
const TEXT_IN = 1.4; // teksten kommer etter så mange sekunder
const LOOK = 0.035; // radianer musa kan dreie kameraet
const FOV = 40; // kameraets synsfelt (grader, loddrett)
const OVERSCAN = 1.22; // lagene er så mye større enn det kameraet ser, så kanten aldri synes

function ease(t) {
  return t * t * (3 - 2 * t);
}

// Kameraet langs nøklene (myk start og slutt, rett mellom nøklene).
function cameraAt(keys, t) {
  const u = ease(Math.min(Math.max(t, 0), 1));
  let i = 0;
  while (i < keys.length - 2 && u > keys[i + 1].at) i++;
  const a = keys[i];
  const b = keys[Math.min(i + 1, keys.length - 1)];
  const f = b === a ? 0 : Math.min(Math.max((u - a.at) / Math.max(b.at - a.at, 1e-6), 0), 1);
  const s = ease(f);
  const lerp = (k, d = 0) => (a[k] ?? d) + ((b[k] ?? d) - (a[k] ?? d)) * s;
  return { x: lerp("x"), y: lerp("y"), z: lerp("z"), yaw: lerp("yaw"), pitch: lerp("pitch"), zoom: lerp("zoom", 1) };
}

function el(tag, cls, parent, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text) e.textContent = text;
  parent?.appendChild(e);
  return e;
}

// En myk, rund prikk (punktene er ellers firkanter).
let dot = null;
function softDot() {
  if (dot) return dot;
  const c = document.createElement("canvas");
  c.width = c.height = 64;
  const g = c.getContext("2d");
  const r = g.createRadialGradient(32, 32, 0, 32, 32, 32);
  r.addColorStop(0, "rgba(255,255,255,1)");
  r.addColorStop(0.4, "rgba(255,255,255,0.45)");
  r.addColorStop(1, "rgba(255,255,255,0)");
  g.fillStyle = r;
  g.fillRect(0, 0, 64, 64);
  dot = new THREE.CanvasTexture(c);
  return dot;
}

// Støv i lyset, glør, snø eller drivende tåke: små lysende punkter.
function particles(kind) {
  if (!kind) return null;
  const n = kind === "mist" ? 60 : 420;
  const pos = new Float32Array(n * 3);
  const speed = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    pos[i * 3] = (Math.random() - 0.5) * 30;
    pos[i * 3 + 1] = (Math.random() - 0.5) * 18;
    pos[i * 3 + 2] = -3 - Math.random() * 22;
    speed[i] = 0.3 + Math.random();
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(pos, 3));
  const color = { dust: 0xffe2a8, embers: 0xff8a3a, glitch: 0xff3cc8, snow: 0xffffff, mist: 0xc8d4e8, motes: 0x9fc0ff }[kind] ?? 0xffffff;
  const size = kind === "mist" ? 3.2 : kind === "embers" ? 0.09 : 0.06;
  const m = new THREE.PointsMaterial({ color, size, map: softDot(), alphaTest: 0.001, transparent: true, opacity: kind === "mist" ? 0.08 : 0.75, depthWrite: false, blending: THREE.AdditiveBlending });
  const p = new THREE.Points(g, m);
  p.userData = { kind, speed };
  return p;
}

function moveParticles(p, dt, t) {
  if (!p) return;
  const a = p.geometry.attributes.position;
  const { kind, speed } = p.userData;
  for (let i = 0; i < speed.length; i++) {
    let x = a.getX(i);
    let y = a.getY(i);
    if (kind === "embers") y += dt * speed[i] * 0.9;
    else if (kind === "snow") y -= dt * speed[i] * 0.8;
    else y += Math.sin(t * 0.4 + i) * dt * 0.08 * speed[i];
    x += dt * (kind === "mist" ? 0.25 : 0.06) * speed[i];
    if (kind === "glitch") {
      x += (Math.random() - 0.5) * 0.3;
      y += (Math.random() - 0.5) * 0.3;
    }
    if (y > 9) y = -9;
    if (y < -9) y = 9;
    if (x > 15) x = -15;
    a.setXY(i, x, y);
  }
  a.needsUpdate = true;
}

// Et lag: et plan på dybden `depth` (meter foran kameraet), stort nok til
// å fylle bildet der, med bildet tegnet uten lys.
// Et lag uten `h` fyller bildet (bakgrunnen); med `h` er det så stor del av
// høyden kameraet ser på den dybden, med midten `x` (−0,5 til 0,5 av
// bredden) og foten `bottom` (−0,5 er nederst i bildet).
function layerMesh(tex, depth, aspect, opts) {
  const viewH = 2 * depth * Math.tan(THREE.MathUtils.degToRad(FOV / 2));
  const imgAspect = tex.image.width / tex.image.height;
  const m = new THREE.MeshBasicMaterial({ map: tex, transparent: true, depthWrite: false, fog: true });
  let w;
  let hh;
  let x = 0;
  let y = 0;
  if (opts.h) {
    hh = viewH * opts.h;
    w = hh * imgAspect;
    x = (opts.x ?? 0) * viewH * aspect;
    y = (opts.bottom ?? -0.5) * viewH + hh / 2;
  } else {
    const h = viewH * OVERSCAN;
    w = Math.max(h * aspect, h * imgAspect);
    hh = w / imgAspect;
  }
  const mesh = new THREE.Mesh(new THREE.PlaneGeometry(w, hh), m);
  mesh.position.set(x, y, -depth);
  mesh.userData = { ...opts, x0: x, y0: y };
  return mesh;
}

async function loadTexture(loader, url) {
  try {
    const t = await loader.loadAsync(url);
    t.colorSpace = THREE.SRGBColorSpace;
    t.anisotropy = 4;
    return t;
  } catch {
    return null;
  }
}

// Spiller filmen. `music.set(navn)` bytter musikken, `onLast()` kalles
// idet den siste scenen begynner å tone ut (spillet gjøres klart bak).
export async function playFilm({ music, url = "film/opening.json", base = "film/", onLast } = {}) {
  let script;
  try {
    script = await (await fetch(url)).json();
  } catch (e) {
    console.warn("Fant ikke filmen:", e);
    return;
  }
  const root = el("div", "film", document.body);
  const canvas = el("canvas", "film-canvas", root);
  el("div", "film-bar film-bar--top", root);
  el("div", "film-bar film-bar--bottom", root);
  const black = el("div", "film-black", root);
  const words = el("div", "film-words", root);
  const heading = el("h2", "film-heading", words);
  const line = el("p", "film-text", words);
  const glitch = el("div", "film-glitch", root);
  const loading = el("p", "film-loading", root, "…");
  const skip = el("button", "fs-button film-skip", root, "Hopp over");
  skip.dataset.variant = "ghost";
  document.body.classList.add("filming");

  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio || 1, 1.5));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  const camera = new THREE.PerspectiveCamera(FOV, 16 / 9, 0.1, 400);
  const fit = () => {
    renderer.setSize(innerWidth, innerHeight, false);
    camera.aspect = innerWidth / innerHeight;
    camera.updateProjectionMatrix();
  };
  fit();
  addEventListener("resize", fit);

  // Alle scenene lastes før filmen begynner (de er små).
  const loader = new THREE.TextureLoader();
  const scenes = [];
  for (const s of script.scenes) {
    const scene = new THREE.Scene();
    const [fc, near, far] = s.fog ?? ["#000000", 200, 400];
    scene.fog = new THREE.Fog(new THREE.Color(fc), near, far);
    scene.background = new THREE.Color(s.backdrop ?? "#000000");
    const layers = [];
    for (const l of s.layers) {
      const tex = await loadTexture(loader, base + l.src);
      if (!tex) continue;
      const mesh = layerMesh(tex, l.depth, camera.aspect, l);
      scene.add(mesh);
      layers.push(mesh);
    }
    const p = particles(s.particles);
    if (p) scene.add(p);
    scenes.push({ ...s, scene, layers, particles: p });
  }
  loading.remove();

  let mouse = [0, 0];
  const onMove = (e) => {
    mouse = [(e.clientX / innerWidth) * 2 - 1, (e.clientY / innerHeight) * 2 - 1];
  };
  let over = false;
  const end = () => (over = true);
  const onKey = (e) => {
    if (e.code === "Escape" || e.code === "Space") {
      e.preventDefault();
      e.stopPropagation();
      end();
    }
  };
  addEventListener("mousemove", onMove);
  addEventListener("keydown", onKey, true);
  skip.addEventListener("click", end);

  const total = scenes.reduce((s, x) => s + x.dur, 0);
  let current = -1;
  let lastCalled = false;
  let prev = performance.now();
  const start = prev;
  const look = [0, 0];
  await new Promise((done) => {
    const tick = (now) => {
      const dt = Math.min((now - prev) / 1000, 0.1);
      prev = now;
      const t = (now - start) / 1000;
      if (over || t >= total) {
        done();
        return;
      }
      let k = 0;
      let at = t;
      while (k < scenes.length - 1 && at >= scenes[k].dur) {
        at -= scenes[k].dur;
        k++;
      }
      const s = scenes[k];
      if (k !== current) {
        current = k;
        if (s.music) music?.set?.(s.music === "none" ? "silence" : s.music);
        heading.textContent = s.heading ?? "";
        heading.hidden = !s.heading;
        line.textContent = s.text ?? "";
        root.dataset.fx = s.fx ?? "";
      }
      const last = k === scenes.length - 1;
      if (last && !lastCalled && s.dur - at < 2.5) {
        lastCalled = true;
        onLast?.();
      }
      // Kameraet, med et lite drag mot musa.
      look[0] += (mouse[0] - look[0]) * Math.min(dt * 2, 1);
      look[1] += (mouse[1] - look[1]) * Math.min(dt * 2, 1);
      const c = cameraAt(s.camera, at / s.dur);
      camera.position.set(c.x, c.y, c.z);
      camera.rotation.set(c.pitch - look[1] * LOOK, c.yaw - look[0] * LOOK, 0, "YXZ");
      camera.zoom = c.zoom;
      camera.updateProjectionMatrix();
      // Lagene puster og svaier litt; glitchen rykker i dem.
      for (const m of s.layers) {
        const o = m.userData;
        if (o.fade) {
          const [a, b] = o.fade;
          m.material.opacity = ease(Math.min(Math.max((at / s.dur - a) / (b - a), 0), 1));
        }
        if (o.breathe) m.scale.y = 1 + Math.sin(t * 1.7 + o.depth) * o.breathe;
        if (o.sway) m.position.x = o.x0 + Math.sin(t * 0.6 + o.depth) * o.sway;
        if (o.drift) m.position.set(o.x0 + o.drift[0] * (at / s.dur), o.y0 + o.drift[1] * (at / s.dur), -o.depth - o.drift[2] * (at / s.dur));
        if (s.fx === "glitch" && o.glitch) {
          const g = at / s.dur;
          const hit = Math.random() < 0.06 + g * 0.4;
          m.position.x = o.x0 + (hit ? (Math.random() - 0.5) * 0.6 * g : 0);
          m.material.color.setRGB(1, hit ? 0.7 : 1, 1);
        }
      }
      moveParticles(s.particles, dt, t);
      renderer.render(s.scene, camera);
      // Til og fra svart; den siste scenen toner ut i spillet.
      const fadeIn = k === 0 ? Math.min(at / 1.8, 1) : Math.min(at / FADE, 1);
      const fadeOut = Math.min((s.dur - at) / (last ? 1.6 : FADE), 1);
      black.style.opacity = String(1 - Math.min(fadeIn, fadeOut));
      const ti = Math.min(Math.max((at - TEXT_IN) / 1.2, 0), 1);
      const to = Math.min(Math.max((s.dur - at - 0.7) / 1.0, 0), 1);
      words.style.opacity = String(Math.min(ti, to));
      if (s.fx === "glitch") {
        const g = at / s.dur;
        const hit = Math.random() < 0.06 + g * 0.45;
        glitch.style.opacity = hit ? String(0.25 + g * 0.5) : "0";
        glitch.style.transform = hit ? `translate(${(Math.random() - 0.5) * 40 * g}px, ${(Math.random() - 0.5) * 12}px)` : "none";
        glitch.style.clipPath = hit ? `inset(${Math.random() * 80}% 0 ${Math.random() * 15}% 0)` : "none";
      } else {
        glitch.style.opacity = "0";
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
  if (!lastCalled) onLast?.();
  removeEventListener("mousemove", onMove);
  removeEventListener("keydown", onKey, true);
  removeEventListener("resize", fit);
  for (const sc of scenes) {
    for (const m of sc.layers) {
      m.material.map?.dispose();
      m.material.dispose();
      m.geometry.dispose();
    }
    sc.particles?.geometry.dispose();
  }
  renderer.dispose();
  renderer.forceContextLoss?.();
  root.remove();
  document.body.classList.remove("filming");
}
