"""Katalogen over alt spillet er laget av (`docs/katalog.md`): teksturene,
modellene, forbildene og det som ventes, med bilde, hva det viser, hvor det
brukes, kontrollresultatene og eierens dom. Den lages av dataene, så den
alltid er oppdatert:

- `assets/catalog/textures.json` og `models.json`: det som ikke kan regnes
  ut (hva tingen viser, slaget, temaet), skrevet for hånd.
- `assets/catalog/approvals.json`: eierens dom (godkjent eller underkjent,
  dato og hvorfor). Bare eieren setter den.
- `target/qa/world.json` (`cargo run --release --bin catalog`): hvor hver
  modell står og hva som mangler eller ventes.
- Filene selv: glb-ene (trekanter, teksturlag, miniatyr), teksturene (P-T),
  forbildene og oppsettene i `sheet2model/examples/fristad/`.

    python3 -m tools.qa catalog
"""
import glob
import json
import os
from collections import defaultdict

import numpy as np
from PIL import Image

from . import texture as ptex
from .gate import OUT, ROOT
from .glb import Mesh
from .raster import render

DOC = os.path.join(ROOT, "docs", "katalog.md")
IMG = os.path.join(ROOT, "docs", "katalog")
DATA = os.path.join(ROOT, "assets", "catalog")
REFS = ["monsters", "people", "props", "weapons_ref"]
THEMES = {"by": "Byen", "natur": "Naturen", "figur": "Figurer og utstyr", "krypt": "Krypten: gravkjelleren", "kloakk": "Krypten: kloakken", "gruve": "Krypten: gruva", "borg": "Krypten: borgen", "kjerne": "Krypten: den korrupte kjernen"}


def load(name):
    with open(os.path.join(DATA, name)) as f:
        return json.load(f)


def thumb(im, path, size=160):
    im = im.convert("RGB")
    im.thumbnail((size, size))
    im.save(path, quality=82)


def verdict(ap, kind, name):
    v = ap.get(kind, {}).get(name)
    if not v:
        return "ikke vurdert"
    return f"**{v['verdict']}** ({v['date']}): {v['why']}"


def md_escape(s):
    return s.replace("|", "\\|")


def build():
    os.makedirs(IMG, exist_ok=True)
    tex_info = load("textures.json")["textures"]
    mod_info = load("models.json")["models"]
    ap = load("approvals.json")
    world_path = os.path.join(OUT, "world.json")
    if not os.path.exists(world_path):
        raise SystemExit("Mangler target/qa/world.json: kjør `cargo run --release --bin catalog > target/qa/world.json` først.")
    with open(world_path) as f:
        world = json.load(f)

    # Modellene: miniatyr, trekanter og teksturlag fra glb-fila.
    models = []
    layer_users = defaultdict(list)
    for m in world["models"]:
        path = os.path.join(ROOT, "assets", "models", m["file"])
        mesh = Mesh(path)
        thumb(Image.fromarray(render(mesh, 320)), os.path.join(IMG, f"m_{m['name']}.jpg"))
        lo, hi = mesh.bounds
        layers = mesh.layers()
        for l in layers:
            layer_users[l].append(m["name"])
        models.append(dict(m, layers=layers, size=(hi - lo).round(2).tolist()))
    in_table = {m["file"] for m in world["models"]}
    stray = sorted(os.path.relpath(p, os.path.join(ROOT, "assets", "models")) for p in glob.glob(os.path.join(ROOT, "assets", "models", "**", "*.glb"), recursive=True) if os.path.relpath(p, os.path.join(ROOT, "assets", "models")) not in in_table)

    # Teksturene: miniatyr av kilden (eller laget), P-T og hvem som bruker dem.
    textures = []
    for name in ptex.names():
        r = ptex.check(name)
        r.save()
        src = os.path.join(ptex.PAINTED, name, "albedo.png")
        lay = os.path.join(ptex.LAYERS, name, "albedo.jpg")
        thumb(Image.open(src if os.path.exists(src) else lay), os.path.join(IMG, f"t_{name}.jpg"))
        info = tex_info.get(name, {})
        checks = r.checks
        textures.append({
            "name": name, "info": info, "source": os.path.exists(src), "layer": os.path.exists(lay),
            "ok": all(c.ok for c in checks), "failed": [f"{c.name.split(': ')[1]} {c.value:.2f}" for c in checks if not c.ok],
            "users": layer_users.get(name, []),
        })

    # Forbildene og oppsettene i sheet2model.
    configs = {}
    for p in glob.glob(os.path.join(ROOT, "sheet2model", "examples", "fristad", "*.json")):
        with open(p) as f:
            c = json.load(f)
        configs.setdefault(os.path.normpath(c.get("ref", "")), []).append(f"{os.path.splitext(os.path.basename(p))[0]} ({c.get('layout', '?')})")
    refs = []
    for d in REFS:
        for p in sorted(glob.glob(os.path.join(ROOT, "assets", "textures", d, "**", "*"), recursive=True)):
            if os.path.splitext(p)[1].lower() not in (".png", ".jpg", ".jpeg", ".webp"):
                continue
            rel = os.path.relpath(p, ROOT)
            key = os.path.basename(rel).replace(".", "_")
            thumb(Image.open(p), os.path.join(IMG, f"r_{d}_{key}.jpg"))
            refs.append({"path": rel, "group": d, "img": f"r_{d}_{key}.jpg", "configs": configs.get(os.path.normpath(rel), [])})

    write(models, textures, refs, world, ap, mod_info, stray)
    with open(os.path.join(IMG, "katalog.json"), "w") as f:
        json.dump({"models": models, "textures": textures, "refs": refs, "planned": world["planned"], "missing": world["missing"]}, f, indent=1, ensure_ascii=False)
    return models, textures, refs


def write(models, textures, refs, world, ap, mod_info, stray):
    L = []
    w = L.append
    t_ok = sum(t["ok"] for t in textures)
    m_bad = sum(1 for m in models if ap.get("models", {}).get(m["name"], {}).get("verdict") == "underkjent")
    m_good = sum(1 for m in models if ap.get("models", {}).get(m["name"], {}).get("verdict") == "godkjent")
    w("# Katalogen: alt spillet er laget av\n")
    w("Laget av `python3 -m tools.qa catalog` fra dataene (se `tools/qa/catalog.py`); rett i `assets/catalog/*.json`, ikke her. Eierens dom står i `assets/catalog/approvals.json` og settes bare av eieren.\n")
    w("## Oversikt\n")
    w(f"- **Teksturer:** {len(textures)}, {t_ok} består teksturkontrollen (P-T), {sum(1 for t in textures if not t['layer'])} har bare et malt bilde og er ikke tatt i bruk som lag.")
    w(f"- **Modeller:** {len(models)} i spillet: {m_good} godkjent, {m_bad} underkjent, {len(models) - m_good - m_bad} ikke vurdert. {sum(1 for m in models if m['instances'] == 0 and m['reserve'] == 0)} står ikke noe sted.")
    w(f"- **Ventes:** {len(world['planned'])} modeller systemene venter på (fiender, figurer, våpen, dyr …).")
    w(f"- **Forbilder:** {len(refs)} bilder, {sum(1 for r in refs if r['configs'])} brukt av et oppsett i sheet2model.")
    if stray:
        w(f"- **Glb-filer som ikke er i bruk:** {', '.join(stray)}.")
    w("")
    w("## Ventes (skal lages)\n")
    w("| Modell | Hva |\n|---|---|")
    for p in world["planned"]:
        w(f"| `{p['name']}` | {p['why']} |")
    w("")
    w("## Teksturer\n")
    w("P-T: kilden stor nok, uten ramme og med jevnt lys; laget flisbart og skarpt (`tools/qa/texture.py`). Arkene (dører, list, skilt, vinduer) sjekkes bare for størrelse og skarphet. Perspektiv vurderes med øyet.\n")
    by_theme = defaultdict(list)
    for t in textures:
        by_theme[t["info"].get("theme", "ukjent")].append(t)
    for theme in list(THEMES) + [k for k in by_theme if k not in THEMES]:
        if theme not in by_theme:
            continue
        w(f"### {THEMES.get(theme, theme)}\n")
        w("| Bilde | Lag | Viser | Brukt av | P-T | Eierens dom |\n|---|---|---|---|---|---|")
        for t in by_theme[theme]:
            kind = "ark" if t["info"].get("kind") == "sheet" else "flis"
            state = "" if t["layer"] else " (bare malt, ikke lag)"
            users = ", ".join(f"`{u}`" for u in t["users"][:6]) + (f" og {len(t['users']) - 6} til" if len(t["users"]) > 6 else "") if t["users"] else "ingen modell"
            pt = "ok" if t["ok"] else "feil: " + ", ".join(t["failed"])
            w(f"| ![](katalog/t_{t['name']}.jpg) | `{t['name']}` ({kind}){state} | {md_escape(t['info'].get('shows', 'ikke beskrevet'))} | {users} | {pt} | {verdict(ap, 'textures', t['name'])} |")
        w("")
    w("## Modeller\n")
    groups = defaultdict(list)
    for m in models:
        groups[m["file"].split("/")[0]].append(m)
    for g, ms in sorted(groups.items()):
        w(f"### {g}\n")
        w("| Bilde | Navn (fil) | Hva | Står ute | Trekanter | Mål (m) | Teksturlag | Eierens dom |\n|---|---|---|---|---|---|---|---|")
        for m in ms:
            where = f"{m['instances']}×" if m["instances"] else ("posert av spillet" if m["reserve"] else "**ingen steder**")
            size = " × ".join(f"{v:.1f}" for v in m["size"])
            layers = ", ".join(m["layers"][:5]) + (" …" if len(m["layers"]) > 5 else "") if m["layers"] else "bakt tekstur"
            w(f"| ![](katalog/m_{m['name']}.jpg) | `{m['name']}` ({m['file']}) | {md_escape(mod_info.get(m['name'], 'ikke beskrevet'))} | {where} | {m['triangles']:,} | {size} | {layers} | {verdict(ap, 'models', m['name'])} |".replace(",", " "))
        w("")
    w("## Forbilder\n")
    w("Bildene modellene lages fra (`assets/textures/{monsters,people,props,weapons_ref}`), og oppsettet i `sheet2model/examples/fristad/` som bruker dem.\n")
    for g in REFS:
        rs = [r for r in refs if r["group"] == g]
        if not rs:
            continue
        w(f"### {g}\n")
        w("| Bilde | Fil | Oppsett |\n|---|---|---|")
        for r in rs:
            w(f"| ![](katalog/{r['img']}) | `{r['path']}` | {', '.join(r['configs']) or 'ikke i bruk'} |")
        w("")
    with open(DOC, "w") as f:
        f.write("\n".join(L) + "\n")
