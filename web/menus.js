// Det menyene har felles: overlegget (dialogen midt på skjermen), fanene,
// lista med én rad per ting, kortet med detaljene og sammenligningstabellen.

import { $, clickHint, docs, engine, enter, esc, locked, setDragging, setPlaying } from "./shared.js";

// Smal skjerm: lista alene, og detaljene i stedet for lista med «Tilbake».
export const narrow = () => innerWidth <= 720;

// Butikk og verksted: en LO-TEK-dialog midt på skjermen med spillet på
// pause mens den står. Den ligger i HUD-en, så den tar imot musa selv.
export let overlayOpen = false;
export function openOverlay(title, subtitle = "") {
  overlayOpen = true;
  setPlaying(false);
  setDragging(false);
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  $("overlay-title").textContent = title;
  $("overlay-sub").textContent = subtitle;
  $("overlay-sub").hidden = !subtitle;
  $("overlay").querySelector(".overlay-inner").classList.remove("menu");
  $("overlay-body").replaceChildren();
  $("overlay-actions").replaceChildren();
  $("overlay").hidden = false;
  return $("overlay-body");
}

// Lukkes med knappen eller E (en brukerhandling), fanges musa igjen med en
// gang, så du kan se deg rundt uten å klikke. Med Esc gir ikke nettleseren
// oss musa; da sier en melding at et klikk i bildet tar deg tilbake.
export function closeOverlay(resume = true, relock = true) {
  if (!overlayOpen) return;
  forgeSkip?.();
  overlayOpen = false;
  $("overlay").hidden = true;
  if (resume) enter(relock);
  if (resume && !relock) clickHint();
}

// Felles for inventaret og Smia (plan 13a): sjeldenheten i farge
// (LO-TEKs tokens: vanlig grå, sjelden blå, episk lilla, legendarisk gull),
// ikonet (`icons/<id>.png`), raden med oppsummeringen og
// sammenligningstabellen.
export const SLOT_NAMES = { weapon: "Våpen", shield: "Skjold", armor: "Rustning", lantern: "Lys" };
const RARITIES = ["vanlig", "sjelden", "episk", "legendarisk"];
export const rarityRank = (r) => Math.max(0, RARITIES.indexOf(r));
const cap = (s) => (s ? s[0].toUpperCase() + s.slice(1) : "");
// «Gammelt rundskjold (vanlig)» → ["Gammelt rundskjold", "vanlig"].
export function splitRarity(name) {
  const m = /^(.*?)\s*\((vanlig|sjelden|episk|legendarisk)\)$/.exec(name ?? "");
  return m ? [m[1], m[2]] : [name ?? "", ""];
}
export const rarityTag = (r) => (r ? `<span class="lt-tag rarity-tag" data-rarity="${esc(r)}">${cap(esc(r))}</span>` : "");
export const itemKey = (g) => `${g?.slot ?? ""}:${g?.id ?? ""}`;
export const compTitle = (c) => docs[c]?.title ?? c;
export const SLOTS = ["weapon", "shield", "armor", "lantern"];
// Fanerekken over lista (LO-TEKs `lt-tabs__list`, lenker i en liste): én
// rad som aldri brytes, med antallet som en liten badge. Lista byttes ut
// når en fane velges, så det er ingen paneler å peke på, og klikket skriver
// ikke i adressen. [[nøkkel, navn, antall]].
export function menuTabs(tabs, current, label) {
  return `<ul class="lt-tabs__list menu-tabs" role="tablist" aria-label="${esc(label)}">
      ${tabs.map(([k, n, count]) => `<li role="presentation"><a href="#${k}" role="tab" data-tab="${k}" aria-selected="${k === current}"
        tabindex="${k === current ? 0 : -1}">${esc(n)}<span class="tab-count">${count}</span></a></li>`).join("")}
    </ul>`;
}
export function bindTabs(root, pick) {
  const tabs = [...root.querySelectorAll(".menu-tabs [data-tab]")];
  for (const t of tabs)
    t.addEventListener("click", (ev) => {
      ev.preventDefault();
      if (t.getAttribute("aria-selected") !== "true") pick(t.dataset.tab);
    });
  root.querySelector(".menu-tabs")?.addEventListener("keydown", (ev) => {
    const i = tabs.indexOf(document.activeElement);
    if (i < 0 || (ev.key !== "ArrowRight" && ev.key !== "ArrowLeft")) return;
    ev.preventDefault();
    pick(tabs[(i + (ev.key === "ArrowRight" ? 1 : tabs.length - 1)) % tabs.length].dataset.tab);
  });
  root.querySelector('.menu-tabs [aria-selected="true"]')?.scrollIntoView({ block: "nearest", inline: "nearest" });
}
export const SILHOUETTES = {
  weapon: '<svg viewBox="0 0 24 24"><path d="M14 3h7v7L10 21l-3-3L18 7M5 15l4 4M3 21l3-3"/></svg>',
  shield: '<svg viewBox="0 0 24 24"><path d="M12 3l8 3v6c0 5-4 8-8 9-4-1-8-4-8-9V6z"/></svg>',
  armor: '<svg viewBox="0 0 24 24"><path d="M8 3l4 2 4-2 4 3-2 4v11H6V10L4 6z"/></svg>',
  lantern: '<svg viewBox="0 0 24 24"><path d="M9 3h6M10 3v3h4V3M7 6h10v13H7zM9 21h6M12 10v5"/></svg>',
};
export function gearIcon(item, size) {
  const r = item?.rarity || splitRarity(item?.name)[1] || "vanlig";
  if (!item?.id) return `<span class="gear-ico ${size} silhouette" data-rarity="${esc(r)}" aria-hidden="true">${SILHOUETTES[item?.slot] ?? ""}</span>`;
  return `<span class="gear-ico ${size}" data-rarity="${esc(r)}"><img src="icons/${esc(item.id)}.png" alt="" loading="lazy" onerror="this.hidden=true"></span>`;
}
export const NO_ARMOR = { id: "", name: "Ingen rustning", slot: "armor", rarity: "vanlig", special: "", about: "Bare skjorta og kappa du kom til porten i.", equipped: true, owned: true, stats: [
  { label: "Skade du tar", value: 1, text: "100 %", up: false },
  { label: "Utholdenhet tilbake", value: 1, text: "×1.00", up: true },
  { label: "Unnamanøver koster", value: 1, text: "×1.00", up: false },
] };

// Egenskaper der lavt er bra, vises slik at høyt er bra der det går
// («Tåler 75 %» i stedet for «Slipper gjennom 25 %»); ellers følger fargen.
const FLIP = { "Slipper gjennom": "Tåler", "Skade du tar": "Demper skade" };
function shownStat(st) {
  if (FLIP[st.label]) {
    const v = 1 - st.value;
    return { label: FLIP[st.label], value: v, text: `${Math.round(v * 100)} %`, up: true };
  }
  return { ...st, text: String(st.text).replace(/(\d)\.(\d)/g, "$1,$2") };
}
function fmtDelta(st, d) {
  const sign = d > 0 ? "+" : "−";
  const a = Math.abs(d);
  const n = (x, k) => x.toFixed(k).replace(".", ",");
  if (/%$/.test(st.text)) return `${sign}${Math.round(a * 100)} %`;
  if (/ m$/.test(st.text)) return `${sign}${n(a, 1)} m`;
  if (/ s$/.test(st.text)) return `${sign}${n(a, 2)} s`;
  if (/min$/.test(st.text)) return `${sign}${Math.round(a)} min`;
  return `${sign}${n(a, 2)}`;
}
// Hvor god en verdi er mot de andre tingene i samme meny (0–100): lengre
// stolpe er bedre, også der lavt er bra.
function goodness(st, label, pool) {
  if (st.max) return (100 * st.value) / st.max;
  const vals = pool.flatMap((g) => g?.stats ?? []).filter((x) => x.label === label).map((x) => shownStat(x).value);
  if (!vals.length) return 100;
  return st.up ? (100 * st.value) / Math.max(...vals, 1e-6) : (100 * Math.min(...vals)) / Math.max(st.value, 1e-6);
}
export function compareTable(sel, other, pool) {
  const bar = (st, label) => Math.round(goodness(st, label, pool));
  const cell = (st, label) => `<span class="stat-val">${esc(st.text)}</span><progress class="lt-progress mini" max="100" value="${bar(st, label)}" aria-hidden="true"></progress>`;
  const rows = sel.stats
    .map((raw) => {
      const st = shownStat(raw);
      const oraw = other?.stats.find((x) => x.label === raw.label);
      const o = oraw && shownStat(oraw);
      let delta = "";
      if (o) {
        const d = st.value - o.value;
        if (Math.abs(d) > 1e-4) {
          const good = st.up ? d > 0 : d < 0;
          delta = `<span class="delta ${good ? "better" : "worse"}" aria-label="${good ? "bedre" : "dårligere"}">${fmtDelta(st, d)}</span>`;
        }
      }
      const low = st.up ? "" : ' <span class="low-good" title="Lavt er bra">(lavt er bra)</span>';
      const worn = other ? `<td data-align="end">${o ? cell(o, raw.label) : "–"}</td>` : "";
      const diff = other ? `<td data-align="end" class="delta-cell">${delta}</td>` : "";
      return `<tr><th scope="row">${esc(st.label)}${low}</th>${worn}<td data-align="end">${cell(st, raw.label)}</td>${diff}</tr>`;
    })
    .join("");
  const otherName = splitRarity(other?.name)[0];
  return `<table class="lt-table compare">
      <caption class="lt-sr-only">${other ? `${esc(splitRarity(sel.name)[0])} mot ${esc(otherName)}` : "Verdiene"}</caption>
      <thead><tr><th scope="col">Egenskap</th>${other ? `<th scope="col" data-align="end">Har på</th>` : ""}<th scope="col" data-align="end">${other ? "Denne" : "Verdi"}</th>${
        other ? '<th scope="col" data-align="end">Forskjell</th>' : ""
      }</tr></thead>
      <tbody>${rows}</tbody>
    </table>`;
}
export function detailHead(item, badge = "") {
  const [name, r] = splitRarity(item.name);
  const rarity = item.rarity || r;
  return `<header class="detail-head" data-rarity="${esc(rarity)}">
      ${gearIcon(item, "big")}
      <div class="detail-title">
        <h3 class="lt-heading" data-size="m">${esc(name)}</h3>
        <div class="gear-tags">${rarityTag(rarity)}<span class="lt-tag">${SLOT_NAMES[item.slot] ?? ""}</span>${
          item.equipped && item.slot !== "lantern" ? '<span class="lt-badge" data-color="success">På deg</span>' : ""
        }${badge}</div>
        ${item.special ? `<p class="detail-special"><span class="lt-tag" data-variant="filled">${esc(cap(item.special))}</span></p>` : ""}
      </div>
    </header>
    ${item.about ? `<p class="lt-paragraph detail-about">${esc(item.about)}</p>` : ""}`;
}
// Én rad i lista: ikonet, navnet, sjeldenheten og et merke.
export function pickRow(key, item, badge, selected, sub = "") {
  const [name, r] = splitRarity(item.name);
  const rarity = item.rarity || r;
  return `<li class="pick-row" role="option" data-key="${esc(key)}" data-rarity="${esc(rarity)}" aria-selected="${selected}" tabindex="${selected ? 0 : -1}">
      ${gearIcon(item, "small")}
      <span class="pick-text"><span class="pick-name">${esc(name)}</span><span class="pick-meta">${rarityTag(rarity)}${sub ? `<span class="pick-sub">${esc(sub)}</span>` : ""}${badge}</span></span>
    </li>`;
}
// Musa og tastene i lista: klikk velger (på smal skjerm åpner det
// detaljene), to klikk eller Enter bruker, piltastene flytter.
let lastPick = { key: "", at: 0 };
export function bindPickList(root, { select, activate }) {
  const list = root.querySelector(".pick-list");
  if (!list) return;
  const rows = [...list.querySelectorAll(".pick-row")];
  for (const row of rows) {
    row.addEventListener("click", () => {
      const key = row.dataset.key;
      const now = performance.now();
      if (activate && lastPick.key === key && now - lastPick.at < 450) {
        lastPick = { key: "", at: 0 };
        activate(key);
        return;
      }
      lastPick = { key, at: now };
      select(key, narrow());
    });
  }
  list.addEventListener("keydown", (ev) => {
    const i = rows.indexOf(document.activeElement);
    if (i < 0) return;
    if (ev.key === "ArrowDown" || ev.key === "ArrowUp") {
      ev.preventDefault();
      const next = rows[Math.min(rows.length - 1, Math.max(0, i + (ev.key === "ArrowDown" ? 1 : -1)))];
      next.focus();
      select(next.dataset.key, false);
    } else if (ev.key === "Enter") {
      ev.preventDefault();
      if (activate) activate(rows[i].dataset.key);
      else select(rows[i].dataset.key, true);
    } else if (ev.key === " ") {
      ev.preventDefault();
      select(rows[i].dataset.key, true);
    }
  });
}

let forgeSkip = null;
// Esc, E, Enter eller mellomrom hopper over (før de andre tastene, så Esc
// ikke også lukker Smia).
document.addEventListener("keydown", (e) => {
  if (!forgeSkip || !["Escape", "KeyE", "Enter", "Space"].includes(e.code)) return;
  e.preventDefault();
  e.stopImmediatePropagation();
  forgeSkip();
}, true);

export const setForgeSkip = (f) => {
  forgeSkip = f;
};
