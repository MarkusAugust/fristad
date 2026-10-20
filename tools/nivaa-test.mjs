// Nivåredigeringen i nettleseren (`?nivaa`, web/nivaa.js): kjører den ferdige
// nettsiden i Chromium uten skjerm og sjekker at kartet, paletten og 3D kommer
// opp, at en modul kan settes ut med musa (fra paletten til kartet), at angre
// virker, at reglene vises, at prøveturen starter og Esc går tilbake, og at
// Lagre uten nøkkel gir en zip (utkastet huskes), «Rett» ved bruddene,
// snittet i 3D og et utvalg som ny modul. Skjermbildene skrives i UT.
//   node tools/nivaa-test.mjs KATALOG [webgl|webgpu] [UT-mappe]
// Nettsiden hentes fra grenen web-dist: `sh tools/pull-dist.sh`.
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { readFile, stat, mkdir } from "node:fs/promises";
import { extname, join } from "node:path";

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_DIR ?? "playwright");
const [dir = "dist", mode = "webgl", out = "target/nivaa-test"] = process.argv.slice(2);
await mkdir(out, { recursive: true });
const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".css": "text/css", ".json": "application/json", ".png": "image/png", ".jpg": "image/jpeg" };
const server = createServer(async (req, res) => {
  const path = join(dir, decodeURIComponent(req.url.split("?")[0]).replace(/\/$/, "/index.html"));
  try {
    await stat(path);
    res.writeHead(200, { "content-type": types[extname(path)] ?? "application/octet-stream" });
    res.end(await readFile(path));
  } catch {
    res.writeHead(404);
    res.end();
  }
});
await new Promise((r) => server.listen(0, r));
const port = server.address().port;
const args =
  mode === "webgpu"
    ? ["--enable-unsafe-webgpu", "--enable-features=Vulkan", "--use-angle=swiftshader", "--use-vulkan=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"]
    : ["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"];
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM ?? "/opt/pw-browsers/chromium", args, headless: true });
const page = await browser.newPage({ viewport: { width: 1600, height: 900 }, acceptDownloads: true });
const errors = [];
page.on("console", (m) => {
  if (m.type() === "error") errors.push(m.text());
  console.log(`[${m.type()}] ${m.text().slice(0, 300)}`);
});
page.on("pageerror", (e) => {
  errors.push(e.message);
  console.log(`[pageerror] ${e.message}`);
});
page.on("response", (r) => r.status() >= 400 && !r.url().endsWith(".jpg") && console.log(`[http ${r.status()}] ${r.url()}`));

let failed = 0;
const check = (ok, what) => {
  console.log(`${ok ? "OK  " : "FEIL"} ${what}`);
  if (!ok) failed++;
};
const shot = async (name) => {
  await page.screenshot({ path: `${out}/${name}.png`, timeout: 300000 });
  console.log(`[test] ${out}/${name}.png`);
};

// `?nivaa` på forsiden sender videre til nivaa.html.
await page.goto(`http://127.0.0.1:${port}/?nivaa${mode === "webgl" ? "&webgl" : ""}&fil=test`, { waitUntil: "load" });
const t0 = Date.now();
await page.waitForFunction(() => window.fristad?.engine && document.querySelector("#overlay")?.hidden, null, { timeout: 600000 });
console.log(`[test] klar etter ${Math.round((Date.now() - t0) / 1000)} s på ${await page.evaluate(() => window.fristad.engine.backend())}`);
check(page.url().includes("nivaa.html"), "?nivaa sender til nivaa.html");
await page.waitForTimeout(3000);
const map0 = await page.evaluate(() => window.fristad.state.map);
check(map0 && map0.modules.length >= 10, `prøvenivået er åpnet (${map0?.modules.length} moduler)`);
check(await page.evaluate(() => document.querySelectorAll("#modlist .tile").length) >= 10, "paletten har modulene");
check(await page.evaluate(() => [...document.querySelectorAll("#modlist img")].some((i) => i.complete && i.naturalWidth > 0)), "bildene av modulene kommer");
// Kartet er tegnet (ikke bare bakgrunn).
const painted = await page.evaluate(() => {
  const c = document.querySelector("#plan");
  const d = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
  let n = 0;
  for (let i = 0; i < d.length; i += 16) if (d[i] > 60) n++;
  return n;
});
check(painted > 500, `kartet er tegnet (${painted} lyse punkter)`);
// 3D: noe er tegnet (WebGL: les tilbake pikslene fra et kopi-canvas).
const litNow = () =>
  page.evaluate(async () => {
    const c = document.querySelector("#world");
    const tmp = document.createElement("canvas");
    tmp.width = 160;
    tmp.height = 90;
    // Uten preserveDrawingBuffer kan bildet være tømt: les i neste bilde,
    // og prøv noen ganger.
    let best = 0;
    for (let k = 0; k < 5 && best < 1000; k++) {
      // I samme bilde som motoren tegnet (dens løkke ligger før i køen).
      await new Promise((r) =>
        requestAnimationFrame(() => {
          tmp.getContext("2d").clearRect(0, 0, 160, 90);
          tmp.getContext("2d").drawImage(c, 0, 0, 160, 90);
          r();
        }),
      );
      const d = tmp.getContext("2d").getImageData(0, 0, 160, 90).data;
      let n = 0;
      for (let i = 0; i < d.length; i += 4) if (d[i] + d[i + 1] + d[i + 2] > 60) n++;
      best = Math.max(best, n);
    }
    return best;
  });
const lit = await litNow();
check(lit > 200, `3D-vinduet viser nivået (${lit} lyse punkter av 14400)`);
await shot("1_prøvenivået");

// Sett ut en modul: dra «Rom 2 × 2» fra paletten og slipp på kartet øst for nivået.
const n0 = map0.modules.length;
const tile = page.locator('#modlist .tile[data-module="grav/rom_2x2"]');
const target = await page.evaluate(() => {
  const st = window.fristad.state;
  // Under nivået (sør), ved den venstre kanten: fritt og innenfor kartet.
  const cells = st.map.modules.flatMap((m) => m.cells);
  const i = Math.min(...cells.map((c) => c[0])) + 1;
  const j = Math.max(...cells.map((c) => c[2])) + 2;
  const r = document.querySelector("#plan").getBoundingClientRect();
  const [px, py] = st.view.toScreen(i * 4, j * 4);
  return [r.left + px, r.top + py, i];
});
const tb = await tile.boundingBox();
await page.mouse.move(tb.x + tb.width / 2, tb.y + tb.height / 2);
await page.mouse.down();
await page.mouse.move(target[0] - 30, target[1], { steps: 8 });
await page.mouse.move(target[0], target[1], { steps: 4 });
await page.mouse.up();
await page.waitForTimeout(500);
let map1 = await page.evaluate(() => window.fristad.state.map);
check(map1.modules.length === n0 + 1, `en modul er satt ut med musa (${n0} → ${map1.modules.length})`);
const added = map1.modules.find((m) => m.module === "grav/rom_2x2" && !map0.modules.some((o) => o.id === m.id));
check(!!added && added.at[0] === target[2], `modulen står der den ble sluppet (${added?.at})`);
await page.waitForTimeout(1500);
await shot("2_modul_satt_ut");
// R dreier, angre og gjør om.
await page.keyboard.press("KeyR");
await page.waitForTimeout(300);
map1 = await page.evaluate(() => window.fristad.state.map);
check(map1.modules.find((m) => m.id === added?.id)?.rot === 1, "R dreier det valgte");
await page.keyboard.press("Control+KeyZ");
await page.keyboard.press("Control+KeyZ");
await page.waitForTimeout(300);
check((await page.evaluate(() => window.fristad.state.map.modules.length)) === n0, "Ctrl+Z angrer");
await page.keyboard.press("Control+KeyY");
await page.waitForTimeout(300);
check((await page.evaluate(() => window.fristad.state.map.modules.length)) === n0 + 1, "Ctrl+Y gjør om");
// Reglene: et rom i lufta øst for alt annet har ingen vei inn: «kan ikke nås»
// meldes for innhold der. Sett en kiste i det.
await page.evaluate((i) => window.fristad.level({ op: "add_thing", kind: "chest", at: [i * 4 + 40, 0, 40] }), target[2]);
const issues = await page.evaluate(() => [...document.querySelectorAll("#issues li")].map((l) => l.textContent));
check(issues.some((t) => t.includes("gulvet") || t.includes("nås")), `reglene sier fra (${issues.length} i lista)`);
await page.locator("#issues li.feil").first().click();
await page.waitForTimeout(400);
await shot("3_reglene");
await page.keyboard.press("Control+KeyZ");
// Åpningen: klikk en pil og velg port.
await page.locator("#plan").click({ position: { x: 5, y: 5 }, button: "right" });
await page.keyboard.press("Home");
await page.waitForTimeout(300);
const opening = await page.evaluate(() => {
  const st = window.fristad.state;
  const m = st.map.modules.find((m) => m.module === "grav/rom_2x2") || st.map.modules[0];
  const o = m.openings[0];
  const d = [[0, -1], [1, 0], [0, 1], [-1, 0]][o.side];
  const r = document.querySelector("#plan").getBoundingClientRect();
  const [px, py] = st.view.toScreen(o.at[0] * 4 + d[0] * 1.8, o.at[1] * 4 + d[1] * 1.8);
  return [r.left + px, r.top + py, m.id, o.id];
});
await page.mouse.click(opening[0], opening[1]);
await page.waitForTimeout(300);
check(await page.evaluate(() => !!document.querySelector("#op-kind")), "klikk på en åpning gir valget i panelet");
if (await page.evaluate(() => !!document.querySelector("#op-kind"))) {
  await page.selectOption("#op-kind", "port");
  await page.waitForTimeout(400);
  const ov = await page.evaluate((m) => window.fristad.state.map.modules.find((x) => x.id === m)?.overrides, opening[2]);
  check(ov?.some((o) => o.kind === "port"), "åpningen er en port");
}
// ---- Fase 4 ----
// Paletten: gruppene, og bildene for alle temaene.
const groups = await page.evaluate(() => [...document.querySelectorAll("#modlist h2")].map((h) => h.dataset.group));
check(["rom", "gang", "trapp", "mekanisme", "bosshall"].every((g) => groups.includes(g)), `paletten er gruppert (${groups.join(", ")})`);
for (const theme of ["grav", "kloakk", "gruve", "borg", "kjerne"]) {
  await page.selectOption("#theme", theme);
  await page.waitForTimeout(300);
  const imgs = await page.evaluate(async () => {
    const vis = [...document.querySelectorAll("#modlist .tile")].filter((t) => !t.hidden);
    for (const t of vis) t.querySelector("img").loading = "eager";
    await new Promise((r) => setTimeout(r, 1500));
    return [vis.length, vis.filter((t) => t.querySelector("img").naturalWidth > 0).length];
  });
  check(imgs[0] >= 15 && imgs[1] === imgs[0], `paletten for ${theme}: ${imgs[1]} av ${imgs[0]} bilder`);
}
await page.selectOption("#theme", "grav");
await shot("8_paletten_gruppert");
// Stemningen: velg kapellet og gi det tåke, lyd og kald farge.
await page.keyboard.press("Escape");
const kapell = await page.evaluate(() => (window.fristad.state.map.modules.find((m) => m.module === "grav/kapell") || window.fristad.state.map.modules[0]).id);
await page.evaluate((id) => window.fristad.select([id]), kapell);
await page.waitForTimeout(300);
check(await page.evaluate(() => !!document.querySelector("#mood-fog")), "panelet har stemningen");
await page.evaluate(() => {
  const set = (id, v) => {
    const el = document.querySelector(id);
    el.value = v;
    el.dispatchEvent(new Event("change"));
  };
  set("#mood-fog", "0.6");
  set("#mood-sound", "vann");
  set("#mood-color", "kald");
  set("#mood-light", "0.6");
});
await page.waitForTimeout(500);
const mood = await page.evaluate((id) => window.fristad.state.map.modules.find((m) => m.id === id)?.mood, kapell);
const near = (a, b) => Math.abs(a - b) < 1e-4;
check(mood && near(mood.fog, 0.6) && mood.sound === "vann" && near(mood.light, 0.6) && mood.color, `stemningen er lagret (${JSON.stringify(mood)})`);
await shot("9_stemningen");
// Et lys eieren setter ut, og panelet for det.
const lightId = await page.evaluate((id) => {
  const m = window.fristad.state.map.modules.find((x) => x.id === id);
  const c = m.cells[0];
  const out = window.fristad.level({ op: "add_light", at: [c[0] * 4 + 1, 2.4, c[2] * 4 + 1] });
  const l = out.made[0];
  window.fristad.select([l]);
  return l;
}, kapell);
await page.waitForTimeout(300);
check(await page.evaluate(() => !!document.querySelector("#l-color") && !!document.querySelector("#l-flicker")), `lyset ${lightId} har farge og flamme i panelet`);
await page.evaluate(() => {
  const el = document.querySelector("#l-int");
  el.value = "4.5";
  el.dispatchEvent(new Event("change"));
});
await page.waitForTimeout(300);
check(await page.evaluate((id) => window.fristad.state.map.lights.find((l) => l.id === id)?.own, lightId), "lyset står på kartet som eierens");
await shot("10_lyset");
// Spillets lys med tåka i 3D.
await page.click("#light");
await page.waitForTimeout(1500);
await shot("11_spillets_lys_og_take");
await page.click("#light");
// Malens innhold: løsne kista i en mekanisme og flytt den.
const mech = await page.evaluate(() => {
  const st = window.fristad.state;
  const cells = st.map.modules.flatMap((m) => m.cells);
  const i = Math.max(...cells.map((c) => c[0])) + 4;
  const out = window.fristad.place("grav/porthall", [i, 0, 0]);
  const t = out.things.find((x) => x.kind === "chest" && x.module === out.made[0]);
  return { id: t?.id, pos: t?.pos };
});
check(!!mech.id, `porthallen har kista ${mech.id}`);
if (mech.id) {
  await page.evaluate((m) => window.fristad.level({ op: "move_thing", id: m.id, to: [m.pos[0] + 1, m.pos[1], m.pos[2]] }), mech);
  const t = await page.evaluate((id) => window.fristad.state.map.things.find((x) => x.id === id), mech.id);
  check(t?.own && t?.template, `kista er flyttet i nivået og husker malen (${t?.template})`);
  await page.evaluate((id) => window.fristad.select([id]), mech.id);
  await page.waitForTimeout(200);
  check(await page.evaluate(() => !!document.querySelector("#t-back")), "«Tilbake til malen» i panelet");
  await shot("12_malens_innhold");
}
// Etasjene: ny etasje over, og snittet.
await page.selectOption("#floor", { index: (await page.evaluate(() => document.querySelector("#floor").options.length)) - 2 });
await page.waitForTimeout(300);
const fl = await page.evaluate(() => window.fristad.state.floor);
check(fl >= 6, `ny etasje over (${fl} m)`);
await page.click("#cut");
await page.waitForTimeout(200);
check(await page.evaluate(() => window.fristad.state.cut), "snittet er på");
await shot("13_ny_etasje");
await page.click("#cut");
await page.evaluate(() => window.fristad.setFloor(0));
// Bokmerker: Shift+1 husker, 1 går tilbake.
await page.locator("#plan").focus();
const v0 = await page.evaluate(() => [window.fristad.state.view.cx, window.fristad.state.view.cz]);
await page.keyboard.press("Shift+Digit1");
await page.evaluate(() => {
  window.fristad.state.view.cx += 40;
});
await page.keyboard.press("Digit1");
const v1 = await page.evaluate(() => [window.fristad.state.view.cx, window.fristad.state.view.cz]);
check(Math.abs(v1[0] - v0[0]) < 0.01 && Math.abs(v1[1] - v0[1]) < 0.01, "bokmerke 1 tar kartet tilbake");
// Malene: lagre utvalget (Ctrl+G) og sett det ut igjen dreid.
await page.evaluate((id) => window.fristad.select([id]), kapell);
page.once("dialog", (d) => d.accept("Prøvemal"));
await page.locator("#plan").focus();
await page.keyboard.press("Control+KeyG");
await page.waitForTimeout(400);
const tplChip = page.locator('#tpllist .chip[data-key="tpl:provemal"]');
check((await tplChip.count()) === 1, "malen står i paletten");
if (await tplChip.count()) {
  const nBefore = await page.evaluate(() => window.fristad.state.map.modules.length);
  await tplChip.click();
  await page.keyboard.press("KeyR");
  // En ledig plass på 3 × 3 ruter (kapellet) som synes i kartet (ikke under minikartet).
  const spot = await page.evaluate(() => {
    const st = window.fristad.state;
    st.view.s = 5;
    const taken = new Set(st.map.modules.flatMap((m) => m.cells.filter((c) => Math.abs(c[1]) < 3).map((c) => `${c[0]},${c[2]}`)));
    const r = document.querySelector("#plan").getBoundingClientRect();
    for (let d = 0; d < 30; d++) {
      for (let i = -d; i <= d; i++) {
        for (let j = -d; j <= d; j++) {
          const ci = Math.round(st.view.cx / 4) + i;
          const cj = Math.round(st.view.cz / 4) + j;
          let free = true;
          for (let a = -1; a <= 1; a++) for (let b = -1; b <= 1; b++) free = free && !taken.has(`${ci + a},${cj + b}`);
          const [px, py] = st.view.toScreen(ci * 4, cj * 4);
          if (free && px > 60 && py > 60 && px < r.width - 220 && py < r.height - 200) return [r.left + px, r.top + py];
        }
      }
    }
    return null;
  });
  check(!!spot, "en ledig plass til malen i kartet");
  if (spot) {
    await page.mouse.move(spot[0], spot[1]);
    await page.mouse.click(spot[0], spot[1]);
  }
  await page.waitForTimeout(400);
  const after = await page.evaluate(() => window.fristad.state.map);
  check(after.modules.length === nBefore + 1 && after.modules.at(-1).rot === 1, `malen er satt ut dreid (${after.modules.length - nBefore} ny, rot ${after.modules.at(-1)?.rot})`);
  await page.keyboard.press("Home");
  await shot("14_malen_satt_ut");
}
// «Rett» (del 27): en tom lapp gir tekstfeltet, og en spak uten mål kobles
// til nærmeste port med ett klikk (og angres).
await page.keyboard.press("Escape");
const kc = await page.evaluate((id) => window.fristad.state.map.modules.find((m) => m.id === id).cells[0], kapell);
await page.evaluate((c) => window.fristad.level({ op: "add_thing", kind: "lore", at: [c[0] * 4 + 0.5, c[1], c[2] * 4 + 0.5] }), kc);
await page.waitForTimeout(200);
const loreFix = page.locator("#issues .chip.fix", { hasText: "Skriv teksten" });
check((await loreFix.count()) >= 1, "den tomme lappen har «Skriv teksten»");
if (await loreFix.count()) {
  await loreFix.first().click();
  await page.waitForTimeout(200);
  check(await page.evaluate(() => document.activeElement?.id === "t-text"), "«Skriv teksten» går til tekstfeltet");
}
await page.evaluate((c) => window.fristad.level({ op: "add_thing", kind: "lever", at: [c[0] * 4 - 0.5, c[1], c[2] * 4 - 0.5] }), kc);
await page.waitForTimeout(200);
const hasGate = await page.evaluate(() => window.fristad.state.map.things.some((t) => ["gate", "bridge", "water", "lift"].includes(t.kind)));
const leverFix = page.locator("#issues .chip.fix", { hasText: "Koble til" });
check(!hasGate || (await leverFix.count()) >= 1, `spaken uten mål har «Koble til …» (${hasGate ? "en port finnes" : "ingen port i nivået"})`);
if (await leverFix.count()) {
  await leverFix.first().click();
  await page.waitForTimeout(300);
  check(await page.evaluate(() => !window.fristad.state.map.issues.some((i) => i.msg.includes("åpner ingenting"))), "spaken er koblet etter «Rett»");
  await shot("17_rett");
}
// Snittet i 3D: kameraet over kapellet ser ned; med snittet kommer rommet fram.
await page.evaluate((c) => window.fristad.level({ cmd: "place", x: c[0] * 4, y: c[1] + 14, z: c[2] * 4 + 12, yaw: 0, pitch: -50 }), kc);
await page.click("#v-3d");
const lum3d = () =>
  page.evaluate(async () => {
    const c = document.querySelector("#world");
    const tmp = document.createElement("canvas");
    tmp.width = 160;
    tmp.height = 90;
    await new Promise((r) => requestAnimationFrame(() => (tmp.getContext("2d").drawImage(c, 0, 0, 160, 90), r())));
    const d = tmp.getContext("2d").getImageData(0, 0, 160, 90).data;
    let sum = 0;
    for (let i = 0; i < d.length; i += 4) sum += d[i] + d[i + 1] + d[i + 2];
    return sum / (d.length / 4) / 3;
  });
await page.waitForTimeout(1500);
const before3d = await lum3d();
await shot("18_uten_snitt_3d");
await page.click("#cut");
await page.waitForTimeout(1500);
const after3d = await lum3d();
check(Math.abs(after3d - before3d) > 0.3, `snittet endrer 3D (${before3d.toFixed(1)} → ${after3d.toFixed(1)})`);
await shot("19_snitt_3d");
await page.click("#cut");
await page.click("#v-split");
// Et utvalg som ny modul (Ctrl+Shift+G): i paletten uten bilde, og kan settes ut.
await page.evaluate((id) => window.fristad.select([id]), kapell);
page.once("dialog", (d) => d.accept("Prøvemodul"));
await page.locator("#plan").focus();
await page.keyboard.press("Control+Shift+KeyG");
await page.waitForTimeout(600);
const own = await page.evaluate(() => {
  const t = [...document.querySelectorAll("#modlist .tile")].find((b) => b.dataset.module.endsWith("/provemodul"));
  return t ? { id: t.dataset.module, missing: !!t.querySelector(".noimg") || !t.querySelector("img")?.naturalWidth } : null;
});
check(!!own, `den nye modulen står i paletten (${own?.id})`);
check(own?.missing, "paletten viser at bildet mangler");
if (own) {
  const nBefore = await page.evaluate(() => window.fristad.state.map.modules.length);
  const free = await page.evaluate(() => {
    const st = window.fristad.state;
    const taken = new Set(st.map.modules.flatMap((m) => m.cells.map((c) => `${c[0]},${c[2]}`)));
    for (let i = 20; i < 60; i += 4) if (![...Array(5).keys()].some((a) => taken.has(`${i + a - 2},-12`))) return [i, 0, -12];
    return [40, 0, -12];
  });
  const out = await page.evaluate(([id, at]) => window.fristad.place(id, at), [own.id, free]);
  check(out && !out.error && out.modules.length === nBefore + 1, "den nye modulen settes ut som en vanlig modul");
  check(await page.evaluate(() => !window.fristad.state.map.issues.some((i) => i.rule === "modul" || i.rule === "bit")), "ingen brudd i modulen eller bitene");
  check(await page.evaluate(() => Object.keys(JSON.parse(localStorage.getItem("fristad.nivaa.moduler") || "{}")).length === 1), "modulen huskes i nettleseren til den lagres");
  await shot("20_ny_modul");
}
// Minikartet: klikk flytter kartet.
await page.keyboard.press("Escape");
const mini = await page.evaluate(() => {
  const r = document.querySelector("#plan").getBoundingClientRect();
  // Øverst til venstre i minikartet (midten er der Home allerede står).
  return [r.right - 170, r.bottom - 150, window.fristad.state.view.cx];
});
await page.mouse.click(mini[0], mini[1]);
await page.waitForTimeout(200);
check((await page.evaluate(() => window.fristad.state.view.cx)) !== mini[2], "klikk i minikartet flytter kartet");
// Blob-sha-en til konfliktsjekken er den samme som git sin.
check((await page.evaluate(() => import("./github.js").then((m) => m.blobSha("hei")))) === "986ebda7e895742cf7961bb0476ee98699d01def", "blob-sha som git (konfliktsjekken)");
// Skyene: kameraet høyt over nivået ser ingen himmel.
await page.evaluate(() => window.fristad.level({ cmd: "place", x: 0, y: 80, z: 60, yaw: 0, pitch: 10 }));
await page.click("#v-3d");
await page.waitForTimeout(1500);
const sky = await page.evaluate(async () => {
  const c = document.querySelector("#world");
  const tmp = document.createElement("canvas");
  tmp.width = 160;
  tmp.height = 90;
  await new Promise((r) => requestAnimationFrame(() => (tmp.getContext("2d").drawImage(c, 0, 0, 160, 90), r())));
  const d = tmp.getContext("2d").getImageData(0, 0, 160, 30).data;
  let sum = 0;
  for (let i = 0; i < d.length; i += 4) sum += d[i] + d[i + 1] + d[i + 2];
  return sum / (d.length / 4) / 3;
});
check(sky < 12, `ingen himmel eller skyer over nivået i 3D (lysstyrke ${sky.toFixed(1)})`);
await shot("15_ingen_himmel");
await page.click("#v-split");
// Delt skjerm og full størrelse.
await page.keyboard.press("Escape");
// Kameraet tilbake i kapellet (det forrige «gå dit» sto ved kista i fjellet).
await page.evaluate(() => {
  const m = window.fristad.state.map.modules.find((m) => m.module === "grav/kapell") || window.fristad.state.map.modules[0];
  const c = m.cells[Math.floor(m.cells.length / 2)];
  window.fristad.level({ cmd: "place", x: c[0] * 4, y: c[1] + 1.7, z: c[2] * 4 + 1, yaw: 0, pitch: -5 });
});
await page.click("#v-3d");
await page.waitForTimeout(1500);
const lit3d = await litNow();
check(lit3d > 1000, `bare 3D fyller vinduet (${lit3d} lyse punkter)`);
await shot("4_bare_3d");
await page.click("#v-map");
await page.waitForTimeout(400);
await shot("5_bare_kartet");
await page.click("#v-split");
await page.waitForTimeout(800);
// Prøveturen og Esc tilbake.
const nBeforePlay = await page.evaluate(() => window.fristad.state.map.modules.length);
await page.click("#play");
await page.waitForFunction(() => document.body.classList.contains("playing"), null, { timeout: 600000 }).catch(() => {});
const playing = await page.evaluate(() => window.fristad.engine.level_playing());
check(playing, "Spill fra start bygger verden og starter prøveturen");
await page.waitForTimeout(3000);
const litPlay = await litNow();
check(litPlay > 500, `prøveturen viser nivået (${litPlay} lyse punkter)`);
await shot("6_proveturen");
await page.keyboard.press("Escape");
await page.waitForTimeout(800);
check(!(await page.evaluate(() => window.fristad.engine.level_playing())), "Esc går tilbake til redigeringen");
const nPlay = await page.evaluate(() => window.fristad.state.map.modules.length);
check(nPlay === nBeforePlay, "redigeringen er som den var etter prøveturen");
// Lagre uten nøkkel: zip, og utkastet er i nettleseren.
page.once("dialog", (d) => d.accept(""));
const dl = page.waitForEvent("download", { timeout: 90000 }).catch(() => null);
await page.click("#save", { force: true });
const file = await dl;
check(!!file && file.suggestedFilename().endsWith(".zip"), `Lagre uten nøkkel laster ned ${file?.suggestedFilename()}`);
const draft = await page.evaluate(() => Object.keys(localStorage).filter((k) => k.startsWith("fristad.nivaa.utkast:")));
check(draft.length > 0, `utkastet er i nettleseren (${draft.join(", ")})`);
// Last siden på nytt: utkastet åpnes.
await page.reload({ waitUntil: "load" });
await page.waitForFunction(() => window.fristad?.engine && document.querySelector("#overlay")?.hidden, null, { timeout: 600000 });
check((await page.evaluate(() => window.fristad.state.map.modules.length)) === nBeforePlay, "utkastet åpnes igjen");
await page.waitForTimeout(2000);
await shot("7_etter_omlasting");
// «Merk klart»: et lite nivå 1 merket klart spilles som nivå 1 (det samme
// som erstatter det gamle i spillet, `Data::compose_levels`).
const small = await page.evaluate(() => {
  const f = window.fristad;
  const text = f.blank("Lite nivå", 1, "grav");
  const out = JSON.parse(f.engine.level_open(text));
  if (out.error) return out.error;
  f.level({ op: "add_module", module: "grav/nedgang", at: [2, 0, 0], rot: 1 });
  f.level({ op: "add_thing", kind: "rest", at: [0, 0, 2] });
  return null;
});
check(!small, `et lite nivå 1 (${small || "ok"})`);
await page.waitForTimeout(500);
await page.locator("#ready").click();
await page.waitForTimeout(500);
check((await page.evaluate(() => window.fristad.state.map.status)) === "klar", "«Merk klart» merker nivået klart");
await page.click("#play");
await page.waitForFunction(() => document.body.classList.contains("playing"), null, { timeout: 600000 }).catch(() => {});
await page.waitForTimeout(3000);
check(await page.evaluate(() => window.fristad.engine.level_playing()), "det klare nivået spilles som nivå 1");
await shot("16_klart_niva_1");
await page.keyboard.press("Escape");
check(!errors.some((e) => /panic|unreachable|RuntimeError/.test(e)), `ingen krasj i konsollen (${errors.length} feil)`);
console.log(failed ? `[test] ${failed} sjekker feilet` : "[test] alt i orden");
await browser.close();
server.close();
process.exit(failed ? 1 : 0);
