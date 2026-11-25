// Smia: skjemaet med oppskriftene og smie-animasjonen.

import { bindPickList, bindTabs, compareTable, compTitle, detailHead, gearIcon, menuTabs, narrow, openOverlay, pickRow, rarityRank, setForgeSkip, SLOT_NAMES, SLOTS, splitRarity } from "./menus.js";
import { $, engine, esc, remember, remembered, sfx } from "./shared.js";

// Smie-animasjonen (plan 13c): komponentene flyr fra lista inn mot midten,
// lyser i LO-TEKs fyllfarger og smelter sammen i en glød, hammeren slår
// to ganger (lyden `craft`), og ikonet for det nye kommer fram med en kort
// blink. Rundt 1,8 s; klikk, Esc, E, Enter eller mellomrom hopper over.
// Med `prefers-reduced-motion` toner bare ikonet inn. `done` kalles én
// gang, også når den hoppes over. Gir funksjonen som hopper over.
const FORGE_GLOWS = ["accent", "brand1", "brand2", "success", "warning", "danger", "brand3"];
function forgeAnimation(box, item, done) {
  const calm = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const timers = [];
  const anims = [];
  const lit = [...box.querySelectorAll(".comp.have")];
  const stage = document.createElement("div");
  stage.className = `forge-stage${calm ? " calm" : ""}`;
  stage.setAttribute("role", "status");
  stage.innerHTML = `<div class="forge-core" aria-hidden="true"></div><div class="forge-sparks" aria-hidden="true"></div>
    <div class="forge-item">${gearIcon(item, "big")}<p class="forge-text">${esc(splitRarity(item.name)[0])} er smidd</p></div>
    <div class="forge-blink" aria-hidden="true"></div><p class="forge-skip">Klikk eller Esc for å hoppe over</p>`;
  box.classList.add("forge-host");
  box.appendChild(stage);
  const at = (ms, f) => timers.push(setTimeout(f, ms));
  const anim = (el, frames, opts) => {
    const a = el.animate(frames, { fill: "both", ...opts });
    anims.push(a);
    return a;
  };
  let over = false;
  const finish = () => {
    if (over) return;
    over = true;
    setForgeSkip(null);
    timers.forEach(clearTimeout);
    anims.forEach((a) => a.cancel());
    stage.remove();
    box.classList.remove("forge-host");
    lit.forEach((c) => c.classList.remove("lit"));
    done();
  };
  setForgeSkip(finish);
  stage.addEventListener("click", finish);
  const shown = stage.querySelector(".forge-item");
  if (calm) {
    anim(shown, [{ opacity: 0 }, { opacity: 1 }], { duration: 300, easing: "ease-out" });
    at(700, finish);
    return finish;
  }
  // Komponentlista rulles fram, så de flyr fra der du ser dem.
  box.querySelector(".comp-list")?.scrollIntoView({ block: "nearest" });
  const s = stage.getBoundingClientRect();
  const cx = s.width / 2;
  const cy = s.height * 0.42;
  stage.style.setProperty("--forge-x", `${cx}px`);
  stage.style.setProperty("--forge-y", `${cy}px`);
  // Ingrediensene lyser opp i hver sin farge og flyr inn mot midten.
  lit.forEach((c, i) => {
    const glow = `var(--lt-color-${FORGE_GLOWS[i % FORGE_GLOWS.length]}-fill)`;
    c.style.setProperty("--glow", glow);
    c.classList.add("lit");
    const ico = c.querySelector(".comp-ico");
    const r = ico?.getBoundingClientRect();
    if (!r || !r.width) return;
    const fly = document.createElement("img");
    fly.className = "forge-fly";
    fly.src = ico.src;
    fly.alt = "";
    fly.style.setProperty("--glow", glow);
    fly.style.left = `${r.left - s.left}px`;
    fly.style.top = `${r.top - s.top}px`;
    fly.style.width = `${r.width}px`;
    fly.style.height = `${r.height}px`;
    stage.appendChild(fly);
    const a = (i / lit.length) * Math.PI * 2;
    const dx = cx + Math.cos(a) * 28 - (r.left - s.left + r.width / 2);
    const dy = cy + Math.sin(a) * 28 - (r.top - s.top + r.height / 2);
    const mx = cx - (r.left - s.left + r.width / 2);
    const my = cy - (r.top - s.top + r.height / 2);
    anim(fly, [
      { transform: "translate(0, 0) scale(1)", opacity: 1, offset: 0 },
      { transform: `translate(${dx * 0.5}px, ${dy * 0.5 - 40}px) scale(1.25)`, opacity: 1, offset: 0.35 },
      { transform: `translate(${dx}px, ${dy}px) scale(0.8)`, opacity: 1, offset: 0.62 },
      { transform: `translate(${mx}px, ${my}px) scale(0.15)`, opacity: 0, offset: 1 },
    ], { duration: 1000, delay: i * 70, easing: "cubic-bezier(.45,0,.25,1)" });
  });
  // Gløden der de smelter sammen, som slår ut ved hvert hammerslag.
  const core = stage.querySelector(".forge-core");
  anim(core, [
    { transform: "translate(-50%, -50%) scale(0.2)", opacity: 0, offset: 0 },
    { transform: "translate(-50%, -50%) scale(0.5)", opacity: 0.7, offset: 0.4 },
    { transform: "translate(-50%, -50%) scale(1)", opacity: 1, offset: 0.55 },
    { transform: "translate(-50%, -50%) scale(0.85)", opacity: 1, offset: 0.62 },
    { transform: "translate(-50%, -50%) scale(1.15)", opacity: 1, offset: 0.72 },
    { transform: "translate(-50%, -50%) scale(1.6)", opacity: 0, offset: 0.85 },
    { transform: "translate(-50%, -50%) scale(1.6)", opacity: 0, offset: 1 },
  ], { duration: 1500, easing: "ease-out" });
  const sparks = stage.querySelector(".forge-sparks");
  for (const t of [820, 1080]) {
    at(t, () => {
      sfx.play("craft");
      const out = `translate(-50%, -50%) scale(2.6) rotate(${t === 820 ? 25 : -40}deg)`;
      anim(sparks, [{ transform: "translate(-50%, -50%) scale(0.3) rotate(0deg)", opacity: 1 }, { transform: out, opacity: 0 }], { duration: 450, easing: "ease-out", fill: "none" });
      anim(box, [{ transform: "translate(0, 0)" }, { transform: "translate(0, 3px)" }, { transform: "translate(0, -1px)" }, { transform: "translate(0, 0)" }], { duration: 160, fill: "none" });
    });
  }
  // Det nye kommer fram med en kort blink.
  at(1250, () => {
    anim(stage.querySelector(".forge-blink"), [{ opacity: 0.85 }, { opacity: 0 }], { duration: 260, easing: "ease-out" });
    anim(shown, [
      { transform: "scale(0.6)", opacity: 0, filter: "brightness(3)" },
      { transform: "scale(1.08)", opacity: 1, filter: "brightness(1.6)", offset: 0.45 },
      { transform: "scale(1)", opacity: 1, filter: "brightness(1)" },
    ], { duration: 420, easing: "ease-out" });
  });
  at(1900, finish);
  return finish;
}

// Smia (plan 13c, eierens revisjon 18. oktober; fra Skriptoriet 26.
// oktober): en dialog med fast
// høyde. Til venstre filtrene med antall og lista som ruller for seg, med
// én oppsummering per oppskrift (ikon, navn, sjeldenhet og «Kan smis»,
// «Mangler N» eller «Har du»); de du kan smi nå øverst, det du har nederst
// og dempet. Til høyre detaljene for den valgte: beskrivelsen,
// sammenligningen med det du har på, komponentene med «har 2 av 3», og
// «Smi» fast nederst, av med grunnen. På smal skjerm står detaljene i
// stedet for lista, med «Tilbake». Fanene er slagene (eieren 18. oktober:
// ingen «Alt», du velger alltid et slag), og den sist valgte huskes.
let workshopFilter = remembered("fristad.workshop.tab", "weapon");
let workshopSel = null;
export function openWorkshop(recipes) {
  const body = openOverlay("Smia", "Søknad om smiing: har du komponentene i lageret, godkjenner smeden skjemaet og smir tingen.");
  $("overlay").querySelector(".overlay-inner").classList.add("menu");
  let detail = false;
  const lack = (r) => r.needs.reduce((s, [, n, have]) => s + Math.max(0, n - have), 0) + r.requires.filter(([, ok]) => !ok).length;
  const state = (r) => (r.item.owned ? 2 : r.ok ? 0 : 1);
  const sorted = [...recipes].sort((a, b) => state(a) - state(b) || lack(a) - lack(b) || rarityRank(a.item.rarity) - rarityRank(b.item.rarity) || a.item.name.localeCompare(b.item.name, "nb"));
  const pool = recipes.flatMap((r) => [r.item, r.current]).filter(Boolean);
  const filters = SLOTS.filter((k) => recipes.some((r) => r.item.slot === k)).map((k) => [k, SLOT_NAMES[k], recipes.filter((r) => r.item.slot === k).length]);
  if (!filters.some(([k]) => k === workshopFilter)) workshopFilter = filters[0]?.[0] ?? "weapon";
  // Etter smiinga: tingen er din og på deg, og komponentene er brukt.
  const crafted = (r) => {
    const made = { ...r.item, owned: true, equipped: true };
    for (const x of recipes) {
      for (const need of x.needs) {
        const used = r.needs.find(([c]) => c === need[0]);
        if (used) need[2] = Math.max(0, need[2] - used[1]);
      }
      if (x.id === r.id) x.item = made;
      else if (x.item.slot === made.slot && made.slot !== "lantern") x.current = made;
      x.ok = !x.item.owned && x.needs.every(([, n, have]) => have >= n) && x.requires.every(([, ok]) => ok);
    }
  };
  const tag = (color, text) => `<span class="lt-badge" data-color="${color}">${text}</span>`;
  const badge = (r) => (r.item.owned ? tag("neutral", "Har du") : r.ok ? tag("success", "Kan smis") : tag("warning", `Mangler ${lack(r)}`));
  const wrap = document.createElement("div");
  wrap.className = "workshop picker";
  body.appendChild(wrap);
  // «Smi» står fast i bunnlinja til venstre for «Tilbake».
  const actions = $("overlay-actions");
  const recipeDetail = (r) => {
    const where = new Map((r.sources ?? []).map(([c, text, cryptOnly]) => [c, { text, cryptOnly }]));
    const missing = [];
    const comp = (c, have, n, keep) => {
      const ok = have >= n;
      if (!ok) missing.push(compTitle(c));
      const w = where.get(c);
      return `<li class="comp ${ok ? "have" : "lack"}">
          <img class="comp-ico" src="components/${esc(c)}.jpg" alt="" loading="lazy" onerror="this.style.visibility='hidden'">
          <div class="comp-main">
            <div class="comp-top"><span class="comp-title">${esc(compTitle(c))}</span><span
              class="comp-count">${keep ? (ok ? "ligger i lageret" : "må ligge i lageret") : `har ${have} av ${n}`}</span>${
              w?.cryptOnly ? '<span class="lt-badge" data-color="warning">Bare i krypten</span>' : ""
            }</div>
            <progress class="lt-progress mini" data-color="${ok ? "success" : "warning"}" max="${n}" value="${Math.min(have, n)}" aria-hidden="true"></progress>
            ${ok || !w ? "" : `<p class="comp-where">${esc(w.text)}</p>`}
          </div>
        </li>`;
    };
    const comps = r.needs.map(([c, n, have]) => comp(c, have, n, false)).concat(r.requires.map(([c, ok]) => comp(c, ok ? 1 : 0, 1, true)));
    const [name] = splitRarity(r.item.name);
    const owned = r.item.owned;
    const can = r.ok && !owned;
    const list = (xs) => (xs.length > 1 ? `${xs.slice(0, -1).join(", ")} og ${xs.at(-1)}` : xs[0] ?? "");
    const why = owned ? "Du har den allerede." : can ? "Komponentene trekkes fra lageret." : `Mangler ${list(missing)}.`;
    const other = r.current && r.current.id !== r.item.id ? r.current : null;
    actions.innerHTML = narrow() && !detail ? "" : `<button type="button" class="lt-button" data-variant="primary" data-craft="${esc(r.id)}" ${can ? "" : "disabled"}
        aria-describedby="craft-why">Smi<span class="craft-name"> ${esc(name)}</span></button>
        <span class="lt-help-text" id="craft-why">${esc(why)}</span>`;
    return `<div class="detail-scroll">
        <button type="button" class="lt-button detail-back" data-variant="ghost" data-back>← Tilbake til lista</button>
        ${detailHead(r.item, badge(r))}
        <h4 class="lt-heading detail-sub" data-size="xs">${other ? `Mot ${esc(splitRarity(other.name)[0])}, som du har på` : "Verdiene"}</h4>
        ${compareTable(r.item, other, pool)}
        <h4 class="lt-heading detail-sub" data-size="xs">Komponentene</h4>
        <ul class="comp-list">${comps.join("")}</ul>
      </div>`;
  };
  const render = () => {
    const shown = sorted.filter((r) => r.item.slot === workshopFilter);
    if (!shown.some((r) => r.id === workshopSel)) workshopSel = shown[0]?.id ?? null;
    const sel = shown.find((r) => r.id === workshopSel);
    const hadFocus = wrap.contains(document.activeElement) && document.activeElement.classList.contains("pick-row");
    wrap.classList.toggle("show-detail", detail);
    actions.innerHTML = "";
    const detailHtml = sel ? recipeDetail(sel) : '<p class="lt-paragraph">Ingen oppskrifter her.</p>';
    wrap.innerHTML = `${menuTabs(filters, workshopFilter, "Slagene")}
      <div class="pick-side">
        <ul class="pick-list" role="listbox" aria-label="Oppskriftene">
          ${shown.map((r) => pickRow(r.id, r.item, badge(r), r.id === workshopSel).replace('class="pick-row"', `class="pick-row${r.item.owned ? " dim" : ""}" data-recipe="${esc(r.id)}"`)).join("")}
        </ul>
      </div>
      <section class="detail" aria-live="polite">${detailHtml}</section>`;
    bindTabs(wrap, (k) => {
      workshopFilter = k;
      remember("fristad.workshop.tab", k);
      workshopSel = null;
      detail = false;
      sfx.play("click");
      render();
      wrap.querySelector(`[data-tab="${k}"]`)?.focus();
    });
    bindPickList(wrap, {
      select(key, open) {
        if (key === workshopSel && !open) return;
        workshopSel = key;
        if (open) detail = true;
        sfx.play("click");
        render();
      },
    });
    wrap.querySelector("[data-back]")?.addEventListener("click", () => {
      detail = false;
      render();
      wrap.querySelector(".pick-row[aria-selected='true']")?.focus();
    });
    actions.querySelector("[data-craft]")?.addEventListener("click", (ev) => {
      const b = ev.currentTarget;
      if (b.disabled) return;
      if (engine.craft(b.dataset.craft)) {
        // Smie-animasjonen, så står raden med «Har du». Lageret er trukket
        // i motoren; her trekkes det samme fra oppskriftene som vises.
        b.disabled = true;
        const r = recipes.find((x) => x.id === b.dataset.craft);
        if (!r) return;
        forgeAnimation($("overlay").querySelector(".overlay-inner"), r.item, () => {
          crafted(r);
          render();
          wrap.querySelector(".pick-row[aria-selected='true']")?.focus();
        });
      } else {
        sfx.play("deny");
      }
    });
    const row = wrap.querySelector(".pick-row[aria-selected='true']");
    row?.scrollIntoView({ block: "nearest" });
    if (hadFocus) row?.focus();
  };
  render();
  if (!narrow()) wrap.querySelector(".pick-row[aria-selected='true']")?.focus();
}
