// Viser Skriptoriet, inventaret og butikken med falske data uten motoren
// (`?uitest` i main.js) og tar skjermbilder i tre størrelser:
//   node tools/ui-test.mjs DIST UT [WEB]
// DIST er den ferdige nettsiden (dist-web fra `sh tools/pull-dist.sh`, for
// fristil.css og pkg/), WEB er mappa med main.js, style.css og index.html
// (standard web/). Skriver UT-<bredde>-<meny>.png og feiler om en meny
// stikker utenfor vinduet eller lista ikke ruller for seg.
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join } from "node:path";
const [dist = "dist", out = "ui", web = "web"] = process.argv.slice(2);
const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json", ".png": "image/png", ".jpg": "image/jpeg" };
const roots = (p) => {
  if (p.startsWith("/components/")) return [join("assets/textures", p)];
  if (p.startsWith("/icons/")) return [join("assets/textures", p)];
  return [join(web, p), join(dist, p)];
};
const server = createServer(async (req, res) => {
  const p = decodeURIComponent(req.url.split("?")[0]).replace(/\/$/, "/index.html");
  for (const f of roots(p)) {
    try {
      const body = await readFile(f);
      res.writeHead(200, { "content-type": types[extname(f)] ?? "application/octet-stream" });
      res.end(body);
      return;
    } catch {
      /* neste */
    }
  }
  res.writeHead(404);
  res.end();
});
await new Promise((r) => server.listen(0, r));

// Utstyret og oppskriftene slik Rust sender dem (`gear_json`, `RecipeView`).
const stat = (label, value, text, up) => ({ label, value, text, up });
const W = (id, name, rarity, special, d, s, r, p, st, about) => ({ id, name, slot: "weapon", rarity, special, about, stats: [stat("Skade", d, `×${d.toFixed(2)}`, true), stat("Fart", s, `×${s.toFixed(2)}`, true), stat("Rekkevidde", r, `${r.toFixed(1)} m`, true), stat("Etter parade", p, `×${p.toFixed(1)}`, true), stat("Utholdenhet per hugg", st, `×${st.toFixed(2)}`, false)] });
const S = (id, name, rarity, b, pw, st, about) => ({ id, name, slot: "shield", rarity, special: "", about, stats: [stat("Slipper gjennom", b, `${Math.round(b * 100)} %`, false), stat("Paradevindu", pw, `${pw.toFixed(2)} s`, true), stat("Utholdenhet per blokk", st, `×${st.toFixed(2)}`, false)] });
const A = (id, name, rarity, t, rg, dg, about) => ({ id, name, slot: "armor", rarity, special: "", about, stats: [stat("Skade du tar", t, `${Math.round(t * 100)} %`, false), stat("Utholdenhet tilbake", rg, `×${rg.toFixed(2)}`, true), stat("Unnamanøver koster", dg, `×${dg.toFixed(2)}`, false)] });
const gear = {
  rusty_sword: W("rusty_sword", "Rustent sverd", "vanlig", "", 0.8, 1, 2.2, 2, 1, "Sverdet du kom til porten med. Rusten i eggen, men det biter ennå."),
  component_sword: W("component_sword", "Komponentsverd", "vanlig", "", 1, 1.05, 2.3, 2, 1, "Smidd etter mønsterboka av Button og Divider: rett, enkelt og til å stole på."),
  tag_dagger: W("tag_dagger", "Taggdolk", "vanlig", "dobbelt i ryggen", 0.7, 1.5, 1.6, 2, 0.6, "Kort og lett. Den som ser en annen vei, får stikket dobbelt i ryggen."),
  card_axe: W("card_axe", "Kortøks", "sjelden", "feier bredt", 1.45, 0.8, 2, 2, 1.3, "Bladet er slått av Card. Feier bredt og tar flere i samme hugg."),
  alert_mace: W("alert_mace", "Alertkølle", "sjelden", "knuser panser og balanse", 1.25, 0.85, 1.9, 2, 1.4, "Køllehodet ringer som stormklokka. Slår gjennom panser og velter balansen."),
  tooltip_stiletto: W("tooltip_stiletto", "Tooltipstikker", "sjelden", "dobbelt i ryggen", 0.95, 1.6, 1.7, 2.5, 0.6, "Tynn som en nål og like rask."),
  divider_blade: W("divider_blade", "Dividerklinge", "episk", "parade gir trippel", 1.35, 1.1, 2.5, 3, 1, "Skiller rent der leddene sitter, slik Gorvek gjorde. En parade gir trippelt."),
  tabs_axe: W("tabs_axe", "Tabsøks", "episk", "feier bredt", 1.8, 0.75, 2.1, 2, 1.5, "Den tyngste øksa i Fristad, smidd av delene fra den korrupte kjernen."),
  old_shield: S("old_shield", "Gammelt rundskjold", "vanlig", 0.25, 0.22, 1, "Et slitt rundskjold av bord og jernbeslag. Bedre enn ingenting."),
  card_shield: S("card_shield", "Kortskjold", "vanlig", 0.15, 0.3, 0.9, "Lett, blåmalt rundskjold."),
  badge_heater: S("badge_heater", "Badgeskjold", "sjelden", 0.1, 0.26, 1, "Trekantskjold med merket fra mønsterboka."),
  fieldset_tower: S("fieldset_tower", "Fieldset-tårnskjold", "episk", 0.03, 0.18, 1.5, "Et tårnskjold som et helt skjema."),
  dialog_heater: S("dialog_heater", "Dialogskjold", "episk", 0.08, 0.4, 0.8, "Fiolett trekantskjold med det lengste paradevinduet i byen."),
  badge_jerkin: A("badge_jerkin", "Badgevams", "vanlig", 0.85, 1, 1, "Lærvams med Badge sydd på brystet."),
  tag_cuirass: A("tag_cuirass", "Taggkyrass", "sjelden", 0.78, 1.05, 0.85, "Mørkt herdet lær."),
  fieldset_mail: A("fieldset_mail", "Fieldset-brynje", "sjelden", 0.65, 0.85, 1.2, "Ringbrynje knyttet ring for ring."),
  search_plate: A("search_plate", "Searchharnisk", "episk", 0.5, 0.7, 1.4, "Harnisk av stål."),
  dialog_plate: A("dialog_plate", "Dialogharnisk", "episk", 0.42, 0.8, 1.25, "Mørkt harnisk fra den korrupte kjernen, det beste vernet i riket."),
  search_lantern: { id: "search_lantern", name: "Søkelykt", slot: "lantern", rarity: "sjelden", special: "lyser lenger inn i mørket", about: "Lykt med slipt glass som kaster lyset dobbelt så langt inn i mørket.", stats: [stat("Bærelyset", 2.2, "×2.2", true)] },
};
const noArmor = A("", "Ingen rustning", "vanlig", 1, 1, 1, "Bare skjorta og kappa du kom til porten i.");
const owned = ["rusty_sword", "component_sword", "tag_dagger", "card_axe", "old_shield", "card_shield", "badge_jerkin"];
const worn = { weapon: "component_sword", shield: "old_shield", armor: "badge_jerkin" };
const view = (id, extra = {}) => ({ ...gear[id], equipped: worn[gear[id].slot] === id, owned: owned.includes(id), ...extra });
const stash = { button: 3, divider: 1, tag: 4, badge: 2, card: 1, alert: 1, progress: 1, label: 1 };
const src = { button: ["Kister i Gravkjelleren · Buttonsmeden 12 tokens", false], divider: ["Kister i Gravkjelleren og Kloakken", true], tag: ["Kister i Gravkjelleren · Buttonsmeden 8 tokens", false], badge: ["Kister i Kloakken · elite i Gravkjelleren", true], card: ["Kister i Kloakken og Gruva", true], alert: ["Elite i Kloakken", true], progress: ["Kister i Gruva", true], label: ["Kister i Gruva · Buttonsmeden 20 tokens", false], tooltip: ["Kister i Kloakken", true], "help-text": ["Elite i Gruva", true], search: ["Kister i Borgen under jorda", true], fieldset: ["Kister i Borgen under jorda · elite i Gruva", true], "file-upload": ["Elite i Borgen under jorda", true], tabs: ["Rammeverket", true], dialog: ["Kister i Den korrupte kjernen · Rammeverket", true] };
const R = (id, needs, requires = []) => {
  const item = gear[id];
  const n = needs.map(([c, k]) => [c, k, stash[c] ?? 0]);
  const q = requires.map((c) => [c, (stash[c] ?? 0) > 0]);
  const ok = n.every(([, k, h]) => h >= k) && q.every(([, h]) => h);
  const cur = item.slot === "lantern" ? null : item.slot === "armor" && !worn.armor ? { ...noArmor, equipped: true, owned: true } : view(worn[item.slot]);
  return { id, name: item.name, rarity: item.rarity, result: "", needs: n, requires: q, sources: [...needs.map(([c]) => c), ...requires].map((c) => [c, ...(src[c] ?? ["Krypten", true])]), ok, item: { ...view(id), equipped: false }, current: cur };
};
const recipes = [
  R("component_sword", [["button", 2], ["divider", 1]]),
  R("tag_dagger", [["tag", 3], ["button", 1]]),
  R("badge_jerkin", [["badge", 3], ["tag", 2]]),
  R("card_shield", [["card", 1], ["badge", 1]]),
  R("card_axe", [["button", 1], ["card", 2], ["badge", 1]], ["alert"]),
  R("tag_cuirass", [["tag", 3], ["badge", 2], ["progress", 1]]),
  R("alert_mace", [["alert", 2], ["progress", 1], ["card", 1]], ["label"]),
  R("tooltip_stiletto", [["tooltip", 2], ["help-text", 1], ["tag", 1]], ["label"]),
  R("badge_heater", [["badge", 2], ["label", 1], ["card", 1]], ["help-text"]),
  R("search_lantern", [["search", 1], ["progress", 1]], ["label", "help-text"]),
  R("fieldset_mail", [["fieldset", 2], ["badge", 3], ["tag", 3]], ["file-upload"]),
  R("divider_blade", [["divider", 3], ["fieldset", 1], ["alert", 1]], ["file-upload", "search"]),
  R("fieldset_tower", [["fieldset", 1], ["card", 2], ["label", 2]], ["file-upload"]),
  R("search_plate", [["search", 2], ["fieldset", 2], ["alert", 1]], ["file-upload", "label"]),
  R("tabs_axe", [["tabs", 1], ["dialog", 1], ["card", 2], ["fieldset", 1]]),
  R("dialog_heater", [["dialog", 1], ["search", 1], ["badge", 2]]),
  R("dialog_plate", [["dialog", 1], ["tabs", 1], ["fieldset", 2], ["search", 1]]),
];
recipes[0].ok = false; // komponentsverdet har du allerede
const inventory = {
  type: "inventory",
  stash: Object.entries(stash),
  carried: [["tooltip", 1]],
  weapon: "Komponentsverd (vanlig)",
  weapon_special: "",
  shield: "Gammelt rundskjold (vanlig)",
  armor: "Badgevams (vanlig)",
  lantern: false,
  max_hp: 120,
  max_stamina: 100,
  tokens: 34,
  read: ["button"],
  gear: owned.map((id) => view(id)),
};
const wares = [
  { id: "button", name: "Button", price: 12 },
  { id: "tag", name: "Tag", price: 8 },
  { id: "label", name: "Label", price: 20 },
  { id: "oil", name: "Lampeolje (full lykt)", price: 6 },
  { id: "potion", name: "Helsedrikk", price: 15 },
  { id: "big_potion", name: "Fullt liv", price: 22 },
  { id: "bandage", name: "Bind", price: 4 },
  { id: "whet", name: "Brynestein", price: 10 },
  { id: "rope", name: "Tau", price: 5 },
  { id: "candle", name: "Talglys", price: 2 },
];
// Byttene hos Buttonsmeden (`SwapView`, plan 14).
const R2 = { button: "vanlig", tag: "vanlig", badge: "vanlig", divider: "vanlig", card: "vanlig", progress: "vanlig", label: "vanlig", "help-text": "vanlig", alert: "sjelden", tooltip: "sjelden", fieldset: "sjelden", "file-upload": "sjelden", search: "sjelden", dialog: "episk", tabs: "episk" };
const SW = (id, give, n, get, fee) => ({ id, give, n, have: stash[give] ?? 0, give_rarity: R2[give], get, get_have: stash[get] ?? 0, get_rarity: R2[get], fee, source: src[get]?.[0] ?? "Krypten", ok: (stash[give] ?? 0) >= n && 34 >= fee });
const swaps = [SW("tag_divider", "tag", 3, "divider", 4), SW("badge_divider", "badge", 3, "divider", 4), SW("badge_card", "badge", 3, "card", 6), SW("button_progress", "button", 3, "progress", 6), SW("card_alert", "card", 3, "alert", 8), SW("divider_label", "divider", 3, "label", 8), SW("card_help", "card", 3, "help-text", 8), SW("label_tooltip", "label", 3, "tooltip", 10), SW("label_fieldset", "label", 3, "fieldset", 12), SW("alert_search", "alert", 3, "search", 12), SW("alert_upload", "alert", 3, "file-upload", 12), SW("fieldset_dialog", "fieldset", 2, "dialog", 20), SW("search_tabs", "search", 2, "tabs", 20)];

const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", headless: true });
let bad = 0;
for (const [w, h] of [[1920, 1080], [1280, 720], [390, 844]]) {
  for (const menu of ["workshop", "workshop-detail", "inventory", "inventory-armor", "inventory-lantern", "shop", "smith-swap", "smith-swap-detail", "smith-buy", "mead", "mead-poor", "mead-singing"]) {
    const page = await browser.newPage({ viewport: { width: w, height: h } });
    // TEXTSIZE=1.4 prøver den største tekststørrelsen (Innstillinger, `fristad.textsize`).
    if (process.env.TEXTSIZE) await page.addInitScript((v) => localStorage.setItem("fristad.textsize", v), process.env.TEXTSIZE);
    page.on("pageerror", (e) => console.log("sidefeil:", e.message));
    if (menu.startsWith("smith")) await page.addInitScript((tab) => localStorage.setItem("fristad.shop.tab", tab), menu === "smith-buy" ? "buy" : "swap");
    await page.route("**/*.wasm", (r) => r.abort());
    await page.goto(`http://127.0.0.1:${server.address().port}/?uitest&nofilm`);
    await page.waitForFunction(() => window.fristadUi);
    await page.evaluate(async ({ menu, inventory, recipes, wares, swaps }) => {
      const docs = await (await fetch("docs.json")).json();
      const engine = { set_paused() {}, craft: () => false, equip: () => true, buy: () => true, swap: () => false, read_doc() {} };
      const ui = window.fristadUi;
      ui.set({ engine, docs, hud: { hp: 96, stamina: 80, tokens: 34, keys: 1, light: 62, cellar: false } });
      document.getElementById("gate").hidden = true;
      document.getElementById("hud").hidden = false;
      ui.handle({ type: "hud", hp: 96, stamina: 80, tokens: 34, keys: 1, light: 62, cellar: false, boss: null });
      ui.handle(inventory);
      if (menu.startsWith("workshop")) ui.handle({ type: "workshop", recipes });
      if (menu.startsWith("inventory")) ui.toggleInventory();
      if (menu === "shop") ui.handle({ type: "shop", npc: "Apotekeren", wares });
      if (menu.startsWith("smith")) ui.handle({ type: "shop", npc: "Buttonsmeden", wares: wares.slice(0, 4), swaps });
      // Trubaduren (E): et krus mjød for kvadet om Gorvek, uten råd og mens han synger.
      if (menu === "mead-poor") ui.handle({ type: "hud", hp: 96, stamina: 80, tokens: 0, keys: 1, light: 62, cellar: false, boss: null });
      const offer = "Et krus mjød, fremmede, så synger jeg kvadet om Gorvek: han som kom gjennom regnet med øksa over skulderen og hogg beistet der leddene satt, natta gjennom. Men strupen er tørr, og kvadet er langt.";
      if (menu.startsWith("mead")) ui.handle({ type: "mead", npc: "Trubaduren", text: menu === "mead-singing" ? "Han er midt i kvadet, og kruset ditt står ennå ved krakken." : offer, price: 1, singing: menu === "mead-singing" });
    }, { menu, inventory, recipes, wares, swaps });
    await page.waitForTimeout(300);
    if (menu === "smith-swap-detail") {
      await page.locator(".smith .pick-row").nth(5).click({ timeout: 2000 }).catch(() => {});
      await page.waitForTimeout(200);
    }
    if (menu === "workshop-detail") {
      // Velg en oppskrift lenger ned (på smal skjerm åpner den detaljene).
      await page.locator("[data-recipe]").nth(4).click({ timeout: 2000 }).catch(() => {});
      await page.waitForTimeout(200);
    }
    if (menu === "inventory-armor" || menu === "inventory-lantern") {
      // Plassen er filteret; på smal skjerm åpner raden detaljene.
      await page.locator(`[data-slot="${menu.slice(10)}"]`).first().click({ timeout: 2000 }).catch(() => {});
      await page.waitForTimeout(200);
      if (w <= 720) await page.locator(".pick-row").first().click({ timeout: 2000 }).catch(() => {});
      await page.waitForTimeout(200);
    }
    const r = await page.evaluate(() => {
      const box = document.querySelector("dialog[open]") ?? document.querySelector("#overlay:not([hidden]) .overlay-inner");
      if (!box) return { missing: true };
      const b = box.getBoundingClientRect();
      const list = box.querySelector(".pick-list");
      const bars = document.getElementById("bars");
      return {
        x: b.x, y: b.y, w: b.width, h: b.height,
        scrolls: list ? getComputedStyle(list).overflowY : "",
        listH: list?.clientHeight ?? 0, listS: list?.scrollHeight ?? 0,
        bars: bars.getBoundingClientRect().height > 0 && getComputedStyle(bars).visibility !== "hidden",
      };
    });
    const off = r.missing || r.x < 0 || r.y < 0 || r.x + r.w > w + 0.5 || r.y + r.h > h + 0.5;
    const tall = !r.missing && r.h > Math.min(h * 0.9, 800);
    const fail = off || r.bars || (menu !== "shop" && tall);
    console.log(`${w}x${h} ${menu}: ${r.missing ? "FEIL ingen meny" : `${Math.round(r.w)}×${Math.round(r.h)}${r.listS ? ` liste ${r.listH}/${r.listS} (${r.scrolls})` : ""}${off ? " FEIL utenfor" : ""}${tall ? " FEIL for høy" : ""}${r.bars ? " FEIL stolpene synes" : ""}`}`);
    if (fail) bad++;
    await page.screenshot({ path: `${out}-${w}-${menu}.png` });
    await page.close();
  }
}
await browser.close();
server.close();
process.exit(bad ? 1 : 0);
