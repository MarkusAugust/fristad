"""Bildene av modulene til eieren (`docs/nivaaredigering.md`, del 21):
hver modul ferdig satt sammen, sett innenfra med faklene og lyset, og et
kart ovenfra over prøvenivået med modulene, åpningene og innholdet.

Modulene står side om side i nivået `moduler` (`compose::modules::gallery`,
`snapshot --level-doc moduler`), så alle bildene tas i én kjøring. Hver
modul får to bilder fra motsatte hjørner (eller fra hver ende av en gang),
satt sammen til ett bilde per modul. Kartet tegnes av det kompilatoren
faktisk lager (`snapshot --dump-doc`): cellene farget per modul, modulenes
omriss, murene, dørene, portene, buene og gangåpningene, og innholdet.

    python3 tools/crypt/module_shots.py [modul ...]

Skriver `target/qa/review/moduler/<modul>.png`, `oversikt_test.png`, og
dokumentene `moduler.json` og `test.json` (til «Se i 3D»).
"""

import json
import math
import os
import subprocess
import sys

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
OUT = os.path.join(ROOT, "target", "qa", "review", "moduler")
SNAP = os.path.join(ROOT, "target", "release", "snapshot")
CELL, HALF, FACE = 4.0, 2.0, 1.6
EYE = 1.65


def dump(name):
    path = os.path.join(OUT, f"{name}.json")
    subprocess.run([SNAP, "--level-doc", name, "--dump-doc", path], cwd=ROOT, check=True, capture_output=True)
    with open(path) as f:
        return json.load(f)


def yaw_to(dx, dz):
    """Yaw i grader (0 mot nord, −z; 90 mot vest) for å se i retningen (dx, dz)."""
    return math.degrees(math.atan2(-dx, -dz))


def cameras(name, cells, hosts):
    """To kameraer (øye, yaw, pitch) for en modul. `cells` er rutene
    (i, y, j) modulen dekker, `hosts` naborommene ved åpningene (ruta og
    siden i nivået). Gangene og trappene ses fra naborommene gjennom
    åpningene; salene langs midtgangen fra sør og langs tverrgangen fra øst
    (søylene står i hjørnene mellom fagene); de små rommene fra hjørne til
    hjørne."""
    i0, i1 = min(c[0] for c in cells), max(c[0] for c in cells)
    j0, j1 = min(c[2] for c in cells), max(c[2] for c in cells)
    y = min(c[1] for c in cells)
    x0, x1 = i0 * CELL - FACE + 0.7, i1 * CELL + FACE - 0.7
    z0, z1 = j0 * CELL - FACE + 0.7, j1 * CELL + FACE - 0.7
    cams = []
    for (hi, hy, hj), side in hosts[:2]:
        ox, oz = [(0, -1), (1, 0), (0, 1), (-1, 0)][side]
        eye = (hi * CELL + ox * 1.25, hy + EYE, hj * CELL + oz * 1.25)
        # Nedgangen: på skrå ned mot hullet med trappa (mot sør i modulen).
        look = (-ox - oz * 0.0 + (0.7 if name == "nedgang" and ox else 0.0) * 0, -oz + (0.7 if name == "nedgang" else 0.0))
        cams.append((eye, yaw_to(*look), -28.0 if name == "nedgang" else 3.0))
    if name == "rotunde":
        cx, cz = (i0 + i1) / 2 * CELL, (j0 + j1) / 2 * CELL
        return [((cx, y + EYE, z1 + 0.2), 0.0, 16.0), ((cx + 3.0, y + EYE, cz + 3.0), yaw_to(-1, -1), 24.0)]
    if name == "nedgang":
        cams.append(((x0, y + EYE, z0), yaw_to(1, 1), -30.0))
    elif name == "skattkammer":
        cams.append(((x1, y + EYE, (z0 + z1) / 2 + 0.6), yaw_to(-1, -0.2), -14.0))
    if cams:
        return cams[:2]
    if i0 == i1 and j0 == j1 or (i0 == i1 and j1 - j0 == 1):
        return [((x0, y + EYE, z1), yaw_to(x1 - x0, -(z1 - z0)), 12.0), ((x1, y + EYE, z0), yaw_to(-(x1 - x0), z1 - z0), 12.0)]
    # Midtgangen (fagenes midte, ikke søylerekka) fra sør, tverrgangen fra øst.
    xc = (i0 + i1) // 2 * CELL
    zc = (j0 + j1) // 2 * CELL
    return [((xc, y + EYE, z1 + 0.3), 0.0, 14.0), ((x1 + 0.3, y + EYE, zc), 90.0, 14.0)]


def font(h):
    for f in ("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"):
        try:
            return ImageFont.truetype(f, h)
        except OSError:
            pass
    return ImageFont.load_default()


def sheet(name, title, plan_img, files, out):
    """Kartet over modulen og de to bildene side om side, med navnet over."""
    ims = [Image.open(f).convert("RGB") for f in files if os.path.exists(f)]
    if not ims:
        return
    w, h = ims[0].size
    head = 56
    p = plan_img.copy()
    p.thumbnail((h, h))
    pw = h
    img = Image.new("RGB", (pw + 8 + len(ims) * (w + 8), h + head), (24, 26, 30))
    d = ImageDraw.Draw(img)
    d.text((14, 12), f"{title}  ({name})", fill=(240, 236, 226), font=font(28))
    img.paste(p, ((pw - p.width) // 2, head + (h - p.height) // 2))
    for k, im in enumerate(ims):
        img.paste(im, (pw + 8 + k * (w + 8), head))
    img.save(out)


def shoot(only):
    gal = dump("moduler")
    modules = {m["id"]: m for m in load_modules()}
    lines, cards = [], []
    for inst in gal["doc"]["modules"]:
        if "_nabo" in inst["id"]:
            continue
        m = modules[inst["module"]]
        name = inst["id"]
        if only and name not in only and inst["module"] not in only:
            continue
        cells = [tuple(c["at"]) for c in gal["flat"]["cells"] if c["id"].startswith(name + "/")]
        # De sammensatte bitene dekker flere celler (rotunden 3 × 3).
        if any("rotunde" in c["bit"] for c in gal["flat"]["cells"] if c["id"].startswith(name + "/")):
            c0 = cells[0]
            cells = [(c0[0] + di, c0[1], c0[2] + dj) for di in (-1, 0, 1) for dj in (-1, 0, 1)]
        hosts = []
        for h in gal["doc"]["modules"]:
            if h["id"].startswith(name + "_nabo"):
                # Siden i modulen som naboen står inntil.
                for o in gal["openings"]:
                    ox, oz = [(0, -1), (1, 0), (0, 1), (-1, 0)][o["side"]]
                    if o["module"] == name and [o["at"][0] + ox, o["at"][1] + oz] == [h["at"][0], h["at"][2]]:
                        hosts.append(((h["at"][0], h["at"][1], h["at"][2]), o["side"]))
                        break
        files = []
        for k, (eye, yaw, pitch) in enumerate(cameras(name, cells, hosts)):
            f = os.path.join(OUT, "skudd", f"{name}_{k}.png")
            files.append(f)
            lines.append(f"{f} {eye[0]:.2f},{eye[1]:.2f},{eye[2]:.2f} {yaw:.1f} {pitch:.1f}")
        keep = {name} | {h["id"] for h in gal["doc"]["modules"] if h["id"].startswith(name + "_nabo")}
        cards.append((name, m["name"], files, draw_plan(gal, keep, scale=110, focus=name)))
    os.makedirs(os.path.join(OUT, "skudd"), exist_ok=True)
    shots = os.path.join(OUT, "shots.txt")
    with open(shots, "w") as f:
        f.write("\n".join(lines) + "\n")
    subprocess.run([SNAP, "--level-doc", "moduler", "--level", "1", "--film", "--size", "800x560", "--frames", "2", "--shots", shots], cwd=ROOT, check=True, capture_output=True)
    for name, title, files, p in cards:
        sheet(name, title, p, files, os.path.join(OUT, f"{name}.png"))
        print(os.path.join(OUT, f"{name}.png"))


def load_modules():
    out = []
    base = os.path.join(ROOT, "assets", "levels", "moduler")
    for theme in sorted(os.listdir(base)):
        for f in sorted(os.listdir(os.path.join(base, theme))):
            if f.endswith(".json"):
                with open(os.path.join(base, theme, f)) as fh:
                    m = json.load(fh)
                m.setdefault("id", f"{theme}/{f[:-5]}")
                out.append(m)
    return out


# --- Kartet ---------------------------------------------------------------------

PALETTE = [(196, 170, 128), (150, 172, 140), (176, 150, 170), (140, 160, 186), (196, 158, 120), (160, 186, 176), (186, 176, 136), (170, 140, 140), (136, 170, 160), (180, 160, 120), (150, 150, 186), (190, 170, 150), (160, 180, 130), (176, 136, 156)]
EDGE_LOOK = {
    "grav_vegg": "mur", "grav_vegg_b": "mur", "grav_vegg_dor": "dør", "grav_vegg_port": "port", "grav_vegg_gang": "gang",
    "grav_vegg_falsk": "falsk", "grav_gjordbue": "bue",
}
MARKS = {"start": ("S", (60, 160, 70)), "up": ("↑", (60, 120, 200)), "down": ("↓", (60, 120, 200)), "rest": ("H", (220, 150, 40)), "chest": ("K", (170, 110, 40)),
         "mob": ("F", (190, 50, 50)), "lever": ("L", (120, 80, 160)), "hint": ("~", (110, 110, 110))}


def overview(name="test"):
    """Kartet over et nivådokument ovenfra (det kompilatoren lager)."""
    d = dump(name)
    img = draw_plan(d, None, scale=96)
    dr = ImageDraw.Draw(img)
    pad = 40
    dr.text((pad, 18), f"{d['flat'].get('name', name)}: {len(d['doc']['modules'])} moduler, {len(d['flat']['rooms'])} rom", fill=(250, 246, 236), font=font(26))
    legend = ["Hvitt: modulenes omriss · svart: mur · oransje: dør · rød: port · lilla: falsk vegg · åpent: gang eller bue",
              "S start · ↑ opp · ↓ ned · H hvilested · K kiste · F fiende · L spak · ~ sprekk · +1 m: gulvet over startplanet"]
    for k, line in enumerate(legend):
        dr.text((pad, 54 + 20 * k), line, fill=(210, 204, 192), font=font(14))
    out = os.path.join(OUT, f"oversikt_{name}.png")
    img.save(out)
    print(out)


def draw_plan(d, keep, scale=96, focus=None):
    """Kartet ovenfra over modulene i `keep` (alle om None). Med `focus`
    står naboene grå, og åpningene modulen har som ingen bruker, er grønne
    piler ut (der den kan kobles)."""
    flat, doc = d["flat"], d["doc"]
    insts = [m["id"] for m in doc["modules"] if keep is None or m["id"] in keep]
    color = {m: PALETTE[k % len(PALETTE)] for k, m in enumerate(insts)}
    if focus:
        color = {m: ((196, 170, 128) if m == focus else (92, 88, 84)) for m in insts}
    owner = lambda cid: cid.split("/")[0] if "/" in cid else None
    inside = lambda o: keep is None or o in keep
    # Delcellene i de sammensatte bitene (rotunden): midtcella og ringen rundt.
    cells, rounds = [], []
    for c in flat["cells"]:
        if not inside(owner(c["id"])):
            continue
        if "rotunde" in c["bit"]:
            cells += [(c["at"][0] + di, c["at"][1], c["at"][2] + dj, owner(c["id"])) for di in (-1, 0, 1) for dj in (-1, 0, 1)]
            rounds.append((c["at"][0] * CELL, c["at"][2] * CELL, owner(c["id"])))
        else:
            cells.append((c["at"][0], c["at"][1], c["at"][2], owner(c["id"])))
    i0, i1 = min(c[0] for c in cells) - 1, max(c[0] for c in cells) + 1
    j0, j1 = min(c[2] for c in cells) - 1, max(c[2] for c in cells) + 1
    S = scale  # piksler per celle
    pad = 40
    top = 90 if keep is None else 0
    W, H = (i1 - i0 + 1) * S + 2 * pad, (j1 - j0 + 1) * S + 2 * pad + top
    img = Image.new("RGB", (W, H), (46, 44, 42))
    dr = ImageDraw.Draw(img)
    px = lambda x, z: (pad + (x / CELL - i0 + 0.5) * S, pad + top + (z / CELL - j0 + 0.5) * S)
    near = lambda x, z: i0 - 0.5 <= x / CELL <= i1 + 0.5 and j0 - 0.5 <= z / CELL <= j1 + 0.5
    for i, y, j, o in cells:
        a, b = px(i * CELL - HALF, j * CELL - HALF), px(i * CELL + HALF, j * CELL + HALF)
        dr.rectangle([a, b], fill=color.get(o, (120, 120, 120)))
        if y:
            dr.text((a[0] + 4, b[1] - 18), f"+{y} m", fill=(40, 30, 20), font=font(13))
    # Rotunden: det runde rommet med gangene ut midt på hver side.
    for x, z, o in rounds:
        a, b = px(x - 3 * HALF, z - 3 * HALF), px(x + 3 * HALF, z + 3 * HALF)
        dr.rectangle([a, b], fill=(46, 44, 42))
        r = 5.6
        for dx, dz in ((0, -1), (1, 0), (0, 1), (-1, 0)):
            p0 = px(x + dx * 4.0 - abs(dz) * FACE - abs(dx) * 2.0, z + dz * 4.0 - abs(dx) * FACE - abs(dz) * 2.0)
            p1 = px(x + dx * 4.0 + abs(dz) * FACE + abs(dx) * 2.0, z + dz * 4.0 + abs(dx) * FACE + abs(dz) * 2.0)
            dr.rectangle([p0, p1], fill=color.get(o), outline=(28, 24, 22), width=5)
        c0, c1 = px(x - r, z - r), px(x + r, z + r)
        dr.ellipse([c0, c1], fill=color.get(o), outline=(28, 24, 22), width=7)
        for dx, dz in ((0, -1), (1, 0), (0, 1), (-1, 0)):
            p0 = px(x + dx * 4.6 - abs(dz) * (FACE - 0.15) - abs(dx) * 1.5, z + dz * 4.6 - abs(dx) * (FACE - 0.15) - abs(dz) * 1.5)
            p1 = px(x + dx * 4.6 + abs(dz) * (FACE - 0.15) + abs(dx) * 1.5, z + dz * 4.6 + abs(dx) * (FACE - 0.15) + abs(dz) * 1.5)
            dr.rectangle([p0, p1], fill=color.get(o))
    # Modulenes omriss: kanter mellom celler i ulike moduler (eller mot fjell).
    at = {(i, j): o for i, y, j, o in cells}
    for (i, j), o in at.items():
        for di, dj, side in ((0, -1, "N"), (1, 0, "E"), (0, 1, "S"), (-1, 0, "W")):
            if at.get((i + di, j + dj)) != o:
                if side in ("N", "S"):
                    zz = j * CELL + dj * HALF
                    dr.line([px(i * CELL - HALF, zz), px(i * CELL + HALF, zz)], fill=(250, 246, 236), width=3)
                else:
                    xx = i * CELL + di * HALF
                    dr.line([px(xx, j * CELL - HALF), px(xx, j * CELL + HALF)], fill=(250, 246, 236), width=3)
    # Kantene slik kompilatoren satte dem ut.
    for inst in d["instances"]:
        look = EDGE_LOOK.get(inst["model"])
        if not look or not near(inst["pos"][0], inst["pos"][2]):
            continue
        x, _, z = inst["pos"]
        horiz = abs(math.sin(inst["yaw"])) < 0.5  # halvkanten langs x (N eller S)
        a, b = ((x - HALF, z), (x + HALF, z)) if horiz else ((x, z - HALF), (x, z + HALF))
        if look == "mur":
            dr.line([px(*a), px(*b)], fill=(28, 24, 22), width=7)
        else:
            w = {"dør": 0.8, "port": 1.4, "gang": 1.6, "falsk": 1.0, "bue": 0}[look]
            mid = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
            u = (1.0, 0.0) if horiz else (0.0, 1.0)
            for s0, s1 in ((-HALF, -w), (w, HALF)):
                if s1 > s0:
                    dr.line([px(mid[0] + u[0] * s0, mid[1] + u[1] * s0), px(mid[0] + u[0] * s1, mid[1] + u[1] * s1)], fill=(28, 24, 22), width=7)
            col = {"dør": (210, 120, 40), "port": (200, 40, 40), "gang": None, "falsk": (150, 60, 170), "bue": None}[look]
            if col:
                dr.line([px(mid[0] - u[0] * w, mid[1] - u[1] * w), px(mid[0] + u[0] * w, mid[1] + u[1] * w)], fill=col, width=5)
    # Søylene.
    for inst in d["instances"]:
        if inst["model"] == "grav_soyle" and near(inst["pos"][0], inst["pos"][2]):
            c = px(inst["pos"][0], inst["pos"][2])
            dr.ellipse([c[0] - 7, c[1] - 7, c[0] + 7, c[1] + 7], fill=(28, 24, 22))
    # Innholdet.
    f = font(18)
    for m in d["marks"]:
        if m["kind"] not in MARKS or not m.get("pos") or not near(m["pos"][0], m["pos"][2]):
            continue
        t, col = MARKS[m["kind"]]
        c = px(m["pos"][0], m["pos"][2])
        dr.ellipse([c[0] - 13, c[1] - 13, c[0] + 13, c[1] + 13], fill=col, outline=(250, 246, 236), width=2)
        bb = dr.textbbox((0, 0), t, font=f)
        dr.text((c[0] - (bb[2] - bb[0]) / 2, c[1] - (bb[3] - bb[1]) / 2 - bb[1]), t, fill=(255, 255, 255), font=f)
    # Navnene på modulene.
    for inst in doc["modules"]:
        own = [(i, j) for (i, j), o in at.items() if o == inst["id"]]
        if not own:
            continue
        ci = sum(p[0] for p in own) / len(own)
        cj = sum(p[1] for p in own) / len(own)
        c = px(ci * CELL, cj * CELL - 0.9)
        label = inst["module"].split("/")[-1] if not focus or inst["id"] == focus else "nabo"
        bb = dr.textbbox((0, 0), label, font=font(14))
        dr.rectangle([c[0] - (bb[2] - bb[0]) / 2 - 4, c[1] - 2, c[0] + (bb[2] - bb[0]) / 2 + 4, c[1] + bb[3] + 4], fill=(30, 28, 26))
        dr.text((c[0] - (bb[2] - bb[0]) / 2, c[1]), label, fill=(250, 240, 220), font=font(14))
    # Åpningene modulen har som ingen bruker: grønne piler ut.
    if focus:
        used = {(h["at"][0], h["at"][2]) for h in doc["modules"] if h["id"] != focus and inside(h["id"])}
        for o in d.get("openings", []):
            if o["module"] != focus:
                continue
            ox, oz = [(0, -1), (1, 0), (0, 1), (-1, 0)][o["side"]]
            if (o["at"][0] + ox, o["at"][1] + oz) in used:
                continue
            mx, mz = o["at"][0] * CELL + ox * HALF, o["at"][1] * CELL + oz * HALF
            tip = px(mx + ox * 1.1, mz + oz * 1.1)
            l = px(mx - oz * 0.8 - ox * 0.2, mz - ox * 0.8 - oz * 0.2)
            r = px(mx + oz * 0.8 - ox * 0.2, mz + ox * 0.8 - oz * 0.2)
            col = {"falsk": (170, 80, 190), "bue": (90, 170, 220)}.get(o["kind"], (90, 190, 90))
            dr.polygon([tip, l, r], fill=col, outline=(250, 246, 236))
    return img


def main():
    only = sys.argv[1:]
    os.makedirs(OUT, exist_ok=True)
    if not only or "oversikt" in only:
        overview("test")
    only = [o for o in only if o != "oversikt"]
    if not sys.argv[1:] or only:
        shoot(only)


if __name__ == "__main__":
    main()
