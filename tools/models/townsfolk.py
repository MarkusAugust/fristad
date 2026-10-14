"""Innbyggerne i byen (`npc_<navn>`), bygd etter eierens ark
(`assets/textures/people/ref/<navn>_body.jpeg`: forfra, fra siden og
bakfra). Kjør:

    python3.13 tools/models/townsfolk.py [smith ...] [størrelse]

Formen leses av arket (`folk_sheet.measure`): kroppen med hodet, to armer
og to bein, hver som en stabel tverrsnitt med bredden forfra og dybden fra
siden, så armene og beina alltid er egne lemmer. Arket legges på som bilde
fra visningen hver flate vender mot (`monsters.project_sheet`). Skjelettet
står i landemerkene fra arket, og hver del vektes bare mot sine egne bein
(armen aldri mot brystet), så klippene fra `mob_motion.py` (stilen «folk»:
`idle`, `walk`, `talk`, `wave`) ikke drar kroppen med armen.

Origo i bakken midt under figuren, fronten mot +X (Blender), som fiendene."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import Clip, armature, bake_textured, clear  # noqa: E402
from mathutils import Vector  # noqa: E402
import folk_motion  # noqa: E402
import folk_sheet  # noqa: E402
from sheetproj import blended_sheet_material, filled_sheet  # noqa: E402

OUT = "assets/models/npcs"

# Høyden (m) per innbygger; navnet på arket er nøkkelen.
FOLK = {
    "smith": 1.82,
    "clerk": 1.74,
    "apothecary": 1.72,
    "innkeeper": 1.66,
    "gorvek": 1.9,
    "farmer": 1.76,
    "maid": 1.6,
    "guard": 1.84,
    "merchant": 1.72,
    "fishwife": 1.62,
    "boy": 1.36,
    "washerwoman": 1.6,
}
RING = {"body": 24, "arm": 14, "leg": 16}  # hjørner rundt hvert tverrsnitt
THIN = 2.5  # tverrsnitt som ligger innen så mange piksler av naboenes linje, tas bort
TAPER = 0.35  # hodet: så mye smalere framme ved ansiktet enn ved ørene
SQUARE = 2.4  # superellipse: 2 er ellipse, høyere er mer kantet (klær)


def smoothed(rows, k=2):
    """Glidende snitt over tverrsnittene, så piksler i omrisset ikke blir
    riller i formen (radtallet beholdes)."""
    out = []
    for i, r in enumerate(rows):
        w = rows[max(0, i - k): i + k + 1]
        out.append((r[0], *[sum(x[j] for x in w) / len(w) for j in range(1, len(r))]))
    return out


def thinned(rows, tol=THIN):
    """Tverrsnittene som trengs (Douglas–Peucker over midte og bredder):
    der formen endrer seg jevnt, holder det med endene."""
    if len(rows) <= 2:
        return rows

    def dev(i, a, b):
        t = (rows[i][0] - rows[a][0]) / max(1e-6, rows[b][0] - rows[a][0])
        return max(abs(rows[i][j] - (rows[a][j] + (rows[b][j] - rows[a][j]) * t)) for j in range(1, len(rows[i])))

    keep = {0, len(rows) - 1}
    stack = [(0, len(rows) - 1)]
    while stack:
        a, b = stack.pop()
        if b - a < 2:
            continue
        i = max(range(a + 1, b), key=lambda i: dev(i, a, b))
        if dev(i, a, b) > tol:
            keep.add(i)
            stack += [(a, i), (i, b)]
    return [rows[i] for i in sorted(keep)]


def ring_mesh(name, rows, m, mat, n=24, square=SQUARE, cap_top=True, cap_bottom=True):
    """En stabel tverrsnitt (y, midte forfra, halv bredde, midte fra siden,
    halv dybde; piksler) som lukket mesh i meter."""
    px, ground, facing = m["px"], m["ground"], m["facing"]
    bm = bmesh.new()
    loops = []
    # Et tverrsnitt med «split» har egen dybde foran og bak aksen
    # (y, midte, halv bredde, akse, foran, bak).
    split = rows and rows[0][-1] == "split"
    if split:
        rows = [r[:-1] for r in rows]
    for r in thinned(smoothed(rows)):
        y, cy, hw, sx = r[:4]
        front, back = (r[4], r[5]) if split else (r[4], r[4])
        z = (ground - y) * px
        cx, cyy = sx * px * facing, cy * px
        a = max(hw * px, 0.004)
        loop = []
        for i in range(n):
            t = 2 * math.pi * i / n
            c, s = math.cos(t), math.sin(t)
            # Superellipse: x = a·sgn(c)|c|^(2/n)
            ex = math.copysign(abs(c) ** (2 / square), c)
            ey = math.copysign(abs(s) ** (2 / square), s)
            b = max((front if c > 0 else back) * px, 0.004)
            if split and c > 0:
                # Hodet smalner av mot ansiktet (håret står ut på sidene,
                # ikke foran): ellers fikk kinnet fra profilen plassen der
                # bildet forfra viser hår, som hengekinn.
                ey *= 1.0 - TAPER * c * c
            loop.append(bm.verts.new((cx + b * ex, cyy + a * ey, z)))
        loops.append(loop)
    for l0, l1 in zip(loops, loops[1:]):
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((l0[i], l0[j], l1[j], l1[i]))
    if cap_top:
        bm.faces.new(list(reversed(loops[0])))
    if cap_bottom:
        bm.faces.new(loops[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    for p in me.polygons:
        p.use_smooth = True
    ob = bpy.data.objects.new(name, me)
    bpy.context.collection.objects.link(ob)
    me.materials.append(mat)
    return ob


def figure(name):
    m = folk_sheet.measure(name, FOLK[name])
    path = os.path.join(folk_sheet.REF, f"{name}_body.jpeg")
    mat = blended_sheet_material(f"{name}_sheet", filled_sheet(path))
    fit = folk_sheet.head_fit(m)
    parts = dict(m["parts"])
    if fit:
        # Hodet (over halsen) får hodearket, som er to–fire ganger så
        # skarpt som hodet på kroppsarket; kroppen begynner litt over
        # halsen, så de to overlapper.
        # Formen på hodet er lest av hodearket selv, så bildet forfra og
        # profilen ligger på samme form (`folk_sheet.head_rows`).
        rows = parts.pop("body")
        head, chin = folk_sheet.head_rows(m, fit)
        parts["head"] = head
        m["y"]["chin"] = int(chin)
        # Kroppen begynner like over der hodet slutter, og er ikke bredere
        # enn hodet der de overlapper (den stakk ellers ut under haka).
        end = head[-1]
        deep = (end[4] + end[5]) / 2
        below = end[0] + 0.04 * m["H"]

        def joined(r):
            # Over skjøten ligger kroppen inne i hodet; like under er den
            # minst like bred som bunnen av hodet (et ringbrynjeskjørt eller
            # håret var ellers en hylle over en smal hals).
            if r[0] <= end[0]:
                return (r[0], r[1], min(r[2], end[2] * 0.92), r[3], min(r[4], deep * 0.92))
            if r[0] < below:
                return (r[0], r[1], max(r[2], end[2] * 0.98), r[3], max(r[4], deep * 0.98))
            return r

        parts["body"] = [joined(r) for r in rows if r[0] >= end[0] - 2 * folk_sheet.STEP]
    obs = {}
    for part, rows in parts.items():
        if len(rows) < 2:
            continue
        kind = "body" if part == "head" else part.split(".")[0]
        # Hodet er rundt (en kantet skalle får ansiktet forfra og profilen
        # på samme hjørne), uten lokk under (det står inne i kroppen).
        obs[part] = ring_mesh(part.replace(".", "_"), rows, m, mat, RING[kind], 2.0 if part.startswith("arm") or part == "head" else SQUARE, cap_bottom=part != "head")
    bpy.context.view_layer.update()
    for part, ob in obs.items():
        if part == "head":
            # Hodet bytter fra ansiktet forfra til profilen ved 45°: bildet
            # forfra lenger ut på kinnet blir strukket.
            ob.data.materials[0] = blended_sheet_material(f"{name}_head_sheet", filled_sheet(fit["file"]), soft=(0.45, 0.55), front_attr="src_front")
            project_head(ob, m, fit, ("front", "back"), "src")
            project_head(ob, m, fit, ("side",), "src_side")
            front_first(ob)
            continue
        project_part(ob, m, part)
    return m, obs


def front_first(ob, start=(0.22, 0.1), sideways=(0.55, 0.75)):
    """Foran på hodet viser bildet forfra også der flaten vender litt til
    siden: der viser bildet forfra hår og profilen kinnet, og forfra ble det
    hengekinn (eieren møter folk ansikt til ansikt). Vekten «src_front» går
    mykt fra 0 til 1 med hvor langt fram hjørnet står (`start`: brøk av
    dybden fra forkanten) og hvor lite flaten vender til siden
    (`sideways`), så det ikke blir en søm."""
    me = ob.data
    mw = ob.matrix_world
    nm = mw.to_3x3().inverted().transposed()
    xs = {}
    for v in me.vertices:
        p = mw @ v.co
        xs.setdefault(round(p.z, 4), []).append(p.x)
    span = {z: (min(a), max(a)) for z, a in xs.items()}
    attr = me.color_attributes.get("src_front") or me.color_attributes.new("src_front", "FLOAT_COLOR", "CORNER")

    def ramp(v, a, b):
        t = max(0.0, min(1.0, (v - a) / (b - a)))
        return t * t * (3 - 2 * t)

    for poly in me.polygons:
        n = (nm @ poly.normal).normalized()
        side = abs(n.y) / (abs(n.x) + abs(n.y) + 1e-6)
        for li in poly.loop_indices:
            p = mw @ me.vertices[me.loops[li].vertex_index].co
            lo, hi = span[round(p.z, 4)]
            depth = (hi - p.x) / max(1e-6, hi - lo)
            w = ramp(-depth, -start[0], -start[1]) * (1 - ramp(side, *sideways)) * (1.0 if n.x > 0 else 0.0)
            attr.data[li].color = (w, w, w, 1.0)


INSET = 0.12  # forfra trekkes flater som vender til siden så mye inn mot midten (omrisset på arket er en mørk strek)
INSET_HIDDEN = 0.45  # og så mye der sidebildet ikke viser flaten (armen, eller det armen dekker)
SOLE = 0.012  # sålen tar fargen så høyt over bakken (del av høyden), ikke bakkestreken


def project_part(ob, m, part):
    """Uv-lagene «src» (forfra eller bakfra) og «src_side» (fra siden) for
    en del av kroppen; materialet blander dem etter flatens retning
    (`blended_sheet_material`).

    - Forfra trekkes flater som vender til siden litt inn mot midten av
      tverrsnittet, så de ikke får den mørke streken rundt figuren.
    - Sidebildet brukes ikke der det viser noe annet enn flaten: armen
      (den henger et annet sted på sidebildet) og det armen dekker på
      sidebildet (låret og hofta bak hånda ble svarte flekker når figuren
      gikk). Der tar sida bildet forfra, trukket godt inn.
    - Sålen tar fargen fra skoen, ikke fra bakkestreken."""
    W, Hs = m["size"]
    v, px, H = m["views"], m["px"], m["H"]
    me = ob.data
    if not me.uv_layers:
        me.uv_layers.new(name="UVMap")
    src = me.uv_layers.get("src") or me.uv_layers.new(name="src")
    side = me.uv_layers.get("src_side") or me.uv_layers.new(name="src_side")
    mw = ob.matrix_world
    nm = mw.to_3x3().inverted().transposed()
    arm = part.startswith("arm")
    # Midten av hvert tverrsnitt (ringene ligger i hver sin høyde).
    rings = {}
    for vert in me.vertices:
        p = mw @ vert.co
        rings.setdefault(round(p.z, 4), []).append(p)
    mid = {z: (sum(q.x for q in ps) / len(ps), sum(q.y for q in ps) / len(ps)) for z, ps in rings.items()}
    # Armene på sidebildet: midt i kroppen sett fra siden, fra skulderen til fingrene.
    band = []
    for s in ("L", "R"):
        for y, cy, hw, sx, sd in m["parts"].get(f"arm.{s}", []):
            band.append(((v["ground"] - y) * px, sx * px * m["facing"], hw * px * 1.1))
    band.sort()

    def covered(p):
        if arm:
            return True
        near = [b for b in band if abs(b[0] - p.z) < 0.03]
        return any(abs(p.x - bx) < r for _, bx, r in near)

    def front_uv(p, n, inset):
        cx, cy = mid[round(p.z, 4)]
        y = cy + (p.y - cy) * (1 - inset)
        back = n.x < -0.05 or (abs(n.x) <= 0.05 and p.x < cx)
        X = v["back"] - y / px if back else v["front"] + y / px
        return X

    for poly in me.polygons:
        n = (nm @ poly.normal).normalized()
        ax, ay = abs(n.x), abs(n.y)
        share = ay / (ax + ay + 1e-6)
        # Hele flaten tar samme bilde (blandes hjørnene, blir flaten et
        # smurt sagtakk).
        hidden = covered(mw @ poly.center)
        for li in poly.loop_indices:
            p = mw @ me.vertices[me.loops[li].vertex_index].co
            Y = v["ground"] - p.z / px
            if part.startswith("leg"):
                Y = min(Y, v["ground"] - SOLE * H)
            X = front_uv(p, n, (INSET_HIDDEN if arm else INSET) * share)
            src.data[li].uv = (X / W, 1.0 - Y / Hs)
            if hidden:
                Xs = front_uv(p, n, INSET_HIDDEN)
            else:
                Xs = v["side"] + v.get("side_dir", 1) * p.x / px
            side.data[li].uv = (Xs / W, 1.0 - Y / Hs)


def project_head(ob, m, fit, views=None, layer="src"):
    """Uv-laget «src» for hodet fra hodearket: hver flate fra visningen den
    vender mot, gjennom passingen i `folk_sheet.head_fit` (kroppsarkets
    piksel → hodearkets)."""
    from PIL import Image

    W, H = Image.open(fit["file"]).size
    v, px = m["views"], m["px"]
    me = ob.data
    if not me.uv_layers:
        me.uv_layers.new(name="UVMap")
    src = me.uv_layers.get(layer) or me.uv_layers.new(name=layer)
    mw = ob.matrix_world
    nm = mw.to_3x3().inverted().transposed()
    for poly in me.polygons:
        n = (nm @ poly.normal).normalized()
        if views == ("side",):
            view = "side"
        elif views == ("front", "back") or abs(n.x) >= abs(n.y) * 0.5:
            view = "front" if (n.x > 0 if abs(n.x) > 0.02 else (mw @ poly.center).x > 0) else "back"
        else:
            view = "side"
        _, s, tx, ty, flip, (qx0, qx1) = fit[view]
        for li in poly.loop_indices:
            p = mw @ me.vertices[me.loops[li].vertex_index].co
            if view == "front":
                X = v["front"] + p.y / px
            elif view == "back":
                X = v["back"] - p.y / px
            else:
                X = v["side"] + v.get("side_dir", 1) * p.x / px
            Y = v["ground"] - p.z / px
            hx, hy = (X - tx) / s, (Y - ty) / s
            if flip:
                hx = 2 * (qx1 - 1) - hx
            src.data[li].uv = (hx / W, 1.0 - hy / H)


def joints(m):
    """Leddene i meter (Blender: x fram, y mot venstre, z opp) fra
    landemerkene på arket og delene (midten av armen og beinet i raden)."""
    px, ground, H, Y = m["px"], m["ground"], m["H"], m["y"]
    z = lambda y: (ground - y) * px  # noqa: E731

    def along(part, y):
        rows = m["parts"][part]
        r = min(rows, key=lambda r: abs(r[0] - y))
        return Vector((r[3] * px * m["facing"], r[1] * px, z(y)))

    def mid(y):
        rows = m["parts"]["body"] + [r[:5] for r in m["parts"].get("head", [])]
        r = min(rows, key=lambda r: abs(r[0] - y))
        return Vector((r[3] * px * m["facing"], 0.0, z(y)))

    J = {}
    pelvis = Y["hip"] - int(0.04 * H)
    J["pelvis"] = mid(pelvis)
    J["spine03"] = mid(pelvis - int(0.07 * H))
    J["spine01"] = mid(Y["shoulder"] + int(0.12 * H))
    J["neck"] = mid(Y["shoulder"] - int(0.01 * H))
    # Hodeleddet under haka, lest av hodets form (profilen trekker seg inn
    # der), så hodet dreies som én ting over halsen.
    J["head"] = mid(Y.get("chin", Y["shoulder"] - int(0.07 * H)))
    if J["head"].z <= J["neck"].z + 0.02:
        J["head"] = J["neck"] + Vector((0, 0, 0.04))
    J["top"] = mid(m["top"] + int(0.01 * H))
    for s in ("L", "R"):
        J[f"shoulder.{s}"] = along(f"arm.{s}", Y["shoulder"] + int(0.02 * H))
        J[f"elbow.{s}"] = along(f"arm.{s}", Y[f"elbow.{s}"])
        J[f"wrist.{s}"] = along(f"arm.{s}", Y[f"wrist.{s}"])
        J[f"finger.{s}"] = along(f"arm.{s}", Y[f"hand_end.{s}"] - int(0.03 * H))
        hip = along(f"leg.{s}", Y["crotch"])
        J[f"hip.{s}"] = Vector((J["pelvis"].x, hip.y * 0.85, z(Y["hip"])))
        J[f"knee.{s}"] = along(f"leg.{s}", Y["knee"]) + Vector((0.01, 0, 0))
        J[f"ankle.{s}"] = along(f"leg.{s}", Y["ankle"])
        J[f"ankle.{s}"].x = J[f"knee.{s}"].x - 0.01
        # Tåa: lengst fram i sålen sett fra siden.
        rows = [r for r in m["parts"][f"leg.{s}"] if r[0] > Y["ankle"]]
        front = max(r[3] * m["facing"] + r[4] for r in rows) * px if rows else J[f"ankle.{s}"].x + 0.15
        J[f"toe.{s}"] = Vector((front - 0.03, J[f"ankle.{s}"].y, 0.02))
    return J


def bones(J):
    """Spillets 18 bein (samme navn og rekkefølge som `folk_motion` venter)."""
    T = lambda k: tuple(J[k])  # noqa: E731
    out = [
        ("root", (0, 0, 0), (0, 0, 0.1), None),
        ("hips", T("pelvis"), T("spine03"), "root"),
        ("spine", T("spine03"), T("spine01"), "hips"),
        ("chest", T("spine01"), T("neck"), "spine"),
        ("neck", T("neck"), T("head"), "chest"),
        ("head", T("head"), T("top"), "neck"),
    ]
    for s in ("L", "R"):
        out += [
            (f"upper_arm.{s}", T(f"shoulder.{s}"), T(f"elbow.{s}"), "chest"),
            (f"forearm.{s}", T(f"elbow.{s}"), T(f"wrist.{s}"), f"upper_arm.{s}"),
            (f"hand.{s}", T(f"wrist.{s}"), T(f"finger.{s}"), f"forearm.{s}"),
        ]
    for s in ("L", "R"):
        out += [
            (f"thigh.{s}", T(f"hip.{s}"), T(f"knee.{s}"), "hips"),
            (f"shin.{s}", T(f"knee.{s}"), T(f"ankle.{s}"), f"thigh.{s}"),
            (f"foot.{s}", T(f"ankle.{s}"), T(f"toe.{s}"), f"shin.{s}"),
        ]
    return out


# Hver del vektes bare mot sine egne bein, så armen som henger inntil
# kroppen aldri drar brystet med seg (og omvendt).
OWN = {
    "head": ("neck", "head"),
    "body": ("hips", "spine", "chest", "neck", "head"),
    "arm.L": ("upper_arm.L", "forearm.L", "hand.L"),
    "arm.R": ("upper_arm.R", "forearm.R", "hand.R"),
    "leg.L": ("thigh.L", "shin.L", "foot.L"),
    "leg.R": ("thigh.R", "shin.R", "foot.R"),
}
BLEND = 0.05  # m der to bein deler et hjørne


def weigh(ob, rig, names):
    """Vektene i `ob` mot beina i `names` etter avstanden til beinet (det
    nærmeste mest, det nest nærmeste blandet inn innen `BLEND`)."""
    segs = [(b.name, Vector(b.head_local), Vector(b.tail_local)) for b in rig.data.bones if b.name in names]
    groups = {n: ob.vertex_groups.get(n) or ob.vertex_groups.new(name=n) for n, _, _ in segs}
    mw = ob.matrix_world
    for v in ob.data.vertices:
        p = mw @ v.co
        ds = []
        for n, h, t in segs:
            ab = t - h
            u = max(0.0, min(1.0, (p - h).dot(ab) / max(1e-9, ab.length_squared)))
            ds.append(((p - (h + ab * u)).length, n))
        ds.sort()
        if len(ds) == 1:
            groups[ds[0][1]].add([v.index], 1.0, "REPLACE")
            continue
        (d1, n1), (d2, n2) = ds[0], ds[1]
        w2 = max(0.0, 1.0 - (d2 - d1) / BLEND) * 0.5
        groups[n1].add([v.index], 1.0 - w2, "REPLACE")
        if w2 > 0:
            groups[n2].add([v.index], w2, "REPLACE")


def skirt_follows_thighs(ob, J):
    """Det kroppen har nedenfor hofta (forkle, kjortel, skjørt) følger
    lårene på sin side mer jo lenger ned det henger, så beina ikke går
    gjennom det."""
    hz = J["hip.L"].z
    knee = J["knee.L"].z
    gl, gr = ob.vertex_groups.get("thigh.L") or ob.vertex_groups.new(name="thigh.L"), ob.vertex_groups.get("thigh.R") or ob.vertex_groups.new(name="thigh.R")
    mw = ob.matrix_world
    for v in ob.data.vertices:
        p = mw @ v.co
        if p.z >= hz:
            continue
        t = min(1.0, (hz - p.z) / max(0.05, hz - knee))
        side = max(-1.0, min(1.0, p.y / 0.08))
        share = 0.75 * t
        for g in v.groups:
            g.weight *= 1.0 - share
        if side > -1.0:
            gl.add([v.index], share * (side + 1) / 2, "ADD")
        if side < 1.0:
            gr.add([v.index], share * (1 - side) / 2, "ADD")


def build(name, size):
    m, obs = figure(name)
    J = joints(m)
    rig = armature(f"npc_{name}_rig", bones(J))
    for part, ob in obs.items():
        weigh(ob, rig, OWN[part])
    skirt_follows_thighs(obs["body"], J)
    style, grips = folk_motion.FIGURES.get(name, ("default", {}))
    st = dict(folk_motion.STYLES.get(style, {}))
    # Armene henger alt ut fra kroppen i hvileposen (som på arket);
    # stilens `arm_out` er vinkelen fra loddrett, så hvileposens trekkes fra.
    out = max(math.degrees(math.atan2(abs(J[f"wrist.{s}"].y - J[f"shoulder.{s}"].y), J[f"shoulder.{s}"].z - J[f"wrist.{s}"].z)) for s in ("L", "R"))
    st["arm_out"] = folk_motion.style_of(style)["arm_out"] * 0.5 - out * 0.65
    folk_motion.clips(rig, st, grips, only=("idle", "walk", "talk", "wave"))
    # Ansiktet får tre ganger så mye plass i teksturen som flaten tilsier.
    bake_textured(f"{OUT}/npc_{name}.glb", size, keep=("glass", "magic"), armature=rig, density={"head": 3.0})
    return m


if __name__ == "__main__":
    args = sys.argv[1:]
    names = [a for a in args if a in FOLK] or list(FOLK)
    size = next((int(a) for a in args if a.isdigit()), 2048)
    os.makedirs(OUT, exist_ok=True)
    for n in names:
        clear()
        build(n, size)
