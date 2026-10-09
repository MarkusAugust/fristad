// Teksturlagene til motoren, samlet i assets/external/ med manifest.json:
//
// 1. Bilder eieren har laget (bildegenerator eller hånd) i
//    assets/textures/<lag>/: albedo.png (eller .jpg), og normal, rough og
//    height som tools/textures/derive.py avleder. Disse går foran.
// 2. Fotoskannede CC0-materialer fra Poly Haven for lagene som mangler.
//
//   bun tools/fetch-assets.ts            henter det som mangler
//   bun tools/fetch-assets.ts --force    henter alt på nytt
//   bun tools/fetch-assets.ts --list     viser bare hva som ville blitt valgt
//
// Kjøres i GitHub Actions (nettversjonen, og grenen external-assets som
// `sh tools/pull-assets.sh` henter til skymiljøet). Lagnavnene er de i
// `Layer::name` i src/domain/geometry.rs.

import { copyFileSync, existsSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const API = "https://api.polyhaven.com";
const OUT = "assets/external";
const OWN = "assets/textures";
const RES = "1k";

/**
 * Teksturlag → søkeord på Poly Haven i prioritert rekkefølge, og hvor
 * mange meter én flis dekker om Poly Haven ikke sier det. Valgt for en
 * nordisk kjøpstad rundt 1350.
 */
const TEXTURES: Record<string, string[]> = {
  // Brostein på torget og i Storgata.
  cobble: ["cobblestone_floor_04", "cobblestone_floor_07", "cobblestone_floor_08", "cobblestone"],
  // Kalkpuss mellom bindingsverket.
  plaster: ["plastered_wall_04", "plastered_wall_05", "plastered_wall", "plaster"],
  // Grå, værbitte bord (kledning, skur, boder).
  cladding: ["weathered_planks", "old_planks", "wood_planks_grey", "planks"],
  // Skifer på takene.
  slate: ["roof_slates_02", "roof_slates_03", "roof_slates", "slate"],
  // Spon og halm på takene.
  tiles: ["thatch_roof_angled", "thatch", "roof_shingles", "shingles"],
  // Bruddstein i murene.
  stone: ["castle_wall_slates", "medieval_wall_01", "stone_wall_04", "castle_wall", "stone_wall"],
  // Hogd stein i borgen og kirka.
  brick: ["castle_brick_02", "castle_brick", "stone_brick_wall", "brick"],
  // Mørk, grovhogd eik i bjelkene.
  oak: ["wood_planks_dirt", "dark_wood", "old_wood", "wood_table"],
  // Smijern med rust.
  metal: ["rusty_metal_02", "rust_coarse_01", "rusty_metal", "metal"],
  // Eng.
  grass: ["aerial_grass_rock", "grass_path_2", "leafy_grass", "grass"],
  // Bark.
  bark: ["bark_brown_02", "bark_willow", "pine_bark", "bark"],
  // Tråkket jord og søle på veiene.
  gravel: ["brown_mud_leaves_01", "forest_ground_01", "brown_mud", "dirt", "mud"],
  // Steinheller i krypten.
  concrete: ["stone_tiles_02", "floor_tiles_06", "flagstone", "stone_floor"],
  // Lysere tømmer (laft, planker).
  timber: ["wood_planks", "pine_planks", "wooden_planks", "planks"],
  // Skogbunn med strå og løv.
  plywood: ["forest_leaves_02", "forrest_ground_01", "forest_floor", "leaves"],
  // Berg.
  rock: ["rock_face", "cliff_side", "rock_wall", "rock"],
  snow: ["snow_02", "snow_field", "snow"],
  // Tråkket jord uten hjulspor på gårdsplassene.
  thatch: ["thatch_roof_angled", "thatch"],
  logs: ["log_wall", "logs"],
  sod: ["grass_roof", "moss"],
  // Klærne og læret til figurene.
  wool: ["wool", "fabric"],
  linen: ["linen", "fabric"],
  leather: ["leather"],
  cryptwall: ["damp_wall", "stone_wall"],
  straw: ["straw", "hay"],
  floor: ["wood_floor", "planks"],
  soot: ["burnt_wall", "stone_wall"],
  earth: ["brown_mud_02", "dirt_floor", "mud_forest", "dirt"],
  // Malte lag fra eieren (bare egne bilder).
  plaster_white: [],
  plaster_red: [],
  plaster_old: [],
  weatherboard: [],
  weatherboard_grey: [],
  wattle: [],
  shingles: [],
  shingles_tar: [],
  trim: [],
  door_oak: [],
  door_dark: [],
  timber_grey: [],
  church_boards: [],
  ruin: [],
  wall_stone: [],
  flagstone: [],
  pebbles: [],
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

const manifest: any = { textures: {}, source: "assets/textures (egne) og https://polyhaven.com (CC0)" };

// 1. Egne bilder.
if (existsSync(OWN)) {
  for (const layer of readdirSync(OWN)) {
    const dir = join(OWN, layer);
    const files = existsSync(dir) ? readdirSync(dir) : [];
    const find = (name: string) => files.find((f) => f.startsWith(`${name}.`));
    const albedo = find("albedo");
    if (!albedo || !(layer in TEXTURES)) continue;
    const entry: any = { id: `egen:${layer}`, name: `Egen ${layer}`, size_m: 2 };
    try {
      const meta = JSON.parse(await Bun.file(join(dir, "meta.json")).text());
      if (meta.size_m) entry.size_m = meta.size_m;
    } catch {
      /* standard 2 m */
    }
    const out = join(OUT, "textures", layer);
    mkdirSync(out, { recursive: true });
    for (const [key, name] of [["diffuse", "albedo"], ["normal", "normal"], ["rough", "rough"], ["height", "height"], ["ao", "ao"]]) {
      const f = find(name);
      if (!f) continue;
      const ext = f.split(".").pop();
      if (!listOnly) copyFileSync(join(dir, f), join(out, `${key}.${ext}`));
      entry[key] = `textures/${layer}/${key}.${ext}`;
    }
    console.log(`${layer.padEnd(9)} ← egne bilder (${entry.size_m} m)`);
    manifest.textures[layer] = entry;
  }
}

// 2. Poly Haven for resten. Når Poly Haven ikke kan nås (skymiljøet),
// beholdes lagene fra forrige manifest.
let previous: any = { textures: {} };
try {
  previous = JSON.parse(await Bun.file(join(OUT, "manifest.json")).text());
} catch {
  /* første gang */
}
let textures: Record<string, Asset> = {};
try {
  textures = await json<Record<string, Asset>>(`${API}/assets?t=textures`);
} catch (e) {
  console.warn(`Poly Haven kan ikke nås (${e}); beholder lagene som er hentet fra før`);
  for (const [layer, entry] of Object.entries<any>(previous.textures ?? {})) {
    if (!manifest.textures[layer] && !String(entry.id ?? "").startsWith("egen:")) manifest.textures[layer] = entry;
  }
}
for (const [layer, searches] of Object.entries(TEXTURES)) {
  if (manifest.textures[layer] || Object.keys(textures).length === 0) continue;
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
    ["height", ["Displacement", "disp", "Height"], ["jpg", "png"]],
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

if (!listOnly) {
  mkdirSync(OUT, { recursive: true });
  writeFileSync(join(OUT, "manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
  console.log(`Skrev ${OUT}/manifest.json`);
}
