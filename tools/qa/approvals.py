"""Godkjenningssiden: eierens dom over hver tekstur og modell (`docs/plan.md` 0c).

    python3 -m tools.qa approvals page UT      # bilder og data til siden i mappa UT
    python3 -m tools.qa approvals apply SVAR   # svarene fra siden inn i approvals.json
    python3 -m tools.qa approvals round SPEC STUDIO UT  # en runde med bare det som er endret
    python3 -m tools.qa approvals new SPEC STUDIO UT    # en runde med nye modeller

`page` bygger på katalogen (`docs/katalog/katalog.json`, så `catalog` først):
hver modell som står ute fotograferes i spillet med `snapshot --shots`, i
øyehøyde foran modellen (eller fra retningen i `VIEW` der noe står i
veien), med modellen alene som skisse i hjørnet; de som bare poseres av
spillet får skissen alene. Hver tekstur vises som laget spillet bruker,
lagt 2 × 2 så skjøtene synes. Rekkefølgen er hvor mye tingen synes
(`ORDER`). Siden selv er `tools/qa/approvals.html`; den lagrer svarene i
samlingen `dom` (dokument `t_<lag>` eller `m_<modell>`: `verdict`, `why`,
`at`).

`apply` leser svarene (en mappe med ett json-dokument per ting, `<id>.json`,
slik ArtifactData lagrer samlingen med `out_dir`, eller en json-liste med
`{id, data}`) og skriver dem inn i `assets/catalog/approvals.json`. Eieren
bestemmer; dette fører bare inn.
"""
import json
import math
import os
import shutil
import subprocess
import sys

from PIL import Image

from .catalog import DATA, IMG
from .gate import ROOT
from .glb import Mesh
from .raster import render

HERE = os.path.dirname(os.path.abspath(__file__))
SNAPSHOT = os.path.join(ROOT, "target", "release", "snapshot")
VERDICTS = ("godkjent", "underkjent", "usikker")

# Hvor mye tingen synes i byen: først husene på torget og innsidene deres,
# så de vanlige husene, muren, kirka og torget, så småtingene, gårdene,
# portalen, galleriet og krypten. Det som ikke står her, kommer etter
# antall instanser.
ORDER = [
    "tavern", "tavern_interior", "forge", "forge_interior", "apothecary", "apothecary_interior",
    "scriptorium", "scriptorium_interior", "home", "home_interior",
    *[f"house_{c}" for c in "abcdefghijklmnopqrst"],
    "town_wall", "wall_tower", "gatehouse", "stave_church", "stave_church_interior", "crypt",
    "market_cross", "well", "well_winch", "well_rope", "well_bucket", "market_stall", "market_stall_b",
    "lantern_post", "signpost", "barrel", "crate", "chest", "chest_lid", "woodpile",
    "training_dummy", "weapon_rack", "picket_fence", "rail_fence", "wattle_fence", "stone_fence",
    "shed_a", "shed_b", "privy", "coop", "haystack", "garden", "cart",
    "portal_ring", "portal_runes", "portal_swirl", "portal_shard",
]

# Retningen kameraet står i der rett forfra havner inni noe: k × 45 grader
# rundt modellen fra forsida (valgt med øyet 11. okt. fra åtte bilder rundt).
VIEW = {
    "haystack": 3, "house_a": 5, "house_c": 3, "house_f": 5, "house_i": 7, "house_l": 2,
    "house_m": 1, "house_n": 3, "house_p": 1, "house_r": 4, "house_s": 2, "kit_ladder": 2,
    "kit_plate": 4, "privy": 1, "picket_fence": 3, "rail_fence": 3, "stone_fence": 3,
    "wall_tower": 7, "wattle_fence": 3,
}
SHOT = (800, 560)

# Runden eieren dømmer i. En dom fra en tidligere runde vises bare som
# «Forrige dom»; feltet og knappene starter tomme, så ingenting arves.
# Økes for hver ny runde (1: alt, 11. okt.; 2: nye bilder, 11. okt.; 3: husene, 12. okt.; 4: hønsehuset og høystakken; 5: møblene, 12. okt.; 6: møblene rettet).
ROUND = 6

# Nye bilder som eieren bør se på igjen: modellene med disse lagene ble
# fotografert uten lagenes bilder (hvite dører, «murstein», 11. okt.).
# Siden merker dem «Nytt bilde», med grunnen.
RESHOT = {
    "layers": {"door", "door_b", "ashlar", "planks", "iron", "oak_planed", "straw_floor"},
    "why": "Forrige bilde var feil: dørene, hogd stein, planker, jern og høvlet eik manglet teksturen sin og ble hvite eller murstein. Se om dommen din står.",
}


def rank(models):
    pos = {n: i for i, n in enumerate(ORDER)}
    return sorted(models, key=lambda m: (pos.get(m["name"], len(ORDER)), m["name"].startswith("crypt_01"), -m["instances"], m["name"]))


def shot_line(m, out):
    """Linja til `snapshot --shots`: kameraet i modellens rom (Blenders
    akser, −y er forsida) i øyehøyde, så langt unna at modellen fyller bildet."""
    sx, sy, sz = m["size"]
    n = m["name"]
    if n.endswith("_interior"):
        return f"{out} {n} {-sx * 0.3:.2f},{-sz * 0.3:.2f},1.6 0 -5"
    big = max(sx, sy)
    k = VIEW.get(n, 0)
    r = (max(sx, sz) / 2 if k else sz / 2) + (big * 0.85 if big > 3 else max(1.4, big * 1.6))
    eye = 1.7
    pitch = -35.0 if n == "kit_plate" else math.degrees(math.atan2(sy * 0.45 - eye, r))
    a = math.radians(45 * k)
    return f"{out} {n} {r * math.sin(a):.2f},{-r * math.cos(a):.2f},{eye} 0 {pitch:.1f}"


def sketch(m, size):
    path = os.path.join(ROOT, "assets", "models", m["file"])
    return Image.fromarray(render(Mesh(path), size, background=(34, 36, 40)))


def page(out):
    with open(os.path.join(IMG, "katalog.json")) as f:
        cat = json.load(f)
    with open(os.path.join(DATA, "approvals.json")) as f:
        prior = json.load(f)
    with open(os.path.join(DATA, "models.json")) as f:
        about = json.load(f)["models"]
    img = os.path.join(out, "img")
    raw = os.path.join(out, "raw")
    os.makedirs(img, exist_ok=True)
    os.makedirs(raw, exist_ok=True)
    models = rank(cat["models"])

    # Bildene fra spillet, i én kjøring.
    placed = [m for m in models if m["instances"] > 0 and not m["name"].startswith("crypt_01_")]
    lines = [shot_line(m, os.path.join(raw, m["name"] + ".png")) for m in placed]
    with open(os.path.join(raw, "shots.txt"), "w") as f:
        f.write("\n".join(lines) + "\n")
    subprocess.run([SNAPSHOT, "--shots", os.path.join(raw, "shots.txt"), "--size", f"{SHOT[0]}x{SHOT[1]}", "--frames", "2"], check=True, cwd=ROOT)

    items = []
    for m in models:
        n = m["name"]
        shot = os.path.join(raw, n + ".png")
        if os.path.exists(shot):
            im = Image.open(shot).convert("RGB")
            s = sketch(m, 180)
            im.paste((70, 72, 78), (SHOT[0] - 192, SHOT[1] - 192, SHOT[0] - 8, SHOT[1] - 8))
            im.paste(s, (SHOT[0] - 190, SHOT[1] - 190))
        else:
            im = Image.new("RGB", SHOT, (34, 36, 40))
            im.paste(sketch(m, SHOT[1]), ((SHOT[0] - SHOT[1]) // 2, 0))
        im.save(os.path.join(img, f"m_{n}.jpg"), quality=84)
        where = f"står {m['instances']} steder" if m["instances"] else ("poseres av spillet" if m["reserve"] else "står ingen steder")
        items.append({
            "id": f"m_{n}", "kind": "modell", "name": n, "about": about.get(n, ""),
            "facts": [f"{m['triangles']:,} trekanter".replace(",", " "), where, " × ".join(f"{v:.1f}" for v in m["size"]) + " m",
                      ("teksturlag: " + ", ".join(m["layers"])) if m["layers"] else "bakt tekstur"],
            "check": None, "game": os.path.exists(shot), "prior": prior.get("models", {}).get(n),
            "fresh": RESHOT["why"] if RESHOT["layers"] & set(m["layers"]) and (prior.get("models", {}).get(n) or {}).get("verdict") != "godkjent" else None,
        })

    # Teksturene: laget 2 × 2 (eller den malte kilden om laget mangler).
    first = {}
    for i, m in enumerate(models):
        for layer in m["layers"]:
            first.setdefault(layer, i)
    textures = sorted(cat["textures"], key=lambda t: (first.get(t["name"], len(models)), -len(t["users"]), t["name"]))
    for t in textures:
        n = t["name"]
        lay = os.path.join(ROOT, "assets", "textures", n, "albedo.jpg")
        src = os.path.join(ROOT, "assets", "textures", "painted", n, "albedo.png")
        tile = t["info"].get("kind", "tile") == "tile"
        im = Image.open(lay if os.path.exists(lay) else src).convert("RGB")
        if tile:
            im = im.resize((320, 320), Image.LANCZOS)
            big = Image.new("RGB", (640, 640))
            for x in (0, 320):
                for y in (0, 320):
                    big.paste(im, (x, y))
            im = big
        else:
            im.thumbnail((640, 640), Image.LANCZOS)
        im.save(os.path.join(img, f"t_{n}.jpg"), quality=86)
        users = t["users"]
        items.append({
            "id": f"t_{n}", "kind": "tekstur", "name": n, "about": t["info"].get("shows", ""),
            "facts": ["flis, vist 2 × 2" if tile else "ark", ("brukt av " + ", ".join(users[:6]) + (f" og {len(users) - 6} til" if len(users) > 6 else "")) if users else "ingen modell bruker den",
                      "" if t["layer"] else "bare malt, ikke tatt i bruk"],
            "check": "består teksturkontrollen" if t["ok"] else "teksturkontrollen: " + ", ".join(t["failed"]),
            "game": t["layer"], "prior": prior.get("textures", {}).get(n),
        })
    # Det viktigste først: modellene og teksturene flettet etter rekkefølgen,
    # så teksturene står like etter den første modellen som bruker dem.
    order = []
    ts = {f"t_{t['name']}": first.get(t["name"], len(models)) for t in textures}
    for i, it in enumerate(i for i in items if i["kind"] == "modell"):
        order.append(it)
        order += [x for x in items if x["kind"] == "tekstur" and ts[x["id"]] == i]
    order += [x for x in items if x["kind"] == "tekstur" and ts[x["id"]] >= len(models)]
    assert len(order) == len(items)
    with open(os.path.join(out, "data.json"), "w") as f:
        json.dump({"round": ROUND, "items": order}, f, ensure_ascii=False)
    shutil.copy(os.path.join(HERE, "approvals.html"), os.path.join(out, "index.html"))
    shutil.rmtree(raw)
    return order


def round_page(spec_path, studio, out):
    """En runde med bare det som er endret (`assets/catalog/rounds/N.json`):
    hvert kort viser eierens punkter (`history` i approvals.json), hvordan
    hvert er rettet, og før- og etterbildene fra `tools/qa/studio.py`
    (`before_<modell>_<vinkel>.png` og `after_...` i mappa `studio`) side om
    side."""
    with open(spec_path) as f:
        spec = json.load(f)
    with open(os.path.join(DATA, "approvals.json")) as f:
        prior = json.load(f)
    with open(os.path.join(DATA, "models.json")) as f:
        about = json.load(f)["models"]
    img = os.path.join(out, "img")
    os.makedirs(img, exist_ok=True)
    items = []
    for n, it in spec["items"].items():
        p = prior.get("models", {}).get(n) or {}
        views = sorted({f[len(f"before_{n}_"):-4] for f in os.listdir(studio) if f.startswith(f"before_{n}_") and f.endswith(".png")})
        pairs = []
        for v in views:
            pair = []
            for side in ("before", "after"):
                src = os.path.join(studio, f"{side}_{n}_{v}.png")
                if not os.path.exists(src):
                    raise SystemExit(f"mangler {src}")
                Image.open(src).convert("RGB").save(os.path.join(img, f"m_{n}_{v}_{side}.jpg"), quality=86)
                pair.append(f"img/m_{n}_{v}_{side}.jpg")
            pairs.append(pair)
        items.append({
            "id": f"m_{n}", "kind": "modell", "name": n, "about": about.get(n, ""), "facts": [],
            "check": None, "game": True, "prior": {k: p[k] for k in ("verdict", "date", "why") if k in p} or None,
            "points": p.get("history") or ([p["why"]] if p.get("why") else []),
            "fixed": [spec["fix"][k] for k in it["fix"]], "pairs": pairs,
        })
    with open(os.path.join(out, "data.json"), "w") as f:
        json.dump({"round": spec.get("round", ROUND), "items": items}, f, ensure_ascii=False)
    shutil.copy(os.path.join(HERE, "approvals.html"), os.path.join(out, "index.html"))
    return items


def new_page(spec_path, studio, out):
    """En runde med nye modeller (`assets/catalog/rounds/N.json` med
    `new`: navnene): hvert kort viser studiobildene fra `tools/qa/studio.py`
    (`<modell>_<vinkel>.png` i mappa `studio`) side om side, hva den er,
    trekantene og husene den står i (`assets/layout/inside/`)."""
    import struct
    with open(spec_path) as f:
        spec = json.load(f)
    with open(os.path.join(DATA, "models.json")) as f:
        about = json.load(f)["models"]
    where = {}
    inside = os.path.join(ROOT, "assets", "layout", "inside")
    for fn in sorted(os.listdir(inside)):
        with open(os.path.join(inside, fn)) as f:
            for it in json.load(f):
                where.setdefault(it.get("model"), []).append(fn[:-5])
    houses = {"tavern": "Vertshuset", "home": "spillerens hus", "forge": "Smia", "apothecary": "Apoteket", "scriptorium": "Skriptoriet", "stave_church": "stavkirka"}
    img = os.path.join(out, "img")
    os.makedirs(img, exist_ok=True)
    with open(os.path.join(DATA, "approvals.json")) as f:
        prior = json.load(f).get("models", {})
    reshot = spec.get("reshot", {})
    items = []
    for n in spec["new"] + reshot.get("names", []):
        order = {"corner": 0, "front": 1, "back": 2}
        views = sorted((f for f in os.listdir(studio) if f.startswith(f"{n}_") and f.endswith(".png") and f[len(n) + 1:-4] in order), key=lambda f: order[f[len(n) + 1:-4]])
        if not views:
            raise SystemExit(f"mangler studiobilder av {n}")
        ims = [Image.open(os.path.join(studio, v)).convert("RGB") for v in views]
        w, h = ims[0].size
        sheet = Image.new("RGB", (w * len(ims), h))
        for k, im in enumerate(ims):
            sheet.paste(im, (k * w, 0))
        sheet.thumbnail((1200, 1200), Image.LANCZOS)
        sheet.save(os.path.join(img, f"m_{n}.jpg"), quality=86)
        old = n in reshot.get("names", [])
        b = open(os.path.join(ROOT, "assets", "models", "town" if old else "furniture", f"{n}.glb"), "rb").read()
        doc = json.loads(b[20:20 + struct.unpack("<I", b[12:16])[0]])
        tris = sum(doc["accessors"][q["indices"]]["count"] // 3 for m in doc["meshes"] for q in m["primitives"])
        hs = where.get(n, [])
        at = ", ".join(sorted({houses.get(x, x) for x in hs}))
        items.append({
            "id": f"m_{n}", "kind": "modell", "name": n, "about": about.get(n, ""),
            "facts": [f"{tris:,} trekanter".replace(",", " "), (f"står ett sted: {at}" if len(hs) == 1 else f"står {len(hs)} steder: {at}") if hs else "står ingen steder"],
            "check": None, "game": False, "studio": True, "prior": prior.get(n) if old else None,
            "fresh": reshot.get("why") if old else None,
        })
        if old:
            items[-1]["facts"] = items[-1]["facts"][:1]
    with open(os.path.join(out, "data.json"), "w") as f:
        json.dump({"round": spec["round"], "items": items}, f, ensure_ascii=False)
    shutil.copy(os.path.join(HERE, "approvals.html"), os.path.join(out, "index.html"))
    return items


def apply(path):
    """Svarene fra siden inn i approvals.json. `path` er en mappe med
    `<id>.json` (`{verdict, why, at}`) eller en json-liste med `{id, data}`."""
    if os.path.isdir(path):
        rows = []
        for name in sorted(os.listdir(path)):
            if name.endswith(".json"):
                with open(os.path.join(path, name)) as f:
                    rows.append({"id": name[:-5], "data": json.load(f)})
    else:
        with open(path) as f:
            rows = json.load(f)
    p = os.path.join(DATA, "approvals.json")
    with open(p) as f:
        ap = json.load(f)
    n = 0
    for row in rows:
        d = row["data"]
        if d.get("verdict") not in VERDICTS:
            continue
        kind, name = row["id"].split("_", 1)
        group = {"t": "textures", "m": "models"}[kind]
        entry = {"verdict": d["verdict"], "date": d.get("at", "")[:10], "why": d.get("why", "").strip()}
        old = ap.setdefault(group, {}).get(name)
        # Ny dom: den forrige tas vare på. Og har grunnen ikke endret seg med
        # dommen, ble den arvet fra forrige runde (feltet var fylt ut, 11.
        # okt.), så den hører til den gamle dommen, ikke den nye.
        if old and old["verdict"] != entry["verdict"]:
            if old.get("why", "") == entry["why"]:
                entry["why"] = ""
            entry["before"] = {k: old[k] for k in ("verdict", "date", "why")}
        elif old and old.get("before"):
            entry["before"] = old["before"]
        # Alle punktene eieren har skrevet om tingen, på tvers av rundene,
        # så ingen grunn blir borte når dommen står eller grunnen endres.
        seen = list((old or {}).get("history", []))
        for w in ((old or {}).get("why", ""), d.get("why", "").strip()):
            if w and w not in seen:
                seen.append(w)
        if seen:
            entry["history"] = seen
        if old != entry:
            ap[group][name] = entry
            n += 1
    with open(p, "w") as f:
        json.dump(ap, f, indent=1, ensure_ascii=False)
        f.write("\n")
    return n


def main(argv):
    if argv[:1] == ["page"] and len(argv) == 2:
        items = page(argv[1])
        print(f"{argv[1]}: {len(items)} ting")
        return 0
    if argv[:1] == ["round"] and len(argv) == 4:
        items = round_page(argv[1], argv[2], argv[3])
        print(f"{argv[3]}: {len(items)} ting i runden")
        return 0
    if argv[:1] == ["new"] and len(argv) == 4:
        items = new_page(argv[1], argv[2], argv[3])
        print(f"{argv[3]}: {len(items)} nye ting i runden")
        return 0
    if argv[:1] == ["apply"] and len(argv) == 2:
        print(f"{apply(argv[1])} dommer ført inn i assets/catalog/approvals.json")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
