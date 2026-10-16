// Åpningsfilmen (docs/plan.md del 8; eieren 16. oktober: «spilleren skal
// fly gjennom historien»): kameraet flyr gjennom det ekte Fristad, tegnet av
// motoren, og historien står i verdenen som malerier der den skjedde:
// Skriptoriet i gullalderen foran Skriptoriets dør, forfallet i samme rom,
// begravelsen ved krypten på kirkegården, krypten nede i krypten, torget i
// 1350 over torget. Kameraet flyr inn i maleriene, og inne i dem står hvert
// punkt så langt unna som maleriet sier (dybdekartet fra Depth Anything,
// `tools/film/layers.py depth`), så alt forskyves mot hverandre når kameraet
// beveger seg. Mellom maleriene flyr det over byen, gjennom portene og ned i
// jorda, og legger seg i svingene.
//
// Filmen tegner maleriene, figurene, lysene, støvet og teksten oppå motoren
// med samme kamera (gjennomsiktig lerret over spillet), og styrer motorens
// kamera hvert bilde (`engine.film`). Dag og natt, krypten og torget byttes
// mens et maleri eller et slør fyller bildet, så ingen ser sømmen. Manuset
// er `opening.json`: maleriene og ankrene deres, figurene i verden,
// kamerabanen, tekstene, slørene, musikken og lyset.

// three.js 0.160.0 (MIT, web/film/THREE-LICENSE), lagt ved så filmen ikke trenger nett.
import * as THREE from "./three.module.min.js";
import { createSoundscape } from "./soundscape.js";

const FOV = 50; // synsfeltet maleriene er løftet for (grader, loddrett)
const OVER = 1.16; // maleriet strekker seg så mye utenfor det kameraet ser fra ankeret
const LOOK = 0.03; // radianer musa kan dreie kameraet
const rad = THREE.MathUtils.degToRad;
const deg = THREE.MathUtils.radToDeg;
const clamp = (x, a = 0, b = 1) => Math.min(Math.max(x, a), b);
const ease = (t) => t * t * (3 - 2 * t);

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
  t.minFilter = THREE.LinearMipmapLinearFilter;
  t.needsUpdate = true;
  return t;
}

// Dybdekartet som tall (0 langt unna, 1 nær).
function pixels(img, w = Math.min(img.width, 640)) {
  const h = Math.round((img.height * w) / img.width);
  const cv = document.createElement("canvas");
  cv.width = w;
  cv.height = h;
  const g = cv.getContext("2d", { willReadFrequently: true });
  g.drawImage(img, 0, 0, w, h);
  return { w, h, data: g.getImageData(0, 0, w, h).data };
}

function sample(P, u, v) {
  const x = clamp(u) * (P.w - 1);
  const y = clamp(v) * (P.h - 1);
  const x0 = Math.floor(x);
  const y0 = Math.floor(y);
  const x1 = Math.min(x0 + 1, P.w - 1);
  const y1 = Math.min(y0 + 1, P.h - 1);
  const fx = x - x0;
  const fy = y - y0;
  const at = (xx, yy) => P.data[(yy * P.w + xx) * 4] / 255;
  return (at(x0, y0) * (1 - fx) + at(x1, y0) * fx) * (1 - fy) + (at(x0, y1) * (1 - fx) + at(x1, y1) * fx) * fy;
}

// Fra dybdekartets nærhet til avstand foran ankeret; himmelen står like
// bak det fjerneste.
function distance(d, near, far, sky) {
  if (sky && d < sky) return far * 1.25;
  return 1 / (d * (1 / near - 1 / far) + 1 / far);
}

const TEAR = 1.7; // en rute der dybden hopper mer enn dette (forhold), rives opp

// Masken som maler bildet fram og tar det bort: verdistøy i tre skalaer, så
// maleriet kommer til syne som blekk som trekker inn i papiret, eller i
// blokker som rykker (`glitch`), og kanten av bildet går mykt over i verden.
const MASK = `
  uniform float uFade, uGlitch, uTime, uEdge, uMix, uFlip;
  float fh(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
  float fn(vec2 p) { vec2 i = floor(p), f = fract(p); f = f * f * (3.0 - 2.0 * f);
    return mix(mix(fh(i), fh(i + vec2(1, 0)), f.x), mix(fh(i + vec2(0, 1)), fh(i + vec2(1, 1)), f.x), f.y); }
  float filmMask(vec2 q) {
    float edge = mix(1.0, smoothstep(0.0, 0.08, min(min(q.x, 1.0 - q.x), min(q.y, 1.0 - q.y))), uEdge);
    float n = fn(q * 6.0) * 0.6 + fn(q * 21.0) * 0.28 + fn(q * 70.0) * 0.12;
    if (uGlitch > 0.5) n = fh(floor(q * vec2(48.0, 27.0)) + floor(uTime * 9.0) * 0.17) * 0.75 + fh(floor(q * vec2(12.0, 7.0))) * 0.25;
    float f = uFade * 1.25 - 0.12;
    return edge * smoothstep(n - 0.05, n + 0.05, f) * mix(uMix, 1.0 - uMix, uFlip);
  }`;

function maskShader(sh, uniforms) {
  Object.assign(sh.uniforms, uniforms);
  sh.fragmentShader = sh.fragmentShader
    .replace("#include <common>", "#include <common>\n" + MASK)
    .replace("#include <dithering_fragment>", "#include <dithering_fragment>\n gl_FragColor.a *= filmMask(vMapUv);");
}

// Et maleri som rutenett i ankerets rom (kameraet i origo ser mot −Z),
// løftet etter dybden og revet opp der dybden hopper.
function plate(tex, D, o, uniforms, flat = null) {
  const aspect = tex.image.width / tex.image.height;
  const cols = 220;
  const rows = Math.max(2, Math.round(cols / aspect));
  const tanV = Math.tan(rad(FOV / 2)) * OVER;
  const tanH = tanV * aspect;
  const near = o.near ?? 3;
  const far = o.far ?? 60;
  const W = cols + 1;
  const H = rows + 1;
  const pos = new Float32Array(W * H * 3);
  const uv = new Float32Array(W * H * 2);
  const rr = new Float32Array(W * H);
  for (let j = 0; j < H; j++) {
    for (let i = 0; i < W; i++) {
      const u = i / cols;
      const v = j / rows;
      const r = flat ?? (D ? distance(sample(D, u, v), near, far, o.sky) : far);
      const k = j * W + i;
      rr[k] = r;
      pos[k * 3] = (u * 2 - 1) * tanH * r;
      pos[k * 3 + 1] = (1 - v * 2) * tanV * r;
      pos[k * 3 + 2] = -r;
      uv[k * 2] = u;
      uv[k * 2 + 1] = 1 - v;
    }
  }
  const idx = [];
  const ok = (a, b, c) => flat !== null || Math.max(rr[a], rr[b], rr[c]) / Math.min(rr[a], rr[b], rr[c]) < TEAR;
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
  const m = new THREE.MeshBasicMaterial({ map: tex, transparent: true, depthWrite: true, side: THREE.DoubleSide });
  m.onBeforeCompile = (sh) => maskShader(sh, { ...uniforms, uFlip: { value: flat === null ? 0 : 1 } });
  const mesh = new THREE.Mesh(g, m);
  mesh.userData = { tanV, tanH, near, far, sky: o.sky, D, fade: o.fade };
  return mesh;
}

// Maleriet sett utenfra er et flatt bilde som henger i lufta (`FLAT`); jo
// nærmere kameraet kommer stedet maleriet er sett fra, jo mer dybde får det
// (`uMix`, de to toner over i hverandre mens de faller sammen sett derfra).
const FLAT = (o) => Math.sqrt((o.near ?? 3) * (o.far ?? 60)) * 0.9;
function deepPlate(tex, D, o, uniforms) {
  const deep = plate(tex, D, o, uniforms);
  const flat = plate(tex, D, o, uniforms, FLAT(o));
  flat.renderOrder = deep.renderOrder - 1;
  deep.add(flat);
  return deep;
}

// Stedet bak punktet (u, v) i maleriet, i ankerets rom.
function pointOn(mesh, u, v, pull = 0) {
  const o = mesh.userData;
  const r = (o.D ? distance(sample(o.D, u, v), o.near, o.far, o.sky) : o.far) - pull;
  return new THREE.Vector3((u * 2 - 1) * o.tanH * r, (1 - v * 2) * o.tanV * r, -r);
}

// Et utklipp med relieff etter dybdekartet: `height` høyt, foten i origo.
// Det puster, svaier og går i hjørneskyggeren.
function cutout(tex, D, o, uniforms) {
  const aspect = tex.image.width / tex.image.height;
  const hh = o.height;
  const w = hh * aspect;
  const cols = 72;
  const rows = Math.max(2, Math.round(cols / aspect));
  const n = (cols + 1) * (rows + 1);
  const pos = new Float32Array(n * 3);
  const uv = new Float32Array(n * 2);
  const relief = o.relief ?? hh * 0.15;
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
  const own = { uT: { value: 0 }, uSway: { value: o.sway ?? 0 }, uBreathe: { value: o.breathe ?? 0 }, uH: { value: hh }, uStep: { value: o.walk ? 1 : 0 }, uNoEdge: { value: 0 } };
  const m = new THREE.MeshBasicMaterial({ map: tex, transparent: true, depthWrite: false, alphaTest: 0.02, side: THREE.DoubleSide });
  m.onBeforeCompile = (sh) => {
    Object.assign(sh.uniforms, own);
    sh.vertexShader = sh.vertexShader
      .replace("#include <common>", "#include <common>\nuniform float uT, uSway, uBreathe, uH, uStep;")
      .replace(
        "#include <begin_vertex>",
        `#include <begin_vertex>
        float hf = clamp(transformed.y / uH, 0.0, 1.0);
        transformed.y *= 1.0 + sin(uT * 1.6) * uBreathe * hf;
        transformed.x += sin(uT * 0.7 + transformed.y * 0.3) * uSway * hf * hf;
        transformed.y += uStep * abs(sin(uT * 3.4)) * uH * 0.025;
        transformed.x += uStep * sin(uT * 3.4) * uH * 0.01 * hf;`,
      );
    maskShader(sh, { ...uniforms, uEdge: { value: 0 }, uFlip: { value: 0 }, uMix: { value: 1 } });
  };
  const mesh = new THREE.Mesh(g, m);
  mesh.userData = { ...o, own };
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

function glow(at, color, size) {
  const m = new THREE.SpriteMaterial({ map: softDot(), color: new THREE.Color(color), transparent: true, blending: THREE.AdditiveBlending, depthWrite: false, depthTest: false });
  const s = new THREE.Sprite(m);
  s.position.copy(at);
  s.scale.setScalar(size);
  s.renderOrder = 200;
  s.userData = { size, phase: Math.random() * 100 };
  return s;
}

// Støvet i lufta rundt kameraet: punkter i en boks som følger kameraet (de
// som faller ut, kommer inn på andre siden), så de forskyves som alt annet.
const DUST = 12;
function dustField() {
  const n = 900;
  const pos = new Float32Array(n * 3);
  for (let i = 0; i < n * 3; i++) pos[i] = (Math.random() - 0.5) * DUST * 2;
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(pos, 3));
  const m = new THREE.PointsMaterial({ color: 0xffe2a8, size: 0.04, map: softDot(), transparent: true, opacity: 0, depthWrite: false, depthTest: false, blending: THREE.AdditiveBlending });
  const p = new THREE.Points(g, m);
  p.frustumCulled = false;
  p.renderOrder = 150;
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

// --- Rammer og kamerabanen -------------------------------------------------------------

// Et rom med origo, dreining (grader) og målestokk: verden, ankeret til et
// maleri, startstedet i krypten, stedet du begynner.
function frame(pos, yaw = 0, pitch = 0, scale = 1) {
  const q = new THREE.Quaternion().setFromEuler(new THREE.Euler(rad(pitch), rad(yaw), 0, "YXZ"));
  return { pos: new THREE.Vector3(...pos), q, scale };
}
const toWorld = (f, p) => new THREE.Vector3(...p).multiplyScalar(f.scale).applyQuaternion(f.q).add(f.pos);
const dirOf = (yaw, pitch) => new THREE.Vector3(-Math.sin(rad(yaw)) * Math.cos(rad(pitch)), Math.sin(rad(pitch)), -Math.cos(rad(yaw)) * Math.cos(rad(pitch)));

// Catmull–Rom med tiden som parameter (farten rykker ikke mellom nøklene);
// endene av en bit står stille.
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

// Bygger filmen. `engine` er motoren (film, set_theme, level_start,
// spawn_eye), `score.cue(navn)` bytter musikken (ellers `music.set`), og
// `audio()` gir lydkonteksten. Gir { tick(dt), done }: tick kalles hvert
// bilde før motoren tegner, og `done` løses når filmen er slutt (med
// `skipped` om den ble hoppet over).
export async function createFilm({ engine, score, music, audio, url = "film/opening.json", base = "film/", start = 0 } = {}) {
  const script = await (await fetch(url)).json();
  const root = el("div", "film", document.body);
  const canvas = el("canvas", "film-canvas", root);
  el("div", "film-bar film-bar--top", root);
  el("div", "film-bar film-bar--bottom", root);
  const veil = el("div", "film-veil", root);
  const textLayer = el("div", "film-texts", root);
  const skip = el("button", "fs-button film-skip", root, "Hopp over");
  skip.dataset.variant = "ghost";
  document.body.classList.add("filming");
  const sound = createSoundscape(audio);

  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true, premultipliedAlpha: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio || 1, 1.5));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.setClearColor(0x000000, 0);
  const camera = new THREE.PerspectiveCamera(60, 16 / 9, 0.02, 3000);
  const fit = () => {
    renderer.setSize(innerWidth, innerHeight, false);
    camera.aspect = innerWidth / innerHeight;
    camera.updateProjectionMatrix();
  };
  fit();
  addEventListener("resize", fit);

  // Rammene: verden, stedene motoren vet om, og ankrene til maleriene.
  const frames = { world: frame([0, 0, 0]) };
  const spawn = engine?.spawn_eye?.() ?? script.fallback?.spawn_eye ?? [1.28, 1.75, 36, 0];
  frames.spawn = frame([spawn[0], spawn[1], spawn[2]], spawn[3]);
  for (let n = 1; n <= 4; n++) {
    const s = engine?.level_start?.(n) ?? script.fallback?.levels?.[n - 1] ?? [];
    if (s.length) frames[`crypt${n}`] = frame([s[0], s[1] + 1.75, s[2]], s[3]);
  }
  const scene = new THREE.Scene();
  const paintings = [];
  for (const p of script.paintings) {
    const f0 = frames[p.frame ?? "world"];
    const at = toWorld(f0, p.pos);
    const q = f0.q.clone().multiply(new THREE.Quaternion().setFromEuler(new THREE.Euler(rad(p.pitch ?? 0), rad(p.yaw ?? 0), 0, "YXZ")));
    frames[p.id] = { pos: at, q, scale: p.scale ?? 0.35 };
    const group = new THREE.Group();
    group.position.copy(at);
    group.quaternion.copy(q);
    group.scale.setScalar(p.scale ?? 0.35);
    const uniforms = { uFade: { value: 0 }, uGlitch: { value: p.mask === "glitch" ? 1 : 0 }, uTime: { value: 0 }, uEdge: { value: 1 }, uMix: { value: 0 } };
    const parts = [];
    let main = null;
    for (const l of p.layers) {
      const [img, dimg] = await Promise.all([loadImage(base + l.src), loadImage(base + l.src.replace(".webp", ".depth.png"))]);
      if (!img) continue;
      const D = dimg ? pixels(dimg) : null;
      const tex = texture(img);
      if (l.height) {
        const m = cutout(tex, D, l, uniforms);
        m.position.set(...l.at);
        m.rotation.y = rad(l.yaw ?? 0);
        m.renderOrder = 10;
        group.add(m);
        parts.push(m);
      } else {
        const m = deepPlate(tex, D, { ...p.space, ...l }, uniforms);
        m.renderOrder = l.fade ? 2 : 1;
        group.add(m);
        main ??= m;
        parts.push(m);
      }
    }
    for (const [u, v, color, size, flicker = 1] of p.lights ?? []) {
      const g = glow(pointOn(main, u, v, 0.4), color, size);
      g.userData.flicker = flicker;
      group.add(g);
      parts.push(g);
    }
    scene.add(group);
    paintings.push({ ...p, group, uniforms, parts });
  }
  // Figurene i verden (den fremmede på veien, skiltet ved veikanten).
  const props = [];
  for (const p of script.props ?? []) {
    const [img, dimg] = await Promise.all([loadImage(base + p.src), loadImage(base + p.src.replace(".webp", ".depth.png"))]);
    if (!img) continue;
    const uniforms = { uFade: { value: 0 }, uGlitch: { value: 0 }, uTime: { value: 0 }, uMix: { value: 1 } };
    const m = cutout(texture(img), dimg ? pixels(dimg) : null, p, uniforms);
    const f = frames[p.frame ?? "world"];
    const path = (p.path ?? [[0, ...p.pos]]).map(([t, x, y, z]) => ({ t, p: toWorld(f, [x, y, z]) }));
    m.renderOrder = 120;
    scene.add(m);
    props.push({ ...p, mesh: m, uniforms, path });
  }
  const dust = dustField();
  scene.add(dust);

  // Kamerabanen: nøklene i verden, gjort om fra rammene, delt i biter der
  // banen hopper (bak et maleri eller et slør som fyller bildet).
  const segments = [[]];
  for (const k of script.camera) {
    const f = frames[k.frame ?? "world"];
    if (!f) continue;
    const p = toWorld(f, k.p);
    const d = k.look ? toWorld(f, k.look).sub(p).normalize() : dirOf(k.yaw ?? 0, k.pitch ?? 0).applyQuaternion(f.q).normalize();
    if (k.cut && segments[segments.length - 1].length) segments.push([]);
    segments[segments.length - 1].push({ t: k.t, p, d, x: new THREE.Vector3(k.fov ?? 60, k.roll ?? 0, k.bank ?? 1) });
  }
  const poseAt = (t) => {
    let seg = segments[0];
    for (const s of segments) if (t >= s[0].t) seg = s;
    const p = hermite(seg, t, (k) => k.p);
    const d = hermite(seg, t, (k) => k.d).normalize();
    const x = hermite(seg, t, (k) => k.x);
    return { p, d, fov: x.x, roll: x.y, bank: x.z };
  };

  // Tekstene: store, fritt plassert, ord for ord inn.
  const word = (w, i) => {
    const s = document.createElement("span");
    s.className = "film-word";
    s.textContent = w + " ";
    s.dataset.i = i;
    return s;
  };
  const texts = (script.texts ?? []).map((x) => {
    const box = el("div", `film-say film-say--${x.size ?? "l"}`, textLayer);
    box.style.left = x.x ?? "8vw";
    box.style.top = x.y ?? "60vh";
    if (x.w) box.style.width = x.w;
    box.style.textAlign = x.align ?? "left";
    if (x.heading) el("h2", "film-say-heading", box).append(...x.heading.split(" ").map((w, i) => word(w, i)));
    if (x.text) el("p", "film-say-text", box).append(...x.text.split(" ").map((w, i) => word(w, i + 3)));
    return { ...x, box, words: [...box.querySelectorAll(".film-word")] };
  });

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
  let lastYaw = null;
  let roll = 0;
  const look = [0, 0];
  const fired = new Set();
  const env = (e, x) => (!e || x < e[0] || x > e[3] ? 0 : x < e[1] ? ease((x - e[0]) / Math.max(e[1] - e[0], 1e-3)) : x > e[2] ? 1 - ease((x - e[2]) / Math.max(e[3] - e[2], 1e-3)) : 1);

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
    if (over) return;
    dt = Math.min(dt, 0.1);
    t += dt;
    if (skipped || t >= script.end) {
      close();
      finish({ skipped });
      return;
    }
    // Hendelsene: musikk, lydbilde, dag og natt.
    (script.events ?? []).forEach((e, i) => {
      if (t < e.t || fired.has(i)) return;
      fired.add(i);
      if (e.cue) score?.cue ? score.cue(e.cue, e.fade ?? 2.5) : music?.set?.(e.music ?? "town");
      if (e.sound !== undefined) sound.set(e.sound);
      if (e.theme) engine?.set_theme?.(e.theme === "night");
    });
    // Kameraet, med musa og krengning i svingene.
    const c = poseAt(t);
    const yaw = Math.atan2(-c.d.x, -c.d.z);
    const pitch = Math.asin(clamp(c.d.y, -1, 1));
    if (lastYaw !== null) {
      const dy = Math.atan2(Math.sin(yaw - lastYaw), Math.cos(yaw - lastYaw));
      const want = clamp((dy / Math.max(dt, 1e-3)) * 0.3 * c.bank, -0.35, 0.35);
      roll += (want - roll) * Math.min(dt * 2.0, 1);
    }
    lastYaw = yaw;
    look[0] += (mouse[0] - look[0]) * Math.min(dt * 2, 1);
    look[1] += (mouse[1] - look[1]) * Math.min(dt * 2, 1);
    const fy = yaw - look[0] * LOOK;
    const fp = pitch - look[1] * LOOK;
    const fr = roll + rad(c.roll);
    camera.position.copy(c.p);
    camera.rotation.set(fp, fy, fr, "YXZ");
    camera.fov = c.fov;
    camera.updateProjectionMatrix();
    engine?.film?.(c.p.x, c.p.y, c.p.z, deg(fy), deg(fp), c.fov, deg(fr));

    // Maleriene kommer til syne og går; lagene puster, lysene blafrer.
    for (const p of paintings) {
      const f = env(p.show, t);
      p.group.visible = f > 0.001;
      p.uniforms.uFade.value = f;
      p.uniforms.uTime.value = t;
      if (!p.group.visible) continue;
      const local = p.group.worldToLocal(camera.position.clone());
      p.uniforms.uMix.value = 1 - THREE.MathUtils.smoothstep(local.length(), 0.6, 3.5);
      for (const m of p.parts) {
        const o = m.userData;
        if (o.own) o.own.uT.value = t;
        if (o.fade) {
          const k = ease(clamp((t - o.fade[0]) / (o.fade[1] - o.fade[0])));
          m.material.opacity = k;
          for (const c of m.children) c.material.opacity = k;
        }
        if (o.glitch) m.material.color.setRGB(1, Math.random() < 0.08 ? 0.55 : 1, 1);
        if (o.flicker !== undefined) {
          const k = 0.8 + 0.2 * Math.sin(t * 9 + o.phase) * Math.sin(t * 3.7 + o.phase * 2) * o.flicker;
          m.material.opacity = k * f;
          m.scale.setScalar(o.size * (0.9 + 0.1 * k));
        }
      }
    }
    for (const p of props) {
      const f = env(p.show, t);
      p.mesh.visible = f > 0.001;
      p.uniforms.uFade.value = f;
      p.mesh.userData.own.uT.value = t;
      let i = 0;
      while (i < p.path.length - 2 && t > p.path[i + 1].t) i++;
      const a = p.path[i];
      const b = p.path[Math.min(i + 1, p.path.length - 1)];
      const s = a === b ? 0 : clamp((t - a.t) / (b.t - a.t));
      p.mesh.position.copy(a.p).lerp(b.p, s);
      // Utklippet vender seg mot kameraet (det er malt forfra eller bakfra).
      p.mesh.rotation.y = Math.atan2(camera.position.x - p.mesh.position.x, camera.position.z - p.mesh.position.z);
    }
    // Støvet: farge og mengde etter delen av filmen.
    const dz = (script.dust ?? []).reduce((acc, d) => (t >= d.t ? d : acc), { amount: 0 });
    dust.material.opacity += ((dz.amount ?? 0) - dust.material.opacity) * Math.min(dt * 1.5, 1);
    if (dz.color) dust.material.color.set(dz.color);
    moveDust(dust, camera.position, dt, t, dz.drift ?? [0.05, 0.02, 0]);
    renderer.render(scene, camera);

    // Slørene: svart i begynnelsen, svart eller lys der banen hopper.
    let black = 0;
    let white = 0;
    for (const v of script.veils ?? []) {
      const k = env(v.t, t);
      if (v.color === "white") white = Math.max(white, k);
      else black = Math.max(black, k);
    }
    veil.style.background = white > black ? `rgba(255, 246, 225, ${white})` : `rgba(0, 0, 0, ${black})`;
    // Ordene kommer ett og ett ut av en dis og går samlet, etter filmens klokke.
    for (const x of texts) {
      const on = t >= x.t[0] - 0.1 && t < x.t[1] + 1;
      x.box.hidden = !on;
      if (!on) continue;
      for (const w of x.words) {
        const a = Math.min(ease(clamp((t - x.t[0] - w.dataset.i * 0.08) / 1.1)), ease(clamp((x.t[1] - t) / 0.8)));
        w.style.opacity = a.toFixed(3);
        w.style.filter = a > 0.99 ? "none" : `blur(${((1 - a) * 10).toFixed(1)}px)`;
        w.style.transform = `translateY(${((1 - a) * 0.35).toFixed(3)}em)`;
      }
    }
  };

  return { tick, done, time: () => t, poseAt };
}
