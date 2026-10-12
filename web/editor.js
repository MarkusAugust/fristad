// Redigeringsmodusen i nettleseren (docs/redigering.md, steg 4), bare for
// eieren: `?edit` i adressen. Motoren gjør jobben (velger, flytter, sjekker
// reglene, angrer); her er panelet med katalogen, det valgte og reglene, og
// tastene og musa.
//
// Musa er fri i redigeringen: venstre klikk velger det under pekeren eller
// tar tak i et håndtak (pilene og ringen rundt det valgte) og drar, høyre
// knapp holdt nede ser rundt. WASD, mellomrom og C flyr.

const STEP = 0.1;
const TURN = (15 * Math.PI) / 180;

const VERDICTS = { godkjent: "Godkjent", underkjent: "Underkjent", usikker: "Usikker" };

function esc(s) {
  return String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

export function createEditor({ engine, canvas, toast, onPlay, onLock, onEdit }) {
  // Redigerer nå (`active`), modellen som skal settes ut, og linjeverktøyet.
  let active = false;
  let placing = null;
  let lining = false;
  let looking = false;
  let pressed = false;
  let view = null;
  const catalog = JSON.parse(engine.catalog());

  const panel = document.createElement("aside");
  panel.id = "editor";
  panel.className = "editor";
  panel.hidden = true;
  panel.innerHTML = `
    <div class="editor-head">
      <h2 class="fs-heading" data-size="s">Redigering</h2>
      <span id="ed-state" class="fs-badge">Grønt</span>
    </div>
    <div class="editor-actions">
      <button id="ed-undo" class="fs-button" data-variant="secondary" type="button" title="Ctrl+Z">Angre</button>
      <button id="ed-redo" class="fs-button" data-variant="secondary" type="button" title="Ctrl+Shift+Z">Gjør om</button>
      <button id="ed-play" class="fs-button" data-variant="primary" type="button" title="P">Spill (P)</button>
      <button id="ed-save" class="fs-button" data-variant="secondary" type="button" disabled title="Lagring til GitHub kommer i steg 5">Lagre</button>
    </div>
    <section class="editor-section">
      <h3 class="fs-heading" data-size="xs">Valgt</h3>
      <p id="ed-selected" class="fs-paragraph" data-size="small">Ingenting. Klikk på noe i bildet.</p>
      <p id="ed-under" class="fs-paragraph" data-size="small" hidden></p>
      <div id="ed-wall" hidden>
        <label class="fs-label" for="ed-wall-h">Høyde på veggen (m)</label>
        <input id="ed-wall-h" class="fs-input" type="number" min="0.5" max="20" step="0.5" value="3">
      </div>
    </section>
    <section class="editor-section">
      <h3 class="fs-heading" data-size="xs">Reglene</h3>
      <ul id="ed-breaches" class="editor-breaches"></ul>
    </section>
    <section class="editor-section editor-catalog">
      <h3 class="fs-heading" data-size="xs">Katalogen</h3>
      <div class="editor-filters">
        <input id="ed-search" class="fs-input" type="search" placeholder="Søk" aria-label="Søk i katalogen">
        <select id="ed-verdict" class="fs-select" aria-label="Dom">
          <option value="">Alle</option>
          <option value="godkjent">Godkjent</option>
          <option value="underkjent">Underkjent</option>
          <option value="usikker">Usikker</option>
          <option value="none">Ikke vurdert</option>
        </select>
        <select id="ed-group" class="fs-select" aria-label="Kategori"><option value="">Alle kategorier</option></select>
      </div>
      <p id="ed-placing" class="fs-paragraph" data-size="small" hidden></p>
      <div id="ed-grid" class="editor-grid"></div>
    </section>
    <section class="editor-section">
      <h3 class="fs-heading" data-size="xs">Linje</h3>
      <p class="fs-paragraph" data-size="small"><kbd>L</kbd> begynner: klikk punkter på bakken, <kbd>Enter</kbd> gir en rad av modellen du har valgt i katalogen (gjerde), <kbd>V</kbd> en usynlig vegg, <kbd>Esc</kbd> avbryter.</p>
    </section>
    <section class="editor-section editor-keys">
      <p class="fs-paragraph" data-size="small">
        <kbd>WASD</kbd> fly, <kbd>Mellomrom</kbd>/<kbd>C</kbd> opp og ned, <kbd>Shift</kbd> fort, høyre knapp ser rundt.
        Piltastene flytter det valgte langs bakken, <kbd>PgUp</kbd>/<kbd>PgDn</kbd> opp og ned (<kbd>Alt</kbd> små steg, <kbd>Shift</kbd> store),
        <kbd>Q</kbd>/<kbd>E</kbd> dreier, <kbd>Tab</kbd> velger det neste under pekeren (kirka, så gjerdet inni boksen dens), <kbd>G</kbd> setter ned på det som er under, <kbd>Ctrl+D</kbd> kopierer, <kbd>Delete</kbd> sletter.
      </p>
    </section>`;
  document.body.append(panel);
  // I spillet: et lite merke om at P tar deg tilbake i redigeringen.
  const back = document.createElement("p");
  back.className = "editor-back fs-tag";
  back.hidden = true;
  back.innerHTML = "<kbd>P</kbd> tilbake til redigeringen";
  document.body.append(back);
  const $ = (id) => panel.querySelector(`#${id}`);

  // Gruppene i katalogen (mappene modellene ligger i).
  // Kategoriene i fast rekkefølge (husene først).
  const ORDER = ["Hus", "Gjenstander", "Møbler", "Gjerder", "Mur og porter", "Krypten"];
  const rank = (g) => (ORDER.indexOf(g) + 1 || 99);
  catalog.sort((a, b) => rank(a.group) - rank(b.group) || a.name.localeCompare(b.name));
  for (const g of [...new Set(catalog.map((m) => m.group))].sort((a, b) => rank(a) - rank(b))) {
    const o = document.createElement("option");
    o.value = g;
    o.textContent = g;
    $("ed-group").append(o);
  }

  function renderCatalog() {
    const q = $("ed-search").value.trim().toLowerCase();
    const v = $("ed-verdict").value;
    const g = $("ed-group").value;
    const shown = catalog.filter((m) => (!q || m.name.includes(q)) && (!g || m.group === g) && (!v || (v === "none" ? !m.verdict : m.verdict === v)));
    $("ed-grid").innerHTML = shown
      .map(
        (m) => `<button type="button" class="editor-item${placing === m.name ? " is-active" : ""}" data-model="${esc(m.name)}" title="${esc(m.why || m.name)}">
          <img src="catalog/${esc(m.name)}.jpg" alt="" loading="lazy" onerror="this.style.visibility='hidden'">
          <span class="editor-item-name">${esc(m.name)}</span>
          <span class="fs-badge" data-variant="${m.verdict === "godkjent" ? "success" : m.verdict === "underkjent" ? "danger" : "warning"}">${esc(VERDICTS[m.verdict] ?? "Ikke vurdert")}</span>
        </button>`,
      )
      .join("");
  }
  for (const id of ["ed-search", "ed-verdict", "ed-group"]) $(id).addEventListener("input", renderCatalog);
  $("ed-grid").addEventListener("click", (e) => {
    const b = e.target.closest("[data-model]");
    if (!b) return;
    setPlacing(placing === b.dataset.model ? null : b.dataset.model);
  });

  function setPlacing(model) {
    placing = model;
    const p = $("ed-placing");
    p.hidden = !model;
    p.textContent = model ? `Klikk i bildet for å sette ut ${model} (Shift: flere). Esc avbryter.` : "";
    renderCatalog();
  }

  function send(cmd) {
    return engine.edit(JSON.stringify(cmd));
  }

  $("ed-undo").addEventListener("click", () => send({ cmd: "undo" }));
  $("ed-redo").addEventListener("click", () => send({ cmd: "redo" }));
  $("ed-play").addEventListener("click", () => play());
  $("ed-wall-h").addEventListener("change", (e) => send({ cmd: "wall_height", h: Number(e.target.value) || 3 }));

  // Hendelsen fra motoren etter hver endring.
  function update(v) {
    view = v;
    if (!v) return;
    const st = $("ed-state");
    st.textContent = v.clean ? (v.dirty ? "Grønt, endret" : "Grønt") : `${v.breaches.length} brudd`;
    st.dataset.variant = v.clean ? "success" : "danger";
    $("ed-undo").disabled = !v.can_undo;
    $("ed-redo").disabled = !v.can_redo;
    const sel = v.selected;
    $("ed-selected").textContent = sel.length ? sel.map((s) => s.label).join(", ") : "Ingenting. Klikk på noe i bildet.";
    // Det som lå under pekeren: klikk igjen eller Tab for det neste.
    const under = v.under ?? [];
    $("ed-under").hidden = under.length < 2;
    $("ed-under").innerHTML = `Under pekeren: ${under.map((u) => (u.selected ? `<strong>${esc(u.label)}</strong>` : esc(u.label))).join(" › ")}. Klikk igjen eller <kbd>Tab</kbd> for neste.`;
    $("ed-wall").hidden = !sel.some((s) => s.label === "usynlig vegg");
    const list = $("ed-breaches");
    list.innerHTML = v.breaches.length
      ? v.breaches
          .slice(0, 30)
          .map((b) => `<li class="fs-paragraph" data-size="small">${esc(b.why)}</li>`)
          .join("") + (v.breaches.length > 30 ? `<li class="fs-paragraph" data-size="small">… og ${v.breaches.length - 30} til</li>` : "")
      : `<li class="fs-paragraph" data-size="small">Alt er grønt.</li>`;
  }

  function begin() {
    if (!send({ cmd: "enter" })) return false;
    active = true;
    panel.hidden = false;
    back.hidden = true;
    document.body.classList.add("editing");
    if (document.pointerLockElement) document.exitPointerLock?.();
    renderCatalog();
    onEdit?.(true);
    return true;
  }

  // P: spill det du har laget (spillet bygges på nytt fra dataene).
  function play() {
    if (!active) return;
    // Musa fanges mens tastetrykket ennå teller som en handling fra deg.
    onLock?.();
    toast?.show?.("Bygger byen på nytt …");
    setTimeout(() => {
      if (!engine.play()) {
        toast?.show?.("Kunne ikke bygge byen på nytt.");
        return;
      }
      active = false;
      lining = false;
      setPlacing(null);
      panel.hidden = true;
      back.hidden = false;
      document.body.classList.remove("editing");
      onPlay?.();
    }, 30);
  }

  // Pekeren i 0..1 fra øverst til venstre i bildet.
  function at(e) {
    const r = canvas.getBoundingClientRect();
    return { x: (e.clientX - r.left) / r.width, y: (e.clientY - r.top) / r.height };
  }

  function onCanvas(e) {
    return e.target === canvas;
  }

  // Musa: gir true når redigeringen tok den.
  function mouseDown(e) {
    if (!active) return false;
    if (!onCanvas(e)) return true;
    e.preventDefault();
    if (e.button === 2) {
      looking = true;
      return true;
    }
    if (e.button !== 0) return true;
    const p = at(e);
    if (lining) {
      send({ cmd: "line_point", ...p });
      return true;
    }
    if (placing) {
      send({ cmd: "place", model: placing, ...p });
      if (!e.shiftKey) setPlacing(null);
      return true;
    }
    pressed = true;
    send({ cmd: "press", ...p, add: e.shiftKey });
    return true;
  }

  function mouseMove(e) {
    if (!active) return false;
    if (looking) engine.look(e.movementX, e.movementY);
    else if (pressed) send({ cmd: "drag", ...at(e) });
    return true;
  }

  function mouseUp(e) {
    if (!active) return false;
    if (e.button === 2) looking = false;
    if (e.button === 0 && pressed) {
      pressed = false;
      send({ cmd: "release" });
    }
    return true;
  }

  // Det valgte langs bakken etter hvor du ser (piltastene).
  function along(forward, right) {
    const yaw = engine.view()[3];
    const f = { x: -Math.sin(yaw), z: -Math.cos(yaw) };
    const r = { x: -f.z, z: f.x };
    return { x: f.x * forward + r.x * right, z: f.z * forward + r.z * right };
  }

  // Tastene: gir true når redigeringen tok tasten (resten går til motoren:
  // flyging med WASD, mellomrom og C).
  function key(e) {
    if (!active) {
      if (e.code === "KeyP" && !e.repeat && engine.can_edit()) {
        e.preventDefault();
        begin();
        return true;
      }
      return false;
    }
    if (e.target instanceof Element && e.target.closest("input, select, textarea")) return true;
    const step = e.altKey ? STEP / 10 : e.shiftKey ? STEP * 10 : STEP;
    const turn = e.altKey ? TURN / 15 : TURN;
    const ctrl = e.ctrlKey || e.metaKey;
    const move = (x, y, z) => send({ cmd: "move", x, y, z });
    switch (e.code) {
      case "ArrowUp":
      case "ArrowDown":
      case "ArrowLeft":
      case "ArrowRight": {
        e.preventDefault();
        const f = e.code === "ArrowUp" ? 1 : e.code === "ArrowDown" ? -1 : 0;
        const r = e.code === "ArrowRight" ? 1 : e.code === "ArrowLeft" ? -1 : 0;
        const d = along(f * step, r * step);
        move(d.x, 0, d.z);
        return true;
      }
      case "PageUp":
      case "PageDown":
        e.preventDefault();
        move(0, e.code === "PageUp" ? step : -step, 0);
        return true;
      case "KeyQ":
      case "KeyE":
        if (e.repeat && !e.altKey) return true;
        send({ cmd: "rotate", a: e.code === "KeyQ" ? turn : -turn });
        return true;
      case "KeyG":
        if (!e.repeat) send({ cmd: "drop" });
        return true;
      case "KeyD":
        if (ctrl) {
          e.preventDefault();
          if (!e.repeat) send({ cmd: "copy" });
          return true;
        }
        return false;
      case "Delete":
      case "Backspace":
        e.preventDefault();
        if (!e.repeat) send({ cmd: "delete" });
        return true;
      case "KeyZ":
        if (ctrl) {
          e.preventDefault();
          send({ cmd: e.shiftKey ? "redo" : "undo" });
          return true;
        }
        return false;
      case "KeyY":
        if (ctrl) {
          e.preventDefault();
          send({ cmd: "redo" });
          return true;
        }
        return false;
      case "KeyL":
        if (!e.repeat) {
          lining = true;
          send({ cmd: "line_cancel" });
          toast?.show?.("Linje: klikk punkter på bakken. Enter gir en rad av modellen du har valgt i katalogen, V en usynlig vegg, Esc avbryter.");
        }
        return true;
      case "Enter":
        if (lining) {
          e.preventDefault();
          if (!placing) {
            toast?.show?.("Velg en modell i katalogen (et gjerde) først, eller trykk V for en usynlig vegg.");
            return true;
          }
          send({ cmd: "line_finish", kind: "row", model: placing });
          lining = false;
          setPlacing(null);
        }
        return true;
      case "KeyV":
        if (lining) {
          send({ cmd: "line_finish", kind: "wall", height: Number($("ed-wall-h").value) || 3 });
          lining = false;
          return true;
        }
        return false;
      case "Escape":
        if (lining || placing) {
          e.preventDefault();
          lining = false;
          send({ cmd: "line_cancel" });
          setPlacing(null);
          return true;
        }
        return false;
      case "Tab":
        e.preventDefault();
        if (!e.repeat) send({ cmd: "cycle" });
        return true;
      case "KeyP":
        if (!e.repeat) play();
        return true;
      default:
        return false;
    }
  }

  return {
    get active() {
      return active;
    },
    begin,
    update,
    key,
    mouseDown,
    mouseMove,
    mouseUp,
  };
}
