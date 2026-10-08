"""Gatelivet og det som står ute i landskapet, modellert i Blender (bpy) og
eksportert til assets/models/town/. Kjør:

    python3 tools/models/street.py [well wattle_fence woodpile stone_wall boulder_a boulder_b boulder_c signpost]

Brønnen (`well`): en rund kum av kampestein med dekkstein, to stolper med
et lite spontak, vinde med sveiv, tau og bøtte. Flettverksgjerdet
(`wattle_fence`): et stykke på 2 m langs lokal x, med staker og vidjer
flettet inn og ut. Vedstabelen (`woodpile`): kløyvd ved stablet mellom to
staker, med et lite tak av bord. Steingjerdet (`stone_wall`): et stykke
tørrmur på 2 m langs lokal x, av enkeltsteiner i lag, med mose; foten går
40 cm ned så det kan følge terrenget. Steinene (`boulder_a`–`c`):
forvitret granitt med mose på oversida, halvt nedgravd.

Akser som i town.py: −Y fram, Z opp, origo midt på grunnflaten."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import clear  # noqa: E402
from mathutils import Vector, noise  # noqa: E402

import town  # noqa: E402
from town import M, PIECES, beam, block, finish, layer, plain, prism  # noqa: E402
from walls import link, pyramid, revolve  # noqa: E402

OUT = "assets/models/town"


def mats():
    m = M(roof="tiles")
    m.rock = layer("rock", (0.6, 0.58, 0.55), 0.5)
    m.field = layer("stone", (0.55, 0.53, 0.5), 0.5)
    m.moss = layer("sod", (0.55, 0.65, 0.45), 0.5)
    m.bark = layer("bark", (0.8, 0.75, 0.7), 0.5)
    m.withy = layer("bark", (0.6, 0.5, 0.42), 0.5)
    m.wood = layer("cladding", (0.5, 0.42, 0.34), 0.5)
    m.end = layer("oak", (0.9, 0.75, 0.6), 0.5)
    m.dark_iron = layer("metal", (0.3, 0.28, 0.27), 0.5)
    m.rope = layer("thatch", (0.55, 0.47, 0.36), 0.5)
    return m


def rock(name, size, mat, at=(0, 0, 0), seed=0, rough=0.25, subdiv=3, moss=None, rot=(0, 0, 0)):
    """En stein: en ikosfære forskjøvet med støy, skalert til `size` (hele
    mål), med uv i meter. Flatene som vender opp får mose om `moss`."""
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=subdiv, radius=0.5)
    off = Vector((seed * 3.1, seed * 1.7, seed * 2.3))
    for v in bm.verts:
        n = v.co.normalized()
        d = noise.noise(n * 1.6 + off) * rough + noise.noise(n * 4.0 + off) * rough * 0.35
        # Litt flat under og kantete: klem den nederste delen.
        v.co = n * (0.5 + d)
        if v.co.z < -0.15:
            v.co.z = -0.15 + (v.co.z + 0.15) * 0.3
        v.co = Vector((v.co.x * size[0], v.co.y * size[1], v.co.z * size[2]))
    ob = link(name, bm, mat)
    if moss is not None:
        ob.data.materials.append(moss)
        for p in ob.data.polygons:
            if p.normal.z > 0.55 and noise.noise(p.center * 2.0 + off) > -0.1:
                p.material_index = 1
    town.uv_box(ob)
    ob.rotation_euler = rot
    ob.location = at
    return ob


def well():
    m = mats()
    rng = random.Random(71)
    R, H = 0.85, 0.85
    # Kummen: ring av stein, med enkeltsteiner som står ut, og dekkstein.
    revolve("curb", [(R, -0.3), (R, H), (R - 0.3, H), (R - 0.3, -0.3)], m.field, n=28)
    for row in range(4):
        n = 14
        for k in range(n):
            a = 2 * math.pi * (k + 0.5 * (row % 2)) / n + rng.uniform(-0.05, 0.05)
            z = 0.1 + row * 0.2
            rock("curb_stone", (0.38, 0.14, 0.2), m.field, at=(math.cos(a) * (R + 0.02), math.sin(a) * (R + 0.02), z), seed=row * 20 + k, rough=0.12, subdiv=2, rot=(0, 0, a + math.pi / 2))
    for k in range(10):
        a = 2 * math.pi * k / 10
        block("cap", (0.5, 0.36, 0.12), m.field, at=(math.cos(a) * (R - 0.14), math.sin(a) * (R - 0.14), H + 0.04), rot=(0, 0, a + math.pi / 2), bevel=0.04, segments=2)
    block("water", (1.0, 1.0, 0.02), m.glass, at=(0, 0, 0.05), bevel=0)
    # Stolpene og taket.
    for sx in (-1, 1):
        beam("post", (sx * (R - 0.1), 0, -0.2), (sx * (R - 0.1), 0, 2.2), 0.14, 0.14, m.timber, roll=math.pi / 2)
    beam("plate", (-R - 0.1, 0, 2.2), (R + 0.1, 0, 2.2), 0.14, 0.14, m.timber)
    for sy in (-1, 1):
        ob = block("roof", (2.2, 0.85, 0.08), m.roof, at=(0, sy * 0.36, 2.48), rot=(sy * 0.65, 0, 0), bevel=0.01)
        town._roof_uv(ob, along_y=False)
    beam("ridge", (-1.1, 0, 2.75), (1.1, 0, 2.75), 0.16, 0.06, m.wood)
    # Lagrene for vinda i stolpene (vinda, tauet og bøtta er egne modeller
    # som spillet dreier og heiser: well_winch, well_rope, well_bucket).
    for sx in (-1, 1):
        block("bearing", (0.16, 0.2, 0.12), m.dark_iron, at=(sx * (R - 0.1), 0, 1.55), bevel=0.01)
    # Steinheller rundt.
    for k in range(9):
        a = 2 * math.pi * k / 9 + rng.uniform(-0.1, 0.1)
        rock("flag", (0.55, 0.45, 0.1), m.field, at=(math.cos(a) * (R + 0.35), math.sin(a) * (R + 0.35), -0.02), seed=50 + k, rough=0.08, subdiv=2, rot=(0, 0, a))
    return m


def wattle_fence():
    m = mats()
    rng = random.Random(72)
    L, H = 2.0, 1.0
    stakes = [-L / 2 + L * k / 5 for k in range(6)]
    for x in stakes:
        beam("stake", (x, 0, -0.25), (x + rng.uniform(-0.02, 0.02), rng.uniform(-0.02, 0.02), H + rng.uniform(0.0, 0.12)), 0.05, 0.05, m.bark, roll=rng.uniform(0, 3))
    # Vidjene: vannrette ris som går foran og bak stakene i tur og orden.
    for row in range(14):
        z = 0.08 + row * (H - 0.12) / 13
        phase = row % 2
        pts = []
        for i in range(len(stakes) * 2 + 1):
            x = -L / 2 - 0.05 + (L + 0.1) * i / (len(stakes) * 2)
            k = (x + L / 2) / (L / 5)
            y = 0.035 * math.cos(math.pi * (k + phase) * 0.5 * 2 / 1)
            pts.append(Vector((x, y, z + rng.uniform(-0.01, 0.01))))
        for a, b in zip(pts, pts[1:]):
            beam("withy", a, b, 0.022, 0.022, m.withy)
    return m


def log(m, a, b, r, rng):
    """En kløyvd vedkubbe: halvsylinder med bark og lys endeved."""
    a, b = Vector(a), Vector(b)
    d = b - a
    ob = revolve("log", [(r, 0.0), (r, d.length)], m.bark, n=8, cap=False)
    ob.rotation_mode = "QUATERNION"
    ob.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(d.normalized())
    ob.location = a
    for p, s in ((a, -1), (b, 1)):
        e = revolve("end", [(0.0, -0.005), (r, -0.005), (r, 0.005), (0.0, 0.005)], m.end, n=8)
        e.rotation_mode = "QUATERNION"
        e.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(d.normalized())
        e.location = p


def woodpile():
    m = mats()
    rng = random.Random(73)
    L, D, H = 1.8, 0.45, 1.2
    for sx in (-1, 1):
        beam("pile_post", (sx * L / 2, 0, -0.2), (sx * L / 2, 0, H + 0.25), 0.08, 0.08, m.timber, roll=math.pi / 2)
    rows = int(H / 0.2)
    for row in range(rows):
        z = 0.1 + row * 0.2
        x = -L / 2 + 0.12
        while x < L / 2 - 0.1:
            r = rng.uniform(0.07, 0.1)
            log(m, (x, -D / 2 + rng.uniform(-0.04, 0.04), z), (x + rng.uniform(-0.03, 0.03), D / 2 + rng.uniform(-0.04, 0.04), z + rng.uniform(-0.02, 0.02)), r, rng)
            x += r * 2 + 0.005
    ob = block("pile_roof", (L + 0.4, D + 0.4, 0.04), m.wood, at=(0, 0, H + 0.3), rot=(0.18, 0, 0), bevel=0.01)
    _ = ob
    block("chopping_block", (0.45, 0.45, 0.5), m.end, at=(L / 2 + 0.6, -0.4, 0.25), bevel=0.05, segments=2)
    beam("axe_haft", (L / 2 + 0.6, -0.4, 0.5), (L / 2 + 0.75, -0.55, 1.15), 0.04, 0.04, m.wood)
    block("axe_head", (0.04, 0.18, 0.12), m.dark_iron, at=(L / 2 + 0.6, -0.4, 0.55), bevel=0.01)
    return m


def stone_wall():
    m = mats()
    rng = random.Random(74)
    L = 2.15
    # Kjernen (skjult) og lagene av stein, smalere oppover.
    block("core", (L, 0.45, 1.25), m.field, at=(0, 0, 0.22), bevel=0.05, segments=2)
    z = -0.35
    seed = 0
    for row, (h, w) in enumerate([(0.32, 0.7), (0.28, 0.62), (0.26, 0.56), (0.24, 0.5)]):
        x = -L / 2 - rng.uniform(0.0, 0.2)
        while x < L / 2:
            ln = rng.uniform(0.3, 0.55)
            seed += 1
            cx = min(x + ln / 2, L / 2 - 0.05)
            rock("stone", (ln + 0.08, w, h + 0.06), m.field, at=(cx, rng.uniform(-0.03, 0.03), z + h / 2), seed=seed, rough=0.16, subdiv=2, rot=(rng.uniform(-0.05, 0.05), 0, rng.uniform(-0.08, 0.08)))
            x += ln
        z += h
    # Dekksteiner på høykant og mose.
    x = -L / 2
    while x < L / 2:
        ln = rng.uniform(0.18, 0.3)
        seed += 1
        rock("cap", (ln, 0.5, 0.3), m.field, at=(min(x + ln / 2, L / 2 - 0.05), 0, z + 0.12), seed=seed, rough=0.18, subdiv=2, moss=m.moss)
        x += ln
    return m


def boulder(seed, size):
    m = mats()
    rock("boulder", size, m.rock, at=(0, 0, size[2] * 0.15), seed=seed, rough=0.28, subdiv=4, moss=m.moss)
    rng = random.Random(seed)
    for k in range(rng.randint(2, 5)):
        a = rng.uniform(0, math.tau)
        r = max(size[0], size[1]) * 0.55
        s = rng.uniform(0.15, 0.3)
        rock("pebble", (s, s * 0.8, s * 0.5), m.rock, at=(math.cos(a) * r, math.sin(a) * r, 0.02), seed=seed * 10 + k, rough=0.2, subdiv=2)
    return m


def well_winch():
    """Vinda: valsen med taukveil, jernbånd og sveiv. Origo i aksen, som
    går langs x; spillet dreier modellen om x."""
    m = mats()
    roller = revolve("roller", [(0.08, -0.7), (0.08, 0.7)], m.bark, n=14)
    roller.rotation_euler = (0, math.pi / 2, 0)
    for x in (-0.68, 0.68):
        band = revolve("roller_band", [(0.085, -0.025), (0.085, 0.025)], m.dark_iron, n=14, cap=False)
        band.rotation_euler = (0, math.pi / 2, 0)
        band.location = (x, 0, 0)
    # Kveilen: tauet surret rundt valsen i vindinger.
    for k in range(9):
        x = -0.25 + k * 0.045
        bpy.ops.mesh.primitive_torus_add(major_radius=0.092, minor_radius=0.014, major_segments=16, minor_segments=6, location=(x, 0, 0), rotation=(0, math.pi / 2, 0))
        ob = bpy.context.active_object
        ob.data.materials.append(m.rope)
        town.ensure_uv(ob.data)
        town.uv_box(ob)
        PIECES.append(ob)
    # Akseltappene gjennom lagrene, og sveiven på østsida.
    beam("axle", (-0.95, 0, 0), (-0.68, 0, 0), 0.04, 0.04, m.dark_iron)
    beam("axle", (0.68, 0, 0), (1.0, 0, 0), 0.04, 0.04, m.dark_iron)
    beam("crank_arm", (1.0, 0, 0.02), (1.0, 0, -0.3), 0.045, 0.03, m.dark_iron)
    beam("crank_grip", (1.0, 0, -0.3), (1.17, 0, -0.3), 0.045, 0.045, m.wood)
    return m


def well_rope():
    """Tauet fra valsen ned til bøtta: én meter, fra origo og ned; spillet
    skalerer lengden."""
    m = mats()
    beam("rope", (0, 0, 0), (0, 0, -1.0), 0.024, 0.024, m.rope)
    return m


def well_bucket():
    """Bøtta: staver i ring med jernbånd, vann i, og bøyla. Origo øverst på
    bøyla, der tauet er knyttet."""
    m = mats()
    m.water = plain("glass", (0.03, 0.05, 0.05), 0.05)
    rim, bottom = -0.13, -0.42
    for k in range(14):
        a = 2 * math.pi * k / 14
        c = Vector((math.cos(a), math.sin(a), 0))
        block("stave", (0.07, 0.025, rim - bottom), m.wood, at=c * 0.148 + Vector((0, 0, (rim + bottom) / 2)), rot=(0, 0, a + math.pi / 2), bevel=0.004)
    revolve("bucket_floor", [(0.0, bottom), (0.14, bottom), (0.14, bottom + 0.03), (0.0, bottom + 0.03)], m.wood, n=14)
    for z in (bottom + 0.05, rim - 0.06):
        revolve("bucket_band", [(0.165, z - 0.02), (0.168, z), (0.165, z + 0.02)], m.dark_iron, n=16, cap=False)
    revolve("bucket_water", [(0.0, rim - 0.05), (0.14, rim - 0.05), (0.0, rim - 0.05 + 0.001)], m.water, n=14)
    # Bøyla: en bue av jern fra øret på hver side opp til tauet.
    pts = []
    for k in range(9):
        a = math.pi * k / 8
        pts.append(Vector((math.cos(a) * 0.16, 0, rim + math.sin(a) * (-rim))))
    for p0, p1 in zip(pts, pts[1:]):
        beam("bail", p0, p1, 0.014, 0.014, m.dark_iron)
    for sx in (-1, 1):
        block("ear", (0.03, 0.05, 0.06), m.dark_iron, at=(sx * 0.165, 0, rim - 0.01), bevel=0.005)
    revolve("knot", [(0.0, -0.03), (0.03, -0.02), (0.03, 0.02), (0.0, 0.03)], m.rope, n=8)
    return m


def carved_text(name, text, size, mat, at, back=False):
    """Bokstaver skåret og svidd inn i en tavle i xz-planet: en tekst
    gjort om til et tynt mesh, vendt mot −Y (eller +Y med `back`)."""
    bpy.ops.object.text_add()
    ob = bpy.context.active_object
    ob.name = name
    ob.data.body = text
    ob.data.size = size
    ob.data.extrude = 0.002
    ob.data.align_x = "CENTER"
    ob.data.align_y = "CENTER"
    ob.rotation_euler = (math.pi / 2, 0, math.pi if back else 0)
    ob.location = at
    bpy.ops.object.convert(target="MESH")
    ob = bpy.context.active_object
    ob.data.materials.clear()
    ob.data.materials.append(mat)
    town.uv_box(ob)
    PIECES.append(ob)
    return ob


def signpost():
    """Veiskiltet til krypten: en tjærebredd stolpe med en pilformet tavle
    der KRYPTEN er svidd inn på begge sider, og et lite jernkors ved
    pilspissen. Pila peker langs lokal +X; tavla står 2 m oppe."""
    m = mats()
    board_m = layer("cladding", (1.0, 0.97, 0.92), 0.6)
    burnt = layer("timber", (0.08, 0.06, 0.05), 0.8)
    beam("sign_post", (0, 0, -0.35), (0, 0, 2.55), 0.13, 0.13, m.timber, roll=math.pi / 2)
    block("post_cap", (0.17, 0.17, 0.05), m.dark_iron, at=(0, 0, 2.57), bevel=0.01)
    z, h, L, d = 2.05, 0.3, 1.25, 0.05
    pts = [(0.08, -h / 2), (L - 0.2, -h / 2), (L, 0.0), (L - 0.2, h / 2), (0.08, h / 2)]
    prism("sign_board", pts, d, board_m, at=(0, 0, z))
    # Beslagene som holder tavla til stolpen.
    for dz in (-0.09, 0.09):
        block("sign_strap", (0.26, d + 0.03, 0.035), m.dark_iron, at=(0.1, 0, z + dz), bevel=0.006)
    for back in (False, True):
        y = (d / 2 + 0.001) * (1 if back else -1)
        carved_text("sign_text", "KRYPTEN", 0.15, burnt, (0.58, y, z), back=back)
        # Et lite kors ved spissen, som på kirkegårdsporter.
        cx = 1.0
        block("sign_cross_v", (0.025, 0.006, 0.13), m.dark_iron, at=(cx, y, z), bevel=0.002)
        block("sign_cross_h", (0.08, 0.006, 0.025), m.dark_iron, at=(cx, y, z + 0.025), bevel=0.002)
    return m


CHEST_W, CHEST_D, CHEST_H = 1.0, 0.56, 0.46


def chest_body():
    """Kista i kjelleren: kassa av eikeplanker med fuger, jernbeslag på
    hjørnene, to jernbånd, nagler, håndtak av smijern på sidene og
    låsplate foran (−Y). Lokket er en egen modell (`chest_lid`) som
    spillet dreier opp om hengslet. Origo midt på gulvet under kassa."""
    m = mats()
    oak = layer("oak", (0.62, 0.48, 0.36), 0.6)
    iron = m.dark_iron
    W, D, H = CHEST_W, CHEST_D, CHEST_H
    # Plankene: fire bord på hver langside, tre på endene, bunn.
    n = 4
    for k in range(n):
        z0 = 0.05 + (H - 0.05) * k / n
        hh = (H - 0.05) / n - 0.006
        for sy in (-1, 1):
            block("plank_long", (W, 0.035, hh), oak, at=(0, sy * (D / 2 - 0.0175), z0 + hh / 2), bevel=0.006, along="x")
        for sx in (-1, 1):
            block("plank_end", (0.035, D - 0.07, hh), oak, at=(sx * (W / 2 - 0.0175), 0, z0 + hh / 2), bevel=0.006, along="y")
    block("bottom", (W - 0.02, D - 0.02, 0.05), oak, at=(0, 0, 0.025), bevel=0.006)
    # Jernbeslag på de åtte hjørnene, bånd rundt kassa og nagler.
    for sx in (-1, 1):
        for sy in (-1, 1):
            block("corner_v", (0.012, 0.012, H), iron, at=(sx * (W / 2 + 0.002), sy * (D / 2 + 0.002), H / 2), bevel=0.003)
            for sz in (0.03, H - 0.03):
                block("corner_plate", (0.1, 0.1, 0.06), iron, at=(sx * (W / 2 - 0.045), sy * (D / 2 - 0.045), sz), bevel=0.004)
    for bx in (-0.3, 0.3):
        for sy in (-1, 1):
            block("band", (0.05, 0.008, H - 0.02), iron, at=(bx, sy * (D / 2 + 0.004), H / 2), bevel=0.002)
            for bz in (0.1, 0.24, 0.38):
                block("rivet", (0.016, 0.01, 0.016), iron, at=(bx, sy * (D / 2 + 0.01), bz), bevel=0.004)
    # Håndtak på endene og låsplata foran.
    for sx in (-1, 1):
        block("handle_plate", (0.01, 0.12, 0.08), iron, at=(sx * (W / 2 + 0.006), 0, H * 0.62), bevel=0.003)
        beam("handle", (sx * (W / 2 + 0.03), -0.07, H * 0.58), (sx * (W / 2 + 0.03), 0.07, H * 0.58), 0.016, 0.016, iron)
    block("lock_plate", (0.13, 0.012, 0.16), iron, at=(0, -(D / 2 + 0.007), H - 0.09), bevel=0.004)
    block("keyhole", (0.014, 0.004, 0.035), layer("metal", (0.05, 0.05, 0.05), 0.5), at=(0, -(D / 2 + 0.014), H - 0.1), bevel=0.001)
    for sx in (-1, 1):
        for sy in (-1, 1):
            block("foot", (0.09, 0.09, 0.03), iron, at=(sx * (W / 2 - 0.06), sy * (D / 2 - 0.06), 0.012), bevel=0.004)
    return m


def chest_lid():
    """Lokket til kista: buet av sju planker, med jernbånd over buen og en
    hasp som henger ned foran. Origo i hengslet (bakre øvre kant, +Y),
    så spillet kan dreie det opp om X."""
    m = mats()
    oak = layer("oak", (0.66, 0.51, 0.38), 0.6)
    iron = m.dark_iron
    W, D = CHEST_W, CHEST_D
    R = D / 2
    k = 7
    # Buen: planker langs X over en flatet halvsirkel (sentrert i y = −D/2
    # fra hengslet), hver plank langs korden mellom to punkter på buen.
    def arc(a, grow=0.0):
        return Vector((0.0, -R + math.cos(a) * (R + grow), math.sin(a) * (R * 0.55 + grow)))

    def along_arc(name, a0, a1, w, t, mat, grow=0.0, lift=0.0, bevel=0.005):
        p0, p1 = arc(a0, grow), arc(a1, grow)
        d = p1 - p0
        mid = (p0 + p1) / 2
        nrm = Vector((0.0, -d.z, d.y)).normalized()
        if nrm.dot(mid - Vector((0.0, -R, 0.0))) < 0:
            nrm = -nrm
        return block(name, (w, d.length * 1.02, t), mat, at=tuple(mid + nrm * (t / 2 + lift)), rot=(math.atan2(d.z, d.y), 0, 0), bevel=bevel, along="x")

    for i in range(k):
        along_arc("lid_plank", math.pi * i / k, math.pi * (i + 1) / k, W, 0.03, oak)
    for sx in (-1, 1):
        # Endestykkene: halvsirkelplater.
        pts = [(math.cos(math.pi * t / 12) * R, math.sin(math.pi * t / 12) * R * 0.55) for t in range(13)]
        pts = [(-R + px, pz) for px, pz in pts]
        bm = bmesh.new()
        front = [bm.verts.new((sx * (W / 2 - 0.0175) - 0.0175, py, pz)) for py, pz in pts]
        back = [bm.verts.new((sx * (W / 2 - 0.0175) + 0.0175, py, pz)) for py, pz in pts]
        bm.faces.new(front)
        bm.faces.new(back[::-1])
        for j in range(len(pts) - 1):
            bm.faces.new((front[j], front[j + 1], back[j + 1], back[j]))
        bm.faces.new((front[-1], front[0], back[0], back[-1]))
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        ob = link("lid_end", bm, oak)
        town.uv_box(ob)
    # Jernbånd over buen og nagler.
    for bx in (-0.3, 0.3):
        for i in range(k):
            b = along_arc("lid_band", math.pi * i / k, math.pi * (i + 1) / k, 0.05, 0.008, iron, lift=0.03, bevel=0.002)
            b.location.x = bx
            r = along_arc("lid_rivet", math.pi * (i + 0.45) / k, math.pi * (i + 0.55) / k, 0.016, 0.012, iron, lift=0.036, bevel=0.004)
            r.location.x = bx
    # Haspen foran som går ned over låsplata.
    block("hasp", (0.07, 0.012, 0.13), iron, at=(0, -D - 0.008, -0.04), bevel=0.003)
    # Hengslene bak.
    for bx in (-0.3, 0.3):
        beam("hinge", (bx - 0.05, 0.01, 0.0), (bx + 0.05, 0.01, 0.0), 0.025, 0.025, iron)
    return m


MODELS = {
    "well": well,
    "well_winch": well_winch,
    "well_rope": well_rope,
    "well_bucket": well_bucket,
    "wattle_fence": wattle_fence,
    "woodpile": woodpile,
    "stone_wall": stone_wall,
    "boulder_a": lambda: boulder(1, (1.6, 1.2, 1.0)),
    "boulder_b": lambda: boulder(2, (1.1, 1.0, 0.7)),
    "boulder_c": lambda: boulder(3, (2.4, 1.6, 1.3)),
    "signpost": signpost,
    "chest": chest_body,
    "chest_lid": chest_lid,
}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        MODELS[name]()
        finish(f"{OUT}/{name}.glb", max_edge=3.0 if name in ("wattle_fence", "woodpile") else 0.6)
