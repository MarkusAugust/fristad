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

import numpy as np

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
# Armer arket viser bøyd forfra (de holder noe foran seg) men hengende
# bakfra: de bygges hengende etter bildet bakfra, og det de holder blir en
# egen gjenstand (`folk_sheet.measure_posed`). To positurer i én arm ble
# bånd og skiver (eieren 15. oktober).
OCC_DIFF, OCC_OVER, OCC_IN, OCC_FEATHER = 90, 0.05, 0.08, 6  # det som skiller seg så mye fra bildet bakfra (sum over rgb) dekkes, høyst så langt over midten og forbi det armen dekket, og innover så langt forbi (andel av høyden), tonet over så mange piksler
REST = {
    "maid": {"R": "krukka"},
    "apothecary": {"R": "morteren"},
}
TOOLS = {
    "gorvek": {"R": "øksa"},
    "farmer": {"L": "høygaffelen"},
    "guard": {"L": "spydet"},
    "smith": {"R": "hammeren"},
}
RING = {"body": 24, "arm": 14, "leg": 16, "held": 14, "head": 36, "hair": 24}  # hjørner rundt hvert tverrsnitt
THIN = 2.5  # tverrsnitt som ligger innen så mange piksler av naboenes linje, tas bort
WIDEN = 0.05  # er hodet fra hodearket så mye smalere eller bredere enn på kroppsarket, rettes bredden
NECK_WIN = 0.02  # halsens bredde er det smaleste så langt under hodet (andel av høyden)
NECK_SLOPE = 3.0  # under hodet blir kroppen høyst så mange piksler bredere per rad
JAW_FROM, JAW_MIN = 0.55, 0.5  # kjeven begynner så langt ned i hodet (fra issen til haka), og er minst så bred som hodet der
SHOULDER_IN = 0.03  # flatene som vender opp på kroppen tar fargen så langt ned (andel av høyden)
CHIN_FROM = 0.2  # hjørner som vender mer ned enn dette tar fargen gradvis høyere opp
CHIN_IN = 0.015  # flatene som vender ned (under hodet, bunnen av en kappe) tar fargen så langt opp (andel av høyden)
HAIR, HAIR_ABOVE, HAIR_LONG, HAIR_IN = 1.05, 0.04, 0.02, 0.92  # håret som henger ned: kroppsarket så mye bredere enn hodet og halsen, fra så langt over haka, minst så langt (andel av høyden), og toppen så smal som hodet
ISSE = 8  # kuppelen over hodet tar fargen så mange piksler innenfor issen på hodearket
RELIEF_W = 0.3  # nesa og munnen er så brede (del av hodets halve bredde)
TAPER = 0.35  # hodet: så mye smalere framme ved ansiktet enn ved ørene
ARMLESS_FROM = 0.03  # armen males bort fra siden fra så langt under skulderen (andel av høyden)
TOOL_GROW, TOOL_COLOR = 0.03, 50  # verktøyet vokser over piksler i sin farge (sum over rgb) så langt utenfor (andel av høyden)
ARMLESS_TIP, ARMLESS_HAND, ARMLESS_RADIUS, ARMLESS_WIDE, TOOL_PAD = 0.03, 0.06, 12, 1.4, 0.015  # hullet går så langt under enden av armen (fingrene), med hele hånda i de nederste radene, innmalt med denne radiusen (px), og armen så mange ganger så bred
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
    m = folk_sheet.measure_posed(name, FOLK[name], tuple(TOOLS.get(name, {})), tuple(REST.get(name, {})))
    path = os.path.join(folk_sheet.REF, f"{name}_body.jpeg")
    sheet = toolless(filled_sheet(path), m)
    mat = blended_sheet_material(f"{name}_sheet", sheet)
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
        # Bredden måles symmetrisk om midten av hodet: en flette på den ene
        # sida (tjenestejenta) er ikke hode, og strakk ansiktet 24 %.
        def sym(r):
            b = min(m["parts"]["body"], key=lambda q: abs(q[0] - r[0]))
            return b[2] - abs(b[1] - r[1])

        ratio = [sym(r) / r[2] for r in head if r[0] <= chin]
        k = float(sorted(ratio)[len(ratio) // 2]) if ratio else 1.0
        fit["kx"] = k if abs(k - 1) > WIDEN else 1.0
        head = [(r[0], r[1], r[2] * fit["kx"], *r[3:7], r[7] * fit["kx"], *r[8:]) for r in head]
        # Henger håret ved siden av kjeven (tjenestejenta), er kjeven like
        # smal som ansiktet hele veien rundt: med hårets bredde ble kjeven en
        # bred kloss med hår på sidene (eieren: «hodet er deformert»). Håret
        # ved siden av blir delen «hair» under.
        top_y = head[0][0]
        jaw = [r[0] > top_y + JAW_FROM * (chin - top_y) and r[7] < r[2] * (1 - folk_sheet.FRAMED_MIN) for r in head]
        if any(jaw):
            w = np.array([max(r[7], r[2] * JAW_MIN) if j else r[2] for r, j in zip(head, jaw)])
            w = np.minimum(w, [r[2] for r in head])
            from scipy.ndimage import gaussian_filter1d
            w = np.minimum(gaussian_filter1d(w, 2.0, mode="nearest"), [r[2] for r in head])
            # Under kjeven blir hodet bare smalere (radene under haka hadde
            # hårets bredde og ble en tallerken).
            i0 = jaw.index(True)
            for i in range(i0 + 1, len(w)):
                w[i] = min(w[i], w[i - 1])
            head = [(r[0], r[1], float(w[i]), *r[3:7], min(r[7], float(w[i])), *r[8:]) for i, r in enumerate(head)]
        parts["head"] = head
        m["y"]["chin"] = int(chin)
        # Kroppen begynner like over der hodet slutter, og er ikke bredere
        # enn hodet der de overlapper (den stakk ellers ut under haka).
        # Henger skjegget under halsen (smeden), måles skjøten ved halsen og
        # ikke ved skjeggspissen (kroppen krympet ellers til spissen).
        end = min(head, key=lambda q: abs(q[0] - min(head[-1][0], fit["neck"] + folk_sheet.STEP)))
        deep = (end[4] + end[5]) / 2

        f = m["facing"]
        # Henger skjegget ned over skuldrene, dekker hodet nakken, og halsen
        # vokser først ut fra skuldrene.
        join_y = m["y"]["shoulder"] if head[-1][0] > m["y"]["shoulder"] else end[0]
        # Hodets forkant og bakkant ved skjøten (forover er +).
        e_ax = end[3] * f
        e_front, e_back = e_ax + end[4] * 0.92, e_ax - end[5] * 0.92

        below_ = [r[2] for r in rows if end[0] < r[0] <= end[0] + NECK_WIN * m["H"]]
        neck_w = min(below_) if below_ else 0.0

        def joined(r):
            # Over skjøten ligger kroppen inne i hodet. Hodet slutter under
            # kjeven, så halsen under er smalere (den ble ellers tvunget like
            # bred som bunnen av hodet, og smeden fikk en skål under skjegget).
            # Under hodet vokser kroppen (håret over skuldrene) gradvis ut
            # fra bunnen av hodet, ikke som en hylle (fiskerkona).
            grow = max(0.0, (r[0] - join_y) * NECK_SLOPE)
            # Forkanten og bakkanten holdes innenfor hodets ved skjøten, ikke
            # bare dybden: bak skjegget sto halsen ellers ut bak nakken som
            # en kloss med flatt lokk (smeden).
            fr_, bk_ = r[3] * f + r[4], r[3] * f - r[4]
            fr_, bk_ = min(fr_, e_front + grow), max(bk_, e_back - grow)
            if fr_ <= bk_:
                fr_, bk_ = (fr_ + bk_) / 2 + 1, (fr_ + bk_) / 2 - 1
            # Under hodet vokser halsen fra halsens egen bredde på arket, ikke
            # fra spissen av hakeskjegget (apotekeren fikk en tynn hals og
            # skuldre malt med halsen).
            base = end[2] * 0.92 if r[0] <= end[0] else max(end[2] * 0.92, neck_w)
            return (r[0], r[1], min(r[2], base + grow), (fr_ + bk_) / 2 * f, (fr_ - bk_) / 2)

        parts["body"] = [joined(r) for r in rows if r[0] >= end[0] - 2 * folk_sheet.STEP]
        # Håret som henger ned ved siden av kjeven og halsen til skuldrene
        # (fiskerkona) er en egen del bak midten av hodet: kroppsarket er der
        # bredere enn hodet og halsen. Som hode ble det en flat kloss under
        # haka, og som hals en krage.
        hair = []
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
        # Skjegget som henger ned under haka (smeden) er en del av hodet
        # (`folk_sheet.head_rows` følger det til spissen). Kroppen i de
        # radene følger brystet under skjegget (det ble en krage rundt
        # halsen med striper av skjegget, eieren 14. oktober «skjegget er
        # merkelig delt»).
        tip = head[-1][0]
        if tip > m["y"]["shoulder"]:
            below = [r[3] + f * r[4] for r in parts["body"] if tip < r[0] <= tip + 6 * folk_sheet.STEP]
            if below:
                chest = float(np.median(below))
                body_rows = parts["body"]
                for i, r in enumerate(body_rows):
                    if r[0] <= tip:
                        b_ = r[3] - f * r[4]
                        f_ = r[3] + f * r[4]
                        f_ = min(f_, chest) if f > 0 else max(f_, chest)
                        if (f_ - b_) * f > 0:
                            body_rows[i] = (r[0], r[1], r[2], (b_ + f_) / 2, abs(f_ - b_) / 2)
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
            ob.data.materials[0] = blended_sheet_material(f"{name}_body_sheet", unoccluded(armless_side(sheet, m), m))
        project_part(ob, m, part)
    return m, obs


def unoccluded(path, m, pad=8):
    """Arket der en arm bygd etter bildet bakfra (`REST`) dekket kroppen
    forfra (underarmen og det den holder, `occluded`), fylt med bildet
    bakfra speilet inn (samme kjole, uten krukka): ellers lå krukka og
    hånda malt på brystet. Speilet fra den andre halvdelen forfra ble det en
    krukke til, for krukka står over midten."""
    occ = m.get("occluded")
    if not occ:
        return path
    import numpy as np
    from PIL import Image
    from scipy import ndimage
    out = path.rsplit(".", 1)[0] + "_open.png"
    im = np.asarray(Image.open(path).convert("RGB")).copy()
    cx, cb, H = m["views"]["front"], m["views"]["back"], m["H"]
    hole = np.zeros(im.shape[:2], bool)
    for y, a, b in occ:
        y0 = int(y) - folk_sheet.STEP // 2
        hole[max(0, y0): y0 + folk_sheet.STEP + 1, max(0, int(cx + a) - pad): int(cx + b) + pad + 1] = True
    xs_all = np.arange(im.shape[1])
    src = np.clip((cb - (xs_all - cx)).astype(int), 0, im.shape[1] - 1)
    # Hullet vokser over det som skiller seg fra bildet bakfra og henger
    # sammen med det (krukka stikker inn over midten av brystet), i de samme
    # radene og ikke langt over midten (fletta).
    rows_ = [int(y) for y, a, b in occ]
    y0, y1 = max(0, min(rows_) - folk_sheet.STEP), max(rows_) + folk_sheet.STEP
    f = im.astype(float)
    diff = np.abs(f - f[:, src]).sum(2) > OCC_DIFF
    # Bare nær det armen og tingen dekket: hos apotekeren vokste det ellers
    # over hele kjortelen, som er annerledes bakfra (reimene og veskene).
    left = min(a for _, a, _ in occ) < 0
    # Innover (mot midten) kan tingen stikke lenger enn det motoren fant
    # (krukka over brystet), utover ikke.
    lo_x = cx + min(a for _, a, _ in occ) - (OCC_OVER if left else OCC_IN) * H
    hi_x = cx + max(b for _, _, b in occ) + (OCC_IN if left else OCC_OVER) * H
    near = (xs_all < min(hi_x, cx + OCC_OVER * H)) & (xs_all > lo_x) if left else (xs_all > max(lo_x, cx - OCC_OVER * H)) & (xs_all < hi_x)
    diff[:y0] = False
    diff[y1:] = False
    diff &= near[None, :]
    lab, _ = ndimage.label(ndimage.binary_closing(diff | hole, iterations=2))
    hit = np.unique(lab[hole])
    hole |= np.isin(lab, hit[hit > 0]) & near[None, :]
    hole = ndimage.binary_dilation(hole, iterations=3)
    # Bildet bakfra ligger i en annen skygge: i hver rad legges forskjellen
    # mellom bildet forfra og bakfra ved kantene av hullet på, jevnt fra
    # den ene kanten til den andre og glattet over radene (én felles
    # lysstyrke ga et mørkt felt over brystet).
    fill = f[:, src].copy()
    off = np.zeros_like(f)
    for y in np.nonzero(hole.any(1))[0]:
        xs = np.nonzero(hole[y])[0]
        for seg in np.split(xs, np.nonzero(np.diff(xs) > 1)[0] + 1):
            a, b = seg[0], seg[-1]
            la, lb = max(0, a - 4), min(im.shape[1], b + 5)
            dl = (f[y, la:a] - fill[y, la:a]).mean(0) if a > la else None
            dr = (f[y, b + 1:lb] - fill[y, b + 1:lb]).mean(0) if lb > b + 1 else None
            # Bare der forfra og bakfra viser samme stoff (ved fletta er
            # forskjellen ikke lys, men en annen ting).
            dl = dl if dl is not None and np.abs(dl).sum() < OCC_DIFF else None
            dr = dr if dr is not None and np.abs(dr).sum() < OCC_DIFF else None
            dl = dl if dl is not None else (dr if dr is not None else np.zeros(3))
            dr = dr if dr is not None else dl
            t = np.linspace(0, 1, b - a + 1)[:, None]
            off[y, a:b + 1] = dl * (1 - t) + dr * t
    for c in range(3):
        off[..., c] = ndimage.gaussian_filter1d(off[..., c], OCC_FEATHER, axis=0)
    im = np.where(hole[..., None], np.clip(fill + off, 0, 255), f).astype(np.uint8)
    Image.fromarray(im).save(out)
    return out


def toolless(path, m, pad=6):
    """Arket uten verktøyet i hånda (`TOOLS`; smedens hammer, Gorveks øks),
    som blir en egen gjenstand: fra siden (`tool_side`) og forfra og bakfra
    (`tool_front`, speilet bakfra), malt inn fra kantene (OpenCV). Alle
    delene tar dette arket (øksa lå ellers malt på låret og hånda)."""
    if not m.get("tool_side") and not m.get("tool_front"):
        return path
    import cv2
    import numpy as np
    out = path.rsplit(".", 1)[0] + "_toolless.png"
    im = cv2.imread(path)
    sx0, H = m["views"]["side"], m["H"]
    hole = np.zeros(im.shape[:2], bool)

    def span(y, a, b):
        y0 = int(y) - folk_sheet.STEP // 2
        hole[max(0, y0): y0 + folk_sheet.STEP + 1, max(0, int(sx0 + a) - pad): int(sx0 + b) + pad + 1] = True

    for y, a, b in m.get("tool_side", []):
        span(y, a, b)
    # Og verktøyet forfra og bakfra (speilet), med margin: hammerhodet lå
    # malt på forkleet, og øksa på låret bakfra.
    cx, cb = m["views"]["front"], m["views"]["back"]
    grow = int(TOOL_PAD * H)
    for y, a, b in m.get("tool_front", []):
        y0 = int(y) - folk_sheet.STEP // 2
        rows = slice(max(0, y0 - grow), y0 + folk_sheet.STEP + 1 + grow)
        hole[rows, max(0, int(cx + a) - grow): int(cx + b) + grow + 1] = True
        hole[rows, max(0, int(cb - b) - grow): int(cb - a) + grow + 1] = True
    # Det av verktøyet som ligger over låret, har verktøyets farge og henger
    # sammen med det som stakk ut (øksehodet foran Gorveks lår).
    from scipy import ndimage
    if hole.any():
        px_ = im.astype(float)
        fg0_ = folk_sheet.mask(os.path.join(folk_sheet.REF, f"{m['name']}_body.jpeg"))
        tool_col = np.median(px_[hole & fg0_], 0) if (hole & fg0_).any() else np.median(px_[hole], 0)
        ys_, xs_ = np.nonzero(hole)
        g = int(TOOL_GROW * H)
        box = np.zeros_like(hole)
        box[max(0, ys_.min() - g): ys_.max() + g, max(0, xs_.min() - g): xs_.max() + g] = True
        fg0 = folk_sheet.mask(os.path.join(folk_sheet.REF, f"{m['name']}_body.jpeg"))
        like = (np.abs(px_ - tool_col).sum(2) < TOOL_COLOR) & box & fg0
        # Bare et kort stykke fra verktøyet (forkleet til smeden har nesten
        # samme farge som skaftet, og hele forkleet ble flekkete).
        hole |= ndimage.binary_dilation(hole, iterations=int(TOOL_GROW * H), mask=like)
    hole = cv2.dilate(hole.astype(np.uint8), np.ones((5, 5), np.uint8)) > 0
    # Fargen hentes bare fra figuren: utenfor omrisset er fyllet fra kanten
    # (rundt øksa grått), og innmalingen ble grå.
    fg = folk_sheet.mask(os.path.join(folk_sheet.REF, f"{m['name']}_body.jpeg"))
    near = ndimage.binary_dilation(hole, iterations=int(TOOL_GROW * H))
    mask = hole | (near & ~fg)
    cv2.imwrite(out, cv2.inpaint(im, mask.astype(np.uint8) * 255, ARMLESS_RADIUS, cv2.INPAINT_TELEA))
    return out


def armless_side(path, m, pad=6):
    """Arket med armen fra siden malt bort, så kroppen ikke får et bilde av
    armen bak den ekte armen (eieren så en arm til, skrått og fra siden).
    Hullet er armen nærmest fra albuen til fingertuppene der motoren
    plasserte den, med margin, og den bare huden der arket viser den
    (`bare_side`).
    Det males inn fra kantene (OpenCV); speilet fra baksida ble det tagget,
    og bare huden alene lot hånda stå igjen forskjøvet."""
    import cv2
    import numpy as np
    out = path.rsplit(".", 1)[0] + "_armless.png"
    im = cv2.imread(path)
    sx0, H = m["views"]["side"], m["H"]
    hole = np.zeros(im.shape[:2], bool)

    def span(y, a, b, grow=0):
        y0 = int(y) - folk_sheet.STEP // 2
        hole[max(0, y0): y0 + folk_sheet.STEP + 1, max(0, int(sx0 + a) - pad - grow): int(sx0 + b) + pad + grow + 1] = True

    bare = m.get("bare_side", [])
    if bare:
        # Armen nærmest: den motoren fant hud på fra siden.
        found = [(y, (a + b) / 2, (b - a) / 2) for y, a, b in bare]
        for chain in folk_sheet.chains(sorted(found), 1, 2):
            ys = np.array([q[0] for q in chain], float)
            ca = np.array([q[1] - q[2] for q in chain], float)
            cb = np.array([q[1] + q[2] for q in chain], float)
            low = ys >= ys[-1] - ARMLESS_HAND * H
            la, lb = ca[low].min(), cb[low].max()
            for y in range(int(ys[0]), int(ys[-1] + ARMLESS_TIP * H) + 1, 2):
                a_, b_ = np.interp(y, ys, ca), np.interp(y, ys, cb)
                if y >= ys[-1] - ARMLESS_HAND * H:
                    a_, b_ = min(a_, la), max(b_, lb)
                span(y, a_, b_)
        near = min(("R", "L"), key=lambda s: min(abs(r[3] - found[0][1]) for r in m["parts"].get(f"arm.{s}", [(0, 0, 0, 1e9, 0)])))
        arm = m["parts"].get(f"arm.{near}", [])
        elbow = m["y"].get(f"elbow.{near}", 0)
        # Og armen der modellen har den, med margin (hånda står ellers igjen
        # ved siden av den bare huden).
        for r in arm:
            if r[0] >= elbow:
                w = max(r[2], r[4]) * ARMLESS_WIDE
                span(r[0], r[3] - w, r[3] + w)
    hole[:, : int(sx0 - 0.3 * H)] = False
    hole[:, int(sx0 + 0.3 * H):] = False
    hole = cv2.dilate(hole.astype(np.uint8), np.ones((5, 5), np.uint8))
    cv2.imwrite(out, cv2.inpaint(im, hole * 255, ARMLESS_RADIUS, cv2.INPAINT_TELEA))
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

    # En arm bygd etter bildet bakfra tar bare det bildet (forfra viser
    # arket den bøyd og med krukka).
    rest = arm and part[-1] in m.get("rest", ())

    def front_uv(p, n, inset):
        cx, cy = mid[round(p.z, 4)]
        y = cy + (p.y - cy) * (1 - inset)
        back = rest or n.x < -0.05 or (abs(n.x) <= 0.05 and p.x < cx)
        X = v["back"] - y / px if back else v["front"] + y / px
        return X

    for poly in me.polygons:
        n = (nm @ poly.normal).normalized()
        ax, ay = abs(n.x), abs(n.y)
        share = ay / (ax + ay + 1e-6)
        # Hele flaten tar samme bilde (blandes hjørnene, blir flaten et
        # smurt sagtakk).
        hidden = rest or covered(mw @ poly.center)
        # En flate som vender rett ned eller opp (under et vidt erme, et
        # lokk) tar fargen fra kanten den henger på: med bildet i sin egen
        # høyde fikk skriveren hudfargede tallerkener under ermene.
        zs = [(mw @ me.vertices[me.loops[li].vertex_index].co).z for li in poly.loop_indices]
        flat_z = (max(zs) if n.z < 0 else min(zs)) if abs(n.z) > FLAT and not (part == "body" and n.z > 0) else None
        for li in poly.loop_indices:
            p = mw @ me.vertices[me.loops[li].vertex_index].co
            Y = v["ground"] - (p.z if flat_z is None else flat_z) / px
            if flat_z is not None and n.z < 0:
                # Litt opp fra kanten: på kanten selv viser arket streken og
                # det under (bunnen av kjøpmannens kappe ble en flekkete skive).
                Y -= CHIN_IN * H
            elif part == "body":
                # Toppen av skuldrene tar fargen litt ned i tøyet: på kanten
                # viser arket halsen, kragen og skjegget, strukket til striper
                # rundt halsen (apotekeren, eieren: «stygg strek på halsen»).
                # Hvor langt følger hjørnets normal jevnt (med en terskel på
                # flatens normal ble det lapper rundt halsen).
                vz = (nm @ me.vertices[me.loops[li].vertex_index].normal).normalized().z
                Y += SHOULDER_IN * H * min(1.0, max(0.0, (vz - CHIN_FROM) / (1 - CHIN_FROM)))
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
            # Under haka (og lokket under hodet) tar flaten fargen litt opp
            # i ansiktet: der den står, viser arket streken langs kjeven, og
            # det ble en mørk strek rundt halsen. Hvor langt følger
            # hjørnets normal jevnt (med flatens normal og en terskel
            # hoppet bildet mellom nabobitene, en trapp under haka).
            # Bare under haka: nesa og leppene vender også ned.
            if Y > m["y"].get("chin", 0) - CHIN_IN * m["H"]:
                vz = (nm @ me.vertices[me.loops[li].vertex_index].normal).normalized().z
                Y -= CHIN_IN * m["H"] * min(1.0, max(0.0, (-vz - CHIN_FROM) / (1 - CHIN_FROM)))
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
    # Skjegget følger hodet.
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
