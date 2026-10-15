// Filmene (docs/plan.md del 8), første utkast: filmet i den ekte byen og
// krypten med motorens filmmodus (`engine.film`, uten armer, kamp og
// fysikk). Manuset er data (`opening.json`): scenene, kamerabanen, lyset,
// været, musikken og teksten i middelalderskrift. Esc, mellomrom eller
// «Hopp over» hopper over; musa dreier kameraet noen grader.

const FADE = 0.8; // sekunder til og fra svart mellom scenene
const TEXT_IN = 1.2; // teksten kommer etter så mange sekunder
const LOOK = 3; // grader musa kan dreie kameraet

// Catmull-Rom gjennom nøklene, med myk start og slutt i hver scene.
function ease(t) {
  return t * t * (3 - 2 * t);
}
function catmull(p0, p1, p2, p3, t) {
  const t2 = t * t;
  const t3 = t2 * t;
  return 0.5 * (2 * p1 + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t2 + (-p0 + 3 * p1 - 3 * p2 + p3) * t3);
}

// Nøklene i verdens rom: høyde over bakken (`ground`) eller i et kryptnivå
// (`level`).
function resolve(engine, scene) {
  return scene.keys.map((k) => {
    let [x, y, z] = k.pos;
    if (k.level) {
      const o = engine.level_origin(k.level);
      if (o.length === 3) {
        x += o[0];
        y += o[1];
        z += o[2];
      }
    } else if (k.ground) {
      y += engine.ground(x, z);
    }
    return { at: k.at, v: [x, y, z, k.yaw, k.pitch, k.fov ?? 60] };
  });
}

function sample(keys, t) {
  const u = ease(Math.min(Math.max(t, 0), 1));
  let i = 0;
  while (i < keys.length - 2 && u > keys[i + 1].at) i++;
  const a = keys[i];
  const b = keys[Math.min(i + 1, keys.length - 1)];
  const span = Math.max(b.at - a.at, 1e-6);
  const f = Math.min(Math.max((u - a.at) / span, 0), 1);
  const p0 = keys[Math.max(i - 1, 0)].v;
  const p3 = keys[Math.min(i + 2, keys.length - 1)].v;
  return a.v.map((_, j) => catmull(p0[j], a.v[j], b.v[j], p3[j], f));
}

function el(tag, cls, parent, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text) e.textContent = text;
  parent?.appendChild(e);
  return e;
}

// Spiller filmen. `music.set(navn)` bytter musikken. Gir et løfte som
// innfris når filmen er ferdig eller hoppet over.
export async function playFilm({ engine, music, url = "film/opening.json" }) {
  let script;
  try {
    script = await (await fetch(url)).json();
  } catch (e) {
    console.warn("Fant ikke filmen:", e);
    return;
  }
  const scenes = script.scenes.map((s) => ({ ...s, path: resolve(engine, s) }));
  const root = el("div", "film", document.body);
  el("div", "film-bar film-bar--top", root);
  el("div", "film-bar film-bar--bottom", root);
  const black = el("div", "film-black", root);
  const words = el("div", "film-words", root);
  const heading = el("h2", "film-heading", words);
  const line = el("p", "film-text", words);
  const glitch = el("div", "film-glitch", root);
  const skip = el("button", "fs-button film-skip", root, "Hopp over");
  skip.dataset.variant = "ghost";
  document.body.classList.add("filming");

  let mouse = [0, 0];
  const onMove = (e) => {
    mouse = [(e.clientX / innerWidth) * 2 - 1, (e.clientY / innerHeight) * 2 - 1];
  };
  let done;
  const finished = new Promise((r) => (done = r));
  let over = false;
  const end = () => {
    if (over) return;
    over = true;
    done();
  };
  const onKey = (e) => {
    if (e.code === "Escape" || e.code === "Space") {
      e.preventDefault();
      end();
    }
  };
  addEventListener("mousemove", onMove);
  addEventListener("keydown", onKey, true);
  skip.addEventListener("click", end);

  const total = scenes.reduce((s, x) => s + x.dur, 0);
  let current = -1;
  const start = performance.now();
  await new Promise((resolveFrame) => {
    const tick = (now) => {
      const t = (now - start) / 1000;
      if (over || t >= total) {
        resolveFrame();
        return;
      }
      // Hvilken scene, og hvor langt i den.
      let k = 0;
      let at = t;
      while (k < scenes.length - 1 && at >= scenes[k].dur) {
        at -= scenes[k].dur;
        k++;
      }
      const s = scenes[k];
      if (k !== current) {
        current = k;
        engine.set_theme(s.daylight === "night");
        engine.set_weather(s.weather ?? "klart");
        if (s.music && s.music !== "none") music.set(s.music);
        else music.set?.("silence");
        heading.textContent = s.heading ?? "";
        heading.hidden = !s.heading;
        line.textContent = s.text ?? "";
        root.dataset.fx = s.fx ?? "";
      }
      const [x, y, z, yaw, pitch, fov] = sample(s.path, at / s.dur);
      engine.film(x, y, z, yaw - mouse[0] * LOOK, pitch - mouse[1] * LOOK, fov);
      // Til og fra svart; den siste scenen går rett over i spillet.
      const last = k === scenes.length - 1;
      const fadeIn = k === 0 ? Math.min(at / 1.5, 1) : Math.min(at / FADE, 1);
      const fadeOut = last ? 1 : Math.min((s.dur - at) / FADE, 1);
      black.style.opacity = String(1 - Math.min(fadeIn, fadeOut));
      // Teksten tones inn og ut.
      const ti = Math.min(Math.max((at - TEXT_IN) / 1.2, 0), 1);
      const to = Math.min(Math.max((s.dur - at - 0.6) / 1.0, 0), 1);
      words.style.opacity = String(Math.min(ti, to));
      // Glitchen vokser gjennom scenen.
      if (s.fx === "glitch") {
        const g = at / s.dur;
        const hit = Math.random() < 0.08 + g * 0.5;
        glitch.style.opacity = hit ? String(0.35 + g * 0.6) : "0";
        glitch.style.transform = hit ? `translate(${(Math.random() - 0.5) * 40 * g}px, ${(Math.random() - 0.5) * 12}px)` : "none";
        glitch.style.clipPath = hit ? `inset(${Math.random() * 80}% 0 ${Math.random() * 15}% 0)` : "none";
      } else {
        glitch.style.opacity = "0";
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
  end();
  await finished;
  removeEventListener("mousemove", onMove);
  removeEventListener("keydown", onKey, true);
  engine.film_end();
  engine.set_theme(false);
  engine.set_weather("klart");
  root.remove();
  document.body.classList.remove("filming");
}
