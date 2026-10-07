// Henter CC0-materialer og modeller fra Poly Haven til assets/external/.
//
//   bun tools/fetch-assets.ts            henter det som mangler
//   bun tools/fetch-assets.ts --force    henter alt på nytt
//   bun tools/fetch-assets.ts --list     viser bare hva som ville blitt valgt
//
// Lista under er kuratert: for hvert teksturlag i motoren (se `Layer` i
// src/domain/geometry.rs) og hver modell byen kan bruke, står søkeordene i
// prioritert rekkefølge. Skriptet slår opp i Poly Havens API, velger første
// treff, og laster ned 1k JPEG (teksturer) eller 1k glTF (modeller).
// Resultatet beskrives i assets/external/manifest.json, som motoren leser.
//
// Kjøres både i GitHub Actions (nettversjonen) og lokalt. Katalogen er ikke
// i git, så eieren kan committe den om Claude skal se materialene i
// skjermbildene sine.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const API = "https://api.polyhaven.com";
const OUT = "assets/external";
const RES = "1k";

/** Teksturlag → søkeord (navn eller tagg), i prioritert rekkefølge. */
const TEXTURES: Record<string, string[]> = {
  cobble: ["cobblestone_floor_08", "cobblestone_large", "cobblestone_floor", "cobblestone"],
  plaster: ["plastered_wall_02", "plastered_wall", "painted_plaster", "plaster"],
  slate: ["roof_slates", "slate_roof", "slates"],
  tiles: ["roof_tiles_14", "roof_tiles", "clay_roof"],
  stone: ["stone_wall_04", "castle_wall", "stone_wall", "stone_bricks"],
  brick: ["red_brick_03", "brick_wall", "red_brick", "brick"],
  oak: ["wood_table_001", "oak_veneer", "wood_floor_deck", "wood_floor"],
  metal: ["metal_plate_02", "painted_metal", "metal_plate", "rusty_metal"],
  grass: ["grass_medium_01", "leafy_grass", "grass_path", "grass"],
  bark: ["bark_willow", "bark_brown", "pine_bark", "bark"],
  gravel: ["gravel_floor", "gravel_road", "gravel"],
  concrete: ["concrete_floor_02", "concrete_wall", "concrete_floor", "concrete"],
  timber: ["wood_planks_dirt", "wooden_planks", "pine_planks", "planks"],
  plywood: ["plywood", "osb", "chipboard", "particle_board"],
};

/**
 * Modeller: navnet motoren ber om → søkeord, ønsket høyde i meter (0 lar
 * modellens egne mål stå, siden Poly Haven er i meter) og rotasjon.
 */
const MODELS: Record<string, { search: string[]; height: number; yaw: number }> = {
  barrel: { search: ["wooden_barrel", "barrel"], height: 0, yaw: 0 },
  crate: { search: ["wooden_crate", "crate"], height: 0, yaw: 0 },
  planter: { search: ["flower_pot", "planter", "plant_pot"], height: 0, yaw: 0 },
  cart: { search: ["wheelbarrow", "wooden_cart", "hand_cart", "wagon"], height: 0, yaw: 0 },
};

const force = process.argv.includes("--force");
const listOnly = process.argv.includes("--list");

type Asset = { name: string; tags?: string[]; categories?: string[]; dimensions?: number[]; type: number };

async function json<T>(url: string): Promise<T> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  return (await res.json()) as T;
}

async function download(url: string, path: string) {
  if (existsSync(path) && !force) return;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(path, new Uint8Array(await res.arrayBuffer()));
}

/** Første asset som matcher et av søkeordene, i rekkefølge. */
function pick(assets: Record<string, Asset>, searches: string[]): string | undefined {
  const ids = Object.keys(assets);
  for (const s of searches) {
    const exact = ids.find((id) => id === s);
    if (exact) return exact;
    const byName = ids.find((id) => id.includes(s));
    if (byName) return byName;
    const byTag = ids.find((id) => (assets[id].tags ?? []).includes(s) || (assets[id].categories ?? []).includes(s));
    if (byTag) return byTag;
  }
  return undefined;
}

/** Finner en fil i Poly Havens filtre: `files[key][res][format]`. */
function file(files: any, keys: string[], res: string, formats: string[]) {
  for (const k of keys) {
    const r = files?.[k]?.[res] ?? files?.[k]?.["2k"];
    if (!r) continue;
    for (const f of formats) if (r[f]) return r[f];
  }
  return undefined;
}

const manifest: any = { textures: {}, models: {}, source: "https://polyhaven.com (CC0)" };

const textures = await json<Record<string, Asset>>(`${API}/assets?t=textures`);
for (const [layer, searches] of Object.entries(TEXTURES)) {
  const id = pick(textures, searches);
  if (!id) {
    console.warn(`${layer}: fant ingen tekstur for ${searches.join(", ")}`);
    continue;
  }
  const a = textures[id];
  // Målene er i millimeter på Poly Haven; standard 2 m om de mangler.
  const size = a.dimensions?.[0] ? a.dimensions[0] / 1000 : 2;
  console.log(`${layer.padEnd(9)} ← ${id} (${a.name}, ${size} m)`);
  manifest.textures[layer] = { id, name: a.name, size_m: size };
  if (listOnly) continue;
  const files = await json<any>(`${API}/files/${id}`);
  const dir = join(OUT, "textures", layer);
  const want: [string, string[], string[]][] = [
    ["diffuse", ["Diffuse", "diff", "Diff"], ["jpg", "png"]],
    ["normal", ["nor_gl", "Normal_gl", "nor"], ["jpg", "png"]],
    ["rough", ["Rough", "rough", "Roughness"], ["jpg", "png"]],
    ["ao", ["AO", "ao"], ["jpg", "png"]],
  ];
  for (const [name, keys, formats] of want) {
    const f = file(files, keys, RES, formats);
    if (!f) continue;
    const ext = f.url.split(".").pop();
    await download(f.url, join(dir, `${name}.${ext}`));
    manifest.textures[layer][name] = `textures/${layer}/${name}.${ext}`;
  }
}

const models = await json<Record<string, Asset>>(`${API}/assets?t=models`);
for (const [name, spec] of Object.entries(MODELS)) {
  const id = pick(models, spec.search);
  if (!id) {
    console.warn(`${name}: fant ingen modell for ${spec.search.join(", ")}`);
    continue;
  }
  console.log(`${name.padEnd(9)} ← ${id} (${models[id].name})`);
  manifest.models[name] = { id, name: models[id].name, height: spec.height, yaw: spec.yaw };
  if (listOnly) continue;
  const files = await json<any>(`${API}/files/${id}`);
  const g = files?.gltf?.[RES]?.gltf ?? files?.gltf?.["2k"]?.gltf;
  if (!g) {
    console.warn(`${name}: ${id} har ingen glTF`);
    delete manifest.models[name];
    continue;
  }
  const dir = join(OUT, "models", name);
  const main = `model.gltf`;
  await download(g.url, join(dir, main));
  const included: string[] = [];
  for (const [rel, f] of Object.entries<any>(g.include ?? {})) {
    await download(f.url, join(dir, rel));
    included.push(rel);
  }
  manifest.models[name].gltf = `models/${name}/${main}`;
  manifest.models[name].files = included;
}

if (!listOnly) {
  mkdirSync(OUT, { recursive: true });
  writeFileSync(join(OUT, "manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
  console.log(`Skrev ${OUT}/manifest.json`);
}
