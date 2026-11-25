// Det som står over bildet: stolpene, målet og journalen, funnet fra
// kista, lappene, fiendekortet, tastene, snakkeboblene og merkene over
// fiendene, og toningen, virvelen og den røde buen der et slag kom fra.

import { readDoc } from "./book.js";
import { lastInv } from "./inventory.js";
import { compTitle } from "./menus.js";
import { $, clickHint, docs, engine, enter, esc, locked, setDragging, setPlaying, sfx } from "./shared.js";

// Treffet: en rød bue i den kanten av bildet slaget eller skuddet kom fra
// (`Event::Hurt`: vinkelen fra blikket, 0 foran = øverst, positiv til høyre,
// ±π bak = nederst). Hver bue toner ut for seg (`.hurt-arc` i style.css).
export function hurtArc(angle) {
  const arcs = document.querySelectorAll(".hurt-arc");
  if (arcs.length >= 4) arcs[0].remove();
  const el = document.createElement("div");
  el.className = "hurt-arc";
  el.setAttribute("aria-hidden", "true");
  el.style.setProperty("--from", `${(angle * 180) / Math.PI}deg`);
  el.addEventListener("animationend", () => el.remove());
  document.body.appendChild(el);
}

// Stolpene over knappelinja: liv og energi med tall (maks fra utstyret).
export function setBars(hp, stamina) {
  const maxHp = lastInv?.max_hp ?? 100;
  const maxSt = lastInv?.max_stamina ?? 100;
  $("hp").max = maxHp;
  $("hp").value = hp;
  $("hp-text").textContent = `${hp} / ${maxHp}`;
  $("stamina").max = maxSt;
  $("stamina").value = stamina;
  $("stamina-text").textContent = `${stamina} / ${maxSt}`;
}
// Ned i krypten og opp igjen: skjermen går i svart et øyeblikk.
export function fade() {
  const el = $("fade");
  el.classList.remove("on");
  void el.offsetWidth;
  el.classList.add("on");
}

// Målet nå (én linje i panelet) og journalen (J) med det du har lest.
// Målet endres når du går ned, slår bossen, finner noe eller leser en lapp,
// så spillet lagres da også.
export function setGoal(g) {
  const goal = $("goal");
  goal.textContent = g.text;
  goal.hidden = !g.text;
  $("journal-goal").textContent = g.text;
  const lore = $("journal-lore");
  lore.replaceChildren();
  if (!g.journal.length) {
    const p = document.createElement("p");
    p.textContent = "Du har ikke lest noe ennå. Lappene og bøkene i krypten havner her.";
    lore.append(p);
  }
  for (const j of g.journal) {
    const h = document.createElement("h3");
    h.className = "lt-heading";
    h.dataset.size = "xs";
    h.textContent = j.title;
    const p = document.createElement("p");
    p.className = "lt-paragraph";
    p.textContent = j.text;
    lore.append(h, p);
  }
  try {
    localStorage.setItem("fristad.save", engine.save_state());
  } catch {
    /* lagring er en bekvemmelighet */
  }
}
export function toggleJournal(on = $("journal").hidden) {
  $("journal").hidden = !on;
}
// Gjennom portalen: en virvel over skjermen (synsfeltet strekkes i motoren).
export function warp() {
  const el = $("warp");
  el.classList.remove("on");
  void el.offsetWidth;
  el.classList.add("on");
}

// Kista: en LO-TEK-dialog som sier hva du fant, og lar deg lese om
// komponenten i boka (panelet til høyre). Spillet står på pause så lenge.
export let lootOpen = false;
// Det som fulgte med i kista utenom kistas egen komponent (`haul`, plan 14).
export let lootMore = [];
export function openLoot(component) {
  lootOpen = true;
  setPlaying(false);
  setDragging(false);
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const d = docs[component] ?? {};
  const title = d.title ?? component;
  const more = lootMore;
  lootMore = [];
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog loot";
  dlg.setAttribute("aria-labelledby", "loot-title");
  const found = component
    ? `<p>Under lokket, mellom råtne filler og knokler, ligger <strong>${esc(title)}</strong>: en bit av LO-TEK, slik den så ut før den ble korrupt.</p>
       <p class="lt-help-text">${esc(d.description ?? "")}</p>
       ${more.length ? `<p>Under den ligger også ${more.map((c) => `<strong>${esc(compTitle(c))}</strong>`).join(more.length > 2 ? ", " : " og ").replace(/, (?=[^,]*$)/, " og ")}.</p>` : ""}
       <p>Bær ${more.length ? "dem" : "den"} opp til byen, så er de${more.length ? "" : "n"} din${more.length ? "e" : ""}. Dør du før det, blir ${more.length ? "de" : "den"} liggende her nede.</p>`
    : `<p>Kista er tom. Noen har vært her før deg.</p>`;
  dlg.innerHTML = `
    <h2 class="lt-dialog__title" id="loot-title">${component ? "Du fant noe i kista" : "Ingenting"}</h2>
    <div class="lt-dialog__body">${found}</div>
    <div class="lt-dialog__footer">
      <button class="lt-button" data-variant="secondary" type="button" data-act="take">${component ? "Ta med og gå videre" : "Gå videre"}</button>
      ${component && docs[component] ? `<button class="lt-button" data-variant="primary" type="button" data-act="read">Les om ${esc(title)}</button>` : ""}
    </div>`;
  document.body.appendChild(dlg);
  const close = (then) => {
    lootOpen = false;
    dlg.close();
    dlg.remove();
    then();
  };
  dlg.querySelector('[data-act="take"]').addEventListener("click", () => close(() => enter()));
  dlg.querySelector('[data-act="read"]')?.addEventListener("click", () => close(() => readDoc(component)));
  dlg.addEventListener("cancel", (ev) => {
    ev.preventDefault();
    close(() => {
      enter(false);
      clickHint();
    });
  });
  dlg.showModal();
  dlg.querySelector('[data-act="read"], [data-act="take"]')?.focus();
}

// Lappene og bøkene i krypten (F på en lapp): et kort midt på skjermen med
// tittel og tekst, og spillet står stille mens du leser. F, E, Esc, mellomrom
// eller et klikk lukker det.
export function openLore(title, text) {
  if (lootOpen) return;
  lootOpen = true;
  setPlaying(false);
  setDragging(false);
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog lore";
  dlg.setAttribute("aria-labelledby", "lore-title");
  dlg.innerHTML = `
    <h2 class="lt-dialog__title" id="lore-title"></h2>
    <div class="lt-dialog__body"><p class="lt-paragraph lore-text"></p></div>
    <div class="lt-dialog__footer">
      <span class="lt-help-text lore-keys"><kbd>F</kbd> <kbd>E</kbd> <kbd>Esc</kbd> eller klikk lukker</span>
      <button class="lt-button" data-variant="primary" type="button" data-act="close">Legg fra deg</button>
    </div>`;
  dlg.querySelector("#lore-title").textContent = title;
  const body = dlg.querySelector(".lore-text");
  body.textContent = text ? `«${text}»` : "";
  body.hidden = !text;
  document.body.appendChild(dlg);
  let closed = false;
  const close = (relock) => {
    if (closed) return;
    closed = true;
    lootOpen = false;
    dlg.close();
    dlg.remove();
    sfx.play("page");
    enter(relock);
    if (!relock) clickHint();
  };
  dlg.addEventListener("click", () => close(true));
  dlg.addEventListener("keydown", (ev) => {
    if (ev.repeat) return;
    if (["KeyF", "KeyE", "Space", "Enter"].includes(ev.code)) {
      ev.preventDefault();
      ev.stopPropagation();
      close(true);
    }
  });
  dlg.addEventListener("cancel", (ev) => {
    ev.preventDefault();
    close(false);
  });
  dlg.showModal();
  dlg.querySelector('[data-act="close"]').focus();
}

// Fiendekortet midt på skjermen om fienden du sikter på: navnet, om den er
// elite eller boss, om den skyter, hva den er (kanon og slaget) og livet som
// følger skaden. Det står på samme plass som siktekortet, aldri samtidig.
export function setFoe(f) {
  const el = $("foe");
  el.hidden = !f;
  if (!f) return;
  $("foe-name").textContent = f.name;
  $("foe-info").textContent = f.info;
  $("foe-rank").textContent = f.rank;
  $("foe-rank").hidden = !f.rank;
  // Slagene som skyter (inputen, Toast-frosken), plan 13e.
  $("foe-ranged").hidden = !f.ranged;
  $("foe-hp").max = f.max_hp;
  $("foe-hp").value = f.hp;
  $("foe-hp-text").textContent = `${f.hp} / ${f.max_hp}`;
}

// Tastepanelet oppe til høyre foldes sammen og ut med F1 eller et klikk på
// overskriften, og huskes. På smale skjermer er det sammenfoldet fra start.
function initKeysPanel() {
  const el = $("keys-panel");
  let open = innerWidth >= 700;
  try {
    const saved = localStorage.getItem("fristad.keys");
    if (saved) open = saved === "open";
  } catch {
    /* ingen lagring */
  }
  el.open = open;
  el.addEventListener("toggle", () => {
    try {
      localStorage.setItem("fristad.keys", el.open ? "open" : "closed");
    } catch {
      /* ingen lagring */
    }
  });
}
initKeysPanel();

// Snakkeboblene over innbyggerne: vanlige bobler, ikke designsystemet.
const bubbleEls = new Map();
export function updateBubbles(bubbles) {
  const root = $("bubbles");
  const seen = new Set();
  for (const b of bubbles) {
    seen.add(b.id);
    let el = bubbleEls.get(b.id);
    if (!el) {
      el = document.createElement("div");
      el.className = "bubble";
      el.innerHTML = '<div class="bubble-name"></div><div class="bubble-text"></div>';
      root.appendChild(el);
      bubbleEls.set(b.id, el);
    }
    el.style.left = `${b.x * 100}%`;
    el.style.top = `${b.y * 100}%`;
    el.firstChild.textContent = b.name;
    el.lastChild.textContent = b.text;
  }
  for (const [id, el] of bubbleEls) {
    if (!seen.has(id)) {
      el.remove();
      bubbleEls.delete(id);
    }
  }
}

// Merkene i kampen: ett element per id, flyttet dit spillet sier.
const markEls = new Map();
export function updateMarks(marks) {
  const root = $("marks");
  const seen = new Set();
  for (const m of marks) {
    seen.add(m.id);
    let el = markEls.get(m.id);
    if (!el) {
      el = document.createElement("div");
      el.className = "mark";
      if (m.kind === "bar") {
        el.innerHTML = '<div class="mark-bar"><i></i></div>';
      } else {
        el.innerHTML = `<span class="mark-text${Number(m.text) >= 50 ? " heavy" : ""}"></span>`;
      }
      root.appendChild(el);
      markEls.set(m.id, el);
    }
    el.style.left = `${m.x * 100}%`;
    el.style.top = `${m.y * 100}%`;
    if (m.kind === "bar") {
      el.firstChild.firstChild.style.width = `${Math.round(m.value * 100)}%`;
    } else {
      el.firstChild.textContent = m.text;
      el.style.opacity = String(Math.max(0, Math.min(1, m.value * 1.5)));
    }
  }
  for (const [id, el] of markEls) {
    if (!seen.has(id)) {
      el.remove();
      markEls.delete(id);
    }
  }
}

export const setLootMore = (m) => {
  lootMore = m;
};
