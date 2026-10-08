"""Borgruinen på haugen, krypten, døra opp fra kjelleren og portgitteret,
modellert i Blender (bpy) og eksportert til assets/models/town/. Kjør:

    python3 tools/models/castle.py [castle_ruin crypt crypt_stair portcullis]

Borgruinen (`castle_ruin`): ringmurer av hogd stein, 40 × 40 m rundt
borggården, med brukne, takkete topper, runde hjørnetårn som er rast
halvveis, et kvadratisk hovedtårn med én side falt ut, porttårn ved
åpningen i sør, og steinrøyser og nedfalne steiner langs murene og midt
i borggården (der står en røys med kollider, så instansen har noe ved
foten). Murene ligger der byen gir dem kollidere (`town::castle`).

Krypten (`crypt`): et lite gravkapell av hogd stein med bratt skifertak,
kors på gavlen, rundbuet portal med eierens jernbeslåtte dør (bildet i
assets/textures/door), trinn ned foran og fakkelholdere. Døra vender mot
sør (lokal +Z i motoren).

Døra opp (`crypt_stair`): en rundbuet steinportal i nordveggen i
startrommet i kjelleren, med samme dør, terskelstein og fakkelholdere.

Portgitteret (`portcullis`): et gitter av smijern, 3,9 × 2,9 m, med
spisser nederst; origo nederst på midten, heises av økta.

Akser i Blender som i town.py: −Y fram, Z opp, origo midt på grunnflaten."""

import math
import os
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import clear  # noqa: E402
from mathutils import Vector  # noqa: E402

import town  # noqa: E402
from town import M, PIECES, beam, block, cut, finish, layer, plain, prism, roof  # noqa: E402
from walls import cutter, link, revolve  # noqa: E402

OUT = "assets/models/town"
DOOR = "assets/textures/door"


def mats(**kw):
    m = M(**kw)
    m.ashlar = layer("brick", (0.6, 0.58, 0.55), 0.5)
    m.rubble = layer("stone", (0.55, 0.53, 0.5), 0.5)
    m.moss = layer("sod", (0.55, 0.62, 0.48), 0.5)
    m.dark_iron = layer("metal", (0.3, 0.28, 0.27), 0.5)
    m.glow = plain("glow", (1.0, 0.55, 0.2), 0.4)
    return m


def photo_door(x, y, z0, w, h, normal_y=-1.0):
    """Eierens jernbeslåtte dør som én flate med bildet (albedo og normal),
    uv 0..1 over hele døra. Blir modellens bakte tekstur i motoren."""
    mat = bpy.data.materials.get("door_photo")
    if mat is None:
        mat = bpy.data.materials.new("door_photo")
        mat.use_nodes = True
        nt = mat.node_tree
        bsdf = nt.nodes["Principled BSDF"]
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = bpy.data.images.load(os.path.abspath(f"{DOOR}/albedo.jpg"))
        nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
        nrm = nt.nodes.new("ShaderNodeTexImage")
        nrm.image = bpy.data.images.load(os.path.abspath(f"{DOOR}/normal.jpg"))
        nrm.image.colorspace_settings.name = "Non-Color"
        nm = nt.nodes.new("ShaderNodeNormalMap")
        nt.links.new(nrm.outputs["Color"], nm.inputs["Color"])
        nt.links.new(nm.outputs["Normal"], bsdf.inputs["Normal"])
        bsdf.inputs["Roughness"].default_value = 0.7
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.new("UVMap")
    # Fronten (mot normal_y) og en tynn bakside, så døra har tykkelse.
    yf = y + normal_y * 0.0
    vs = [bm.verts.new(v) for v in [(x - w / 2, yf, z0), (x + w / 2, yf, z0), (x + w / 2, yf, z0 + h), (x - w / 2, yf, z0 + h)]]
    f = bm.faces.new(vs if normal_y < 0 else vs[::-1])
    for loop, uv in zip(f.loops, [(0, 0), (1, 0), (1, 1), (0, 1)] if normal_y < 0 else [(0, 1), (1, 1), (1, 0), (0, 0)]):
        loop[uvl].uv = uv
    ob = link("door_photo", bm, mat)
    return ob


def rubble(m, rng, cx, cy, spread, n, size=(0.3, 0.7)):
    """En røys av nedfalne steiner."""
    for _ in range(n):
        a = rng.uniform(0, math.tau)
        r = spread * math.sqrt(rng.random())
        s = rng.uniform(*size)
        block("rubble", (s, s * rng.uniform(0.6, 1.0), s * rng.uniform(0.4, 0.7)), m.rubble,
              at=(cx + math.cos(a) * r, cy + math.sin(a) * r, s * 0.2 - 0.05), rot=(rng.uniform(-0.3, 0.3), rng.uniform(-0.3, 0.3), rng.uniform(0, 3)), bevel=0.06, segments=1)


def ruined_wall(m, rng, x0, y0, x1, y1, thick, h_lo, h_hi):
    """Et murløp fra (x0, y0) til (x1, y1): skiver på en meter med ulik
    høyde, så toppen blir takkete, med enkeltsteiner som står ut."""
    a, b = Vector((x0, y0, 0)), Vector((x1, y1, 0))
    d = b - a
    n = max(1, int(d.length / 1.6))
    along = d.normalized()
    side = Vector((-along.y, along.x, 0))
    yaw = math.atan2(d.y, d.x)
    h = rng.uniform(h_lo, h_hi)
    for k in range(n):
        c = a + d * ((k + 0.5) / n)
        # Høyden vandrer langsomt, med brudd her og der.
        h = min(h_hi, max(h_lo, h + rng.uniform(-0.9, 0.9)))
        if rng.random() < 0.08:
            h = h_lo * rng.uniform(0.4, 0.8)
        top = h + rng.uniform(-0.25, 0.25)
        block("wall", (d.length / n + 0.02, thick, top + 0.8), m.ashlar, at=(c.x, c.y, (top - 0.8) / 2), rot=(0, 0, yaw), bevel=0.04, segments=1)
        # Steiner i toppen som er løse og skjeve.
        if rng.random() < 0.6:
            s = rng.uniform(0.3, 0.6)
            block("top_stone", (s, thick * rng.uniform(0.4, 0.9), s * 0.6), m.ashlar, at=(c.x + along.x * rng.uniform(-0.3, 0.3), c.y + along.y * rng.uniform(-0.3, 0.3), top + s * 0.2), rot=(rng.uniform(-0.2, 0.2), rng.uniform(-0.2, 0.2), yaw + rng.uniform(-0.3, 0.3)), bevel=0.04, segments=1)
        # Mose på toppen.
        if rng.random() < 0.35:
            block("moss", (d.length / n * 0.8, thick * 0.9, 0.08), m.moss, at=(c.x, c.y, top + 0.02), rot=(0, 0, yaw), bevel=0.03)
        # Røys ved foten, på begge sider.
        if rng.random() < 0.25:
            sgn = rng.choice((-1, 1))
            rubble(m, rng, c.x + side.x * sgn * (thick / 2 + 0.6), c.y + side.y * sgn * (thick / 2 + 0.6), 0.9, 4)


def ruined_tower(m, rng, x, y, r, h, slant):
    """Et rundt tårn som har rast: sylinder med skrå bruddflate og ei
    side som er falt ut."""
    ob = revolve("tower", [(r + 0.3, -0.8), (r, 1.0), (r, h + 3.0)], m.ashlar, n=28, at=(x, y, 0))
    # Bruddflata: skjær bort alt over et skrått plan.
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(x, y, h + 3.0), rotation=(slant, slant * 0.6, rng.uniform(0, 3)))
    c = bpy.context.active_object
    c.scale = (r * 4, r * 4, 6.0)
    cutter(c, ob)
    # Hulrommet inni og et gap der sida har falt ut.
    inner = revolve("tower_inner", [(r - 0.9, 0.3), (r - 0.9, h + 4.0)], m.ashlar, n=20, at=(x, y, 0))
    PIECES.remove(inner)
    cutter(inner, ob)
    a = rng.uniform(0, math.tau)
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(x + math.cos(a) * r, y + math.sin(a) * r, h * 0.7 + 1.5), rotation=(0, 0, a))
    g = bpy.context.active_object
    g.scale = (2.0, 2.6, h)
    cutter(g, ob)
    town.uv_box(ob)
    rubble(m, rng, x + math.cos(a) * (r + 1.5), y + math.sin(a) * (r + 1.5), 2.2, 18, (0.4, 0.9))


def castle_ruin():
    m = mats()
    rng = random.Random(61)
    S = 20.0
    t = 2.4
    # Ringmurene: nord, vest, øst, og sør på hver side av porten.
    ruined_wall(m, rng, -S, S, S, S, t, 3.0, 7.5)
    ruined_wall(m, rng, -S, -S, -S, S, t, 2.5, 7.0)
    ruined_wall(m, rng, S, -S, S, S, t, 3.0, 7.5)
    ruined_wall(m, rng, -S, -S, -5.0, -S, t, 2.0, 6.0)
    ruined_wall(m, rng, 5.0, -S, S, -S, t, 2.0, 6.0)
    # Hjørnetårnene, rast i ulik grad.
    for (x, y), h, sl in zip([(-S, S), (S, S), (-S, -S), (S, -S)], [9.0, 6.0, 7.5, 4.5], [0.35, 0.5, 0.25, 0.6]):
        ruined_tower(m, rng, x, y, 3.2, h, sl)
    # Porttårnene ved åpningen i sør.
    for sx in (-1, 1):
        block("gate_tower", (3.4, 3.4, 7.5 - sx * 1.5), m.ashlar, at=(sx * 6.6, -S, (7.5 - sx * 1.5) / 2 - 0.8), bevel=0.05, segments=2)
        rubble(m, rng, sx * 6.6, -S - 2.5, 1.6, 10)
    # Hovedtårnet i nordvest: kvadratisk, tre vegger står, den fjerde ligger
    # som en røys inne i borggården.
    kx, ky, kw, kh = -11.0, 11.0, 7.0, 12.0
    for (cx, cy, sx_, sy_, hh) in [(kx, ky + kw / 2, kw, 1.4, kh), (kx - kw / 2, ky, 1.4, kw, kh - 1.5), (kx + kw / 2, ky, 1.4, kw, kh * 0.6)]:
        ob = block("keep_wall", (sx_, sy_, hh + 0.8), m.ashlar, at=(cx, cy, (hh - 0.8) / 2), bevel=0.05, segments=2)
        # Vinduer og en skyteskår.
        for z in (4.0, 8.0):
            if z < hh - 1.0:
                cut(ob, (0.5 if sx_ > sy_ else 2.0, 2.0 if sx_ > sy_ else 0.5, 1.2), (cx, cy, z))
        town.uv_box(ob)
    rubble(m, rng, kx + 1.0, ky - kw / 2 - 1.5, 3.5, 40, (0.4, 1.1))
    # Røysa midt i borggården: et rast brønnhus.
    rubble(m, rng, 0.0, 0.0, 1.8, 26, (0.4, 1.0))
    revolve("well", [(1.2, -0.3), (1.2, 0.9), (0.85, 0.9), (0.85, -0.3)], m.ashlar, n=20)
    # Steiner spredt rundt i gresset.
    for _ in range(16):
        rubble(m, rng, rng.uniform(-S + 3, S - 3), rng.uniform(-S + 3, S - 3), 0.8, 2, (0.25, 0.5))
    return m


def torch_bracket(m, x, y, z, normal_y=-1.0):
    """En fakkelholder av smijern på veggen, med fakkel og flamme."""
    s = normal_y
    beam("bracket", (x, y, z), (x, y + s * 0.25, z + 0.1), 0.04, 0.04, m.dark_iron)
    revolve("bracket_ring", [(0.07, z + 0.05), (0.08, z + 0.1), (0.07, z + 0.15), (0.06, z + 0.1)], m.dark_iron, n=10, at=(x, y + s * 0.25, 0), cap=False)
    beam("torch", (x, y + s * 0.25, z - 0.25), (x, y + s * 0.3, z + 0.3), 0.06, 0.06, m.timber)
    revolve("torch_head", [(0.0, z + 0.25), (0.06, z + 0.28), (0.06, z + 0.4), (0.0, z + 0.42)], m.leather if hasattr(m, "leather") else m.timber, n=8, at=(x, y + s * 0.3, 0))
    revolve("flame", [(0.0, z + 0.4), (0.07, z + 0.48), (0.0, z + 0.72)], m.glow, n=8, at=(x, y + s * 0.3, 0))


def arched_portal(m, front, x, y_face, z0, w, h, t, depth_door):
    """En rundbuet åpning i muren med buestein, og fotodøra innfelt."""
    straight = h - w / 2
    cut(front, (w, t + 0.4, straight), (x, y_face + t / 2, z0 + straight / 2))
    bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=w / 2, depth=t + 0.4, location=(x, y_face + t / 2, z0 + straight), rotation=(math.pi / 2, 0, 0))
    cutter(bpy.context.active_object, front)
    town.uv_box(front)
    for k in range(13):
        a = math.pi * k / 12
        c = Vector((x + math.cos(a) * (w / 2 + 0.16), y_face - 0.05, z0 + straight + math.sin(a) * (w / 2 + 0.16)))
        block("voussoir", (0.22, 0.18, 0.36), m.ashlar, at=c, rot=(0, math.pi / 2 - a, 0), bevel=0.02, segments=2)
    for sx in (-1, 1):
        block("jamb", (0.3, 0.2, straight), m.ashlar, at=(x + sx * (w / 2 + 0.15), y_face - 0.05, z0 + straight / 2), bevel=0.02, segments=2)
    # Døra: bildet er 1:2; den rette delen dekkes av døra, buen over av et
    # tympanon av stein.
    photo_door(x, y_face + depth_door, z0, w, straight + w * 0.15)
    tymp = prism("tympanum", [(-w / 2, 0.0), (w / 2, 0.0), (w / 2, w / 2), (-w / 2, w / 2)], 0.2, m.ashlar, at=(x, y_face + depth_door + 0.12, z0 + straight + w * 0.15))
    bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=w / 2, depth=0.6, location=(x, y_face + depth_door + 0.12, z0 + straight), rotation=(math.pi / 2, 0, 0))
    inv = bpy.context.active_object
    mod = tymp.modifiers.new("keep", "BOOLEAN")
    mod.operation = "INTERSECT"
    mod.object = inv
    mod.solver = "EXACT"
    bpy.context.view_layer.objects.active = tymp
    names = {lay.name for lay in tymp.data.uv_layers}
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.data.objects.remove(inv)
    town.keep_uv(tymp, names)
    town.uv_box(tymp)


def crypt():
    m = mats(roof="slate")
    rng = random.Random(62)
    W, D, H = 5.6, 6.0, 3.6
    y0 = -1.0  # fronten (døra) mot −Y
    t = 0.6
    front = block("front", (W, t, H + 0.8), m.ashlar, at=(0, y0 + t / 2, (H - 0.8) / 2), bevel=0.04, segments=2)
    block("back", (W, t, H + 0.8), m.ashlar, at=(0, y0 + D - t / 2, (H - 0.8) / 2), bevel=0.04, segments=2)
    for sx in (-1, 1):
        block("side", (t, D - 2 * t, H + 0.8), m.ashlar, at=(sx * (W / 2 - t / 2), y0 + D / 2, (H - 0.8) / 2), bevel=0.04, segments=2)
        # Støttepilarer.
        for y in (y0 + 0.4, y0 + D - 0.4):
            block("buttress", (0.6, 0.7, 2.2), m.ashlar, at=(sx * (W / 2 + 0.3), y, 0.7), bevel=0.04, segments=2)
    arched_portal(m, front, 0.0, y0, 0.0, 1.3, 2.7, t, 0.25)
    # Taket: bratt skifertak med gavlen mot sør, gavler av stein og kors.
    before = len(PIECES)
    rise = roof(m, W + 0.1, D, H, 55, overhang=0.35, gable_to_street=True)
    for ob in PIECES[before:]:
        ob.location.y += y0 + D / 2
    for sy in (0, 1):
        yy = y0 + t / 2 + sy * (D - t)
        prism("gable", [(-W / 2, 0.0), (W / 2, 0.0), (0.0, rise)], t, m.ashlar, at=(0, yy, H))
    cz = H + rise + 0.1
    block("cross_v", (0.14, 0.14, 1.0), m.ashlar, at=(0, y0 + 0.3, cz + 0.45), bevel=0.02)
    block("cross_h", (0.6, 0.14, 0.14), m.ashlar, at=(0, y0 + 0.3, cz + 0.65), bevel=0.02)
    # Trinn ned foran døra (krypten ligger lavere enn borggården).
    for k in range(3):
        block("step", (2.2 - k * 0.2, 0.45, 0.16), m.rubble, at=(0, y0 - 0.25 - k * 0.42, 0.0 - k * 0.04), bevel=0.03, segments=2)
    torch_bracket(m, 1.3, y0 - 0.02, 2.0)
    torch_bracket(m, -1.3, y0 - 0.02, 2.0)
    # Mose og nedfalne steiner rundt foten.
    for _ in range(10):
        rubble(m, rng, rng.uniform(-W / 2 - 1, W / 2 + 1), rng.uniform(y0 + 1, y0 + D + 1), 0.4, 1, (0.2, 0.4))
    block("moss", (W + 0.2, 0.25, 0.12), m.moss, at=(0, y0 + D + 0.05, 0.0), bevel=0.05, segments=2)
    return m


def crypt_stair():
    m = mats()
    # Portalen står i nordveggen; døra vender mot rommet (−Y), og bak døra
    # er det mørkt (trappa opp).
    t = 0.5
    front = block("portal_wall", (3.0, t, 3.6), m.ashlar, at=(0, t / 2 - 0.1, 1.8 - 0.1), bevel=0.04, segments=2)
    arched_portal(m, front, 0.0, -0.1, 0.0, 1.3, 2.7, t, 0.2)
    block("threshold", (1.6, 0.5, 0.12), m.ashlar, at=(0, -0.3, 0.02), bevel=0.03, segments=2)
    torch_bracket(m, 1.15, -0.12, 1.9)
    torch_bracket(m, -1.15, -0.12, 1.9)
    return m


def portcullis():
    m = mats()
    W, H = 3.9, 2.9
    n = 9
    for k in range(n):
        x = -W / 2 + 0.15 + (W - 0.3) * k / (n - 1)
        block("bar", (0.07, 0.07, H - 0.25), m.dark_iron, at=(x, 0, 0.25 + (H - 0.25) / 2), bevel=0.012)
        revolve("spike", [(0.05, 0.25), (0.0, 0.0)], m.dark_iron, n=4, at=(x, 0, 0))
    for z in (0.6, 1.3, 2.0, H - 0.1):
        block("rail", (W - 0.1, 0.09, 0.08), m.dark_iron, at=(0, 0, z), bevel=0.012)
        # Nagler i krysningene.
        for k in range(n):
            x = -W / 2 + 0.15 + (W - 0.3) * k / (n - 1)
            for sy in (-1, 1):
                block("rivet", (0.05, 0.03, 0.05), m.dark_iron, at=(x, sy * 0.06, z), bevel=0.01)
    return m


MODELS = {"castle_ruin": castle_ruin, "crypt": crypt, "crypt_stair": crypt_stair, "portcullis": portcullis}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        MODELS[name]()
        finish(f"{OUT}/{name}.glb", max_edge=1.6 if name == "castle_ruin" else 0.6)
