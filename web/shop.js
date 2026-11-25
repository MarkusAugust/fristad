// Butikkene: varene hos Apotekeren og Torgils, kjøp og bytte i
// Skriptoriet, og kruset mjød til trubaduren.

import { bindPickList, bindTabs, closeOverlay, compTitle, menuTabs, narrow, openOverlay, rarityRank, rarityTag } from "./menus.js";
import { $, docs, engine, esc, lastHud, remember, remembered, sfx } from "./shared.js";

// Butikken: én rad per vare (navnet, prisen og «Kjøp»). Lista ruller inne i
// dialogen når den er lang (plan 13c). Skriptoriet har også byttene
// (plan 14): da står varene og byttene i hver sin fane (`openSmith`).
export function openShop(npc, wares, swaps = []) {
  if (swaps.length) return openSmith(npc, wares, swaps);
  const body = openOverlay(npc, "");
  body.appendChild(wareList(wares));
  updateShop();
}

function wareList(wares) {
  const list = document.createElement("ul");
  list.className = "lt-list wares";
  list.dataset.variant = "divided";
  for (const w of wares) {
    const row = document.createElement("li");
    row.className = "ware";
    const text = document.createElement("div");
    text.className = "ware-text";
    const name = document.createElement("span");
    name.className = "ware-name";
    name.textContent = w.name;
    const price = document.createElement("span");
    price.className = "ware-price";
    price.textContent = `${w.price} tokens`;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "lt-button";
    btn.dataset.variant = "secondary";
    btn.dataset.price = String(w.price);
    btn.textContent = "Kjøp";
    btn.setAttribute("aria-label", `Kjøp ${w.name} for ${w.price} tokens`);
    const why = document.createElement("span");
    why.className = "lt-help-text ware-why";
    why.id = `ware-why-${w.id}`;
    btn.setAttribute("aria-describedby", why.id);
    btn.addEventListener("click", () => {
      if (btn.disabled) return;
      engine.buy(w.id);
    });
    text.append(name, why);
    // Varen med ikon (`icons/<id>.png`, helsedrikkene); uten ikon står bare teksten.
    const main = document.createElement("div");
    main.className = "ware-main";
    const ico = document.createElement("img");
    ico.className = "ware-ico";
    ico.src = `icons/${w.id}.png`;
    ico.alt = "";
    ico.loading = "lazy";
    ico.onerror = () => ico.remove();
    main.append(ico, text);
    row.append(main, price, btn);
    list.appendChild(row);
  }
  return list;
}

// Skriptoriet (plan 14; fra Buttonsmeden 26. oktober): fanene «Kjøp» og
// «Bytte», som Smia. I
// «Bytte» står ett bytte per rad (det du får, sjeldenheten og hva du gir),
// de du kan gjøre nå øverst; detaljene viser hva du gir og har, avgiften og
// hvor komponenten ellers finnes, og «Bytt» står fast i bunnlinja. Etter et
// bytte sender motoren butikken på nytt, så tallene er de nye.
let shopTab = remembered("fristad.shop.tab", "swap");
let swapSel = null;
function openSmith(npc, wares, swaps) {
  const body = openOverlay(npc, `Du har ${lastHud?.tokens ?? 0} tokens.`);
  $("overlay").querySelector(".overlay-inner").classList.add("menu");
  let detail = false;
  const lack = (s) => Math.max(0, s.n - s.have);
  const sorted = [...swaps].sort((a, b) => Number(b.ok) - Number(a.ok) || lack(a) - lack(b) || rarityRank(a.get_rarity) - rarityRank(b.get_rarity));
  const tabs = [["buy", "Kjøp", wares.length], ["swap", "Bytte", swaps.length]];
  if (!tabs.some(([k]) => k === shopTab)) shopTab = "swap";
  const warn = (text) => `<span class="lt-badge" data-color="warning">${text}</span>`;
  const badge = (s) => (s.ok ? '<span class="lt-badge" data-color="success">Kan byttes</span>' : lack(s) ? warn(`Mangler ${lack(s)}`) : warn("Mangler tokens"));
  const compIco = (c, size, r) => `<span class="gear-ico ${size} comp-face" data-rarity="${esc(r)}"><img src="components/${esc(c)}.jpg" alt="" loading="lazy" onerror="this.hidden=true"></span>`;
  const wrap = document.createElement("div");
  wrap.className = "smith picker";
  body.appendChild(wrap);
  const actions = $("overlay-actions");
  const swapDetail = (s) => {
    const tokens = lastHud?.tokens ?? 0;
    const why = s.ok ? `Du gir ${s.n} ${compTitle(s.give)} og ${s.fee} tokens.` : lack(s) ? `Mangler ${lack(s)} ${compTitle(s.give)} i lageret.` : `Mangler ${s.fee - tokens} tokens til avgiften.`;
    actions.innerHTML = narrow() && !detail ? "" : `<button type="button" class="lt-button" data-variant="primary" data-swap="${esc(s.id)}" ${s.ok ? "" : "disabled"}
        aria-describedby="swap-why"><span>Bytt<span class="craft-name"> til ${esc(compTitle(s.get))}</span></span></button>
        <span class="lt-help-text" id="swap-why">${esc(why)}</span>`;
    const giveOk = s.have >= s.n;
    const feeOk = tokens >= s.fee;
    return `<div class="detail-scroll">
        <button type="button" class="lt-button detail-back" data-variant="ghost" data-back>← Tilbake til lista</button>
        <header class="detail-head" data-rarity="${esc(s.get_rarity)}">
          ${compIco(s.get, "big", s.get_rarity)}
          <div class="detail-title">
            <h3 class="lt-heading" data-size="m">${esc(compTitle(s.get))}</h3>
            <div class="gear-tags">${rarityTag(s.get_rarity)}<span class="lt-tag">Komponent</span>${badge(s)}</div>
          </div>
        </header>
        <p class="lt-paragraph detail-about">${esc(docs[s.get]?.description ?? "")}</p>
        <h4 class="lt-heading detail-sub" data-size="xs">Du gir</h4>
        <ul class="comp-list">
          <li class="comp ${giveOk ? "have" : "lack"}">
            <img class="comp-ico" src="components/${esc(s.give)}.jpg" alt="" loading="lazy" onerror="this.style.visibility='hidden'">
            <div class="comp-main">
              <div class="comp-top"><span class="comp-title">${s.n} × ${esc(compTitle(s.give))}</span>${rarityTag(s.give_rarity)}<span class="comp-count">har ${s.have} av ${s.n}</span></div>
              <progress class="lt-progress mini" data-color="${giveOk ? "success" : "warning"}" max="${s.n}" value="${Math.min(s.have, s.n)}" aria-hidden="true"></progress>
            </div>
          </li>
          <li class="comp ${feeOk ? "have" : "lack"}">
            <span class="comp-ico token-ico" aria-hidden="true"><svg viewBox="0 0 24 24"><path d="M12 2.5l8.2 4.75v9.5L12 21.5l-8.2-4.75v-9.5z"/></svg></span>
            <div class="comp-main">
              <div class="comp-top"><span class="comp-title">Avgift: ${s.fee} tokens</span><span class="comp-count">har ${tokens}</span></div>
            </div>
          </li>
        </ul>
        <h4 class="lt-heading detail-sub" data-size="xs">Du får</h4>
        <ul class="comp-list">
          <li class="comp have">
            <img class="comp-ico" src="components/${esc(s.get)}.jpg" alt="" loading="lazy" onerror="this.style.visibility='hidden'">
            <div class="comp-main">
              <div class="comp-top"><span class="comp-title">1 × ${esc(compTitle(s.get))}</span><span class="comp-count">har ${s.get_have} nå</span></div>
              <p class="comp-where">Ellers: ${esc(s.source)}</p>
            </div>
          </li>
        </ul>
        <p class="lt-help-text swap-rule">Byttet går bare én vei, og smeden kjøper ingenting tilbake.</p>
      </div>`;
  };
  const render = () => {
    const hadFocus = wrap.contains(document.activeElement) && document.activeElement.classList.contains("pick-row");
    actions.innerHTML = "";
    if (shopTab === "buy") {
      wrap.classList.remove("show-detail");
      wrap.innerHTML = `${menuTabs(tabs, shopTab, "Butikken")}<div class="shop-buy"></div>`;
      wrap.querySelector(".shop-buy").appendChild(wareList(wares));
      updateShop();
    } else {
      if (!sorted.some((s) => s.id === swapSel)) swapSel = sorted[0]?.id ?? null;
      const sel = sorted.find((s) => s.id === swapSel);
      wrap.classList.toggle("show-detail", detail);
      const detailHtml = sel ? swapDetail(sel) : '<p class="lt-paragraph">Ingen bytter her.</p>';
      wrap.innerHTML = `${menuTabs(tabs, shopTab, "Butikken")}
        <div class="pick-side">
          <ul class="pick-list" role="listbox" aria-label="Byttene">
            ${sorted
              .map(
                (s) => `<li class="pick-row" role="option" data-key="${esc(s.id)}" data-rarity="${esc(s.get_rarity)}" aria-selected="${s.id === swapSel}" tabindex="${s.id === swapSel ? 0 : -1}">
                ${compIco(s.get, "small", s.get_rarity)}
                <span class="pick-text"><span class="pick-name">${esc(compTitle(s.get))}</span><span class="pick-meta">${rarityTag(s.get_rarity)}<span
                  class="pick-sub">${s.n} × ${esc(compTitle(s.give))} + ${s.fee} t</span>${badge(s)}</span></span>
              </li>`,
              )
              .join("")}
          </ul>
        </div>
        <section class="detail" aria-live="polite">${detailHtml}</section>`;
    }
    bindTabs(wrap, (k) => {
      shopTab = k;
      remember("fristad.shop.tab", k);
      detail = false;
      sfx.play("click");
      render();
      wrap.querySelector(`[data-tab="${k}"]`)?.focus();
    });
    bindPickList(wrap, {
      select(key, open) {
        if (key === swapSel && !open) return;
        swapSel = key;
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
    actions.querySelector("[data-swap]")?.addEventListener("click", (ev) => {
      const b = ev.currentTarget;
      if (b.disabled) return;
      // Går byttet, kommer butikken på nytt med de nye tallene.
      if (!engine.swap?.(b.dataset.swap)) sfx.play("deny");
    });
    const row = wrap.querySelector(".pick-row[aria-selected='true']");
    row?.scrollIntoView({ block: "nearest" });
    if (hadFocus) row?.focus();
  };
  render();
  if (!narrow()) wrap.querySelector(".pick-row[aria-selected='true']")?.focus();
}

// Det du ikke har råd til, er en deaktivert LO-TEK-knapp med forklaring.
// Kalles når butikken åpnes og hver gang tokenene endres (etter et kjøp).
export function updateShop() {
  const tokens = lastHud?.tokens ?? 0;
  if (!$("overlay").querySelector(".overlay-inner").classList.contains("menu") || $("overlay-body").querySelector(".smith")) {
    $("overlay-sub").textContent = `Du har ${tokens} tokens.`;
    $("overlay-sub").hidden = false;
  }
  for (const btn of document.querySelectorAll("#overlay-body .ware .lt-button[data-price]")) {
    const price = Number(btn.dataset.price);
    const short = price > tokens;
    btn.disabled = short;
    const why = document.getElementById(btn.getAttribute("aria-describedby"));
    if (why) why.textContent = short ? `Har ikke råd (mangler ${price - tokens} tokens)` : "";
  }
}

// Trubaduren (E): det han sier og ett kjøp, et krus mjød, så synger han
// kvadet om Gorvek. Har du ikke råd, er knappen av som i butikkene
// (`updateShop`); synger han allerede, er den av med en forklaring.
export function openMead(e) {
  const body = openOverlay(e.npc, "");
  const say = document.createElement("p");
  say.className = "lt-paragraph mead-text";
  say.textContent = `«${e.text}»`;
  const row = document.createElement("div");
  row.className = "ware mead";
  const text = document.createElement("div");
  text.className = "ware-text";
  const why = document.createElement("span");
  why.className = "lt-help-text ware-why";
  why.id = "ware-why-mead";
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "lt-button";
  btn.dataset.variant = "primary";
  btn.textContent = `Kjøp et krus mjød (${e.price} token)`;
  btn.setAttribute("aria-describedby", why.id);
  if (e.singing) {
    btn.disabled = true;
    why.textContent = "Han synger allerede for kruset ditt.";
  } else btn.dataset.price = String(e.price);
  btn.addEventListener("click", () => {
    if (btn.disabled) return;
    if (engine.buy_mead?.()) closeOverlay();
  });
  text.append(why);
  row.append(text, btn);
  body.append(say, row);
  updateShop();
}
