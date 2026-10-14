// 3D-visningen i dommen: modellen (og for kryptbitene hele nivået rundt) med
// teksturlagene lagt på slik motoren gjør (uv i meter delt på flisens mål,
// tonet med materialfargen), så eieren kan rotere, zoome og gå inn i rommene.
// Bygges av `tools/qa/review.py`, lastes først når et kort åpnes i 3D.
import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

let state = null;

function setup(canvas) {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.1;
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x1b1e23);
  const camera = new THREE.PerspectiveCamera(60, 1, 0.02, 2000);
  scene.add(new THREE.HemisphereLight(0xdfe8f5, 0x3a332b, 1.4));
  const sun = new THREE.DirectionalLight(0xfff1dc, 1.6);
  sun.position.set(3, 6, 2);
  scene.add(sun);
  // Lykta følger kameraet, så rommene innenfra er lyse nok å se i.
  const lamp = new THREE.PointLight(0xffe2b8, 2.2, 0, 1.2);
  camera.add(lamp);
  scene.add(camera);
  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = true;
  controls.screenSpacePanning = true;
  controls.zoomToCursor = true;
  const loader = new GLTFLoader();
  const tex = new Map();
  const st = { renderer, scene, camera, controls, loader, tex, groups: [], raf: 0, canvas, lamp, mixer: null, clock: new THREE.Clock() };
  // Dobbeltklikk: dit du klikker, blir midten du roterer rundt (inn i et rom).
  canvas.addEventListener("dblclick", e => {
    const r = canvas.getBoundingClientRect();
    const p = new THREE.Vector2(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
    const ray = new THREE.Raycaster();
    ray.setFromCamera(p, camera);
    const hit = ray.intersectObjects(st.groups.filter(g => g.visible), true)[0];
    if (!hit) return;
    const d = camera.position.clone().sub(hit.point);
    const near = Math.min(d.length(), 3);
    controls.target.copy(hit.point);
    camera.position.copy(hit.point).add(d.setLength(near));
  });
  const loop = () => {
    st.raf = requestAnimationFrame(loop);
    const w = canvas.clientWidth, h = canvas.clientHeight;
    if (canvas.width !== Math.floor(w * renderer.getPixelRatio()) || canvas.height !== Math.floor(h * renderer.getPixelRatio())) {
      renderer.setSize(w, h, false);
      camera.aspect = w / Math.max(1, h);
      camera.updateProjectionMatrix();
    }
    controls.update();
    const dt = st.clock.getDelta();
    if (st.mixer) st.mixer.update(dt);
    renderer.render(scene, camera);
  };
  loop();
  return st;
}

function layerTexture(st, layers, name) {
  if (!layers[name]) return null;
  if (!st.tex.has(name)) {
    const t = new THREE.TextureLoader().load(layers[name].url);
    t.wrapS = t.wrapT = THREE.RepeatWrapping;
    t.colorSpace = THREE.SRGBColorSpace;
    t.anisotropy = st.renderer.capabilities.getMaxAnisotropy();
    const s = 1 / layers[name].size;
    t.repeat.set(s, s);
    st.tex.set(name, t);
  }
  return st.tex.get(name);
}

function dress(st, root, layers, sides) {
  root.traverse(o => {
    if (!o.isMesh) return;
    const mats = Array.isArray(o.material) ? o.material : [o.material];
    for (const m of mats) {
      const role = (m.userData && m.userData.fristad) || {};
      if (role.layer && !m.map) {
        const t = layerTexture(st, layers, role.layer);
        if (t) { m.map = t; m.needsUpdate = true; }
      }
      m.side = sides;
      if (role.kind === "lamp" || role.kind === "magic") { m.emissive = m.color.clone(); m.emissiveIntensity = 0.6; }
    }
  });
}

// Modellen er glTF i JSON med bufferen som base64 i `fristadBin` (siden
// får ikke laste data:- eller blob:-adresser): settes sammen til en glb i
// minnet, og bildene hentes som filer ved siden av.
async function fetchModel(url) {
  const j = await (await fetch(url)).json();
  const raw = atob(j.fristadBin || "");
  delete j.fristadBin;
  const bin = new Uint8Array(raw.length);
  for (let i = 0; i < raw.length; i++) bin[i] = raw.charCodeAt(i);
  if (j.buffers && j.buffers[0]) delete j.buffers[0].uri;
  let text = new TextEncoder().encode(JSON.stringify(j));
  const pad = (n) => (4 - (n % 4)) % 4;
  const jl = text.length + pad(text.length), bl = bin.length + pad(bin.length);
  const out = new Uint8Array(12 + 8 + jl + (bin.length ? 8 + bl : 0));
  const dv = new DataView(out.buffer);
  dv.setUint32(0, 0x46546c67, true); dv.setUint32(4, 2, true); dv.setUint32(8, out.length, true);
  dv.setUint32(12, jl, true); dv.setUint32(16, 0x4e4f534a, true);
  out.set(text, 20); out.fill(0x20, 20 + text.length, 20 + jl);
  if (bin.length) {
    dv.setUint32(20 + jl, bl, true); dv.setUint32(24 + jl, 0x004e4942, true);
    out.set(bin, 28 + jl);
  }
  return out.buffer;
}

async function load(st, url, layers, sides) {
  const data = await fetchModel(url);
  const g = await st.loader.parseAsync(data, url.slice(0, url.lastIndexOf("/") + 1));
  dress(st, g.scene, layers, sides);
  g.scene.userData.clips = g.animations || [];
  return g.scene;
}

function frame(st, obj, inside) {
  const box = new THREE.Box3().setFromObject(obj);
  const c = box.getCenter(new THREE.Vector3()), size = box.getSize(new THREE.Vector3());
  const r = Math.max(size.x, size.y, size.z) || 1;
  st.controls.target.copy(c);
  if (inside) {
    // Kryptbitene: fra innsida, i øyehøyde over gulvet midt i biten.
    st.camera.position.set(c.x + r * 0.25, box.min.y + 1.7, c.z + r * 0.25);
    st.controls.target.set(c.x, box.min.y + 1.7, c.z);
  } else {
    st.camera.position.copy(c).add(new THREE.Vector3(r * 0.9, r * 0.55, r * 1.1));
  }
  st.camera.near = Math.max(0.01, r / 2000);
  st.camera.far = r * 50 + 100;
  st.camera.updateProjectionMatrix();
}

// Åpner visningen for kortet `it` (fra data.json) i dialogen.
export async function show(it, data, ui) {
  if (!state) state = setup(ui.canvas);
  const st = state;
  for (const g of st.groups) st.scene.remove(g);
  st.groups = [];
  const layers = data.layers || {};
  const sides = ui.both.checked ? THREE.DoubleSide : THREE.FrontSide;
  ui.status.textContent = "laster …";
  const crypt = !!it.level;
  const after = await load(st, it.glb, layers, sides);
  after.name = "etter";
  const before = it.glb_before ? await load(st, it.glb_before, layers, sides) : null;
  if (before) { before.name = "før"; before.visible = false; }
  let level = null;
  if (crypt && data.levels && data.levels[it.level]) {
    level = new THREE.Group();
    level.name = "nivået";
    const urls = data.levels[it.level].filter(u => u !== it.glb);
    const parts = await Promise.all(urls.map(u => load(st, u, layers, sides)));
    for (const p of parts) level.add(p);
    level.visible = ui.whole.checked;
  }
  for (const g of [after, before, level]) if (g) { st.scene.add(g); st.groups.push(g); }
  // Biten du dømmer, lyser svakt når hele nivået står rundt den.
  after.traverse(o => { if (o.isMesh) for (const m of [].concat(o.material)) { m.userData.base = m.emissive ? m.emissive.clone() : null; } });
  const mark = on => after.traverse(o => {
    if (!o.isMesh) return;
    for (const m of [].concat(o.material)) if (m.emissive) { m.emissive.set(on ? 0x3a2a10 : 0x000000); }
  });
  mark(!!(level && level.visible));
  ui.whole.parentElement.hidden = !level;
  ui.which.hidden = !before;
  ui.which.value = "etter";
  ui.which.onchange = () => { after.visible = ui.which.value === "etter"; if (before) before.visible = ui.which.value === "før"; };
  ui.whole.onchange = () => { if (level) { level.visible = ui.whole.checked; mark(level.visible); } };
  ui.both.onchange = () => { for (const g of st.groups) g.traverse(o => { if (o.isMesh) for (const m of [].concat(o.material)) { m.side = ui.both.checked ? THREE.DoubleSide : THREE.FrontSide; m.needsUpdate = true; } }); };
  // Figurer med klipp (innbyggerne, fiendene): velg bevegelsen og se den
  // mens du roterer; «Hvilepose» står stille.
  st.mixer = null;
  const clips = after.userData.clips || [];
  if (ui.clip) {
    ui.clip.hidden = clips.length === 0;
    ui.clip.replaceChildren(new Option("Hvilepose", ""), ...clips.map(c => new Option(c.name, c.name)));
    const pick = clips.find(c => c.name === "idle") ? "idle" : "";
    ui.clip.value = pick;
    const play = name => {
      if (st.mixer) st.mixer.stopAllAction();
      st.mixer = null;
      const c = clips.find(c => c.name === name);
      if (!c) return;
      st.mixer = new THREE.AnimationMixer(after);
      st.mixer.clipAction(c).play();
    };
    ui.clip.onchange = () => play(ui.clip.value);
    play(pick);
  }
  frame(st, after, crypt);
  ui.status.textContent = "";
}
