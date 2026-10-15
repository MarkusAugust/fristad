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
# Det figurene holder i hånda, som blir egne gjenstander (ikke en del av
# kroppen): `folk_check` regner det ikke som hull ved den hånda.
TOOLS = {
    "gorvek": {"R": "øksa"},
    "farmer": {"L": "høygaffelen"},
    "guard": {"L": "spydet"},
    "smith": {"R": "hammeren"},
}
RING = {"body": 24, "arm": 14, "leg": 16, "held": 14, "head": 36, "hair": 24}  # hjørner rundt hvert tverrsnitt
THIN = 2.5  # tverrsnitt som ligger innen så mange piksler av naboenes linje, tas bort
WIDEN = 0.05  # er hodet fra hodearket så mye smalere eller bredere enn på kroppsarket, rettes bredden
NECK_SLOPE = 3.0  # under hodet blir kroppen høyst så mange piksler bredere per rad
CHIN_IN = 0.015  # flatene under hodet tar fargen så langt opp (andel av høyden)
HAIR, HAIR_ABOVE, HAIR_LONG, HAIR_IN = 1.05, 0.04, 0.02, 0.92  # håret som henger ned: kroppsarket så mye bredere enn hodet og halsen, fra så langt over haka, minst så langt (andel av høyden), og toppen så smal som hodet
ISSE = 8  # kuppelen over hodet tar fargen så mange piksler innenfor issen på hodearket
RELIEF_W = 0.3  # nesa og munnen er så brede (del av hodets halve bredde)
TAPER = 0.35  # hodet: så mye smalere framme ved ansiktet enn ved ørene
ARMLESS_FROM = 0.03  # armen males bort fra siden fra så langt under skulderen (andel av høyden)
ARMLESS_TIP, ARMLESS_HAND, ARMLESS_RADIUS = 0.03, 0.06, 12  # og så langt under enden av armen (fingrene), innmalt med denne radiusen (px)
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
    # Hodet tynnes mindre (der ble omrisset kantete).
    for r in thinned(smoothed(rows), THIN / 3 if split else THIN):
        y, cy, hw, sx = r[:4]
        front, back = (r[4], r[5]) if split else (r[4], r[4])
        # Hodet: det som stikker fram av ansiktsplanet (nesa, leppene,
        # haka) er en smal bule midt i ansiktet (`folk_sheet.rounded_head`).
        relief = r[6] if split and len(r) > 6 else 0.0
        # Og foran på hodet kan det være smalere enn bak (håret som henger
        # bak kinnene).
        front_w = r[7] if split and len(r) > 7 else hw
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
            bump = relief * px * math.exp(-(ey / RELIEF_W) ** 2) * max(c, 0.0) ** 2 if relief else 0.0
            # Foran og på sidene holder hodet ansiktets bredde; hele bredden
            # (håret) kommer bak øret, der profilen viser håret.
            af = min(a, max(front_w * px, 0.004))
            lat = af + (a - af) * min(1.0, max(0.0, -c / 0.6)) if split else a
            loop.append(bm.verts.new((cx + b * ex + bump, cyy + lat * ey, z)))
        loops.append(loop)
    # Uv-laget er delen brettet ut som én sylinder (sømmen bak), i meter:
    # rundt etter snittets omkrets, langs etter midtlinja. Utbrettet av
    # Blender ga hundrevis av små øyer med svarte kanter som synes på
    # armene (eieren 14. oktober).
    uvl = bm.loops.layers.uv.new("UVMap")
    mids = [sum((v.co for v in l), Vector()) / n for l in loops]
    per = sum(sum((l[i].co - l[(i + 1) % n].co).length for i in range(n)) for l in loops) / len(loops)
    # Langs etter flaten, ikke midtlinja: en hattebrem er nesten flat, og
    # med høyden mellom radene ble den klemt til en strek i teksturen.
    along = [0.0]
    for l0, l1 in zip(loops, loops[1:]):
        along.append(along[-1] + sum((a.co - b.co).length for a, b in zip(l0, l1)) / n)

    def u_of(i, last=False):
        # Sømmen går bak (indeksen n/2 er bakover, −X).
        k = (i - n // 2) % n
        return (n if last and k == 0 else k) / n * per

    for r, (l0, l1) in enumerate(zip(loops, loops[1:])):
        for i in range(n):
            j = (i + 1) % n
            f = bm.faces.new((l0[i], l0[j], l1[j], l1[i]))
            for lp, (ii, rr) in zip(f.loops, ((i, r), (j, r), (j, r + 1), (i, r + 1))):
                lp[uvl].uv = (u_of(ii, last=ii == j), -along[rr])
    # Lokkene ligger ved siden av sylinderen i uv-laget, hvert for seg (de
    # lå oppå toppen av den og delte piksler med halsen: en prikkete mørk
    # strek under haka, eieren 14. oktober «stygg strek på halsen»).
    R = max((v.co - mids[k]).length for k, l in enumerate(loops) for v in l) + 0.01
    for k, (cap, ring) in enumerate(((cap_top, 0), (cap_bottom, len(loops) - 1))):
        if not cap:
            continue
        verts = loops[ring] if ring else list(reversed(loops[0]))
        f = bm.faces.new(verts)
        c = mids[ring]
        for lp in f.loops:
            lp[uvl].uv = (per + R + 0.01 + lp.vert.co.x - c.x, -(2 * k + 1) * R + lp.vert.co.y - c.y)
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
    m = folk_sheet.measure(name, FOLK[name], tuple(TOOLS.get(name, {})))
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
        # Kroppsarket bestemmer bredden: tegner hodearket hodet smalere enn
        # kroppsarket (tjenestejenta, 18 %), strekkes hodet og bildet av det
        # like mye i bredden.
        ratio = [min(rows_b, key=lambda q: abs(q[0] - r[0]))[2] / r[2] for r in head if r[0] <= chin for rows_b in [m["parts"]["body"]]]
        k = float(sorted(ratio)[len(ratio) // 2]) if ratio else 1.0
        fit["kx"] = k if abs(k - 1) > WIDEN else 1.0
        head = [(r[0], r[1], r[2] * fit["kx"], *r[3:7], r[7] * fit["kx"], *r[8:]) for r in head]
        parts["head"] = head
        m["y"]["chin"] = int(chin)
        # Kroppen begynner like over der hodet slutter, og er ikke bredere
        # enn hodet der de overlapper (den stakk ellers ut under haka).
        end = head[-1]
        deep = (end[4] + end[5]) / 2

        def joined(r):
            # Over skjøten ligger kroppen inne i hodet. Hodet slutter under
            # kjeven, så halsen under er smalere (den ble ellers tvunget like
            # bred som bunnen av hodet, og smeden fikk en skål under skjegget).
            if r[0] <= end[0]:
                return (r[0], r[1], min(r[2], end[2] * 0.92), r[3], min(r[4], deep * 0.92))
            # Under hodet vokser kroppen (håret over skuldrene) gradvis ut
            # fra bunnen av hodet, ikke som en hylle (fiskerkona).
            grow = (r[0] - end[0]) * NECK_SLOPE
            return (r[0], r[1], min(r[2], end[2] * 0.92 + grow), r[3], min(r[4], deep * 0.92 + grow))

        parts["body"] = [joined(r) for r in rows if r[0] >= end[0] - 2 * folk_sheet.STEP]
        # Håret som henger ned ved siden av kjeven og halsen til skuldrene
        # (fiskerkona) er en egen del bak midten av hodet: kroppsarket er der
        # bredere enn hodet og halsen. Som hode ble det en flat kloss under
        # haka, og som hals en krage.
        hair, f = [], m["facing"]
        for r in rows:
            if not chin - HAIR_ABOVE * m["H"] <= r[0] <= m["y"]["shoulder"]:
                continue
            h = min(head, key=lambda q: abs(q[0] - r[0]))
            inner = h[2] if r[0] <= end[0] else joined(r)[2]
            back_, mid_ = r[3] - f * r[4], h[3]
            if r[2] > inner * HAIR and (mid_ - back_) * f > 0:
                hair.append((r[0], r[1], r[2], (back_ + mid_) / 2, abs(mid_ - back_) / 2))
            elif hair:
                break
        if hair:
            # Og videre ned i skuldrene (en flat underkant over skuldrene
            # synes nedenfra).
            last = hair[-1]
            for r in rows:
                if r[0] <= last[0]:
                    continue
                if r[0] > m["y"]["shoulder"] + HAIR_ABOVE * m["H"]:
                    break
                hair.append((r[0], *last[1:]))
        if hair and hair[-1][0] - hair[0][0] >= HAIR_LONG * m["H"]:
            # Toppen begynner inne i hodet (oppover til der hodet er bredere
            # enn håret), så håret vokser ut av hodet der kjeven smalner, ikke
            # som en hylle.
            above = [r for r in rows if r[0] < hair[0][0]]
            for r in reversed(above):
                h = min(head, key=lambda q: abs(q[0] - r[0]))
                back_, mid_ = r[3] - f * r[4], h[3]
                if (mid_ - back_) * f <= 0:
                    break
                hair.insert(0, (r[0], r[1], min(r[2], h[2] * HAIR_IN), (back_ + mid_) / 2, abs(mid_ - back_) / 2))
                if r[2] <= h[2]:
                    break
            parts["hair"] = hair
    obs = {}
    for part, rows in parts.items():
        if len(rows) < 2:
            continue
        kind = part.split(".")[0]
        # Hodet er rundt (en kantet skalle får ansiktet forfra og profilen
        # på samme hjørne), uten lokk under (det står inne i kroppen).
        obs[part] = ring_mesh(part.replace(".", "_"), rows, m, mat, RING[kind], 2.0 if part.startswith("arm") or part in ("head", "hair") else SQUARE)
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
        if part == "body":
            # Fra siden ligger armen over kroppen på arket; kroppen får et
            # sideark der armen er malt bort (det ble en arm til, malt på
            # kroppen bak armen, sett skrått og fra siden).
            ob.data.materials[0] = blended_sheet_material(f"{name}_body_sheet", armless_side(filled_sheet(path), m))
        project_part(ob, m, part)
    return m, obs


def armless_side(path, m, pad=6):
    """Arket med armen fra siden malt bort, så kroppen ikke får et bilde av
    armen bak den ekte armen (eieren så en arm til, skrått og fra siden).
    Hullet er den bare armen og hånda fra siden, slik motoren fant dem
    etter hudfargen (`bare_side`; en maske etter fargen alene tok med
    forkleet). Ermet har kjolens farge og står. Hver rad fylles med kjolen speilet fra
    baksida av hullet, så stoffet beholder strøkene; det som ikke har noe å
    speile, males inn fra kantene (OpenCV)."""
    import cv2
    import numpy as np
    out = path.rsplit(".", 1)[0] + "_armless.png"
    im = cv2.imread(path)
    sx0, H, facing = m["views"]["side"], m["H"], m["facing"]
    top = m["y"]["shoulder"] + int(ARMLESS_FROM * H)
    x0, x1 = int(sx0 - 0.3 * H), int(sx0 + 0.3 * H)
    # Den bare armen der motoren fant den fra siden (`bare_side`, rad for
    # rad): radene imellom fylles, og hånda forlenges til fingertuppene.
    hole = np.zeros(im.shape[:2], bool)
    found = [(y, (a + b) / 2, (b - a) / 2) for y, a, b in m.get("bare_side", [])]
    for chain in folk_sheet.chains(sorted(found), 1, 2):
        ys = np.array([q[0] for q in chain], float)
        ca = np.array([q[1] - q[2] for q in chain], float)
        cb = np.array([q[1] + q[2] for q in chain], float)
        # Nederst er hånda så bred som den er bredest der (fingrene ble
        # funnet som smale biter og sto igjen på skjørtet).
        low = ys >= ys[-1] - ARMLESS_HAND * H
        la, lb = ca[low].min(), cb[low].max()
        for y in range(int(ys[0]), int(ys[-1] + ARMLESS_TIP * H) + 1):
            a_, b_ = np.interp(y, ys, ca), np.interp(y, ys, cb)
            if y >= ys[-1] - ARMLESS_HAND * H:
                a_, b_ = min(a_, la), max(b_, lb)
            hole[y, int(sx0 + a_) - pad: int(sx0 + b_) + pad + 1] = True
    hole[:, :x0] = False
    hole[:, x1:] = False
    rest = hole.copy()
    # Bare stoff innenfor omrisset speiles (bakgrunnen ved siden av er fylt
    # med kantfargene).
    inside = folk_sheet.mask(os.path.join(folk_sheet.REF, f"{m['name']}_body.jpeg"))
    back = -facing  # baksida av figuren fra siden
    for y in np.nonzero(hole.any(1))[0]:
        xs = np.nonzero(hole[y])[0]
        # Bitene av hullet i raden, hver speilet fra baksida.
        for seg in np.split(xs, np.nonzero(np.diff(xs) > 1)[0] + 1):
            a, b = seg[0], seg[-1]
            n_ = b - a + 1
            src = (a - 1 - np.arange(n_)) if back < 0 else (b + 1 + np.arange(n_))
            dst = np.arange(a, b + 1) if back < 0 else np.arange(b, a - 1, -1)
            ok = (src >= 0) & (src < im.shape[1])
            ok[ok] &= ~hole[y, src[ok]] & inside[y, src[ok]]
            im[y, dst[ok]] = im[y, src[ok]]
            rest[y, dst[ok]] = False
    im = cv2.inpaint(im, rest.astype(np.uint8) * 255, ARMLESS_RADIUS, cv2.INPAINT_TELEA)
    cv2.imwrite(out, im)
    return out


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


FLAT = 0.75  # en flate med så stor del av normalen opp eller ned vender opp eller ned
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
        # Armen står fra siden der arket viser den (`folk_sheet`), og både
        # armen og kroppen tar sidebildet: bildet forfra på flater som vender
        # til siden ble utstrukne striper eller lapper.
        return False

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
        # En flate som vender rett ned eller opp (under et vidt erme, et
        # lokk) tar fargen fra kanten den henger på: med bildet i sin egen
        # høyde fikk skriveren hudfargede tallerkener under ermene.
        zs = [(mw @ me.vertices[me.loops[li].vertex_index].co).z for li in poly.loop_indices]
        flat_z = (max(zs) if n.z < 0 else min(zs)) if abs(n.z) > FLAT else None
        for li in poly.loop_indices:
            p = mw @ me.vertices[me.loops[li].vertex_index].co
            Y = v["ground"] - (p.z if flat_z is None else flat_z) / px
            if part.startswith("leg"):
                Y = min(Y, v["ground"] - SOLE * H)
            X = front_uv(p, n, (INSET_HIDDEN if arm else INSET) * share)
            src.data[li].uv = (X / W, 1.0 - Y / Hs)
            if hidden:
                # Nøyaktig samme som bildet forfra: et eget, sterkere
                # innskrudd utsnitt ga lapper på kroppen fra siden.
                Xs = X
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
    tops = {}
    for poly in me.polygons:
        n = (nm @ poly.normal).normalized()
        if views == ("side",):
            view = "side"
        elif views == ("front", "back") or abs(n.x) >= abs(n.y) * 0.5:
            view = "front" if (n.x > 0 if abs(n.x) > 0.02 else (mw @ poly.center).x > 0) else "back"
        else:
            view = "side"
        _, s, tx, ty, flip, (qx0, qx1) = fit[view]
        # Kuppelen over den øverste raden tar fargen innenfor issen, ikke
        # fra fyllet over den (gutten fikk rosa flekker på toppen).
        if view not in tops:
            # Toppen av omrisset i hver kolonne av ruta.
            col = {}
            for hy_, rs in sorted(folk_sheet.quad_rows(fit, view).items()):
                for a_, b_ in rs:
                    for x_ in range(a_, b_):
                        col.setdefault(x_, hy_)
            tops[view] = (min(col.values()), col)
        top_hy, col = tops[view]
        for li in poly.loop_indices:
            p = mw @ me.vertices[me.loops[li].vertex_index].co
            if view == "front":
                X = v["front"] + p.y / px / fit.get("kx", 1.0)
            elif view == "back":
                X = v["back"] - p.y / px / fit.get("kx", 1.0)
            else:
                X = v["side"] + v.get("side_dir", 1) * p.x / px
            Y = v["ground"] - p.z / px
            if n.z < -FLAT:
                # Under haka (og lokket under hodet) tar flaten fargen litt
                # opp i ansiktet: der den står, viser arket streken langs
                # kjeven, og det ble en mørk strek rundt halsen.
                Y -= CHIN_IN * m["H"]
            hx, hy = (X - tx) / s, (Y - ty) / s
            # Over omrisset (kuppelen over issen) tar flaten fargen like
            # innenfor toppen av omrisset i samme kolonne: én rad strukket
            # over kuppelen ble en stripe, en rad langt inn traff huden.
            ct = col.get(int(round(2 * (qx1 - 1) - hx if flip else hx)))
            if ct is not None and hy < ct + ISSE:
                hy = ct + ISSE
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
    # Håret som henger ned følger hodet og halsen.
    "hair": ("chest", "neck", "head"),
    "body": ("hips", "spine", "chest", "neck", "head"),
    "arm.L": ("upper_arm.L", "forearm.L", "hand.L"),
    "arm.R": ("upper_arm.R", "forearm.R", "hand.R"),
    "leg.L": ("thigh.L", "shin.L", "foot.L"),
    "leg.R": ("thigh.R", "shin.R", "foot.R"),
    # Det figuren bærer (kurven) følger underarmen og hånda.
    "held.L": ("forearm.L", "hand.L"),
    "held.R": ("forearm.R", "hand.R"),
    # Det som bare synes fra siden (sekken på ryggen) følger brystet.
    "held.back": ("spine", "chest"),
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
        # En del nummer to (`held.back2`) følger de samme beina som den første.
        weigh(ob, rig, OWN[part.rstrip("0123456789")])
    skirt_follows_thighs(obs["body"], J)
    style, grips = folk_motion.FIGURES.get(name, ("default", {}))
    st = dict(folk_motion.STYLES.get(style, {}))
    # Armene henger alt ut fra kroppen i hvileposen (som på arket);
    # stilens `arm_out` er vinkelen fra loddrett, så hvileposens trekkes fra.
    out = max(math.degrees(math.atan2(abs(J[f"wrist.{s}"].y - J[f"shoulder.{s}"].y), J[f"shoulder.{s}"].z - J[f"wrist.{s}"].z)) for s in ("L", "R"))
    st["arm_out"] = folk_motion.style_of(style)["arm_out"] * 0.5 - out * 0.65
    folk_motion.clips(rig, st, grips, only=("idle", "walk", "talk", "wave"))
    # Ansiktet får tre ganger så mye plass i teksturen som flaten tilsier.
    # Arket har lyset og skyggene malt inn, og armen ligger inntil kroppen:
    # okklusjonen ga svarte flekker på armen (eieren 14. oktober). Øyene er
    # delene brettet ut som sylindre (`ring_mesh`), og delene bakes hver for
    # seg (armen ligger delvis inne i kroppen).
    bake_textured(f"{OUT}/npc_{name}.glb", size, keep=("glass", "magic"), armature=rig, density={"head": 3.0}, uv_angle=None, ao=False, apart=True)
    return m


if __name__ == "__main__":
    args = sys.argv[1:]
    names = [a for a in args if a in FOLK] or list(FOLK)
    size = next((int(a) for a in args if a.isdigit()), 2048)
    os.makedirs(OUT, exist_ok=True)
    import folk_check

    failed = []
    for n in names:
        clear()
        build(n, size)
        # Porten: figuren skal treffe arket (`folk_check`), ellers stopper
        # byggingen (eieren 14. oktober: «treffer mer riktig med en gang»).
        r = folk_check.check(n, os.path.join(folk_sheet.ROOT, "target", "qa", "folk"))
        if r["fails"]:
            failed.append(f"{n}: " + "; ".join(r["fails"]))
    if failed:
        raise SystemExit("figurene treffer ikke arket (se target/qa/folk/<navn>_check.jpg):\n" + "\n".join(failed))
