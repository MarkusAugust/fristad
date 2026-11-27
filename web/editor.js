// Redigeringsmodusen i nettleseren (docs/redigering.md, steg 4), bare for
// eieren: `?edit` i adressen. Motoren gjør jobben (velger, flytter, sjekker
// reglene, angrer); her er panelet med katalogen, det valgte og reglene, og
// tastene og musa.
//
// Musa er fri i redigeringen: venstre klikk velger det under pekeren, og
// dra flytter det langs bakken; med Alt holdt inne heiser draget det valgte
// opp og ned, med Ctrl (Cmd på Mac) dreier det (flere valgt dreies samlet).
// Pilene og ringen rundt det valgte virker fortsatt. Høyre knapp holdt nede
// ser rundt. WASD, mellomrom og C flyr.
//
// Dra en modell fra katalogen ut i bildet: et fantom følger pekeren (grønn
// boks når alt er i orden, rød når en regel brytes), R dreier det 90°, og
// slipp setter det ned på det som er under. «Fest» (eller Ctrl holdt) setter
// på rutenettet (0,5 m) og vinkelen (15°). Det som ligger under pekeren
// står som en liste å klikke i (Tab og Shift+Tab blar, Alt+klikk tar det
// bakerste), Shift+dra i bildet velger alt innenfor rammen, panelet viser
// egenskapene til det valgte å skrive i, og historikken er en liste å
// klikke seg tilbake i. Shift+1–9 (eller Ctrl+1–9) husker hvor kameraet står, 1–9 går dit.
//
// Det du gjør, huskes som et utkast i nettleseren (`fristad.edit-draft`)
// ved hver endring og hvert 30. sekund, og Lagre (Ctrl+S) legger det på
// grenen `redigering`, én commit per økt; uten nøkkel lastes filene ned som
// zip. Neste gang fortsetter du fra utkastet, eller fra det du lagret sist.

import { catalogItem, deviation, esc, historyItems, makePanel, propField } from "./editor-panel.js";
import { createSaving } from "./editor-save.js";

const MARKS = "fristad.edit-marks";
// Så langt (px) musa må gå før et trykk blir et drag (katalogen, rammen).
const SLOP = 5;

const STEP = 0.1;
const TURN = (15 * Math.PI) / 180;

export function createEditor({ engine, canvas, toast, onPlay, onLock, onEdit }) {
  // Redigerer nå (`active`), modellen som skal settes ut, og linjeverktøyet.
  let active = false;
  let placing = null;
  let lining = false;
  let looking = false;
  let pressed = false;
  let dragAt = null;
  let dragFrame = 0;
  let view = null;
  // Fest til rutenett og vinkel: bryteren i panelet, eller Ctrl holdt.
  let snapToggle = false;
  let ctrlHeld = false;
  let snapSent = null;
  // Dra fra katalogen: modellen og der trykket begynte.
  let dragModel = null;
  let ghostShown = false;
  let ghostFrame = 0;
  let ghostAt = null;
  // Shift+trykk i bildet: et klikk (legg til eller ta bort) eller en ramme.
  let boxFrom = null;
  let boxing = false;
  const catalog = JSON.parse(engine.catalog());

  const panel = makePanel();
  // Rammen når Shift+dra velger.
  const box = document.createElement("div");
  box.className = "editor-box";
  box.hidden = true;
  document.body.append(box);
  // I spillet: et lite merke om at P tar deg tilbake i redigeringen.
  const back = document.createElement("p");
  back.className = "editor-back lt-tag";
  back.hidden = true;
  back.innerHTML = "<kbd>P</kbd> tilbake til redigeringen";
  document.body.append(back);
  const $ = (id) => panel.querySelector(`#${id}`);

  // Gruppene i katalogen (mappene modellene ligger i).
  // Kategoriene i fast rekkefølge (husene først).
  const ORDER = ["Hus", "Gjenstander", "Møbler", "Gjerder", "Mur og porter", "Natur"];
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
    const odd = $("ed-odd").checked;
    const g = $("ed-group").value;
    const shown = catalog.filter((m) => (!q || m.name.includes(q)) && (!g || m.group === g) && (!odd || deviation(m)));
    $("ed-grid").innerHTML = shown.map((m) => catalogItem(m, placing === m.name)).join("");
  }
  for (const id of ["ed-search", "ed-odd", "ed-group"]) $(id).addEventListener("input", renderCatalog);
  // Et klikk velger modellen å sette ut; et drag tar den rett ut i bildet.
  $("ed-grid").addEventListener("mousedown", (e) => {
    const b = e.target.closest("[data-model]");
    if (!b || e.button !== 0) return;
    e.preventDefault();
    dragModel = { name: b.dataset.model, x: e.clientX, y: e.clientY, moved: false };
  });

  function setPlacing(model) {
    placing = model;
    const p = $("ed-placing");
    p.hidden = !model;
    p.textContent = model ? `Klikk i bildet for å sette ut ${model} (Shift: flere, R dreier). Esc avbryter.` : "";
    if (!model) hideGhost();
    renderCatalog();
  }

  function send(cmd) {
    return engine.edit(JSON.stringify(cmd));
  }

  // Fantomet følger pekeren (høyst én gang per bilde).
  function ghost(model, p) {
    ghostAt = { model, ...p };
    if (ghostFrame) return;
    ghostFrame = requestAnimationFrame(() => {
      ghostFrame = 0;
      if (!ghostAt) return;
      send({ cmd: "ghost", model: ghostAt.model, x: ghostAt.x, y: ghostAt.y });
      ghostShown = true;
    });
  }
  function hideGhost() {
    ghostAt = null;
    if (ghostShown) send({ cmd: "ghost" });
    ghostShown = false;
  }
  const ghostModel = () => (dragModel?.moved ? dragModel.name : placing);

  // Draget følger pekeren (høyst én gang per bilde: musa melder seg oftere
  // enn bildet tegnes). Det siste punktet sendes før knappen slippes.
  function drag(p) {
    dragAt = p;
    if (!dragFrame) dragFrame = requestAnimationFrame(sendDrag);
  }
  function sendDrag() {
    cancelAnimationFrame(dragFrame);
    dragFrame = 0;
    if (dragAt) send({ cmd: "drag", ...dragAt });
    dragAt = null;
  }

  // Festet: bryteren, eller Ctrl holdt (motoren får det bare når det endres).
  function sendSnap() {
    const on = snapToggle !== ctrlHeld;
    if (on === snapSent) return;
    snapSent = on;
    send({ cmd: "snap", on });
    if (ghostAt) ghost(ghostAt.model, ghostAt);
  }
  $("ed-snap").addEventListener("change", (e) => {
    snapToggle = e.target.checked;
    sendSnap();
  });

  // Kamerabokmerkene (Shift+1–9 eller Ctrl+1–9 husker, 1–9 går dit), i nettleseren.
  function readMarks() {
    try {
      return JSON.parse(localStorage.getItem(MARKS) || "{}");
    } catch {
      return {};
    }
  }
  function bookmark(n, store) {
    const marks = readMarks();
    if (store) {
      marks[n] = engine.view();
      try {
        localStorage.setItem(MARKS, JSON.stringify(marks));
      } catch {
        /* uten lagring huskes det ikke */
      }
      toast?.show?.(`Kameraet er husket på ${n}.`);
      return;
    }
    const v = marks[n];
    if (!v) {
      toast?.show?.(`Ingenting på ${n} ennå. Ctrl+${n} husker der du står.`);
      return;
    }
    engine.place(v[0], v[1], v[2], (v[3] * 180) / Math.PI, (v[4] * 180) / Math.PI);
  }

  // Lagringen og utkastet (web/editor-save.js).
  const saving = createSaving({ engine, toast, status: $("ed-saved"), getView: () => view, isActive: () => active });
  $("ed-save").addEventListener("click", () => saving.openSave(false));

  const makeGroup = () => send({ cmd: "group", name: $("ed-grp-name").value.trim() });
  const freeGroup = () => send({ cmd: "ungroup" });
  $("ed-grp-make").addEventListener("click", makeGroup);
  $("ed-grp-free").addEventListener("click", freeGroup);
  $("ed-undo").addEventListener("click", () => send({ cmd: "undo" }));
  $("ed-redo").addEventListener("click", () => send({ cmd: "redo" }));
  $("ed-play").addEventListener("click", () => play());
  $("ed-wall-h").addEventListener("change", (e) => send({ cmd: "wall_height", h: Number(e.target.value) || 3 }));
  $("ed-under").addEventListener("click", (e) => {
    const b = e.target.closest("[data-k]");
    if (b) send({ cmd: "select_under", k: Number(b.dataset.k), add: e.shiftKey });
  });
  $("ed-history").addEventListener("click", (e) => {
    const b = e.target.closest("[data-n]");
    if (b) send({ cmd: "undo_to", n: Number(b.dataset.n) });
  });
  // Egenskapene: Enter eller å gå ut av feltet setter verdien (ett steg å
  // angre). Fargevelgeren skriver i tekstfeltet ved siden av.
  function setProp(input) {
    const key = input.name;
    const value = input.type === "checkbox" ? (input.checked ? "ja" : "nei") : input.value;
    if (input.dataset.was === value) return;
    input.dataset.was = value;
    send({ cmd: "set_prop", key, value });
  }
  $("ed-props").addEventListener("submit", (e) => e.preventDefault());
  $("ed-props").addEventListener("change", (e) => {
    const t = e.target;
    if (t.dataset.pick) {
      const text = $("ed-props").querySelector(`[name="${t.dataset.pick}"]`);
      text.value = t.value;
      setProp(text);
    } else if (t.name) setProp(t);
  });
  $("ed-props").addEventListener("keydown", (e) => {
    if (e.key === "Enter" && e.target.tagName === "INPUT") {
      e.preventDefault();
      setProp(e.target);
    }
  });

  // Hendelsen fra motoren etter hver endring.
  function update(v) {
    view = v;
    if (!v) return;
    const st = $("ed-state");
    st.textContent = v.clean ? (v.dirty ? "Grønt, endret" : "Grønt") : `${v.breaches.length} brudd`;
    st.dataset.variant = v.clean ? "success" : "danger";
    $("ed-undo").disabled = !v.can_undo;
    $("ed-redo").disabled = !v.can_redo;
    $("ed-save").disabled = !v.dirty;
    $("ed-save").title = v.clean ? "Lagre på grenen redigering (Ctrl+S)" : "Rett reglene som er brutt før du lagrer";
    saving.draftLater();
    const sel = v.selected;
    $("ed-selected").textContent = sel.length ? sel.map((s) => s.label).join(", ") : "Ingenting. Klikk på noe i bildet.";
    // Det som lå under pekeren, nærmest først: klikk for å velge.
    const under = v.under ?? [];
    $("ed-under-box").hidden = under.length < 2;
    $("ed-under").innerHTML = under
      .map((u, k) => `<li><button type="button" class="editor-under-item${u.selected ? " is-active" : ""}" data-k="${k}">${esc(u.label)}</button></li>`)
      .join("");
    renderProps(v.props ?? []);
    renderHistory(v.history ?? [], v.step ?? 0);
    if (v.error) toast?.show?.(v.error);
    if (ghostModel()) {
      const p = $("ed-placing");
      p.hidden = false;
      const g = v.ghost;
      p.textContent = !g ? `Dra ${ghostModel()} ut i bildet.` : g.length ? `${ghostModel()}: ${g.join(", ")}.` : `${ghostModel()}: grønt. Slipp for å sette ut (R dreier).`;
      p.dataset.state = g && g.length ? "bad" : "ok";
    }
    if (typeof v.snap === "boolean") snapSent = v.snap;
    $("ed-wall").hidden = !sel.some((s) => s.label === "usynlig vegg");
    // Gruppen: navnet når én gruppe er valgt, og knappene.
    const grp = $("ed-grp");
    grp.hidden = !v.group;
    grp.textContent = v.group ? `Gruppen «${v.group}» (${sel.length} ting).` : "";
    $("ed-grp-make").disabled = sel.length < 2;
    $("ed-grp-free").disabled = !(v.props ?? []).some((p) => p.key === "group" && p.value) && !v.group;
    // Ruta til en valgt innbygger.
    const route = (v.props ?? []).filter((p) => p.key.startsWith("route.")).length;
    const rt = $("ed-route");
    rt.hidden = !route;
    rt.textContent =
      v.route_point != null
        ? `Punkt ${v.route_point + 1} av ${route} på ruta er tatt: dra det langs bakken, Delete sletter det.`
        : `Ruta har ${route} punkter (stolpene): dra dem, klikk på streken for et nytt punkt.`;
    const list = $("ed-breaches");
    list.innerHTML = v.breaches.length
      ? v.breaches
          .slice(0, 30)
          .map((b) => `<li class="lt-paragraph" data-size="small">${esc(b.why)}</li>`)
          .join("") + (v.breaches.length > 30 ? `<li class="lt-paragraph" data-size="small">… og ${v.breaches.length - 30} til</li>` : "")
      : `<li class="lt-paragraph" data-size="small">Alt er grønt.</li>`;
  }

  // Egenskapene til det valgte: et felt per egenskap (tekst, tall, farge).
  // Panelet tegnes bare på nytt når noe er endret, så feltet du skriver i
  // ikke mister fokus.
  let propsKey = "";
  function renderProps(props) {
    const key = JSON.stringify(props);
    if (key === propsKey) return;
    propsKey = key;
    $("ed-props-box").hidden = !props.length;
    $("ed-props").innerHTML = props.map((p) => `<label class="editor-prop"><span class="lt-label">${esc(p.label)}</span>${propField(p)}</label>`).join("");
    // Teksten i et tekstfelt settes når du går ut av det.
    for (const t of $("ed-props").querySelectorAll("textarea")) t.addEventListener("blur", () => setProp(t));
  }

  // Historikken: «Start» og hvert steg; det du står på er markert, og det
  // som kan gjøres om, er dempet. Klikk går dit.
  function renderHistory(steps, at) {
    const list = $("ed-history");
    list.innerHTML = historyItems(steps, at);
    list.querySelector(".is-active")?.scrollIntoView?.({ block: "nearest" });
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
    saving.restore();
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
    // Ctrl+klikk på Mac kommer som høyre knapp: det er dreiingen.
    const turn = e.ctrlKey || e.metaKey;
    if (e.button === 2 && !turn) {
      looking = true;
      return true;
    }
    if (e.button !== 0 && e.button !== 2) return true;
    const p = at(e);
    if (lining) {
      send({ cmd: "line_point", ...p });
      return true;
    }
    if (placing) {
      send({ cmd: "place", model: placing, ...p });
      if (!e.shiftKey) setPlacing(null);
      else ghost(placing, p);
      return true;
    }
    // Shift: et klikk legger til eller tar bort, et drag er en ramme.
    if (e.shiftKey && e.button === 0 && !turn && !e.altKey) {
      boxFrom = { x: e.clientX, y: e.clientY, p };
      boxing = false;
      return true;
    }
    pressed = true;
    send({ cmd: "press", ...p, add: e.shiftKey, lift: e.altKey && !turn, turn });
    canvas.style.cursor = turn ? "ew-resize" : e.altKey ? "ns-resize" : "grabbing";
    return true;
  }

  // Pekeren sier hva et drag vil gjøre: Alt heiser, Ctrl eller Cmd dreier.
  function hint(e) {
    if (!active || pressed) return;
    canvas.style.cursor = e.ctrlKey || e.metaKey ? "ew-resize" : e.altKey ? "ns-resize" : "";
  }

  function drawBox(e) {
    const x0 = Math.min(boxFrom.x, e.clientX);
    const y0 = Math.min(boxFrom.y, e.clientY);
    Object.assign(box.style, { left: `${x0}px`, top: `${y0}px`, width: `${Math.abs(e.clientX - boxFrom.x)}px`, height: `${Math.abs(e.clientY - boxFrom.y)}px` });
    box.hidden = false;
  }

  function mouseMove(e) {
    if (!active) return false;
    if (dragModel && !dragModel.moved && Math.hypot(e.clientX - dragModel.x, e.clientY - dragModel.y) > SLOP) {
      dragModel.moved = true;
      document.body.classList.add("editor-dragging");
    }
    if (looking) engine.look(e.movementX, e.movementY);
    else if (boxFrom) {
      if (!boxing && Math.hypot(e.clientX - boxFrom.x, e.clientY - boxFrom.y) > SLOP) boxing = true;
      if (boxing) drawBox(e);
    } else if (pressed) drag(at(e));
    else {
      const model = ghostModel();
      if (model && onCanvas(e)) ghost(model, at(e));
      else if (model) hideGhost();
      hint(e);
    }
    return true;
  }

  function mouseUp(e) {
    if (!active) return false;
    if (dragModel) {
      const d = dragModel;
      dragModel = null;
      document.body.classList.remove("editor-dragging");
      if (!d.moved) {
        setPlacing(placing === d.name ? null : d.name);
      } else if (onCanvas(e)) {
        // Slipp: settes ned på det som er under, der fantomet sto.
        send({ cmd: "place", model: d.name, ...at(e) });
        hideGhost();
        setPlacing(placing);
      } else {
        hideGhost();
        setPlacing(placing);
      }
      return true;
    }
    if (e.button === 2 && looking) looking = false;
    else if (boxFrom) {
      const from = boxFrom;
      boxFrom = null;
      box.hidden = true;
      if (boxing) {
        const to = at(e);
        send({ cmd: "select_box", x0: from.p.x, y0: from.p.y, x1: to.x, y1: to.y, add: true });
      } else {
        send({ cmd: "press", ...from.p, add: true, lift: false, turn: false });
        send({ cmd: "release" });
      }
      boxing = false;
    } else if (pressed) {
      pressed = false;
      sendDrag();
      send({ cmd: "release" });
      hint(e);
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
      case "AltLeft":
      case "AltRight":
      case "ControlLeft":
      case "ControlRight":
      case "MetaLeft":
      case "MetaRight":
        // Alt flytter ellers fokus til menylinja i noen nettlesere.
        if (e.code.startsWith("Alt")) e.preventDefault();
        else if (!ctrlHeld) {
          ctrlHeld = true;
          sendSnap();
        }
        hint(e);
        return true;
      case "KeyR":
        if (e.repeat || ctrl) return !ctrl;
        // En kvart runde: fantomet før det settes ut, ellers det valgte.
        if (ghostModel()) send({ cmd: "ghost_turn", a: e.shiftKey ? Math.PI / 2 : -Math.PI / 2 });
        else send({ cmd: "rotate", a: e.shiftKey ? Math.PI / 2 : -Math.PI / 2 });
        return true;
      case "Digit1":
      case "Digit2":
      case "Digit3":
      case "Digit4":
      case "Digit5":
      case "Digit6":
      case "Digit7":
      case "Digit8":
      case "Digit9":
        e.preventDefault();
        // Shift+1–9 husker også (⌘+tall på Mac bytter fane i nettleseren).
        if (!e.repeat) bookmark(e.code.slice(5), ctrl || e.shiftKey);
        return true;
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
      // + og − løfter og senker (som PageUp/PageDown, som små tastaturer mangler).
      case "Minus":
      case "Equal":
      case "Slash":
      case "NumpadAdd":
      case "NumpadSubtract":
        // (⌥ på Mac gir ± og –.)
        if (!["+", "±", "-", "–"].includes(e.key)) return false;
        e.preventDefault();
        move(0, e.key === "+" || e.key === "±" ? step : -step, 0);
        return true;
      case "KeyQ":
      case "KeyE":
        if (e.repeat && !e.altKey) return true;
        send({ cmd: "rotate", a: e.code === "KeyQ" ? turn : -turn });
        return true;
      case "KeyG":
        if (ctrl) {
          e.preventDefault();
          if (!e.repeat) e.shiftKey ? freeGroup() : makeGroup();
          return true;
        }
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
      case "KeyS":
        if (ctrl) {
          e.preventDefault();
          saving.openSave(true);
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
        if (lining || placing || dragModel) {
          e.preventDefault();
          dragModel = null;
          document.body.classList.remove("editor-dragging");
          lining = false;
          send({ cmd: "line_cancel" });
          setPlacing(null);
          return true;
        }
        return false;
      case "Tab":
        e.preventDefault();
        if (!e.repeat) send({ cmd: e.shiftKey ? "cycle_back" : "cycle" });
        return true;
      case "KeyP":
        if (!e.repeat) play();
        return true;
      default:
        return false;
    }
  }

  document.addEventListener("keyup", (e) => {
    if (active && ctrlHeld && !e.ctrlKey && !e.metaKey) {
      ctrlHeld = false;
      sendSnap();
    }
    hint(e);
  });
  // Fanen mister fokus med Ctrl nede: slipp den.
  window.addEventListener("blur", () => {
    if (ctrlHeld) {
      ctrlHeld = false;
      if (active) sendSnap();
    }
  });

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
