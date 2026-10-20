"""Dommen som tilstand, ikke runder (eieren 13. okt.: «det virker som at du
ikke klarer å følge med på hva jeg har sett på og hva du skal vise meg
igjen»).

Hver modell og tekstur har en versjon: git-blobben til fila spillet bruker
(`.glb` eller det malte bildet). En dom i `approvals.json` gjelder versjonen
den ble gitt for (`seen`). Hvert kort står i én av tre bunker:

- `venter`: ikke vurdert, eller endret siden eieren så det (versjonen er en
  annen enn `seen`) – det eieren skal se, med før og etter;
- `hos_meg`: underkjent eller usikker og ikke endret ennå – det jeg skal
  rette, og som ikke vises eieren igjen før det er rettet;
- `godkjent`: godkjent, og ikke endret siden.

Og to som eieren ikke dømmer én og én (`assets/catalog/modules.json`):
`i_modul` for bitene og byggesettets mekanismer, som dømmes gjennom
modulene og de samlede kortene (`mekanismer`: tingene i bruk i et rom), og
`utgaar` for det som erstattes (bitene i de gamle kryptnivåene), som står
sammenfoldet nederst med dommene eieren alt har gitt.

Siden (`approvals.html`) viser alle tingene i faner per kategori, med det
som venter først. Bare kortene som venter, har bilder: ett samlet bilde per
kort (før og etter, alle vinklene), så siden holder seg langt under
grensen på filer. `check` stopper en publisering der noe som venter
mangler bilde, eller det er for mange.

    python3 -m tools.qa approvals migrate          # én gang: `seen` fra svartidene
    python3 -m tools.qa approvals review UT        # bygg siden i UT
    python3 -m tools.qa approvals check UT         # det som venter er med
"""
import datetime
import glob
import json
import math
import os
import shutil
import subprocess
import tempfile

from .gate import DATA, ROOT


class _Lazy:
    """Pillow lastes først når et bilde lages: `audit` (testen
    `dommen_mister_ingenting`) kjører uten den, som i Actions."""

    def __init__(self, name):
        self._name = name

    def __getattr__(self, attr):
        import importlib
        return getattr(importlib.import_module(self._name), attr)


Image, ImageDraw, ImageFont = _Lazy("PIL.Image"), _Lazy("PIL.ImageDraw"), _Lazy("PIL.ImageFont")

HERE = os.path.dirname(os.path.abspath(__file__))
WORK = os.path.join(ROOT, "target", "qa", "review")
STUDIO = os.path.join(HERE, "studio.py")
VIEWS = ["front:0:6:1.15:0.4", "corner:-35:12:1.2:0.4"]
# Siden har plass til høyst 511 filer per versjon; ett bilde per kort, og resten
# (siden, data.json, viewer.js, teksturlagene i 3D) får plass i de siste 31.
# Hevet fra 400 da kryptnivå 8–10 kom (17. oktober).
MAX_WAITING = 480

# Fanene, i rekkefølge.
CATEGORIES = [
    ("arms", "Armer og våpen"),
    ("mobs", "Fiender"),
    ("people", "Innbyggerne"),
    ("crypt", "Krypten"),
    ("town", "Byen: hus og bygninger"),
    ("nature", "Naturen"),
    ("things", "Møbler og småting"),
    ("textures", "Teksturer"),
]
BUILDINGS = ("house_", "tavern", "forge", "apothecary", "scriptorium", "home", "town_wall", "wall_tower",
             "gatehouse", "stave_church", "crypt", "shed_", "privy", "coop", "garden", "haystack")

# Når en side ble publisert (artefaktversjonene 24–26, 13. okt.). Et svar
# gitt mens en side var ute, gjelder filene slik de var da siden ble laget:
# rundene ble committet like etter publiseringen.
PAGES = ["2026-10-13T12:34:31Z", "2026-10-13T13:46:36Z", "2026-10-13T15:20:49Z"]


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout.strip()


def locate(name, kind):
    """Fila spillet bruker for tingen (relativt til repoet), eller None."""
    if kind == "modell":
        hits = sorted(glob.glob(os.path.join(ROOT, "assets", "models", "**", f"{name}.glb"), recursive=True))
        return os.path.relpath(hits[0], ROOT) if hits else None
    for p in (f"assets/textures/painted/{name}/albedo.png", f"assets/textures/{name}/albedo.jpg"):
        if os.path.exists(os.path.join(ROOT, p)):
            return p
    return None


def glb_layers(path):
    """Teksturlagene materialene i en glb bruker (`extras.fristad.layer`)."""
    import struct
    with open(path, "rb") as f:
        b = f.read()
    n = struct.unpack("<I", b[12:16])[0]
    j = json.loads(b[20:20 + n])
    return {((m.get("extras") or {}).get("fristad") or {}).get("layer") for m in j.get("materials", [])} - {None}


def glb_to_json(src, dst):
    """glb som glTF i JSON med dataene innebygd (artefaktene tar ikke imot
    `.glb`; three.js leser begge)."""
    import base64
    import struct
    with open(src, "rb") as f:
        b = f.read()
    n = struct.unpack("<I", b[12:16])[0]
    j = json.loads(b[20:20 + n])
    rest = b[20 + n:]
    binary = b""
    if len(rest) >= 8:
        m = struct.unpack("<I", rest[:4])[0]
        binary = rest[8:8 + m]
        # Siden får ikke laste data:- eller blob:-adresser (artefaktenes
        # CSP): bufferen står som base64 i et eget felt (`viewer.js` setter
        # sammen en glb i minnet), og bildene blir egne filer ved siden av.
        j["fristadBin"] = base64.b64encode(binary).decode()
    base = os.path.splitext(os.path.basename(dst))[0]
    for k, im in enumerate(j.get("images", [])):
        if "bufferView" not in im:
            continue
        bv = j["bufferViews"][im["bufferView"]]
        data = binary[bv.get("byteOffset", 0):bv.get("byteOffset", 0) + bv["byteLength"]]
        ext = ".png" if im.get("mimeType") == "image/png" else ".jpg"
        with open(os.path.join(os.path.dirname(dst), f"{base}_{k}{ext}"), "wb") as f:
            f.write(data)
        del im["bufferView"]
        im.pop("mimeType", None)
        im["uri"] = f"{base}_{k}{ext}"
    with open(dst, "w") as f:
        json.dump(j, f, separators=(",", ":"))


LEVEL_PIECE = __import__("re").compile(r"^(crypt_\d\d)_[a-z]{2}$")


def models_3d(out, items):
    """3D-visningen (`viewer.js`): modellen til hvert kort som venter, den
    eieren så sist (`glb/før/`), alle bitene i et kryptnivå der en bit venter
    (så biten kan ses inne i nivået), og teksturlagene de bruker (`tex/`)."""
    root = os.path.join(out, "glb")
    if os.path.isdir(root):
        shutil.rmtree(root)
    texdir = os.path.join(out, "tex")
    if os.path.isdir(texdir):
        shutil.rmtree(texdir)
    os.makedirs(os.path.join(root, "før"), exist_ok=True)
    os.makedirs(texdir, exist_ok=True)
    used, levels = set(), {}
    modules_3d(root, items, levels, used)
    for it in items:
        if it["state"] != "venter" or it["kind"] != "modell" or not it.get("path"):
            continue
        src = os.path.join(ROOT, it["path"])
        m = LEVEL_PIECE.match(it["name"])
        if KIT in it["path"] and os.path.exists(os.path.join(WORK, "kit", "nivaa_test.json")):
            # Bitene i byggesettet: prøvenivået som instanser av modellene
            # (`kit_shots.py`, `snapshot --dump-level`).
            if "grav_test" not in levels:
                inst = json.load(open(os.path.join(WORK, "kit", "nivaa_test.json")))
                os.makedirs(os.path.join(root, "grav_test"), exist_ok=True)
                urls = {}
                for name in sorted({i["model"] for i in inst}):
                    hits = glob.glob(os.path.join(ROOT, "assets", "models", "crypt", "kit", "*", f"{name}.glb"))
                    if hits:
                        glb_to_json(hits[0], os.path.join(root, "grav_test", f"{name}.json"))
                        urls[name] = f"glb/grav_test/{name}.json"
                        used |= glb_layers(hits[0])
                levels["grav_test"] = {"models": urls, "instances": [i for i in inst if i["model"] in urls]}
            glb_to_json(src, os.path.join(root, f"{it['id']}.json"))
            it["glb"], it["level"] = f"glb/{it['id']}.json", "grav_test"
            used |= glb_layers(src)
        elif m:
            lv = m.group(1)
            if lv not in levels:
                pieces = sorted(glob.glob(os.path.join(os.path.dirname(src), f"{lv}_??.glb")))
                os.makedirs(os.path.join(root, lv), exist_ok=True)
                levels[lv] = []
                for pc in pieces:
                    nm = os.path.basename(pc)[:-4] + ".json"
                    glb_to_json(pc, os.path.join(root, lv, nm))
                    levels[lv].append(f"glb/{lv}/{nm}")
                    used |= glb_layers(pc)
            it["glb"], it["level"] = f"glb/{lv}/{it['name']}.json", lv
        else:
            glb_to_json(src, os.path.join(root, f"{it['id']}.json"))
            it["glb"] = f"glb/{it['id']}.json"
            used |= glb_layers(src)
        seen = (it.get("seen") or "")
        if seen and seen != it["version"]:
            old = subprocess.run(["git", "cat-file", "blob", seen], cwd=ROOT, capture_output=True).stdout
            if old[:4] == b"glTF":
                tmp = os.path.join(WORK, "før.glb")
                os.makedirs(WORK, exist_ok=True)
                with open(tmp, "wb") as f:
                    f.write(old)
                glb_to_json(tmp, os.path.join(root, "før", f"{it['id']}.json"))
                it["glb_before"] = f"glb/før/{it['id']}.json"
                used |= glb_layers(tmp)
    held = held_3d(root, items, used)
    layers = {}
    for name in sorted(used):
        src = os.path.join(ROOT, "assets", "textures", name, "albedo.jpg")
        if not os.path.exists(src):
            continue
        im = Image.open(src).convert("RGB")
        im.thumbnail((1024, 1024))
        im.save(os.path.join(texdir, f"{name}.jpg"), quality=86)
        size, mirror = 2.0, False
        try:
            with open(os.path.join(ROOT, "assets", "textures", name, "meta.json")) as f:
                meta = json.load(f)
            size = float(meta.get("size_m", 2.0))
            mirror = bool(meta.get("mirror"))
        except (OSError, ValueError):
            pass
        layers[name] = {"url": f"tex/{name}.jpg", "size": size}
        if mirror:
            layers[name]["mirror"] = True  # speilvendt annenhver flis, som i spillet
    return layers, levels, held


def held_bones():
    """Det spillet setter på et bein (beinet → modellen), lest av spillets
    egne tabeller, så det ikke finnes to kilder: `mob_rigs::HELD` (buen
    bueskytteren holder) og trubadurens lutt (`npcs::LUTE_MODEL` på beinet
    `r.joint(..)` i `Clips.lute`). Filene fra `models!` i gltf.rs."""
    import re
    dom = os.path.join(ROOT, "crates", "fristad-domain", "src", "domain")
    rigs = open(os.path.join(dom, "mob_rigs.rs")).read()
    npcs = open(os.path.join(dom, "npcs.rs")).read()
    table = open(os.path.join(ROOT, "src", "adapters", "gltf.rs")).read()
    out = {}
    held = re.search(r"pub const HELD: &\[\(&str, &str\)\] = &\[(.*?)\];", rigs)
    for bone, model in re.findall(r'\("(\w+)",\s*"(\w+)"\)', held.group(1) if held else ""):
        out[bone] = model
    lute = re.search(r'pub const LUTE_MODEL: &str = "(\w+)";', npcs)
    bone = re.search(r'lute: r\.joint\("(\w+)"\)', npcs)
    if lute and bone:
        out[bone.group(1)] = lute.group(1)
    files = dict(re.findall(r'^\s*"(\w+)" => "([^"]+\.glb)",', table, re.M))
    return {b: (m, os.path.join(ROOT, "assets", "models", files[m])) for b, m in out.items() if m in files}


def held_3d(root, items, used):
    """De holdte modellene til «Se i 3D» (`glb/held/`): bare de med et bein i
    en modell som venter. viewer.js setter dem på beinet som spillet gjør
    (beinets rom dreid om x, `mob_rigs::held`, `Npcs::instances`)."""
    bones = held_bones()
    need = set()
    for it in items:
        if it["state"] != "venter" or not it.get("glb"):
            continue
        for g in [it["glb"]] + ([it["glb_before"]] if it.get("glb_before") else []):
            try:
                with open(os.path.join(os.path.dirname(root), g)) as f:
                    names = {n.get("name") for n in json.load(f).get("nodes", [])}
            except (OSError, ValueError):
                continue
            need |= names & set(bones)
    out = {}
    for bone in sorted(need):
        model, src = bones[bone]
        if not os.path.exists(src):
            continue
        os.makedirs(os.path.join(root, "held"), exist_ok=True)
        glb_to_json(src, os.path.join(root, "held", f"{model}.json"))
        used |= glb_layers(src)
        out[bone] = {"model": model, "glb": f"glb/held/{model}.json"}
    return out


def modules_3d(root, items, levels, used):
    """«Se i 3D» for modulene: hver modul med naboene sine slik de står i
    nivået `moduler` (`tools/crypt/module_shots.py`, `snapshot --dump-doc`),
    flyttet til origo, som instanser av bitene."""
    # Galleriet står på flere sider (`moduler.json` er side 1, så
    # `moduler_<n>.json`): lyset følger høyst 64 rom per nivå.
    pages = []
    for f in sorted(glob.glob(os.path.join(WORK, "moduler", "moduler*.json"))):
        with open(f) as fh:
            pages.append(json.load(fh))

    def gallery(name):
        return next((g for g in pages if any(m["id"] == name for m in g["doc"]["modules"])), None)
    urls = {}
    for it in items:
        if it["kind"] != "modul" or it["state"] != "venter":
            continue
        name = it["module"]
        gal = gallery(name)
        if gal is None:
            continue
        keep = [m for m in gal["doc"]["modules"] if m["id"] == name or m["id"].startswith(name + "_nabo")]
        if not keep:
            continue
        cells = [c for c in gal["flat"]["cells"] if any(c["id"].startswith(m["id"] + "/") for m in keep)]
        reach = 6.0 if any("rotunde" in c["bit"] or c["bit"].endswith(("/hall", "/galleri")) for c in cells) else 2.0
        xs = [c["at"][0] * 4.0 for c in cells]
        zs = [c["at"][2] * 4.0 for c in cells]
        lo, hi = (min(xs) - reach - 0.01, min(zs) - reach - 0.01), (max(xs) + reach + 0.01, max(zs) + reach + 0.01)
        mine = [c for c in gal["flat"]["cells"] if c["id"].startswith(name + "/")]
        cx = sum(c["at"][0] for c in mine) / len(mine) * 4.0
        cz = sum(c["at"][2] for c in mine) / len(mine) * 4.0
        inst = []
        for i in gal["instances"] + mark_instances(gal["marks"]):
            x, y, z = i["pos"]
            if lo[0] <= x <= hi[0] and lo[1] <= z <= hi[1]:
                inst.append({**i, "pos": [x - cx, y, z - cz]})
        os.makedirs(os.path.join(root, "kit"), exist_ok=True)
        for model in sorted({i["model"] for i in inst}):
            if model in urls:
                continue
            hits = glob.glob(os.path.join(ROOT, "assets", "models", "crypt", "kit", "*", f"{model}.glb")) or glob.glob(os.path.join(ROOT, "assets", "models", "**", f"{model}.glb"), recursive=True)
            if hits:
                glb_to_json(hits[0], os.path.join(root, "kit", f"{model}.json"))
                urls[model] = f"glb/kit/{model}.json"
                used |= glb_layers(hits[0])
        # Steinbyttet per instans (`crypt::stone`): lagene temaet gir, så
        # «Se i 3D» viser kloakken i tegl og borgen i kvader (viewer.js).
        for i in inst:
            if i.get("stone"):
                used |= set(i["stone"]["to"])
        levels[f"modul_{name}"] = {"models": {m: u for m, u in urls.items() if any(i["model"] == m for i in inst)}, "instances": [i for i in inst if i["model"] in urls]}
        it["level"] = f"modul_{name}"


# Byggesettet motoren setter ut etter merkene (`crypt::Crypt::build`), så
# «Se i 3D» viser porten, spaken, stigen, broa, heisen, kistene, vannet og
# pynten i modulen (eieren 20. oktober: «ser ingen port i 3d-visningen»).
MARK_MODELS = {"up": ["kit_up"], "down": ["kit_down"], "rest": ["kit_rest"], "lore": ["kit_lore"], "bonus": ["kit_bonus"], "hint": ["kit_crack"],
               "gate": ["kit_gate"], "trapdoor": ["kit_trapdoor"], "trap": ["kit_plate"], "lift": ["kit_lift", "kit_winch"], "bridge": ["kit_bridge"]}


def mark_instances(marks):
    """Instansene for merkene (som i spillet, i hvile: portene nede, broa
    nede, heisen oppe, kistene lukket)."""
    out = []

    def put(model, m, off=(0.0, 0.0, 0.0), scale=None):
        yaw = m.get("yaw", 0.0)
        c, s_ = math.cos(yaw), math.sin(yaw)
        x, y, z = m["pos"]
        i = {"model": model, "pos": [x + off[0] * c + off[2] * s_, y + off[1], z - off[0] * s_ + off[2] * c], "yaw": yaw}
        if scale:
            i["scale"] = list(scale)
        out.append(i)
    for m in marks:
        if not m.get("pos"):
            continue
        kind = m["kind"]
        if kind == "prop" and m.get("label"):
            put(m["label"], m)
        elif kind == "secret_wall" and m.get("label"):
            put(m["label"], m)
        elif kind in ("chest", "rare"):
            put("chest", m)
            put("chest_lid", m, (0.0, 0.46, -0.28))
        elif kind == "lever":
            put("kit_lever", m)
            put("kit_lever_arm", m, (0.0, 0.56, 0.0))
        elif kind == "shortcut":
            put("kit_shortcut", m)
            put("kit_shortcut_leaf", m, (-0.78, 0.0, 0.0))
        elif kind == "ladder":
            h = max(0.5, (m.get("top") or [0, m["pos"][1] + 3.0, 0])[1] - m["pos"][1])
            n = max(1, round(h))
            for k in range(n):
                put("kit_ladder", m, (0.0, k * h / n, 0.0), (1.0, h / n, 1.0))
            put("kit_ladder_top", m, (0.0, h, 0.0))
        elif kind == "water":
            w, d = (m.get("size") or [4.0, 4.0])[:2]
            put("kit_water", m, scale=(w, 1.0, d))
        elif kind == "gate":
            put("kit_gate", m, scale=(m.get("width", 2.8) / 2.8, 1.0, 1.0))
        elif kind == "bridge":
            w, l = (m.get("size") or [3.6, 8.0])[:2]
            put("kit_bridge", m, scale=(w / 3.6, 1.0, l / 8.0))
        else:
            for model in MARK_MODELS.get(kind, []):
                put(model, m)
    return out


def version(path):
    """Versjonen av fila slik den ligger nå (git-blob, 12 tegn)."""
    return git("hash-object", path)[:12]


def version_at(path, when):
    """Versjonen eieren så da svaret ble gitt (`when`, ISO-tid)."""
    t = datetime.datetime.fromisoformat(when.replace("Z", "+00:00"))
    page = max((p for p in PAGES if datetime.datetime.fromisoformat(p.replace("Z", "+00:00")) <= t), default=None)
    if page:
        commit = git("rev-list", "--reverse", "--since=" + page, "HEAD").split("\n")[0]
    else:
        commit = git("log", "-1", "--until=" + when, "--format=%H")
    if not commit:
        return None
    return git("rev-parse", f"{commit}:{path}")[:12] or None


def category(name, kind, path):
    if kind == "tekstur":
        return "textures"
    if kind in ("modul", "samlet"):
        return "crypt"
    folder = path.split("/")[2]
    if folder in ("arms", "mobs", "crypt"):
        return folder
    if folder == "npcs":
        return "people"
    if folder == "nature":
        return "nature"
    if folder == "town" and name.startswith(BUILDINGS):
        return "town"
    return "things"


def load(name):
    with open(os.path.join(DATA, name)) as f:
        return json.load(f)


def save(name, data):
    with open(os.path.join(DATA, name), "w") as f:
        json.dump(data, f, indent=1, ensure_ascii=False)
        f.write("\n")


def things():
    """Alle modellene og teksturene i katalogen: (id, kind, name, path)."""
    out = []
    for n in load("models.json")["models"]:
        p = locate(n, "modell")
        if p:
            out.append((f"m_{n}", "modell", n, p))
    for n in load("textures.json")["textures"]:
        p = locate(n, "tekstur")
        if p:
            out.append((f"t_{n}", "tekstur", n, p))
    # Modulene i nivåredigeringen (del 21): `u_<tema>_<navn>`.
    for p in sorted(glob.glob(os.path.join(ROOT, "assets", "levels", "moduler", "*", "*.json"))):
        theme, name = os.path.basename(os.path.dirname(p)), os.path.basename(p)[:-5]
        out.append((f"u_{theme}_{name}", "modul", f"{theme}_{name}", os.path.relpath(p, ROOT)))
    # Byggesettets mekanismer og pynt, satt sammen (`mekanismer`): `u_mek_<kort>`.
    for c in mechanism_cards():
        out.append((f"u_mek_{c['id']}", "samlet", f"mek_{c['id']}", "assets/catalog/modules.json"))
    return out


def mechanism_cards():
    """De samlede kortene for byggesettets mekanismer og pynt
    (`tools/crypt/mechanism_shots.py`)."""
    try:
        return load("modules.json").get("mekanismer", {}).get("kort", [])
    except OSError:
        return []


def retired(name):
    """Det som utgår (`utgaar` i `modules.json`): navnene og mønstrene."""
    import re
    try:
        rule = load("modules.json").get("utgaar", {})
    except OSError:
        return False
    return name in rule.get("navn", []) or any(re.search(p, name) for p in rule.get("moenster", []))


def via_modules():
    """Bitene som dømmes gjennom modulene (`assets/catalog/modules.json`)."""
    try:
        return set(load("modules.json").get("gjennom_moduler", [])) | {m for c in mechanism_cards() for m in c["modeller"]}
    except OSError:
        return set()


def group_of(kind):
    return {"modell": "models", "tekstur": "textures", "modul": "modules", "samlet": "modules"}[kind]


def current(kind, name, path):
    """Versjonen eieren dømmer: fila, og for en modul også bitene i temaet
    (en modul ser annerledes ut når en bit endres)."""
    import hashlib
    if kind == "samlet":
        card = next(c for c in mechanism_cards() if f"mek_{c['id']}" == name)
        parts = [version(locate(m, "modell")) for m in card["modeller"]]
        return hashlib.sha1(" ".join(parts).encode()).hexdigest()[:12]
    if kind != "modul":
        return version(path)
    theme = name.split("_", 1)[0]
    parts = [version(path)] + [version(os.path.relpath(g, ROOT)) for g in sorted(glob.glob(os.path.join(ROOT, "assets", "models", "crypt", "kit", theme, "*.glb")))]
    return hashlib.sha1(" ".join(parts).encode()).hexdigest()[:12]


def state_of(kind, name, entry, cur, fix=None):
    """Som `state`, men bitene som dømmes gjennom modulene står i `i_modul`."""
    if kind == "modell" and name in via_modules():
        return "i_modul"
    if kind == "modell" and retired(name):
        return "utgaar"
    return state(entry, cur, fix)


def state(entry, current, fix=None):
    """Bunken for en ting: `venter`, `hos_meg` eller `godkjent`. En retting
    (`fix`, fra `fixes.json`) som svarer på dommen som står, gjør at tingen
    venter selv om modellen ser lik ut: det som ble rettet, kan ligge i en
    annen modell (rundskjoldet: hånda i blokken er armene)."""
    if not entry or entry.get("verdict") not in ("godkjent", "underkjent", "usikker"):
        return "venter"
    if entry.get("seen") != current:
        return "venter"
    ans = (fix or {}).get("answers")
    if ans and ans.get("verdict") == entry["verdict"] and ans.get("why", "") == entry.get("why", ""):
        return "venter"
    return "godkjent" if entry["verdict"] == "godkjent" else "hos_meg"


def fix_for(fixes, name):
    """Den siste rettingen av tingen (`fixes.json`: navn → versjon → retting)."""
    v = fixes.get(name) or {}
    return list(v.values())[-1] if v else None


def migrate(answers_dir):
    """Én gang: `seen` for hver dom fra svartiden i samlingen `dom`
    (`answers_dir`, lagret med ArtifactData `out_dir`)."""
    ap = load("approvals.json")
    at = {}
    for f in glob.glob(os.path.join(answers_dir, "**", "*.json"), recursive=True):
        with open(f) as fh:
            at[os.path.basename(f)[:-5]] = json.load(fh).get("at")
    n = 0
    for id_, kind, name, path in things():
        group = "models" if kind == "modell" else "textures"
        e = ap.get(group, {}).get(name)
        if not e or "seen" in e:
            continue
        when = at.get(id_) or (e.get("date", "") + "T23:59:59Z")
        e["seen"] = version_at(path, when)
        n += 1
    save("approvals.json", ap)
    return n


# --- Bildene -------------------------------------------------------------------


def blob_file(path, ver, tmp):
    """Fila i versjon `ver` lagt i `tmp` (None om den ikke finnes)."""
    full = git("rev-parse", "--verify", "--quiet", ver) if ver else ""
    if not full:
        return None
    out = os.path.join(tmp, f"{ver}{os.path.splitext(path)[1]}")
    with open(out, "wb") as f:
        subprocess.run(["git", "cat-file", "blob", ver], cwd=ROOT, stdout=f, check=True)
    return out


KIT = "/crypt/kit/"  # byggesettet for nivåredigeringen


def kit_shot(name):
    """Bildet av en bit fra byggesettet inne i prøvenivået
    (`tools/crypt/kit_shots.py`), om det finnes."""
    p = os.path.join(WORK, "kit", f"{name}.png")
    return p if os.path.exists(p) else None


def shots(kind, name, path, ver):
    """Bildene av tingen i versjon `ver` (bufret i target/qa/review/shots):
    modellene i studio fra faste vinkler, teksturene lagt 2 × 2. Bitene i
    byggesettet for krypten får i tillegg bildet inne i prøvenivået (bare
    for versjonen som ligger nå)."""
    d = os.path.join(WORK, "shots", f"{name}@{ver}")
    folk = "/npcs/" in path
    names = ["sheet"] if folk else [v.split(":")[0] for v in VIEWS] if kind == "modell" else ["tile"]
    files = [os.path.join(d, f"{v}.png") for v in names]
    if KIT in path and ver == version(path) and kit_shot(name):
        inside = os.path.join(d, "nivaa.png")
        os.makedirs(d, exist_ok=True)
        Image.open(kit_shot(name)).convert("RGB").resize((640, 448)).save(inside)
        return _studio(kind, name, path, ver, d, folk, names, files) + [inside]
    return _studio(kind, name, path, ver, d, folk, names, files)


def _studio(kind, name, path, ver, d, folk, names, files):
    if all(os.path.exists(f) for f in files):
        return files
    os.makedirs(d, exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        src = os.path.join(ROOT, path) if ver == version(path) else blob_file(path, ver, tmp)
        if not src:
            return []
        if folk:
            # Innbyggerne: arket, figuren fra fire kanter, gangen og
            # nærbilder av hodet (`tools/models/folk_preview.py`).
            who = name.removeprefix("npc_")
            out = os.path.join(d, "sheet.jpg")
            subprocess.run(["python3.13", os.path.join(ROOT, "tools", "models", "folk_preview.py"), src, who, out], cwd=ROOT, capture_output=True, check=True)
            Image.open(out).convert("RGB").save(files[0])
        elif kind == "modell":
            env = dict(os.environ, FRISTAD_STUDIO_SIZE="640x448")
            subprocess.run(["python3.13", STUDIO, src, os.path.join(d, "x"), *VIEWS], cwd=ROOT, env=env, capture_output=True, check=True)
            for v in names:
                os.replace(os.path.join(d, f"x_{v}.png"), os.path.join(d, f"{v}.png"))
        else:
            im = Image.open(src).convert("RGB").resize((320, 320), Image.LANCZOS)
            big = Image.new("RGB", (640, 640))
            for x in (0, 320):
                for y in (0, 320):
                    big.paste(im, (x, y))
            big.save(files[0])
    return files


def same_look(a, b):
    """To sett bilder ser like ut (snittforskjell under 1,5 av 255)."""
    import numpy as np

    if not a or not b or len(a) != len(b):
        return False
    for x, y in zip(a, b):
        p = np.asarray(Image.open(x).convert("RGB"), dtype=float)
        q = np.asarray(Image.open(y).convert("RGB"), dtype=float)
        if p.shape != q.shape or abs(p - q).mean() > 1.5:
            return False
    return True


def font(h):
    for f in ("DejaVuSans-Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"):
        try:
            return ImageFont.truetype(f, h)
        except OSError:
            pass
    return ImageFont.load_default()


def sheet(before, after, out):
    """Ett bilde: én rad per vinkel, før til venstre og etter til høyre
    (bare etter når det ikke finnes noe før)."""
    rows = []
    for i, a in enumerate(after):
        b = before[i] if before and i < len(before) else None
        ims = [Image.open(x).convert("RGB") for x in ([b, a] if b else [a])]
        rows.append(ims)
    w, h = rows[0][-1].size
    cols = max(len(r) for r in rows)
    img = Image.new("RGB", (cols * w + (cols - 1) * 8, len(rows) * h + (len(rows) - 1) * 8), (24, 26, 30))
    draw = ImageDraw.Draw(img)
    f = font(max(16, h // 18))
    for r, ims in enumerate(rows):
        for c, im in enumerate(ims):
            x, y = c * (w + 8), r * (h + 8)
            img.paste(im.resize((w, h)), (x, y))
            if len(ims) == 2:
                label = ("Før", "Etter")[c]
                box = draw.textbbox((x + 10, y + 8), label, font=f)
                draw.rectangle((box[0] - 6, box[1] - 4, box[2] + 6, box[3] + 4), fill=(24, 26, 30) if c == 0 else (40, 110, 70))
                draw.text((x + 10, y + 8), label, fill=(255, 255, 255), font=f)
    img.save(out, quality=84)


# --- Siden ---------------------------------------------------------------------


def build(out, render=True):
    """Bygger siden i `out` (index.html, data.json og img/ med ett bilde per
    kort som venter). Ting som er bygd på nytt uten å se annerledes ut,
    beholder dommen (versjonen føres videre). Returnerer kortene."""
    ap = load("approvals.json")
    about = load("models.json")["models"]
    tex = load("textures.json")["textures"]
    fixes = load("fixes.json") if os.path.exists(os.path.join(DATA, "fixes.json")) else {}
    img = os.path.join(out, "img")
    if os.path.isdir(img):
        shutil.rmtree(img)
    os.makedirs(img, exist_ok=True)
    items, carried = [], 0
    mods = {}
    if os.path.exists(os.path.join(WORK, "moduler", "moduler.json")):
        with open(os.path.join(WORK, "moduler", "moduler.json")) as f:
            mods = json.load(f)
    for id_, kind, name, path in things():
        group = group_of(kind)
        e = ap.get(group, {}).get(name)
        cur = current(kind, name, path)
        fix = fix_for(fixes, name)
        st = state_of(kind, name, e, cur, fix)
        before = after = None
        if kind == "modul":
            with open(os.path.join(ROOT, path)) as f:
                mdef = json.load(f)
            item = {
                "id": id_, "kind": kind, "cat": "crypt", "name": name, "version": cur, "state": st,
                "about": f"{mdef.get('name', '')}: {mdef.get('about', '')}",
                "prior": {k: e[k] for k in ("verdict", "date", "why") if k in e} if e else None,
                "points": (e or {}).get("history") or ([e["why"]] if e and e.get("why") else []),
                "fixed": (fix or {}).get("text", []),
                "path": path, "seen": (e or {}).get("seen"), "module": name.split("_", 1)[1] if name.startswith("grav_") else name,
            }
            shot = os.path.join(WORK, "moduler", f"{item['module']}.png")
            seen_shot = os.path.join(WORK, "moduler", "foer", f"{item['module']}.png")
            # Bitene bygd på nytt, men modulen ser ut som det eieren så
            # (bildet i `foer/`): dommen står, som for modellene.
            # Ikke når en retting venter på eieren (`answers`).
            if st == "venter" and render and e and e.get("seen") and e["seen"] != cur and not (fixes.get(name) or {}).get(cur) and state_of(kind, name, dict(e, seen=cur), cur, fix) != "venter" and os.path.exists(shot) and same_look([seen_shot], [shot]):
                e["seen"] = cur
                st = item["state"] = state_of(kind, name, e, cur, fix)
                carried += 1
            if st == "venter" and render and os.path.exists(shot):
                # En bit som bare synes i en annen stein (skyteskårene i
                # kvader, tømmeret i sjakta i berg): rettingens bilder under.
                extra = ((fixes.get(name) or {}).get(cur) or {}).get("images", [])
                with_fix(before_after(shot, os.path.join(WORK, "moduler", "foer", f"{item['module']}.png")), extra).save(os.path.join(img, f"{id_}.jpg"), quality=86)
                item["img"] = f"img/{id_}.jpg"
            items.append(item)
            continue
        if kind == "samlet":
            card = next(c for c in mechanism_cards() if f"mek_{c['id']}" == name)
            item = {
                "id": id_, "kind": kind, "cat": "crypt", "name": card["name"], "version": cur, "state": st,
                "about": card.get("about", "") + " (" + ", ".join(card["modeller"]) + ")",
                "prior": {k: e[k] for k in ("verdict", "date", "why") if k in e} if e else None,
                "points": (e or {}).get("history") or ([e["why"]] if e and e.get("why") else []),
                "fixed": (fix or {}).get("text", []),
                "path": path, "seen": (e or {}).get("seen"),
            }
            shot = os.path.join(WORK, "mekanismer", f"{card['id']}.png")
            if st == "venter" and render and os.path.exists(shot):
                extra = ((fixes.get(name) or {}).get(cur) or {}).get("images", [])
                with_fix(before_after(shot, os.path.join(WORK, "mekanismer", "foer", f"{card['id']}.png")), extra).save(os.path.join(img, f"{id_}.jpg"), quality=86)
                item["img"] = f"img/{id_}.jpg"
            items.append(item)
            continue
        if st == "venter" and render and e and e.get("seen"):
            before, after = shots(kind, name, path, e["seen"]), shots(kind, name, path, cur)
            # Bygd på nytt, men ser ut som det eieren så: dommen står. Ikke
            # når jeg har rettet noe i denne versjonen (`fixes.json`):
            # studiobildene utenfra så ikke at dolkspissen eller hullene i
            # kryptbitene var rettet (13. oktober).
            if not (fixes.get(name) or {}).get(cur) and same_look(before, after):
                e["seen"] = cur
                st = state_of(kind, name, e, cur, fix)
                carried += st != "venter"
        item = {
            "id": id_, "kind": kind, "cat": category(name, kind, path), "name": name, "version": cur, "state": st,
            "about": about.get(name, "") if kind == "modell" else (tex.get(name) or {}).get("shows", ""),
            "prior": {k: e[k] for k in ("verdict", "date", "why") if k in e} if e else None,
            "points": (e or {}).get("history") or ([e["why"]] if e and e.get("why") else []),
            "fixed": (fix or {}).get("text", []),
            "path": path, "seen": (e or {}).get("seen"),
        }
        if st == "utgaar":
            item["about"] = load("modules.json")["utgaar"].get("hvorfor", "")
        if st == "venter" and render:
            if fix and fix.get("images"):
                ims = [Image.open(os.path.join(ROOT, p)).convert("RGB") for p in fix["images"]]
                w = max(i.width for i in ims)
                ims = [i.resize((w, round(i.height * w / i.width))) for i in ims]
                s = Image.new("RGB", (w, sum(i.height for i in ims) + 8 * (len(ims) - 1)), (24, 26, 30))
                y = 0
                for i in ims:
                    s.paste(i, (0, y))
                    y += i.height + 8
                s.save(os.path.join(img, f"{id_}.jpg"), quality=84)
            else:
                after = after or shots(kind, name, path, cur)
                if after:
                    seen_b = before if before and e and e.get("seen") != cur else None
                    sheet(seen_b, after, os.path.join(img, f"{id_}.jpg"))
            item["img"] = f"img/{id_}.jpg" if os.path.exists(os.path.join(img, f"{id_}.jpg")) else None
        items.append(item)
    # Modulkortene som er borte (kopiene per tema, eieren 20. oktober: «det
    # holder med en modell»): dommene står, i «Utgår» med modulen som tar over.
    cards = {n for _, k, n, _ in things() if k in ("modul", "samlet")}
    for name, e in sorted(ap.get("modules", {}).items()):
        if name in cards or name.startswith("mek_"):
            continue
        base = name.split("_", 1)[1] if "_" in name else name
        items.append({
            "id": f"u_{name}", "kind": "modul", "cat": "crypt", "name": name, "version": e.get("seen", ""), "state": "utgaar",
            "about": f"Kopien per tema er borte: modulen er grav_{base}, og steinen velges i nivået." if os.path.exists(os.path.join(ROOT, "assets", "levels", "moduler", "grav", f"{base}.json")) else "Kopien per tema er borte: formen finnes i settet, og steinen velges i nivået.",
            "prior": {k: e[k] for k in ("verdict", "date", "why") if k in e},
            "points": e.get("history") or ([e["why"]] if e.get("why") else []), "fixed": [], "path": None, "seen": e.get("seen"),
        })
    save("approvals.json", ap)
    layers, levels, held = models_3d(out, items) if render else ({}, {}, {})
    for it in items:
        it.pop("path", None)
        it.pop("seen", None)
    with open(os.path.join(out, "data.json"), "w") as f:
        json.dump({"categories": CATEGORIES, "items": items, "layers": layers, "levels": levels, "held": held}, f, ensure_ascii=False)
    shutil.copy(os.path.join(HERE, "approvals.html"), os.path.join(out, "index.html"))
    shutil.copy(os.path.join(HERE, "viewer.js"), os.path.join(out, "viewer.js"))
    return items, carried


def use_assets(out, urls):
    """Bildene til sidens asset-lager (19. oktober: 1147 filer, siden har
    plass til 511 per versjon). `urls` er sti i `out` → adressen lageret ga
    (`/_blob/<id>`); data.json og glTF-filene peker dit, og bildene slettes
    fra `out`, så bare siden, data.json, viewer.js og glTF-filene publiseres
    som filer. Gir stiene som ble igjen."""
    with open(os.path.join(out, "data.json")) as f:
        data = json.load(f)
    for it in data["items"]:
        if it.get("img"):
            it["img"] = urls[it["img"]]
    for layer in data["layers"].values():
        layer["url"] = urls[layer["url"]]
    with open(os.path.join(out, "data.json"), "w") as f:
        json.dump(data, f, ensure_ascii=False)
    for p in glob.glob(os.path.join(out, "glb", "**", "*.json"), recursive=True):
        with open(p) as f:
            j = json.load(f)
        base = os.path.relpath(os.path.dirname(p), out)
        for im in j.get("images", []):
            if im.get("uri") and not im["uri"].startswith("/"):
                im["uri"] = urls[os.path.join(base, im["uri"])]
        with open(p, "w") as f:
            json.dump(j, f, separators=(",", ":"))
    for rel in urls:
        if os.path.exists(os.path.join(out, rel)):
            os.remove(os.path.join(out, rel))
    return sorted(os.path.relpath(os.path.join(d, n), out) for d, _, ns in os.walk(out) for n in ns)


def check(out):
    """Det som venter, står på siden med bilde, og det er plass til alt.
    Gir feilene som tekst (tom liste: alt er med)."""
    with open(os.path.join(out, "data.json")) as f:
        data = json.load(f)
    ap = load("approvals.json")
    fixes = load("fixes.json") if os.path.exists(os.path.join(DATA, "fixes.json")) else {}
    seen = {it["id"]: it for it in data["items"]}
    errors = []
    for id_, kind, name, path in things():
        st = state_of(kind, name, ap.get(group_of(kind), {}).get(name), current(kind, name, path), fix_for(fixes, name))
        it = seen.get(id_)
        if not it:
            errors.append(f"{name} mangler på siden")
        elif it["state"] != st:
            errors.append(f"{name} står som {it['state']} på siden, men er {st}")
        # Etter `assets` ligger bildet i sidens asset-lager (`/_blob/<id>`).
        elif st == "venter" and not (it.get("img") and (it["img"].startswith("/_blob/") or os.path.exists(os.path.join(out, it["img"])))):
            errors.append(f"{name} venter, men har ikke bilde")
    waiting = sum(1 for it in data["items"] if it["state"] == "venter")
    if waiting > MAX_WAITING:
        errors.append(f"{waiting} kort venter, siden har plass til {MAX_WAITING}")
    return errors


def before_after(shot, before):
    """Bildet av et modul- eller mekanismekort, med bildet eieren dømte
    over («Før», lagt i `foer/` før bildene lages på nytt) når det finnes."""
    after = Image.open(shot).convert("RGB")
    if not os.path.exists(before):
        return after
    prev = Image.open(before).convert("RGB")
    prev = prev.resize((after.width, round(prev.height * after.width / prev.width)))
    band = 36
    out = Image.new("RGB", (after.width, prev.height + after.height + 2 * band + 8), (24, 26, 30))
    d = ImageDraw.Draw(out)
    d.text((12, 8), "FØR", fill=(230, 140, 120), font=_font(22))
    out.paste(prev, (0, band))
    y = band + prev.height + 8
    d.text((12, y + 8), "ETTER", fill=(140, 220, 150), font=_font(22))
    out.paste(after, (0, y + band))
    return out


def with_fix(card, images):
    """Kortet med rettingens bilder (`fixes.json`, `images`) under."""
    if not images:
        return card
    ims = [Image.open(os.path.join(ROOT, p)).convert("RGB") for p in images]
    ims = [i.resize((card.width, round(i.height * card.width / i.width))) for i in ims]
    out = Image.new("RGB", (card.width, card.height + sum(i.height + 8 for i in ims)), (24, 26, 30))
    out.paste(card, (0, 0))
    y = card.height + 8
    for i in ims:
        out.paste(i, (0, y))
        y += i.height + 8
    return out


def _font(h):
    for f in ("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"):
        try:
            return ImageFont.truetype(f, h)
        except OSError:
            pass
    return ImageFont.load_default()


def audit():
    """Det som kan sjekkes uten å bygge siden (testen `dommen_mister_ingenting`):
    hver dom har versjonen eieren så, hver ting i katalogen har en fil, og alt
    som venter, får plass på siden. Gir feilene som tekst."""
    ap = load("approvals.json")
    errors = []
    names = {"models": set(load("models.json")["models"]), "textures": set(load("textures.json")["textures"])}
    found = {(group_of(k), n) for _, k, n, _ in things()}
    for group in ("models", "textures"):
        for n in names[group]:
            if (group, n) not in found:
                errors.append(f"{n} har ingen fil")
        for n, e in ap.get(group, {}).items():
            if e.get("verdict") in ("godkjent", "underkjent", "usikker") and "seen" not in e:
                errors.append(f"dommen over {n} mangler versjonen eieren så (`seen`)")
    fixes = load("fixes.json") if os.path.exists(os.path.join(DATA, "fixes.json")) else {}
    # «Rettet slik» er punkter (siden viser én linje per punkt).
    for n, vs in fixes.items():
        for v, fx in vs.items():
            if not isinstance(fx.get("text", []), list):
                errors.append(f"«rettet slik» for {n} ({v}) må være en liste med punkter")
    # Bitene som dømmes gjennom modulene: de finnes, og hver modul har sitt bilde å vises med.
    for n in sorted(via_modules()):
        if n not in names["models"]:
            errors.append(f"{n} står i modules.json, men ikke i models.json")
    for c in mechanism_cards():
        for n in c["modeller"]:
            if n not in names["models"]:
                errors.append(f"{n} står i mekanismekortet {c['id']}, men ikke i models.json")
    for n in load("modules.json").get("utgaar", {}).get("navn", []):
        if n not in names["models"]:
            errors.append(f"{n} står som utgått i modules.json, men ikke i models.json")
    for n, e in ap.get("modules", {}).items():
        if e.get("verdict") in ("godkjent", "underkjent", "usikker") and "seen" not in e:
            errors.append(f"dommen over modulen {n} mangler versjonen eieren så (`seen`)")
    waiting = sum(1 for _, k, n, p in things() if state_of(k, n, ap.get(group_of(k), {}).get(n), current(k, n, p), fix_for(fixes, n)) == "venter")
    if waiting > MAX_WAITING:
        errors.append(f"{waiting} kort venter, siden har plass til {MAX_WAITING}")
    return errors
