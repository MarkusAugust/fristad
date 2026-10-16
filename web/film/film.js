// Filmene (docs/plan.md del 8): eierens malte bilder som rom kameraet går
// gjennom. Hvert bilde løftes etter dybdekartet sitt (`<lag>.depth.png`,
// Depth Anything, `tools/film/layers.py depth`): bakgrunnen blir et
// rutenett der hvert punkt står så langt unna som maleriet sier, så
// kameraet kan gå inn i det og se forbi tingene i forgrunnen. Utklippene
// (figurene, ting foran) står på sin dybde med litt relieff, puster, svaier
// og går. Kameraet går i én sammenhengende bane: hver scene går over i den
// neste gjennom noe i bildet (døra, porten, boka, sprekken), lysene
// blafrer, støvet henger i lufta, og lydbildet følger rommet. Den siste
// scenen venter med en framdriftslinje til spillet er klart, og går så
// gjennom sørporten og over i spillet (`onHandoff`).
//
// Manuset er data (`opening.json`). Esc, mellomrom eller «Hopp over»
// hopper over; musa dreier kameraet litt.

// three.js 0.160.0 (MIT, web/film/THREE-LICENSE), lagt ved så filmen ikke trenger nett.
import * as THREE from "./three.module.min.js";
import { createSoundscape } from "./soundscape.js";

const FOV = 50; // kameraets synsfelt (grader, loddrett), det samme som maleriet er løftet for
const OVER = 1.16; // maleriet strekker seg så mye utenfor bildet, så kanten ikke synes når kameraet går
const LOOK = 0.035; // radianer musa kan dreie kameraet
const TRANS = 2.6; // sekunder en overgang varer (scenene overlapper så lenge)
const TEXT_IN = 0.9; // sekunder etter overgangen før teksten kommer
const rad = THREE.MathUtils.degToRad;

const ease = (t) => t * t * (3 - 2 * t);
const clamp = (x, a = 0, b = 1) => Math.min(Math.max(x, a), b);

function el(tag, cls, parent, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text) e.textContent = text;
  parent?.appendChild(e);
  return e;
}

// --- Bildene og dybden ----------------------------------------------------------------

function loadImage(url) {
  return new Promise((ok) => {
    const im = new Image();
    im.onload = () => ok(im);
    im.onerror = () => ok(null);
    im.src = url;
  });
}

function texture(img) {
  const t = new THREE.Texture(img);
  t.colorSpace = THREE.SRGBColorSpace;
  t.anisotropy = 8;
  t.generateMipmaps = true;
  t.minFilter = THREE.LinearMipmapLinearFilter;
  t.needsUpdate = true;
  return t;
}

// Dybdekartet som tall (0 langt unna, 1 nær), og alfaen til et utklipp.
function pixels(img, w = img.width) {
  const h = Math.round((img.height * w) / img.width);
  const cv = document.createElement("canvas");
  cv.width = w;
  cv.height = h;
  const g = cv.getContext("2d", { willReadFrequently: true });
  g.drawImage(img, 0, 0, w, h);
  return { w, h, data: g.getImageData(0, 0, w, h).data };
}

function sample(P, u, v, ch = 0) {
  const x = clamp(u, 0, 1) * (P.w - 1);
  const y = clamp(v, 0, 1) * (P.h - 1);
  const x0 = Math.floor(x);
  const y0 = Math.floor(y);
  const x1 = Math.min(x0 + 1, P.w - 1);
  const y1 = Math.min(y0 + 1, P.h - 1);
  const fx = x - x0;
  const fy = y - y0;
  const at = (xx, yy) => P.data[(yy * P.w + xx) * 4 + ch] / 255;
  return (at(x0, y0) * (1 - fx) + at(x1, y0) * fx) * (1 - fy) + (at(x0, y1) * (1 - fx) + at(x1, y1) * fx) * fy;
}

// Fra dybdekartets nærhet til meter foran kameraet; himmelen står like bak
// det fjerneste.
function distance(d, near, far, sky) {
  if (sky && d < sky) return far * 1.25;
  return 1 / (d * (1 / near - 1 / far) + 1 / far);
}

const MARGIN = 0.14; // rutenettet går så langt utenfor bildet (kantpikslene strekkes), så kanten ikke synes
const TEAR = 1.7; // en rute der dybden hopper mer enn dette (forhold), rives opp

// Et rutenett over maleriet, løftet etter dybden (`D`, eller flatt på
// `flat` meter). Rutene der dybden hopper (kanten av noe foran) rives
// opp, så det nære står fritt foran det fjerne; bak hullene står bildet
// flatt langt unna (`backdrop`).
function grid(tex, D, o, flat, tear) {
  const aspect = tex.image.width / tex.image.height;
  const cols = o.cols ?? 240;
  const rows = Math.max(2, Math.round(cols / aspect));
  const tanV = Math.tan(rad(FOV / 2)) * OVER;
  const tanH = tanV * aspect;
  const near = o.near ?? 3;
  const far = o.far ?? 60;
  const ext = Math.round(cols * MARGIN);
  const ey = Math.round(rows * MARGIN);
  const W = cols + 2 * ext + 1;
  const H = rows + 2 * ey + 1;
  const pos = new Float32Array(W * H * 3);
  const uv = new Float32Array(W * H * 2);
  const rr = new Float32Array(W * H);
  for (let j = 0; j < H; j++) {
    for (let i = 0; i < W; i++) {
      const u = (i - ext) / cols;
      const v = (j - ey) / rows;
      const cu = clamp(u, 0.0005, 0.9995);
      const cv = clamp(v, 0.0005, 0.9995);
      const r = flat ?? (D ? distance(sample(D, cu, cv), near, far, o.sky) : far);
      const k = j * W + i;
      rr[k] = r;
      pos[k * 3] = (u * 2 - 1) * tanH * r;
      pos[k * 3 + 1] = (1 - v * 2) * tanV * r;
      pos[k * 3 + 2] = -r;
      uv[k * 2] = cu;
      uv[k * 2 + 1] = 1 - cv;
    }
  }
  const idx = [];
  const ok = (p, q, w) => !tear || Math.max(rr[p], rr[q], rr[w]) / Math.min(rr[p], rr[q], rr[w]) < TEAR;
  for (let j = 0; j < H - 1; j++) {
    for (let i = 0; i < W - 1; i++) {
      const a = j * W + i;
      const b = a + 1;
      const c = a + W;
      const d = c + 1;
      if (ok(a, c, b)) idx.push(a, c, b);
      if (ok(b, c, d)) idx.push(b, c, d);
    }
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(pos, 3));
  g.setAttribute("uv", new THREE.BufferAttribute(uv, 2));
  g.setIndex(idx);
  return { g, tanV, tanH, near, far };
}

// Et maleri som fyller bildet, løftet etter dybden.
function plate(tex, D, o) {
  const { g, tanV, tanH, near, far } = grid(tex, D, o, null, true);
  const m = new THREE.MeshBasicMaterial({ map: tex, fog: false, transparent: !!o.fade, depthWrite: !o.fade, side: THREE.DoubleSide });
  const mesh = new THREE.Mesh(g, m);
  mesh.renderOrder = o.fade ? 1 : 0;
  mesh.userData = { ...o, tanV, tanH, near, far, D };
  if (!o.fade) {
    const back = new THREE.Mesh(grid(tex, null, o, far * 1.4, false).g, new THREE.MeshBasicMaterial({ map: tex, fog: false, side: THREE.DoubleSide }));
    back.renderOrder = -1;
    mesh.add(back);
  }
  return mesh;
}

// Stedet i rommet bak punktet (u, v) i maleriet (brøk av bildet fra
// øverst til venstre), trukket `pull` meter mot kameraet.
function pointOn(mesh, u, v, pull = 0) {
  const o = mesh.userData;
  const r = (o.D ? distance(sample(o.D, u, v), o.near, o.far, o.sky) : o.far) - pull;
  return new THREE.Vector3((u * 2 - 1) * o.tanH * r, (1 - v * 2) * o.tanV * r, -r);
}

// Et utklipp (figurer, ting foran): så stor del `h` av høyden kameraet ser
// på dybden `depth`, med midten `x` og foten `bottom` (brøk av bildet), og
// et relieff etter dybdekartet. Det puster, svaier og går i hjørneskyggeren.
function cutout(tex, D, o) {
  const aspect = tex.image.width / tex.image.height;
  const viewH = 2 * o.depth * Math.tan(rad(FOV / 2));
  const hh = viewH * o.h;
  const w = hh * aspect;
  const x0 = (o.x ?? 0) * viewH * (16 / 9);
  const y0 = (o.bottom ?? -0.5) * viewH;
  const cols = 64;
  const rows = Math.max(2, Math.round(cols / aspect));
  const n = (cols + 1) * (rows + 1);
  const pos = new Float32Array(n * 3);
  const uv = new Float32Array(n * 2);
  const relief = o.relief ?? hh * 0.18;
  for (let j = 0; j <= rows; j++) {
    for (let i = 0; i <= cols; i++) {
      const u = i / cols;
      const v = j / rows;
      const k = j * (cols + 1) + i;
      pos[k * 3] = (u - 0.5) * w;
      pos[k * 3 + 1] = (1 - v) * hh;
      pos[k * 3 + 2] = D ? (sample(D, u, v) - 0.5) * relief : 0;
      uv[k * 2] = u;
      uv[k * 2 + 1] = 1 - v;
    }
  }
  const idx = [];
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const a = j * (cols + 1) + i;
      idx.push(a, a + cols + 1, a + 1, a + 1, a + cols + 1, a + cols + 2);
    }
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(pos, 3));
  g.setAttribute("uv", new THREE.BufferAttribute(uv, 2));
  g.setIndex(idx);
  const uniforms = { uT: { value: 0 }, uSway: { value: o.sway ?? 0 }, uBreathe: { value: o.breathe ?? 0 }, uH: { value: hh }, uStep: { value: o.walk ? 1 : 0 } };
  const m = new THREE.MeshBasicMaterial({ map: tex, fog: true, transparent: true, depthWrite: false, depthTest: false, alphaTest: 0.02, side: THREE.DoubleSide });
  // Pust (høyden løftes litt), svai (toppen mer enn foten) og gange (et lite hopp i takt).
  m.onBeforeCompile = (sh) => {
    Object.assign(sh.uniforms, uniforms);
    sh.vertexShader = sh.vertexShader
      .replace("#include <common>", "#include <common>\nuniform float uT, uSway, uBreathe, uH, uStep;")
      .replace(
        "#include <begin_vertex>",
        `#include <begin_vertex>
        float hf = clamp(transformed.y / uH, 0.0, 1.0);
        transformed.y *= 1.0 + sin(uT * 1.6) * uBreathe * hf;
        transformed.x += sin(uT * 0.7 + transformed.y * 0.3) * uSway * hf * hf;
        transformed.y += uStep * abs(sin(uT * 3.6)) * uH * 0.02;
        transformed.x += uStep * sin(uT * 3.6) * uH * 0.008 * hf;`,
      );
  };
  const mesh = new THREE.Mesh(g, m);
  mesh.position.set(x0, y0, -o.depth);
  mesh.renderOrder = 10 + Math.round(100 - o.depth);
  mesh.userData = { ...o, x0, y0, uniforms, hh };
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
  r.addColorStop(0.25, "rgba(255,255,255,0.55)");
  r.addColorStop(1, "rgba(255,255,255,0)");
  g.fillStyle = r;
  g.fillRect(0, 0, 128, 128);
  dot = new THREE.CanvasTexture(c);
  return dot;
}

// Et lys som blafrer: en glorie på stedet i maleriet.
function glow(at, color, size) {
  const m = new THREE.SpriteMaterial({ map: softDot(), color: new THREE.Color(color), transparent: true, blending: THREE.AdditiveBlending, depthWrite: false, depthTest: false, fog: false });
  const s = new THREE.Sprite(m);
  s.position.copy(at);
  s.scale.setScalar(size);
  s.renderOrder = 200;
  s.userData = { size, phase: Math.random() * 100 };
  return s;
}

// Støv i lyset, glør, flak fra glitchen eller tåke, spredt i rommet mellom
// det nære og det fjerne, så de forskyves med kameraet som alt annet.
function particles(kind, near, far) {
  if (!kind) return null;
  const n = kind === "mist" ? 90 : 700;
  const pos = new Float32Array(n * 3);
  const speed = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const z = near * 0.6 + Math.random() * Math.min(far, 30);
    pos[i * 3] = (Math.random() - 0.5) * z * 1.6;
    pos[i * 3 + 1] = (Math.random() - 0.5) * z * 0.9;
    pos[i * 3 + 2] = -z;
    speed[i] = 0.3 + Math.random();
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(pos, 3));
  const color = { dust: 0xffe2a8, embers: 0xff8a3a, glitch: 0xff3cc8, snow: 0xffffff, mist: 0xb8c8e8, motes: 0x9fc0ff }[kind] ?? 0xffffff;
  const size = kind === "mist" ? 4.5 : kind === "embers" ? 0.08 : 0.045;
  const m = new THREE.PointsMaterial({ color, size, map: softDot(), transparent: true, opacity: kind === "mist" ? 0.07 : 0.8, depthWrite: false, blending: THREE.AdditiveBlending, fog: false });
  const p = new THREE.Points(g, m);
  p.renderOrder = 150;
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
    const z = -a.getZ(i);
    if (kind === "embers") y += dt * speed[i] * 0.6;
    else if (kind === "snow") y -= dt * speed[i] * 0.8;
    else y += Math.sin(t * 0.5 + i) * dt * 0.05 * speed[i];
    x += dt * (kind === "mist" ? 0.35 : 0.05) * speed[i] + Math.sin(t * 0.3 + i * 1.7) * dt * 0.03;
    if (kind === "glitch" && Math.random() < 0.02) {
      x += (Math.random() - 0.5) * 0.4;
      y += (Math.random() - 0.5) * 0.4;
    }
    const lim = z * 0.5;
    if (y > lim) y = -lim;
    if (y < -lim) y = lim;
    if (x > z * 0.85) x = -z * 0.85;
    a.setXY(i, x, y);
  }
  a.needsUpdate = true;
}

// Kameraet langs nøklene (meter og radianer fra der maleriet er tatt).
function cameraAt(keys, u) {
  let i = 0;
  while (i < keys.length - 2 && u > keys[i + 1].at) i++;
  const a = keys[i];
  const b = keys[Math.min(i + 1, keys.length - 1)];
  const f = b === a ? 0 : clamp((u - a.at) / Math.max(b.at - a.at, 1e-6));
  const s = keys.length > 2 ? f : ease(f);
  const lerp = (k, d = 0) => (a[k] ?? d) + ((b[k] ?? d) - (a[k] ?? d)) * s;
  return { x: lerp("x"), y: lerp("y"), z: lerp("z"), yaw: lerp("yaw"), pitch: lerp("pitch"), zoom: lerp("zoom", 1) };
}

// Overgangen: den nye scenen åpner seg i bildet av den gamle fra et punkt
// (døra, porten, boka): som et hull som vokser (`iris`), i blokker som
// glitcher (`glitch`), med lys rundt kanten (`flash`) eller ut av mørket (`dark`).
const COMPOSE = {
  uniforms: { tA: { value: null }, tB: { value: null }, uC: { value: new THREE.Vector2(0.5, 0.5) }, uR: { value: 0 }, uAspect: { value: 1 }, uMode: { value: 0 }, uTime: { value: 0 } },
  vertexShader: "varying vec2 vUv; void main(){ vUv = uv; gl_Position = vec4(position.xy, 0.0, 1.0); }",
  fragmentShader: `
    uniform sampler2D tA, tB; uniform vec2 uC; uniform float uR, uAspect, uMode, uTime; varying vec2 vUv;
    float h(vec2 p){ return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453); }
    void main(){
      vec2 d = vUv - uC; d.x *= uAspect; float dist = length(d);
      float m; vec2 shift = vec2(0.0);
      if (uMode < 0.5 || uMode > 1.5) {
        m = 1.0 - smoothstep(uR * 0.72, uR, dist);
      } else {
        vec2 b = floor(vUv * vec2(40.0, 22.0));
        float n = h(b + floor(uTime * 12.0));
        m = step(dist + n * 0.45, uR * 1.2);
        float edge = step(abs(dist + n * 0.45 - uR * 1.2), 0.08);
        shift = vec2((h(b.yy + floor(uTime * 20.0)) - 0.5) * 0.04 * edge, 0.0);
      }
      vec4 A = texture2D(tA, vUv + shift);
      vec4 B = texture2D(tB, vUv - shift);
      vec4 col = mix(A, B, m);
      float rim = exp(-pow((dist - uR) * 7.0, 2.0)) * (1.0 - clamp(uR / 1.6, 0.0, 1.0));
      if (uMode > 1.5 && uMode < 2.5) col.rgb += vec3(1.0, 0.92, 0.78) * rim * 1.4;
      if (uMode > 2.5) col.rgb *= 1.0 - rim * 0.9 - (1.0 - m) * smoothstep(0.0, 0.6, uR) * 0.5;
      if (uMode > 0.5 && uMode < 1.5) col.rgb += vec3(1.0, 0.2, 0.8) * step(abs(dist - uR * 1.2), 0.012) * 0.22;
      gl_FragColor = col;
      #include <colorspace_fragment>
    }`,
};
const MODES = { iris: 0, glitch: 1, flash: 2, dark: 3 };

// Spiller filmen. `music.set(navn)` bytter musikken og `audio()` gir
// lydkonteksten. `ready()` sier om spillet er klart ({ done, progress }),
// og `onHandoff(x, y)` kalles når kameraet går gjennom porten (punktet i
// bildet): spillet tegnes da bak filmen, som åpner seg mot det.
export async function playFilm({ music, audio, url = "film/opening.json", base = "film/", ready, onHandoff, start = 0 } = {}) {
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
  const progress = el("div", "film-progress", root);
  const bar = el("div", "film-progress-fill", progress);
  progress.hidden = true;
  const skip = el("button", "fs-button film-skip", root, "Hopp over");
  skip.dataset.variant = "ghost";
  document.body.classList.add("filming");
  const sound = createSoundscape(audio);

  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio || 1, 1.5));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  const camera = new THREE.PerspectiveCamera(FOV, 16 / 9, 0.05, 600);
  const rtOpts = { type: THREE.HalfFloatType, colorSpace: THREE.LinearSRGBColorSpace, samples: 4 };
  let rtA = new THREE.WebGLRenderTarget(16, 16, rtOpts);
  let rtB = new THREE.WebGLRenderTarget(16, 16, rtOpts);
  const compose = new THREE.ShaderMaterial(COMPOSE);
  const quadScene = new THREE.Scene();
  quadScene.add(new THREE.Mesh(new THREE.PlaneGeometry(2, 2), compose));
  const quadCam = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  let baseFov = FOV;
  const fit = () => {
    renderer.setSize(innerWidth, innerHeight, false);
    camera.aspect = innerWidth / innerHeight;
    // Er skjermen bredere enn maleriet, snevres synsfeltet inn, så kanten ikke synes.
    const tanH = Math.tan(rad(FOV / 2)) * 1.79 * 1.06;
    baseFov = Math.min(FOV, 2 * THREE.MathUtils.radToDeg(Math.atan(tanH / camera.aspect)));
    camera.updateProjectionMatrix();
    const s = renderer.getDrawingBufferSize(new THREE.Vector2());
    rtA.setSize(s.x, s.y);
    rtB.setSize(s.x, s.y);
    compose.uniforms.uAspect.value = camera.aspect;
  };
  fit();
  addEventListener("resize", fit);

  // Alle scenene lastes før filmen begynner.
  const scenes = [];
  for (const s of script.scenes) {
    const scene = new THREE.Scene();
    const [fc, fnear, ffar] = s.fog ?? ["#000000", 200, 600];
    scene.fog = new THREE.Fog(new THREE.Color(fc), fnear, ffar);
    scene.background = new THREE.Color(s.backdrop ?? fc);
    const plates = [];
    const cutouts = [];
    for (const l of s.layers) {
      const [img, dimg] = await Promise.all([loadImage(base + l.src), loadImage(base + l.src.replace(".webp", ".depth.png"))]);
      if (!img) continue;
      const D = dimg ? pixels(dimg, Math.min(dimg.width, 640)) : null;
      const tex = texture(img);
      const mesh = l.h ? cutout(tex, D, l) : plate(tex, D, { ...s.space, ...l });
      scene.add(mesh);
      (l.h ? cutouts : plates).push(mesh);
    }
    const main = plates[0];
    const lights = [];
    for (const [u, v, color, size, flicker = 1] of s.lights ?? []) {
      const g = glow(pointOn(main, u, v, 0.3), color, size);
      g.userData.flicker = flicker;
      scene.add(g);
      lights.push(g);
    }
    const p = particles(s.particles, s.space?.near ?? 3, s.space?.far ?? 60);
    if (p) scene.add(p);
    const portal = s.portal ? { ...s.portal, at: pointOn(main, s.portal.u, s.portal.v, 0.5) } : null;
    scenes.push({ ...s, scene, plates, cutouts, lights, particles: p, portal });
  }
  loading.remove();

  let mouse = [0, 0];
  const onMove = (e) => {
    mouse = [(e.clientX / innerWidth) * 2 - 1, (e.clientY / innerHeight) * 2 - 1];
  };
  let skipped = false;
  const onSkip = () => (skipped = true);
  const onKey = (e) => {
    if (e.code === "Escape" || e.code === "Space") {
      e.preventDefault();
      e.stopPropagation();
      onSkip();
    }
  };
  addEventListener("mousemove", onMove);
  addEventListener("keydown", onKey, true);
  skip.addEventListener("click", onSkip);

  // Tidslinjen: scene k begynner når overgangen inn i den begynner.
  const starts = [];
  let acc = 0;
  for (const s of scenes) {
    starts.push(acc);
    acc += s.dur - TRANS;
  }
  const lastIdx = scenes.length - 1;
  const mainEnd = starts[lastIdx] + scenes[lastIdx].dur;

  const look = [0, 0];
  const v3 = new THREE.Vector3();
  let prev = performance.now();
  const t0 = prev;
  let offset = start; // «Hopp over» før spillet er klart: rett til ventingen i siste scene (og `start` til testene)
  let shown = -1;
  let handoff = null; // { t, x, y } når kameraet går gjennom porten til spillet
  let waitFrom = null;
  let soundKind = null;

  // Plasserer kameraet i scene k ved tiden `at` (sekunder i scenen); `out`
  // er hvor langt overgangen ut av den har kommet (0–1).
  const place = (s, at, out, t) => {
    const u = clamp(at / s.dur, 0, 1.5);
    const c = cameraAt(s.camera, Math.min(u, 1));
    let pos = new THREE.Vector3(c.x, c.y, c.z);
    let yaw = c.yaw;
    let pitch = c.pitch;
    // Venter den siste scenen på spillet, driver kameraet sakte videre.
    if (u > 1) {
      const extra = (u - 1) * s.dur;
      pos.z -= Math.min(extra * 0.05, 1.2);
      yaw += Math.sin(t * 0.15) * 0.01;
    }
    // Overgangen ut: kameraet trekkes mot porten og dreies litt mot den.
    if (out > 0 && s.portal) {
      const e = out * out * (3 - 2 * out);
      const P = s.portal.at;
      pos = pos.lerp(P, e * (s.portal.pull ?? 0.55));
      const dir = v3.copy(P).sub(pos).normalize();
      yaw += (Math.atan2(-dir.x, -dir.z) - yaw) * e * 0.6;
      pitch += (Math.asin(clamp(dir.y, -1, 1)) - pitch) * e * 0.6;
    }
    look[0] += (mouse[0] - look[0]) * 0.03;
    look[1] += (mouse[1] - look[1]) * 0.03;
    camera.position.copy(pos);
    camera.rotation.set(pitch - look[1] * LOOK, yaw - look[0] * LOOK, 0, "YXZ");
    camera.fov = baseFov / c.zoom;
    camera.updateProjectionMatrix();
    // Lagene lever: pust, svai, gange, glitch; lysene blafrer.
    for (const m of s.cutouts) {
      const o = m.userData;
      o.uniforms.uT.value = t + (o.depth ?? 0);
      // Det kameraet kommer nær (på vei mot porten), toner bort.
      m.material.opacity = clamp((pos.z - m.position.z - 1.2) / 1.6);
      if (o.drift) {
        const f = clamp(at / s.dur);
        m.position.set(o.x0 + o.drift[0] * f, o.y0 + o.drift[1] * f, -o.depth - o.drift[2] * f);
      }
      if (o.glitch) {
        const hit = Math.random() < 0.05 + clamp(at / s.dur) * 0.3;
        m.position.x = o.x0 + (hit ? (Math.random() - 0.5) * 0.4 : 0);
        m.material.color.setRGB(1, hit ? 0.6 : 1, 1);
      }
    }
    for (const m of s.plates) {
      const o = m.userData;
      if (o.fade) m.material.opacity = ease(clamp((at / s.dur - o.fade[0]) / (o.fade[1] - o.fade[0])));
    }
    for (const g of s.lights) {
      const o = g.userData;
      const f = 0.82 + 0.18 * Math.sin(t * 9 + o.phase) * Math.sin(t * 3.7 + o.phase * 2) * o.flicker;
      g.material.opacity = f;
      g.scale.setScalar(o.size * (0.9 + 0.1 * f));
    }
  };

  await new Promise((done) => {
    const tick = (now) => {
      const dt = Math.min((now - prev) / 1000, 0.1);
      prev = now;
      const st = ready?.() ?? { done: true, progress: 1 };
      if (skipped && !st.done && onHandoff) offset = Math.max(offset, mainEnd - TRANS + 0.01 - (now - t0) / 1000);
      const t = (now - t0) / 1000 + offset;

      // Hopp over: rett til porten om spillet er klart, ellers en ventelinje.
      if (skipped && !handoff) {
        if (st.done && onHandoff) {
          handoff = { t, x: innerWidth / 2, y: innerHeight / 2, fast: true };
          onHandoff(handoff.x, handoff.y);
        } else if (!onHandoff) {
          done();
          return;
        }
      }

      // Hvilken scene, og hvor langt i den.
      let k = 0;
      while (k < lastIdx && t >= starts[k + 1] + TRANS) k++;
      const s = scenes[k];
      const at = t - starts[k];
      // Den siste scenen: ferdig, venter, eller går gjennom porten.
      if (k === lastIdx && t >= mainEnd - TRANS && !handoff) {
        if (st.done) {
          if (onHandoff) {
            const P = v3.copy(s.portal?.at ?? new THREE.Vector3(0, 0, -20)).project(camera);
            handoff = { t, x: ((P.x + 1) / 2) * innerWidth, y: ((1 - P.y) / 2) * innerHeight };
            onHandoff(handoff.x, handoff.y);
          } else if (t >= mainEnd) {
            done();
            return;
          }
        } else {
          waitFrom ??= t;
          progress.hidden = false;
          bar.style.width = `${Math.round(clamp(st.progress) * 100)}%`;
        }
      }

      if (k !== shown) {
        shown = k;
        if (s.music) music?.set?.(s.music === "none" ? "silence" : s.music);
        heading.textContent = s.heading ?? "";
        heading.hidden = !s.heading;
        line.textContent = s.text ?? "";
        root.dataset.fx = s.fx ?? "";
      }
      if (s.sound !== soundKind) {
        soundKind = s.sound;
        sound.set(s.sound);
      }

      // Overgangen til neste scene (de siste TRANS sekundene av denne).
      const next = k < lastIdx ? scenes[k + 1] : null;
      const out = next ? clamp((at - (s.dur - TRANS)) / TRANS) : 0;
      // Den siste scenen dras mot porten mens spillet åpner seg.
      const hand = handoff ? clamp((t - handoff.t) / (handoff.fast ? 1.0 : 2.2)) : 0;
      place(s, at, next ? out : hand, t);
      moveParticles(s.particles, dt, t);
      if (out > 0 && next) {
        const pa = s.portal ? v3.copy(s.portal.at).project(camera) : new THREE.Vector3();
        compose.uniforms.uC.value.set((pa.x + 1) / 2, (pa.y + 1) / 2);
        renderer.setRenderTarget(rtA);
        renderer.render(s.scene, camera);
        place(next, out * TRANS, 0, t);
        moveParticles(next.particles, dt, t);
        renderer.setRenderTarget(rtB);
        renderer.render(next.scene, camera);
        renderer.setRenderTarget(null);
        compose.uniforms.tA.value = rtA.texture;
        compose.uniforms.tB.value = rtB.texture;
        compose.uniforms.uR.value = Math.pow(out, 1.6) * 1.9;
        compose.uniforms.uMode.value = MODES[s.portal?.mode ?? "iris"] ?? 0;
        compose.uniforms.uTime.value = t;
        renderer.render(quadScene, quadCam);
      } else {
        renderer.setRenderTarget(null);
        renderer.render(s.scene, camera);
      }

      // Svart inn i begynnelsen; teksten kommer etter overgangen og går før neste.
      // Uten spill bak (filmen alene) toner den ut i svart til slutt.
      black.style.opacity = String(Math.max(1 - clamp(t / 2.0), onHandoff ? 0 : clamp((t - (mainEnd - 1.6)) / 1.5)));
      const inStart = k === 0 ? 1.2 : TRANS + TEXT_IN;
      const ti = clamp((at - inStart) / 1.0);
      const to = next ? clamp((s.dur - TRANS - at) / 0.8) : handoff ? 0 : 1;
      words.style.opacity = String(Math.min(ti, to));
      if (s.fx === "glitch") {
        const g = clamp(at / s.dur);
        const hit = Math.random() < 0.05 + g * 0.35;
        glitch.style.opacity = hit ? String(0.18 + g * 0.4) : "0";
        glitch.style.transform = hit ? `translate(${(Math.random() - 0.5) * 30 * g}px, ${(Math.random() - 0.5) * 10}px)` : "none";
        glitch.style.clipPath = hit ? `inset(${Math.random() * 80}% 0 ${Math.random() * 15}% 0)` : "none";
      } else {
        glitch.style.opacity = "0";
      }

      // Gjennom porten: filmen åpner seg som et hull rundt porten, og spillet
      // står bak (det tegnes nå).
      if (handoff) {
        progress.hidden = true;
        const e = Math.pow(hand, 1.7);
        const r = e * Math.hypot(innerWidth, innerHeight) * 1.1;
        const mask = `radial-gradient(circle at ${handoff.x}px ${handoff.y}px, transparent ${r}px, black ${r + 120}px)`;
        root.style.maskImage = mask;
        root.style.webkitMaskImage = mask;
        if (hand >= 1) {
          done();
          return;
        }
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });

  sound.stop(2.5);
  removeEventListener("mousemove", onMove);
  removeEventListener("keydown", onKey, true);
  removeEventListener("resize", fit);
  for (const sc of scenes) {
    sc.scene.traverse((o) => {
      o.geometry?.dispose?.();
      if (o.material) {
        o.material.map?.dispose?.();
        o.material.dispose?.();
      }
    });
  }
  rtA.dispose();
  rtB.dispose();
  renderer.dispose();
  renderer.forceContextLoss?.();
  root.remove();
  document.body.classList.remove("filming");
}
