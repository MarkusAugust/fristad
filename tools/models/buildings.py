"""Bygningene på torget, modellert i Blender (bpy) og eksportert til
assets/models/town/. Kjør:

    python3 tools/models/buildings.py [forge tavern]

Smia (`forge`): et lavt hus av bruddstein med sotet mur og spontak, og
foran et åpent verksted under et skråtak på stolper: esse av stein med
glør og pipe opp gjennom taket, blåsebelg av lær, ambolt på kubbe,
kjølekar med vann, slipestein, verktøy på veggen, vedstabel og et
smijernsskilt på arm.

Vertshuset (`tavern`): et stort bindingsverkshus i to etasjer med
langsida mot torget, framskutt overetasje og halmtak, og ved døra et
skilt på jernarm, en lykt, benker og bord, og tønner.

Samme hjelpere og teksturlag som husene (town.py): origo midt på
grunnflaten, −Y fram (døra mot lokal +Z i motoren)."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402,F401
from common import clear  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

import town  # noqa: E402
from town import M, PIECES, beam, block, cut, door, finish, gable, layer, plain, roof, window  # noqa: E402
from walls import revolve  # noqa: E402
from props import barrel, crate  # noqa: E402

OUT = "assets/models/town"


def mats(**kw):
    m = M(**kw)
    m.planks = layer("cladding", (0.5, 0.42, 0.34), 0.5)
    m.soot = layer("soot", (0.8, 0.8, 0.8), 0.5)
    m.leather = layer("leather", (0.7, 0.62, 0.55), 0.5)
    m.bark = layer("bark", (0.8, 0.75, 0.7), 0.5)
    m.dark_iron = layer("metal", (0.28, 0.26, 0.25), 0.5)
    m.glow = plain("glow", (1.0, 0.45, 0.12), 0.4)
    m.water = plain("glass", (0.03, 0.04, 0.04), 0.05)
    return m


def moved(before, mat):
    """Flytt/drei alle deler laget siden `before` med matrisen `mat`."""
    for ob in PIECES[before:]:
        ob.matrix_world = mat @ ob.matrix_world


def log_cyl(m, name, a, b, r, mat=None):
    """En stokk (sylinder) fra a til b."""
    a, b = Vector(a), Vector(b)
    d = b - a
    ob = revolve(name, [(r, 0.0), (r, d.length)], mat or m.bark, n=10)
    ob.rotation_mode = "QUATERNION"
    ob.rotation_quaternion = Vector((0, 0, 1)).rotation_difference(d.normalized())
    ob.location = a
    return ob


def stone_wall(m, name, x0, x1, y0, y1, z0, z1, mat):
    """En murt vegg (bruddstein) som en kloss, med steiner som står ut."""
    rng = random.Random(hash(name) & 0xFFFF)
    ob = block(name, (x1 - x0, y1 - y0, z1 - z0), mat, at=((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2), bevel=0.04, segments=2)
    along_x = (x1 - x0) > (y1 - y0)
    for _ in range(int(((x1 - x0) + (y1 - y0)) * (z1 - z0) * 0.9)):
        if along_x:
            x = rng.uniform(x0 + 0.2, x1 - 0.2)
            for y in (y0 - 0.04, y1 + 0.04):
                if rng.random() < 0.5:
                    block("stone", (rng.uniform(0.3, 0.6), 0.1, rng.uniform(0.2, 0.35)), mat, at=(x, y, rng.uniform(z0 + 0.2, z1 - 0.2)), rot=(0, rng.uniform(-0.06, 0.06), 0), bevel=0.04, segments=2)
        else:
            y = rng.uniform(y0 + 0.2, y1 - 0.2)
            for x in (x0 - 0.04, x1 + 0.04):
                if rng.random() < 0.5:
                    block("stone", (0.1, rng.uniform(0.3, 0.6), rng.uniform(0.2, 0.35)), mat, at=(x, y, rng.uniform(z0 + 0.2, z1 - 0.2)), rot=(rng.uniform(-0.06, 0.06), 0, 0), bevel=0.04, segments=2)
    return ob


# --- Smia -------------------------------------------------------------------


def forge():
    m = mats(roof="tiles")
    rng = random.Random(21)
    W = 9.0
    yb0, yb1 = -0.4, 4.0  # steinhuset bak
    yf = -4.0  # forkant av verkstedet
    wh = 3.0
    # Steinhuset: fire murer, døra inn til lageret og et lite vindu.
    front = stone_wall(m, "front", -W / 2, W / 2, yb0, yb0 + 0.55, -0.24, wh, m.soot)
    cut(front, (1.0, 2.0, 2.0), (1.6, yb0 + 0.27, 1.0))
    cut(front, (0.6, 2.0, 0.6), (3.4, yb0 + 0.27, 1.8))
    town.uv_box(front)
    stone_wall(m, "back", -W / 2, W / 2, yb1 - 0.55, yb1, -0.24, wh, m.stone)
    for sx in (-1, 1):
        stone_wall(m, f"side{sx}", sx * W / 2 - 0.275 - 0.275 * sx, sx * W / 2 + 0.275 - 0.275 * sx, yb0 + 0.55, yb1 - 0.55, -0.24, wh, m.stone)
    door(m, 1.6, yb0 - 0.0, -1.0, 1.0, 2.0, 0.0)
    window(m, 3.4, 1.8, yb0 - 0.02, 0.6, 0.6, -1.0, shutters=True, rng=rng)
    # Taket over steinhuset: saltak med mønet langs x, flyttet bakover.
    before = len(PIECES)
    d = yb1 - yb0
    rise = roof(m, W + 0.1, d, wh, 42, gable_to_street=False)
    for sx in (-1, 1):
        b2 = len(PIECES)
        gable(m, d, wh, rise, 0.0, -1.0, rng)
        rot = Matrix.Rotation(math.radians(90 if sx > 0 else -90), 4, "Z")
        moved(b2, Matrix.Translation((sx * W / 2, 0, 0)) @ rot)
    moved(before, Matrix.Translation((0, (yb0 + yb1) / 2, 0)))
    # Verkstedet foran: stolper, rem, skråstag og skråtak med spon.
    eave_back, eave_front = wh - 0.1, 2.45
    posts = (-W / 2 + 0.2, -1.3, 1.3, W / 2 - 0.2)
    for x in posts:
        beam("post", (x, yf + 0.2, -0.05), (x, yf + 0.2, eave_front), 0.2, 0.2, m.timber, roll=math.pi / 2)
        block("post_stone", (0.4, 0.4, 0.2), m.stone, at=(x, yf + 0.2, 0.0), bevel=0.04, segments=2)
        beam("brace", (x, yf + 0.2, eave_front - 0.7), (x + (0.6 if x < 0 else -0.6), yf + 0.2, eave_front - 0.05), 0.12, 0.12, m.timber)
    beam("plate", (-W / 2 - 0.1, yf + 0.2, eave_front + 0.1), (W / 2 + 0.1, yf + 0.2, eave_front + 0.1), 0.22, 0.2, m.timber)
    run = yb0 - yf + 0.6
    slope_len = math.hypot(run, eave_back - eave_front)
    tilt = math.atan2(eave_back - eave_front, run)
    for k in range(7):
        x = -W / 2 + W * k / 6
        beam("rafter", (x, yf - 0.4, eave_front + 0.05), (x, yb0 + 0.1, eave_back + 0.05), 0.1, 0.14, m.timber)
    lean = block("lean_roof", (W + 0.6, slope_len, 0.14), m.roof, bevel=0.01)
    town._roof_uv(lean, along_y=False)
    lean.rotation_euler = (tilt, 0, 0)
    lean.location = (0, (yf - 0.4 + yb0 + 0.1) / 2, (eave_front + eave_back) / 2 + 0.2)
    # Essa: murt av stein, med glør i fordypningen og en hette som går
    # over i pipa opp gjennom taket.
    hx, hy = -2.4, yb0 - 0.75
    block("hearth", (1.8, 1.4, 0.85), m.soot, at=(hx, hy, 0.425 - 0.12), bevel=0.05, segments=2)
    block("hearth_rim", (1.9, 1.5, 0.1), m.stone, at=(hx, hy, 0.78), bevel=0.03, segments=2)
    block("coals", (0.9, 0.7, 0.06), m.glow, at=(hx, hy - 0.05, 0.82), bevel=0.02)
    rng2 = random.Random(5)
    for _ in range(14):
        block("coal", (rng2.uniform(0.06, 0.12), rng2.uniform(0.06, 0.12), 0.05), m.dark_iron, at=(hx + rng2.uniform(-0.4, 0.4), hy - 0.05 + rng2.uniform(-0.3, 0.3), 0.86), rot=(0, 0, rng2.uniform(0, 3)), bevel=0.015)
    # Hetta: smalner fra 1,6 til 0,8 m mellom 1,9 og 2,9 m.
    for k in range(5):
        t = k / 4
        w = 1.7 - 0.8 * t
        block("hood", (w, w * 0.85, 0.24), m.soot, at=(hx, hy + 0.1 * t, 1.9 + t * 1.0), bevel=0.03, segments=2)
    town.chimney(m, hx, hy + 0.1, 2.9, 6.6)
    # Blåsebelgen: to bord med lær mellom, og en stang til å trå.
    bx, by = hx - 1.25, hy + 0.2
    block("bellows_top", (0.55, 1.0, 0.05), m.planks, at=(bx, by, 0.9), rot=(0.12, 0, 0), bevel=0.01)
    block("bellows_bottom", (0.55, 1.0, 0.05), m.planks, at=(bx, by, 0.55), bevel=0.01)
    block("bellows_leather", (0.5, 0.85, 0.3), m.leather, at=(bx, by + 0.05, 0.72), bevel=0.08, segments=3)
    beam("bellows_nozzle", (bx, by - 0.55, 0.7), (hx - 0.7, hy - 0.2, 0.7), 0.06, 0.06, m.dark_iron)
    beam("bellows_legs", (bx, by, 0.0), (bx, by, 0.55), 0.12, 0.5, m.timber)
    # Ambolten på en kubbe.
    ax, ay = 0.2, yf + 1.7
    log_cyl(m, "stump", (ax, ay, -0.1), (ax, ay, 0.5), 0.32, m.bark)
    block("stump_top", (0.62, 0.62, 0.02), m.planks, at=(ax, ay, 0.5), bevel=0.0)
    block("anvil_foot", (0.34, 0.24, 0.12), m.dark_iron, at=(ax, ay, 0.57), bevel=0.02)
    block("anvil_waist", (0.18, 0.14, 0.16), m.dark_iron, at=(ax, ay, 0.71), bevel=0.02)
    block("anvil_face", (0.55, 0.16, 0.1), m.dark_iron, at=(ax, ay, 0.84), bevel=0.015)
    horn = revolve("anvil_horn", [(0.06, 0.0), (0.0, 0.28)], m.dark_iron, n=12)
    horn.rotation_euler = (0, math.pi / 2, 0)
    horn.location = (ax + 0.27, ay, 0.84)
    # Hammer og tang på ambolten.
    beam("hammer_handle", (ax - 0.1, ay - 0.05, 0.9), (ax - 0.45, ay - 0.25, 0.9), 0.03, 0.03, m.timber)
    block("hammer_head", (0.06, 0.14, 0.06), m.dark_iron, at=(ax - 0.1, ay - 0.05, 0.9), bevel=0.01)
    for s in (-1, 1):
        beam("tongs", (ax + 0.05, ay + 0.03, 0.9), (ax + 0.5, ay + 0.03 + s * 0.04, 0.9), 0.02, 0.015, m.dark_iron)
    # Kjølekaret: et trau av bord med vann.
    qx, qy = 1.9, yf + 1.4
    block("trough_bottom", (1.2, 0.55, 0.05), m.planks, at=(qx, qy, 0.12), bevel=0.005)
    for sy in (-1, 1):
        block("trough_side", (1.2, 0.05, 0.5), m.planks, at=(qx, qy + sy * 0.27, 0.35), bevel=0.005)
    for sx in (-1, 1):
        block("trough_end", (0.05, 0.6, 0.55), m.planks, at=(qx + sx * 0.6, qy, 0.37), bevel=0.005)
    block("water", (1.12, 0.48, 0.02), m.water, at=(qx, qy, 0.52), bevel=0)
    for sx in (-1, 1):
        block("trough_leg", (0.1, 0.6, 0.12), m.timber, at=(qx + sx * 0.45, qy, 0.06), bevel=0.01)
    # Slipesteinen i en ramme.
    gx, gy = 3.4, yf + 1.2
    wheel = revolve("grindstone", [(0.0, -0.06), (0.32, -0.06), (0.32, 0.06), (0.0, 0.06)], m.stone, n=24)
    wheel.rotation_euler = (math.pi / 2, 0, 0)
    wheel.location = (gx, gy, 0.6)
    for sy in (-1, 1):
        beam("grind_frame", (gx - 0.4, gy + sy * 0.14, 0.0), (gx, gy + sy * 0.14, 0.62), 0.07, 0.07, m.timber)
        beam("grind_frame", (gx + 0.4, gy + sy * 0.14, 0.0), (gx, gy + sy * 0.14, 0.62), 0.07, 0.07, m.timber)
    beam("grind_axle", (gx, gy - 0.25, 0.6), (gx, gy + 0.3, 0.6), 0.03, 0.03, m.dark_iron)
    beam("grind_crank", (gx, gy + 0.3, 0.6), (gx, gy + 0.3, 0.38), 0.025, 0.025, m.dark_iron)
    # Verktøy på murveggen: en list med tenger og hammere.
    beam("tool_rail", (-0.4, yb0 - 0.04, 1.75), (1.0, yb0 - 0.04, 1.75), 0.08, 0.08, m.timber)
    for k in range(6):
        x = -0.3 + k * 0.24
        if k % 2:
            beam("tool", (x, yb0 - 0.1, 1.72), (x, yb0 - 0.1, 1.2), 0.025, 0.025, m.timber)
            block("tool_head", (0.12, 0.05, 0.06), m.dark_iron, at=(x, yb0 - 0.1, 1.2), bevel=0.01)
        else:
            for s in (-1, 1):
                beam("tongs", (x, yb0 - 0.1, 1.72), (x + s * 0.03, yb0 - 0.1, 1.15), 0.015, 0.015, m.dark_iron)
    # Jernstenger og et ferdig sverd lent mot muren.
    for k in range(4):
        beam("iron_bar", (-0.6 + k * 0.06, yb0 - 0.12, 0.0), (-0.5 + k * 0.06, yb0 - 0.05, 1.3), 0.03, 0.03, m.dark_iron)
    # Kulltønne og kasser i hjørnet.
    barrel(m, at=(-4.0, yf + 1.0, 0.0), h=0.8, r=0.28, staves=14, rng=rng)
    crate(m, at=(-3.6, yb0 - 0.6, 0.0), s=0.5, rng=rng)
    # Vedstabelen langs gavlen ute, under takskjegget.
    for row in range(5):
        for k in range(9):
            y = yb0 + 0.5 + k * 0.38
            z = 0.14 + row * 0.24
            log_cyl(m, "firewood", (W / 2 + 0.25, y, z), (W / 2 + 0.95, y + rng.uniform(-0.03, 0.03), z), 0.1 + rng.uniform(-0.015, 0.015))
    # Smijernsskiltet på en arm ut fra stolpen: en ambolt i silhuett.
    sx = W / 2 - 0.2
    beam("sign_arm", (sx, yf + 0.2, 2.2), (sx + 1.1, yf + 0.2, 2.2), 0.05, 0.05, m.dark_iron)
    beam("sign_strut", (sx, yf + 0.2, 1.8), (sx + 0.7, yf + 0.2, 2.18), 0.03, 0.03, m.dark_iron)
    for k in range(2):
        beam("sign_chain", (sx + 0.35 + k * 0.6, yf + 0.2, 2.18), (sx + 0.35 + k * 0.6, yf + 0.2, 1.95), 0.015, 0.015, m.dark_iron)
    block("sign_anvil", (0.75, 0.04, 0.18), m.dark_iron, at=(sx + 0.65, yf + 0.2, 1.88), bevel=0.01)
    block("sign_anvil_waist", (0.25, 0.04, 0.16), m.dark_iron, at=(sx + 0.62, yf + 0.2, 1.72), bevel=0.01)
    block("sign_anvil_foot", (0.5, 0.04, 0.08), m.dark_iron, at=(sx + 0.62, yf + 0.2, 1.6), bevel=0.01)
    return m


# --- Vertshuset -------------------------------------------------------------


def tavern():
    spec = dict(seed=31, w=12.0, d=9.0, floors=2, jetty=0.45, pitch=50, gable=False, roof="thatch",
                plaster=(1.0, 0.93, 0.8), door_x=-1.2, windows0=[-4.4, -2.9, 1.6, 3.6], windows_up=5, plinth=0.6)
    m = town.half_timbered(spec)
    m.planks = layer("cladding", (0.5, 0.42, 0.34), 0.5)
    m.dark_iron = layer("metal", (0.28, 0.26, 0.25), 0.5)
    m.glow = plain("glow", (1.0, 0.75, 0.4), 0.4)
    rng = random.Random(32)
    fy = -spec["d"] / 2
    # Skiltet: et bord på en jernarm over døra, med et krus skåret ut.
    ax = -1.2 + 1.2
    beam("sign_arm", (ax, fy - 0.05, 3.25), (ax, fy - 1.3, 3.25), 0.06, 0.06, m.dark_iron)
    beam("sign_strut", (ax, fy - 0.05, 2.75), (ax, fy - 0.85, 3.22), 0.04, 0.04, m.dark_iron)
    for k in (-1, 1):
        beam("sign_ring", (ax, fy - 0.75 + k * 0.3, 3.22), (ax, fy - 0.75 + k * 0.3, 3.05), 0.02, 0.02, m.dark_iron)
    block("sign_board", (0.05, 0.85, 0.6), m.planks, at=(ax, fy - 0.75, 2.75), bevel=0.01)
    mug = revolve("sign_mug", [(0.0, -0.18), (0.13, -0.18), (0.15, 0.16), (0.0, 0.16)], m.dark_iron, n=12)
    mug.rotation_euler = (0, math.pi / 2, 0)
    mug.scale = (1, 1, 0.25)
    mug.location = (ax, fy - 0.75, 2.75)
    # Lykta ved døra, på en jernarm.
    lx = -1.2 - 0.9
    beam("lamp_arm", (lx, fy - 0.05, 2.5), (lx, fy - 0.45, 2.5), 0.03, 0.03, m.dark_iron)
    block("lamp_cage", (0.2, 0.2, 0.28), m.dark_iron, at=(lx, fy - 0.45, 2.3), bevel=0.01)
    block("lamp_glass", (0.16, 0.16, 0.22), m.glass, at=(lx, fy - 0.45, 2.3), bevel=0)
    block("lamp_flame", (0.04, 0.04, 0.08), m.glow, at=(lx, fy - 0.45, 2.3), bevel=0)
    # Benker og bord foran vinduene.
    for cx in (-3.6, 2.6):
        y = fy - 1.1
        block("table_top", (1.8, 0.75, 0.06), m.planks, at=(cx, y, 0.76), bevel=0.01)
        for sx in (-1, 1):
            block("table_leg", (0.08, 0.6, 0.7), m.timber, at=(cx + sx * 0.7, y, 0.38), bevel=0.01)
        for sy in (-1, 1):
            block("bench", (1.8, 0.3, 0.05), m.planks, at=(cx, y + sy * 0.62, 0.45), bevel=0.01)
            for sx in (-1, 1):
                block("bench_leg", (0.06, 0.26, 0.42), m.timber, at=(cx + sx * 0.75, y + sy * 0.62, 0.21), bevel=0.01)
        # Et krus og en tallerken på bordet.
        revolve("mug", [(0.0, 0.79), (0.05, 0.79), (0.055, 0.92), (0.0, 0.92)], m.planks, n=10, at=(cx - 0.4, y + 0.1, 0))
        revolve("plate", [(0.0, 0.79), (0.12, 0.79), (0.13, 0.81), (0.0, 0.81)], m.planks, n=14, at=(cx + 0.3, y - 0.05, 0))
    # Tønner og kasser ved hjørnet.
    for k, (x, y) in enumerate([(5.3, fy - 0.6), (5.0, fy - 1.25), (5.85, fy - 1.2)]):
        barrel(m, at=(x, y, 0.0), h=0.86, r=0.3, staves=16, rng=rng)
    barrel(m, at=(5.4, fy - 0.9, 0.86), h=0.7, r=0.25, staves=14, rng=rng)
    crate(m, at=(-5.4, fy - 0.6, 0.0), s=0.6, rng=rng)
    return m


MODELS = {"forge": forge, "tavern": tavern}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        MODELS[name]()
        finish(f"{OUT}/{name}.glb")
