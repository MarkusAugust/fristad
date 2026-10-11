"""Godkjenningssiden: eierens dom over hver tekstur og modell (`docs/plan.md` 0c).

    python3 -m tools.qa approvals page UT      # bilder og data til siden i mappa UT
    python3 -m tools.qa approvals apply SVAR   # svarene fra siden inn i approvals.json

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
        json.dump({"items": order}, f, ensure_ascii=False)
    shutil.copy(os.path.join(HERE, "approvals.html"), os.path.join(out, "index.html"))
    shutil.rmtree(raw)
    return order


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
        if ap.setdefault(group, {}).get(name) != entry:
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
    if argv[:1] == ["apply"] and len(argv) == 2:
        print(f"{apply(argv[1])} dommer ført inn i assets/catalog/approvals.json")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
