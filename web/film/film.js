// Åpningsfilmen (docs/film/manus.md, kanon i docs/kanon.md): en selvstendig,
// malt film. Eierens bilder står i lag i dybden i én sammenhengende verden,
// lest som et billedteppe: å fly mot høyre er at tiden går, og kameraet
// dykker inn i hendelsene. Lagene forskyves mot hverandre med parallakse når
// kameraet beveger seg, personene puster og svaier svakt, lys blafrer og
// støvet driver. Tekstene henger i rommet mellom lagene og kommer ord for ord.
// Spillet er ikke med i filmen; til slutt toner filmen bort, og spillet står
// bak ved sørporten (`reveal`).
//
// Manuset er `opening.json` (lagd av `tools/film/intro.py`): lagene, tekstene,
// kamerabanen, musikken, lydbildet, slørene og støvet.

// three.js 0.160.0 (MIT, web/film/THREE-LICENSE), lagt ved så filmen ikke trenger nett.
import * as THREE from "./three.module.min.js";
import { createSoundscape } from "./soundscape.js";

const LOOK = 0.025; // radianer musa kan dreie kameraet
const rad = THREE.MathUtils.degToRad;
const clamp = (x, a = 0, b = 1) => Math.min(Math.max(x, a), b);
const ease = (t) => t * t * (3 - 2 * t);
// En konvolutt [inn fra, helt inn, ut fra, borte] → 0..1.
const env = (e, x) => (!e ? 1 : x < e[0] || x > e[3] ? 0 : x < e[1] ? ease((x - e[0]) / Math.max(e[1] - e[0], 1e-3)) : x > e[2] ? 1 - ease((x - e[2]) / Math.max(e[3] - e[2], 1e-3)) : 1);

function el(tag, cls, parent, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text) e.textContent = text;
  parent?.appendChild(e);
  return e;
}

function loadImage(url) {
  return new Promise((ok) => {
    const im = new Image();
    im.onload = () => ok(im);
    im.onerror = () => ok(null);
    im.src = url;
  });
}

function texture(img) {
  const t = img instanceof HTMLCanvasElement ? new THREE.CanvasTexture(img) : new THREE.Texture(img);
  t.colorSpace = THREE.SRGBColorSpace;
  t.anisotropy = 8;
  t.minFilter = THREE.LinearMipmapLinearFilter;
  t.needsUpdate = true;
  return t;
}

// Masken som maler et lag fram og tar det bort: blekk som trekker inn i
// papiret (verdistøy i tre skalaer), eller blokker som rykker (`glitch`).
const MASK = `
  uniform float uFade, uGlitch, uTime;
  uniform vec4 uFeather;
  float fh(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
  float fn(vec2 p) { vec2 i = floor(p), f = fract(p); f = f * f * (3.0 - 2.0 * f);
    return mix(mix(fh(i), fh(i + vec2(1, 0)), f.x), mix(fh(i + vec2(0, 1)), fh(i + vec2(1, 1)), f.x), f.y); }
  // Myke kanter (brøk av bildet): venstre, høyre, nede, oppe.
  float feather(vec2 q) {
    return smoothstep(0.0, uFeather.x + 1e-4, q.x) * smoothstep(0.0, uFeather.y + 1e-4, 1.0 - q.x) * smoothstep(0.0, uFeather.z + 1e-4, q.y) * smoothstep(0.0, uFeather.w + 1e-4, 1.0 - q.y);
  }
  float filmMask(vec2 q) {
    if (uFade >= 0.999) return feather(q);
    float n = fn(q * 6.0) * 0.6 + fn(q * 21.0) * 0.28 + fn(q * 70.0) * 0.12;
    if (uGlitch > 0.5) n = fh(floor(q * vec2(48.0, 27.0)) + floor(uTime * 9.0) * 0.17) * 0.75 + fh(floor(q * vec2(12.0, 7.0))) * 0.25;
    float f = uFade * 1.25 - 0.12;
    return smoothstep(n - 0.05, n + 0.05, f) * feather(q);
  }`;

// Et lag: et bilde på et plan, `h` høyt, med midten (eller foten, `anchor:
// "bottom"`) i `at`. Det puster (`breathe`), svaier (`sway`) og går (`walk`)
// i hjørneskyggeren, males fram med masken og farges med `tint`.
function layerMesh(tex, o) {
  const aspect = tex.image.width / tex.image.height;
  const h = o.h;
  const w = h * aspect;
  const g = new THREE.PlaneGeometry(w, h, 24, 24);
  if (o.anchor === "bottom") g.translate(0, h / 2, 0);
  if (o.flip) g.scale(-1, 1, 1);
  const u = {
    uFade: { value: 1 },
    uGlitch: { value: o.mask === "glitch" ? 1 : 0 },
    uTime: { value: 0 },
    uSway: { value: o.sway ?? 0 },
    uBreathe: { value: o.breathe ?? 0 },
    uWalk: { value: o.walk ? 1 : 0 },
    uH: { value: h },
    uBase: { value: o.anchor === "bottom" ? 0 : -h / 2 },
    uFeather: { value: new THREE.Vector4(...(o.feather ?? [0, 0, 0, 0])) },
  };
  const m = new THREE.MeshBasicMaterial({ map: tex, transparent: true, depthWrite: false, side: THREE.DoubleSide, color: new THREE.Color(o.tint ?? "#ffffff") });
  m.onBeforeCompile = (sh) => {
    Object.assign(sh.uniforms, u);
    sh.vertexShader = sh.vertexShader.replace("#include <common>", "#include <common>\nuniform float uTime, uSway, uBreathe, uWalk, uH, uBase;").replace(
      "#include <begin_vertex>",
      `#include <begin_vertex>
      float hf = clamp((transformed.y - uBase) / uH, 0.0, 1.0);
      transformed.y = uBase + (transformed.y - uBase) * (1.0 + sin(uTime * 1.6) * uBreathe * hf);
      transformed.x += sin(uTime * 0.7 + transformed.y * 0.3) * uSway * hf * hf;
      transformed.y += uWalk * abs(sin(uTime * 3.4)) * uH * 0.012;`,
    );
    sh.fragmentShader = sh.fragmentShader
      .replace("#include <common>", "#include <common>\n" + MASK)
      .replace("#include <dithering_fragment>", "#include <dithering_fragment>\n gl_FragColor.a *= filmMask(vMapUv);");
  };
  const mesh = new THREE.Mesh(g, m);
  mesh.userData = { u, w, h };
  return mesh;
}

// En myk, rund prikk (til lys og støv).
let dot = null;
function softDot() {
  if (dot) return dot;
  const c = document.createElement("canvas");
  c.width = c.height = 128;
  const g = c.getContext("2d");
  const r = g.createRadialGradient(64, 64, 0, 64, 64, 64);
  r.addColorStop(0, "rgba(255,255,255,1)");
  r.addColorStop(0.25, "rgba(255,255,255,0.5)");
  r.addColorStop(1, "rgba(255,255,255,0)");
  g.fillStyle = r;
  g.fillRect(0, 0, 128, 128);
  dot = new THREE.CanvasTexture(c);
  return dot;
}

function glow(color, size) {
  const m = new THREE.SpriteMaterial({ map: softDot(), color: new THREE.Color(color), transparent: true, blending: THREE.AdditiveBlending, depthWrite: false });
  const s = new THREE.Sprite(m);
  s.scale.setScalar(size);
  s.userData = { size, phase: Math.random() * 100 };
  return s;
}

// Støvet i lufta rundt kameraet: punkter i en boks som følger kameraet.
const DUST = 10;
function dustField() {
  const n = 700;
  const pos = new Float32Array(n * 3);
  for (let i = 0; i < n * 3; i++) pos[i] = (Math.random() - 0.5) * DUST * 2;
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(pos, 3));
  const m = new THREE.PointsMaterial({ color: 0xffe2a8, size: 0.05, map: softDot(), transparent: true, opacity: 0, depthWrite: false, blending: THREE.AdditiveBlending });
  const p = new THREE.Points(g, m);
  p.frustumCulled = false;
  p.renderOrder = 900;
  return p;
}

function moveDust(p, cam, dt, t, drift) {
  const a = p.geometry.attributes.position;
  const wrap = (v, c) => c + ((((v - c + DUST) % (DUST * 2)) + DUST * 2) % (DUST * 2)) - DUST;
  for (let i = 0; i < a.count; i++) {
    const x = a.getX(i) + drift[0] * dt + Math.sin(t * 0.4 + i) * dt * 0.05;
    const y = a.getY(i) + drift[1] * dt + Math.sin(t * 0.3 + i * 1.3) * dt * 0.04;
    const z = a.getZ(i) + drift[2] * dt;
    a.setXYZ(i, wrap(x, cam.x), wrap(y, cam.y), wrap(z, cam.z));
  }
  a.needsUpdate = true;
}

// --- Tekstene i rommet ----------------------------------------------------------------

const STYLES = {
  heading: { font: '"UnifrakturMaguntia", serif', color: "#f3d58c", scale: 1.0 },
  quote: { font: '"UnifrakturMaguntia", serif', color: "#f6e7c4", scale: 0.78 },
  latin: { font: 'italic 1em "Uncial Antiqua", serif', color: "#e9c77a", scale: 0.62 },
  text: { font: '"Uncial Antiqua", serif', color: "#f6ecd6", scale: 0.5 },
};
const PX = 128; // piksler per linjehøyde på lerretet

// Ett ord som et lite plan med skrift og en mørk glød rundt, så det leses
// mot både lyse og mørke bilder.
function wordMesh(word, style, lineH) {
  const st = STYLES[style] ?? STYLES.text;
  const px = PX * st.scale;
  const font = st.font.startsWith("italic") ? `italic ${px}px ${st.font.replace("italic 1em ", "")}` : `${px}px ${st.font}`;
  const c = document.createElement("canvas");
  const g = c.getContext("2d");
  g.font = font;
  const pad = px * 0.45;
  c.width = Math.ceil(g.measureText(word).width + pad * 2);
  c.height = Math.ceil(px * 1.5 + pad);
  g.font = font;
  g.textBaseline = "alphabetic";
  g.shadowColor = "rgba(10, 6, 2, 0.9)";
  g.shadowBlur = px * 0.25;
  g.fillStyle = st.color;
  const base = pad / 2 + px * 1.05;
  for (let k = 0; k < 3; k++) g.fillText(word, pad, base);
  const tex = texture(c);
  const h = (lineH * st.scale * c.height) / px;
  const w = (h * c.width) / c.height;
  const m = new THREE.MeshBasicMaterial({ map: tex, transparent: true, depthWrite: false, depthTest: false, opacity: 0 });
  const mesh = new THREE.Mesh(new THREE.PlaneGeometry(w, h), m);
  mesh.renderOrder = 1000;
  // Bredden av selve ordet (uten luft) til linjeskiftet.
  mesh.userData = { w, adv: (w * (c.width - pad * 2)) / c.width, pad: (w * pad) / c.width, h };
  return mesh;
}

// En tekst: linjer med stil, brutt til `width`, plassert i `at` (midten av
// første linje for `align: "center"`, ellers venstre kant).
function textGroup(spec) {
  const group = new THREE.Group();
  group.position.set(...spec.at);
  const words = [];
  let y = 0;
  for (const [style, line] of spec.lines) {
    const st = STYLES[style] ?? STYLES.text;
    const lh = spec.h * st.scale;
    const space = lh * 0.32;
    const rows = [[]];
    let x = 0;
    for (const w of line.split(" ")) {
      const m = wordMesh(w, style, spec.h);
      if (x > 0 && x + m.userData.adv > spec.width) {
        rows.push([]);
        x = 0;
      }
      rows[rows.length - 1].push(m);
      x += m.userData.adv + space;
    }
    for (const row of rows) {
      const total = row.reduce((s, m) => s + m.userData.adv, 0) + space * (row.length - 1);
      let x0 = spec.align === "center" ? -total / 2 : spec.align === "right" ? -total : 0;
      for (const m of row) {
        m.position.set(x0 + m.userData.adv / 2, y, 0);
        x0 += m.userData.adv + space;
        group.add(m);
        words.push(m);
      }
      y -= lh * 1.25;
    }
    y -= lh * 0.2;
  }
  // Teksten vender mot der kameraet er når den leses.
  if (spec.face) group.lookAt(new THREE.Vector3(...spec.face));
  return { group, words };
}

// --- Kamerabanen ----------------------------------------------------------------------

// Catmull–Rom med tiden som parameter; endene av banen står stille.
function hermite(keys, t, get) {
  let i = 0;
  while (i < keys.length - 2 && t > keys[i + 1].t) i++;
  const a = keys[i];
  const b = keys[Math.min(i + 1, keys.length - 1)];
  if (a === b || t <= a.t) return get(a).clone();
  if (t >= b.t) return get(b).clone();
  const h = b.t - a.t;
  const s = (t - a.t) / h;
  const tangent = (k) => {
    const p = keys[k - 1];
    const n = keys[k + 1];
    if (!p || !n) return get(keys[k]).clone().multiplyScalar(0);
    return get(n).clone().sub(get(p)).multiplyScalar(1 / (n.t - p.t));
  };
  const m0 = tangent(i).multiplyScalar(h);
  const m1 = tangent(i + 1).multiplyScalar(h);
  const s2 = s * s;
  const s3 = s2 * s;
  return get(a)
    .clone()
    .multiplyScalar(2 * s3 - 3 * s2 + 1)
    .add(m0.multiplyScalar(s3 - 2 * s2 + s))
    .add(get(b).clone().multiplyScalar(-2 * s3 + 3 * s2))
    .add(m1.multiplyScalar(s3 - s2));
}

// --- Filmen ---------------------------------------------------------------------------

// Bygger filmen. `score.cue(navn)` bytter musikken, og `audio()` gir
// lydkonteksten. Gir { tick(dt) → hvor mye av filmen som dekker spillet
// (1 til 0 når den toner bort), done, time, poseAt }: `done` løses når
// filmen er slutt (med `skipped` om den ble hoppet over).
export async function createFilm({ score, audio, url = "film/opening.json", base = "film/", start = 0 } = {}) {
  const script = await (await fetch(url)).json();
  const root = el("div", "film", document.body);
  const canvas = el("canvas", "film-canvas", root);
  const veil = el("div", "film-veil", root);
  const skip = el("button", "fs-button film-skip", root, "Hopp over");
  el("p", "film-rotate", root, "Snu telefonen for større bilde.");
  skip.dataset.variant = "ghost";
  document.body.classList.add("filming");
  const sound = createSoundscape(audio);
  await Promise.all(Object.values(STYLES).map((s) => document.fonts?.load?.(`${PX}px ${s.font.replace("italic 1em ", "")}`).catch(() => {})));

  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio || 1, 1.5));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.setClearColor(0x000000, 1);
  const camera = new THREE.PerspectiveCamera(40, 16 / 9, 0.1, 5000);
  // Filmen er komponert i 16:9 (tekstene står der de skal i det bildet), så
  // den vises alltid slik, med svarte felt på smale og høye skjermer.
  const fit = () => {
    const w = Math.min(innerWidth, (innerHeight * 16) / 9);
    const h = (w * 9) / 16;
    renderer.setSize(w, h, false);
    Object.assign(canvas.style, { inset: "auto", width: `${w}px`, height: `${h}px`, left: `${(innerWidth - w) / 2}px`, top: `${(innerHeight - h) / 2}px` });
    camera.aspect = 16 / 9;
    camera.updateProjectionMatrix();
  };
  fit();
  addEventListener("resize", fit);

  const scene = new THREE.Scene();
  const layers = [];
  const images = await Promise.all(script.layers.map((l) => loadImage(base + l.src)));
  script.layers.forEach((l, i) => {
    if (!images[i]) return console.warn("Filmen mangler", l.src);
    const mesh = layerMesh(texture(images[i]), l);
    mesh.position.set(...l.at);
    mesh.renderOrder = l.order ?? 0;
    const lights = (l.lights ?? []).map(([u, v, color, size, flicker = 1]) => {
      const g = glow(color, size);
      g.position.set((u - 0.5) * mesh.userData.w, (l.anchor === "bottom" ? 1 - v : 0.5 - v) * mesh.userData.h, 0.05);
      g.userData.flicker = flicker;
      g.renderOrder = (l.order ?? 0) + 1;
      mesh.add(g);
      return g;
    });
    const path = l.path?.map(([t, x, y, z]) => ({ t, p: new THREE.Vector3(x, y, z) }));
    scene.add(mesh);
    layers.push({ ...l, mesh, lights, path });
  });
  // Lagene tegnes bakfra og fram (lengst unna først), så utklippene legger seg riktig.
  layers.sort((a, b) => (a.order ?? 0) - (b.order ?? 0) || a.at[2] - b.at[2]).forEach((l, i) => (l.mesh.renderOrder = i));
  const texts = (script.texts ?? []).map((x) => {
    const { group, words } = textGroup(x);
    scene.add(group);
    return { ...x, group, words };
  });
  const dust = dustField();
  scene.add(dust);

  const keys = script.camera.map((k) => ({
    t: k.t,
    p: new THREE.Vector3(...k.p),
    look: new THREE.Vector3(...(k.look ?? [k.p[0], k.p[1], k.p[2] - 10])),
    x: new THREE.Vector3(k.fov ?? 40, k.roll ?? 0, 0),
  }));
  const poseAt = (t) => {
    const p = hermite(keys, t, (k) => k.p);
    const look = hermite(keys, t, (k) => k.look);
    const x = hermite(keys, t, (k) => k.x);
    return { p, look, fov: x.x, roll: x.y };
  };

  let mouse = [0, 0];
  const onMove = (e) => {
    mouse = [(e.clientX / innerWidth) * 2 - 1, (e.clientY / innerHeight) * 2 - 1];
  };
  let skipped = false;
  const onKey = (e) => {
    if (e.code === "Escape" || e.code === "Space") {
      e.preventDefault();
      e.stopPropagation();
      skipped = true;
    }
  };
  addEventListener("mousemove", onMove);
  addEventListener("keydown", onKey, true);
  skip.addEventListener("click", () => (skipped = true));

  let finish;
  const done = new Promise((r) => (finish = r));
  let t = start;
  let over = false;
  const look = [0, 0];
  const fired = new Set();
  const fadeOut = script.fade ?? [script.end - 3, script.end];

  const close = () => {
    over = true;
    sound.stop(2.5);
    removeEventListener("mousemove", onMove);
    removeEventListener("keydown", onKey, true);
    removeEventListener("resize", fit);
    scene.traverse((o) => {
      o.geometry?.dispose?.();
      if (o.material) {
        o.material.map?.dispose?.();
        o.material.dispose?.();
      }
    });
    renderer.dispose();
    renderer.forceContextLoss?.();
    root.remove();
    document.body.classList.remove("filming");
  };

  const tick = (dt) => {
    if (over) return 0;
    dt = Math.min(dt, 0.1);
    t += dt;
    if (skipped || t >= script.end) {
      close();
      finish({ skipped });
      return 0;
    }
    for (const [i, e] of (script.events ?? []).entries()) {
      if (t < e.t || fired.has(i)) continue;
      fired.add(i);
      if (e.cue) score?.cue?.(e.cue, e.fade ?? 2.5);
      if (e.sound !== undefined) sound.set(e.sound);
    }
    // Kameraet, med musa.
    const c = poseAt(t);
    look[0] += (mouse[0] - look[0]) * Math.min(dt * 2, 1);
    look[1] += (mouse[1] - look[1]) * Math.min(dt * 2, 1);
    camera.position.copy(c.p);
    camera.lookAt(c.look);
    camera.rotateY(-look[0] * LOOK);
    camera.rotateX(-look[1] * LOOK);
    camera.rotateZ(rad(c.roll));
    camera.fov = c.fov;
    camera.updateProjectionMatrix();

    for (const l of layers) {
      const f = env(l.show, t);
      l.mesh.visible = f > 0.001;
      if (!l.mesh.visible) continue;
      const u = l.mesh.userData.u;
      u.uFade.value = l.mask ? f : 1;
      l.mesh.material.opacity = l.mask ? 1 : f;
      u.uTime.value = t;
      if (l.path) {
        let i = 0;
        while (i < l.path.length - 2 && t > l.path[i + 1].t) i++;
        const a = l.path[i];
        const b = l.path[Math.min(i + 1, l.path.length - 1)];
        const s = a === b ? 0 : clamp((t - a.t) / (b.t - a.t));
        l.mesh.position.copy(a.p).lerp(b.p, l.walk ? s : ease(s));
      }
      // Fjerne lag følger kameraet et stykke (`follow`), så himmelen nesten står stille.
      if (l.follow) {
        l.mesh.position.x = l.at[0] + (c.p.x - l.at[0]) * l.follow;
        l.mesh.position.y = l.at[1] + (c.p.y - (l.camY ?? l.at[1])) * l.follow;
      }
      for (const g of l.lights) {
        const k = 0.8 + 0.2 * Math.sin(t * 9 + g.userData.phase) * Math.sin(t * 3.7 + g.userData.phase * 2) * g.userData.flicker;
        g.material.opacity = k * f;
        g.scale.setScalar(g.userData.size * (0.9 + 0.1 * k));
      }
    }
    // Ordene kommer ett og ett, stiger litt fram, og går samlet.
    for (const x of texts) {
      const on = t >= x.t[0] - 0.1 && t < x.t[1] + 1;
      x.group.visible = on;
      if (!on) continue;
      x.words.forEach((w, i) => {
        const a = Math.min(ease(clamp((t - x.t[0] - i * (x.pace ?? 0.12)) / 0.9)), ease(clamp((x.t[1] - t) / 0.9)));
        w.material.opacity = a;
        w.position.z = (1 - a) * -0.6 * x.h;
      });
    }
    const dz = (script.dust ?? []).reduce((acc, d) => (t >= d.t ? d : acc), { amount: 0 });
    dust.material.opacity += ((dz.amount ?? 0) - dust.material.opacity) * Math.min(dt * 1.5, 1);
    if (dz.color) dust.material.color.set(dz.color);
    moveDust(dust, camera.position, dt, t, dz.drift ?? [0.05, 0.02, 0]);
    renderer.render(scene, camera);

    // Slørene (svart, lyst eller lyn), og til slutt toner filmen bort.
    let black = 0;
    let white = 0;
    for (const v of script.veils ?? []) {
      const k = env(v.t, t);
      if (v.color === "white") white = Math.max(white, k);
      else black = Math.max(black, k);
    }
    veil.style.background = white > black ? `rgba(255, 246, 225, ${white})` : `rgba(0, 0, 0, ${black})`;
    const cover = 1 - ease(clamp((t - fadeOut[0]) / (fadeOut[1] - fadeOut[0])));
    root.style.opacity = cover.toFixed(3);
    return cover;
  };

  return { tick, done, time: () => t, poseAt };
}
