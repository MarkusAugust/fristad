// Inventaret (I): plassene til venstre, lista over det du har, og den
// valgte tingen med sammenligningen mot det du har på.

import { readDoc } from "./book.js";
import { bindPickList, compareTable, compTitle, detailHead, gearIcon, itemKey, narrow, NO_ARMOR, pickRow, rarityRank, SILHOUETTES, SLOT_NAMES, splitRarity } from "./menus.js";
import { $, docs, engine, enter, esc, lastHud, locked, remember, remembered, setDragging, setPlaying, sfx } from "./shared.js";

export let lastInv = null;
export function renderInventory(e) {
  lastInv = e;
  if (invOpen) renderInvGear(e);
  const weapon = `<div><b>Våpen:</b> ${esc(e.weapon)} <i>${esc(e.weapon_special)}</i></div>`;
  $("gear").innerHTML = `${weapon}<div><b>Skjold:</b> ${esc(e.shield)}</div><div><b>Rustning:</b> ${esc(e.armor)}${e.lantern ? " · Søkelykt" : ""}</div>`;
  const inv = $("inventory");
  inv.replaceChildren();
  const chip = (name, n, carried) => {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "chip" + (carried ? " carried" : "") + (e.read.includes(name) ? " read" : "");
    b.textContent = `${name} ×${n}`;
    b.title = carried ? "Bæres: trygt når du er oppe igjen. Klikk for å lese i boka." : "Klikk for å lese i boka (+20 liv første gang per tur).";
    b.addEventListener("click", () => readDoc(name));
    inv.appendChild(b);
  };
  for (const [name, n] of e.stash) chip(name, n, false);
  for (const [name, n] of e.carried) chip(name, n, true);
  $("vitals").hidden = false;
}

// Inventaret (I), plan 13a/13b: plassene for det du har på (Våpen, Skjold,
// Rustning, Lys) til venstre er filteret, lista over utstyret ruller for
// seg i en dialog med fast høyde, hver rad er en oppsummering (ikon, navn,
// sjeldenhet), og den valgte står med beskrivelse og sammenligning mot det
// du har på, med «Bruk <navn>» rett under tabellen. Dobbeltklikk eller
// Enter tar på, og piltastene går i lista. Fanen «Komponenter» er lageret.
export let invOpen = null;
let invSelected = null;
let invFilter = remembered("fristad.inventory.slot", "weapon");
let invDetail = false;
export function toggleInventory() {
  if (invOpen) {
    closeInventory(true);
    return;
  }
  const none = { stash: [], carried: [], weapon: "Rustent sverd", weapon_special: "", shield: "Gammelt rundskjold", armor: "ingen", lantern: false };
  Object.assign(none, { max_hp: 100, max_stamina: 100, tokens: 0, read: [], gear: [] });
  const e = lastInv ?? none;
  setPlaying(false);
  setDragging(false);
  engine.set_paused(true);
  if (locked()) document.exitPointerLock?.();
  const dlg = document.createElement("dialog");
  dlg.className = "lt-dialog menu-dialog inventory-dialog";
  dlg.setAttribute("aria-labelledby", "inv-title");
  dlg.innerHTML = `
    <div class="lt-dialog__header menu-head">
      <h2 class="lt-dialog__title" id="inv-title">Utstyr og lager</h2>
      <div class="inv-stats"></div>
    </div>
    <div class="lt-dialog__body menu-body">
      <lt-tabs>
        <ul class="lt-tabs__list" aria-label="Inventaret">
          <li><a href="#inv-gear">Utstyr</a></li>
          <li><a href="#inv-comps" class="inv-comp-tab">Komponenter</a></li>
        </ul>
        <div class="lt-tabs__panel inv-gear-panel" id="inv-gear"></div>
        <div class="lt-tabs__panel inv-comps" id="inv-comps"></div>
      </lt-tabs>
    </div>
    <div class="lt-dialog__footer menu-foot">
      <button class="lt-button" data-variant="secondary" type="button" data-act="close">Lukk (I)</button>
    </div>`;
  document.body.appendChild(dlg);
  invOpen = dlg;
  invSelected = null;
  invDetail = false;
  renderInvGear(e);
  dlg.querySelector('[data-act="close"]').addEventListener("click", () => closeInventory(true));
  dlg.addEventListener("cancel", (ev) => {
    ev.preventDefault();
    if (invDetail && narrow()) {
      invDetail = false;
      renderInvGear(lastInv ?? e);
      return;
    }
    closeInventory(true);
  });
  sfx.play("bag");
  dlg.showModal();
  dlg.querySelector(".pick-row[aria-selected='true']")?.focus();
}

// Lykta som en ting i plassen «Lys»: oljen og hvor lenge den brenner
// (`lantern::FULL` er 8 minutter).
function lanternItem(e) {
  const light = lastHud?.light ?? 100;
  const min = Math.round((light / 100) * 8);
  return {
    id: e.lantern ? "search_lantern" : "",
    name: e.lantern ? "Søkelykt" : "Lykta i beltet",
    slot: "lantern",
    rarity: e.lantern ? "sjelden" : "vanlig",
    special: "",
    about: e.lantern
      ? "Lykt med slipt glass som kaster lyset dobbelt så langt inn i mørket."
      : "Lykta du bærer i beltet. Oljen brenner bare nede i krypten; fyll den hos Apotekeren eller på hvilestedene.",
    stats: [
      { label: "Olje", value: light / 100, max: 1, text: `${light} %`, up: true },
      { label: "Brenner", value: min, max: 8, text: min ? `ca. ${min} min` : "tom", up: true },
      { label: "Bærelyset", value: e.lantern ? 2.2 : 1, max: 2.2, text: e.lantern ? "×2,2" : "×1,0", up: true },
    ],
    equipped: true,
    owned: true,
    light,
  };
}

function renderInvGear(e) {
  const dlg = invOpen;
  if (!dlg) return;
  const h = lastHud;
  dlg.querySelector(".inv-stats").innerHTML = `
    <label class="lt-label">Liv <span class="bar-num">${h.hp} / ${e.max_hp}</span><progress class="lt-progress" data-color="danger" value="${h.hp}" max="${e.max_hp}"></progress></label>
    <label class="lt-label">Energi <span class="bar-num">${h.stamina} / ${e.max_stamina}</span><progress class="lt-progress" value="${h.stamina}" max="${e.max_stamina}"></progress></label>
    <span class="lt-badge" data-color="warning">${e.tokens} tokens</span>
    <span class="lt-badge" data-color="neutral">${h.keys === 1 ? "1 nøkkel" : `${h.keys ?? 0} nøkler`}</span>`;
  const gear = e.gear ?? [];
  const lantern = lanternItem(e);
  const worn = (slot) => (slot === "lantern" ? lantern : gear.find((g) => g.slot === slot && g.equipped) ?? (slot === "armor" ? NO_ARMOR : null));
  const slotOf = { weapon: 0, shield: 1, armor: 2, lantern: 3 };
  const all = [...gear, lantern]
    .filter((g) => g.slot === invFilter)
    .sort((a, b) => slotOf[a.slot] - slotOf[b.slot] || Number(b.equipped) - Number(a.equipped) || rarityRank(b.rarity) - rarityRank(a.rarity) || a.name.localeCompare(b.name, "nb"));
  // Først det første du ikke har på, så det står mot det du har på med en
  // gang; ellers det du har på.
  if (!all.some((g) => itemKey(g) === invSelected)) invSelected = itemKey(all.find((g) => !g.equipped) ?? all[0] ?? {});
  const sel = all.find((g) => itemKey(g) === invSelected);
  // Plassene: det du har på, og filteret.
  const slotCard = (slot, label, item, empty) => {
    const [name, r] = splitRarity(item?.name ?? "");
    const rarity = item?.rarity || r;
    const real = item && item.id !== "" || slot === "lantern";
    return `<button type="button" class="slot-card" data-slot="${slot}" data-rarity="${esc(rarity || "vanlig")}" aria-pressed="${invFilter === slot}"
        title="${esc(label)}: ${esc(real ? name : empty)}" ${real ? "" : 'data-empty=""'}>
        ${real ? gearIcon(item, "small") : `<span class="gear-ico small silhouette" aria-hidden="true">${SILHOUETTES[slot]}</span>`}
        <span class="slot-text">
          <span class="eyebrow">${label}</span>
          <span class="slot-name">${esc(real ? name : empty)}</span>
          ${slot === "lantern" ? `<progress class="lt-progress mini" data-color="warning" max="100" value="${lantern.light}" aria-label="Olje"></progress>` : ""}
        </span>
      </button>`;
  };
  const wornName = (slot) => worn(slot) ?? null;
  const counts = new Map();
  for (const [c, n] of e.stash) counts.set(c, { stash: n, carried: 0 });
  for (const [c, n] of e.carried) counts.set(c, { stash: counts.get(c)?.stash ?? 0, carried: n });
  const panel = dlg.querySelector(".inv-gear-panel");
  const hadFocus = panel.contains(document.activeElement) && document.activeElement.classList.contains("pick-row");
  panel.innerHTML = `<div class="inv-layout picker${invDetail ? " show-detail" : ""}">
      <div class="inv-slots" role="group" aria-label="Det du har på (velg for å vise bare det slaget)">
        ${slotCard("weapon", "Våpen", wornName("weapon"), "")}
        ${slotCard("shield", "Skjold", wornName("shield"), "")}
        ${slotCard("armor", "Rustning", worn("armor") === NO_ARMOR ? null : worn("armor"), "Finn en rustning, eller få den smidd hos Smeden")}
        ${slotCard("lantern", "Lys", lantern, "")}
      </div>
      <div class="pick-side">
        <p class="pick-count">${SLOT_NAMES[invFilter]} <span class="tab-count">${all.length}</span></p>
        <ul class="pick-list" role="listbox" aria-label="Utstyret ditt">
          ${all.map((g) => pickRow(itemKey(g), g, g.equipped ? '<span class="lt-badge" data-color="success">På deg</span>' : "", invSelected === itemKey(g))).join("")}
        </ul>
      </div>
      <section class="detail" aria-live="polite">${sel ? invDetailHtml(sel, worn(sel.slot), [...gear, NO_ARMOR]) : ""}</section>
    </div>`;
  const tab = dlg.querySelector(".inv-comp-tab");
  tab.innerHTML = `Komponenter <span class="tab-count">${counts.size}</span>`;
  dlg.querySelector(".inv-comps").innerHTML = counts.size
    ? `<div class="menu-scroll"><table class="lt-table" data-variant="striped">
        <caption>Komponentene du har vunnet. Det du bærer, er ditt først når du er oppe igjen.</caption>
        <thead><tr><th scope="col">Komponent</th><th scope="col" data-align="end">I lageret</th><th scope="col" data-align="end">Bærer</th><th scope="col">Boka</th><th scope="col"></th></tr></thead>
        <tbody>${[...counts.entries()]
          .sort((a, b) => compTitle(a[0]).localeCompare(compTitle(b[0]), "nb"))
          .map(([c, k]) => `<tr>
            <th scope="row"><span class="comp-name"><img class="comp-ico" src="components/${esc(c)}.jpg" alt="" loading="lazy" onerror="this.hidden=true">${esc(compTitle(c))}</span></th>
            <td data-align="end">${k.stash}</td>
            <td data-align="end">${k.carried ? `<span class="lt-badge" data-color="warning">${k.carried}</span>` : "0"}</td>
            <td>${e.read.includes(c) ? '<span class="lt-badge" data-color="success">Lest</span>' : '<span class="lt-badge" data-color="neutral">Ulest</span>'}</td>
            <td data-align="end">${docs[c] ? `<button class="lt-button" data-variant="secondary" type="button" data-read="${esc(c)}">Les</button>` : ""}</td>
          </tr>`)
          .join("")}</tbody>
      </table></div>`
    : `<p class="lt-paragraph">Du har ingen komponenter ennå. De ligger i kistene og hos fiendene i krypten.</p>`;
  for (const b of dlg.querySelectorAll("[data-read]")) {
    b.addEventListener("click", () => {
      const c = b.dataset.read;
      closeInventory(false);
      readDoc(c);
    });
  }
  for (const b of panel.querySelectorAll("[data-slot]")) {
    b.addEventListener("click", () => {
      invFilter = b.dataset.slot;
      remember("fristad.inventory.slot", invFilter);
      invSelected = null;
      invDetail = false;
      sfx.play("click");
      renderInvGear(lastInv ?? e);
    });
  }
  const equip = (key) => {
    const g = all.find((x) => itemKey(x) === key);
    if (!g || g.equipped || g.slot === "lantern") return;
    if (engine.equip(g.id)) sfx.play("click");
  };
  bindPickList(panel, {
    select(key, open) {
      invSelected = key;
      if (open) invDetail = true;
      sfx.play("click");
      renderInvGear(lastInv ?? e);
    },
    activate: equip,
  });
  panel.querySelector("[data-equip]")?.addEventListener("click", (ev) => equip(ev.currentTarget.dataset.equip));
  panel.querySelector("[data-back]")?.addEventListener("click", () => {
    invDetail = false;
    renderInvGear(lastInv ?? e);
  });
  const row = panel.querySelector(".pick-row[aria-selected='true']");
  row?.scrollIntoView({ block: "nearest" });
  if (hadFocus) row?.focus();
}

// Detaljene for en ting i inventaret: beskrivelsen, sammenligningen og
// «Bruk <navn>» rett under tabellen.
function invDetailHtml(sel, worn, pool) {
  const [name] = splitRarity(sel.name);
  const same = !worn || itemKey(worn) === itemKey(sel);
  const action =
    sel.slot === "lantern"
      ? `<p class="lt-help-text">${sel.id ? "Søkelykta henger i beltet." : "En søkelykt smis hos Smeden."} Olje får du hos Apotekeren og på hvilestedene.</p>`
      : `<div class="detail-actions">
          ${sel.equipped
            ? '<span class="lt-help-text">Du har den på deg. Velg noe annet i lista for å se det mot denne.</span>'
            : `<button type="button" class="lt-button" data-variant="primary" data-equip="${esc(sel.id)}">Bruk ${esc(name)}</button>
               <span class="lt-help-text">Bytter ut ${esc(splitRarity(worn?.name ?? "")[0])}. Dobbeltklikk eller Enter i lista gjør det samme.</span>`}
        </div>`;
  return `<div class="detail-scroll">
      <button type="button" class="lt-button detail-back" data-variant="ghost" data-back>← Tilbake til lista</button>
      ${detailHead(sel)}
      <h4 class="lt-heading detail-sub" data-size="xs">${same ? "Verdiene" : `Mot ${esc(splitRarity(worn?.name ?? "")[0])}, som du har på`}</h4>
      ${compareTable(sel, same ? null : worn, pool)}
      ${action}
    </div>`;
}

function closeInventory(resume) {
  if (!invOpen) return;
  invOpen.close();
  invOpen.remove();
  invOpen = null;
  if (resume) enter();
}
