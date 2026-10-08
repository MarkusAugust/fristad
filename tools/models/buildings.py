"""Bygningene på torget, modellert i Blender (bpy) og eksportert til
assets/models/town/. Kjør:

    python3 tools/models/buildings.py [forge tavern apothecary scriptorium stave_church gravestone]

Smia (`forge`): et lavt hus av bruddstein med sotet mur og spontak, og
foran et åpent verksted under et skråtak på stolper: esse av stein med
glør og pipe opp gjennom taket, blåsebelg av lær, ambolt på kubbe,
kjølekar med vann, slipestein, verktøy på veggen, vedstabel og et
smijernsskilt på arm.

Vertshuset (`tavern`): et stort bindingsverkshus i to etasjer med
langsida mot torget, framskutt overetasje og halmtak, og ved døra et
skilt på jernarm, en lykt, benker og bord, og tønner.

Apoteket (`apothecary`): et smalt bindingsverkshus med gavlen mot torget
og skifertak, med en salgsluke der skodda er slått ned til et bord med
krukker og glass, urtebunter til tørk under takskjegget, plantekrukker
og et skilt med morter og støter.

Skriptoriet (`scriptorium`): et hus av hogd stein med støttepilarer,
høye rundbuede vinduer med blyglass, rundbuet dør, skifertak, en liten
klokkestol på mønet og et skilt med en åpen bok.

Stavkirka (`stave_church`), bygd i lag som Borgund: svalgang rundt med
stolper og lavt tak, sideskip med skråtak, høyt midtskip med bratt
spontak og drakehoder på gavlene, takrytter med spir, kor og apsis med
eget lite tårn, rundbuet portal med søyler, alt i tjærebredde planker.
Gravsteinen (`gravestone`): en hogd steinhelle med rundet topp og kors.

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


# --- Apoteket ---------------------------------------------------------------


def jar(m, x, y, z, r, h, mat):
    """En krukke eller flaske (dreid), med kork."""
    revolve("jar", [(0.0, z), (r * 0.8, z), (r, z + h * 0.15), (r, z + h * 0.7), (r * 0.45, z + h * 0.88), (r * 0.4, z + h), (0.0, z + h)], mat, n=12, at=(x, y, 0))
    revolve("cork", [(0.0, z + h), (r * 0.36, z + h), (r * 0.36, z + h + 0.03), (0.0, z + h + 0.03)], m.bark, n=8, at=(x, y, 0))


def apothecary():
    spec = dict(seed=41, w=7.4, d=7.0, floors=2, jetty=0.4, pitch=55, gable=True, roof="slate",
                plaster=(0.95, 0.97, 0.9), door_x=1.9, windows0=[-1.4], windows_up=2, plinth=0.7)
    m = town.half_timbered(spec)
    m.planks = layer("cladding", (0.5, 0.42, 0.34), 0.5)
    m.dark_iron = layer("metal", (0.28, 0.26, 0.25), 0.5)
    m.bark = layer("bark", (0.8, 0.75, 0.7), 0.5)
    m.clay = layer("brick", (0.75, 0.5, 0.36), 0.5)
    m.herb = layer("thatch", (0.45, 0.55, 0.3), 0.5)
    m.herb2 = layer("thatch", (0.6, 0.45, 0.4), 0.5)
    rng = random.Random(42)
    fy = -spec["d"] / 2 - 0.11
    # Salgsluka: skodda er slått ned til et bord på to knekter under vinduet.
    wx, wz = -1.4, 0.7 + 1.45
    block("counter", (1.25, 0.5, 0.05), m.planks, at=(wx, fy - 0.27, wz - 0.55), bevel=0.008)
    for sx in (-1, 1):
        beam("bracket", (wx + sx * 0.5, fy - 0.02, wz - 1.0), (wx + sx * 0.5, fy - 0.45, wz - 0.58), 0.05, 0.05, m.timber)
    for k in range(6):
        x = wx - 0.5 + k * 0.2
        if k % 2:
            jar(m, x, fy - 0.27 + rng.uniform(-0.08, 0.08), wz - 0.525, 0.06, rng.uniform(0.14, 0.2), m.clay)
        else:
            jar(m, x, fy - 0.27 + rng.uniform(-0.08, 0.08), wz - 0.525, 0.05, rng.uniform(0.16, 0.24), m.glass)
    # En vekt og en morter på bordet.
    revolve("mortar", [(0.0, 0.0), (0.09, 0.0), (0.11, 0.1), (0.08, 0.1), (0.06, 0.04), (0.0, 0.04)], m.stone, n=14, at=(wx + 0.45, fy - 0.35, wz - 0.525))
    beam("pestle", (wx + 0.45, fy - 0.35, wz - 0.48), (wx + 0.52, fy - 0.3, wz - 0.3), 0.025, 0.025, m.planks)
    # Urtebunter til tørk på en stang under den framskutte etasjen.
    bar_z = 0.7 + 2.7 - 0.25
    beam("herb_bar", (-3.4, fy - 0.55, bar_z), (0.6, fy - 0.55, bar_z), 0.04, 0.04, m.timber)
    for k in range(9):
        x = -3.2 + k * 0.42
        mat = m.herb if k % 3 else m.herb2
        ln = rng.uniform(0.3, 0.45)
        b = revolve("herbs", [(0.0, -ln), (0.07, -ln * 0.8), (0.05, -0.05), (0.015, 0.0)], mat, n=8, at=(x, fy - 0.55, bar_z - 0.02))
        b.rotation_euler = (rng.uniform(-0.1, 0.1), rng.uniform(-0.1, 0.1), 0)
        beam("herb_tie", (x, fy - 0.55, bar_z), (x, fy - 0.55, bar_z - 0.06), 0.012, 0.012, m.planks)
    # Plantekrukker ved døra.
    for k, (x, y) in enumerate([(3.1, fy - 0.5), (3.3, fy - 1.0), (0.7, fy - 0.45)]):
        r = rng.uniform(0.18, 0.24)
        revolve("pot", [(0.0, 0.0), (r * 0.8, 0.0), (r, r * 1.3), (r * 1.05, r * 1.4), (0.0, r * 1.4)], m.clay, n=14, at=(x, y, 0))
        for j in range(6):
            a = j * 1.05
            b = revolve("plant", [(0.0, 0.0), (0.04, 0.12), (0.0, 0.35)], m.herb, n=5, at=(x + math.cos(a) * r * 0.4, y + math.sin(a) * r * 0.4, r * 1.35))
            b.rotation_euler = (math.cos(a) * 0.4, math.sin(a) * 0.4, 0)
    # Skiltet: en morter med støter i smijern på en arm ved hjørnet.
    sx = spec["w"] / 2 - 0.1
    beam("sign_arm", (sx, fy, 3.6), (sx, fy - 1.0, 3.6), 0.05, 0.05, m.dark_iron)
    beam("sign_strut", (sx, fy, 3.1), (sx, fy - 0.7, 3.58), 0.03, 0.03, m.dark_iron)
    beam("sign_chain", (sx, fy - 0.75, 3.58), (sx, fy - 0.75, 3.35), 0.015, 0.015, m.dark_iron)
    revolve("sign_mortar", [(0.0, -0.24), (0.16, -0.24), (0.2, 0.0), (0.16, 0.0), (0.0, -0.08)], m.dark_iron, n=14, at=(sx, fy - 0.75, 3.35))
    beam("sign_pestle", (sx - 0.05, fy - 0.75, 3.2), (sx + 0.12, fy - 0.75, 3.5), 0.04, 0.04, m.dark_iron)
    return m


# --- Skriptoriet ------------------------------------------------------------


def arched_window(m, target, u, face, normal, z0, w, h, thick, side=False):
    """Et høyt rundbuet vindu i en mur: hull, buestein, sålbenk og blyglass
    innfelt. Muren har yta i `face` med normalen `normal` (±1) langs y, eller
    langs x når `side`; `u` er plassen langs muren."""
    from walls import cutter
    s = normal

    def P(along, out, z):
        return Vector((face + s * out, along, z)) if side else Vector((along, face + s * out, z))

    def size(a, b, c):
        return (b, a, c) if side else (a, b, c)

    straight = h - w / 2
    cut(target, size(w, thick + 0.4, straight), P(u, -thick / 2, z0 + straight / 2))
    bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=w / 2, depth=thick + 0.4, location=P(u, -thick / 2, z0 + straight), rotation=(0, math.pi / 2, 0) if side else (math.pi / 2, 0, 0))
    cutter(bpy.context.active_object, target)
    town.uv_box(target)
    for k in range(9):
        a = math.pi * k / 8
        c = P(u + math.cos(a) * (w / 2 + 0.12), 0.04, z0 + straight + math.sin(a) * (w / 2 + 0.12))
        rot = (-(math.pi / 2 - a), 0, 0) if side else (0, math.pi / 2 - a, 0)
        block("voussoir", size(0.16, 0.14, 0.3), m.stone, at=c, rot=rot, bevel=0.02, segments=2)
    block("sill", size(w + 0.3, 0.22, 0.1), m.stone, at=P(u, 0.06, z0 - 0.05), bevel=0.02, segments=2)
    g = -thick * 0.6
    block("glass", size(w, 0.02, straight), m.glass, at=P(u, g, z0 + straight / 2), bevel=0)
    disc = revolve("glass_top", [(0.0, -0.01), (w / 2, -0.01), (w / 2, 0.01), (0.0, 0.01)], m.glass, n=16)
    disc.rotation_euler = (0, math.pi / 2, 0) if side else (math.pi / 2, 0, 0)
    disc.location = P(u, g, z0 + straight)
    for k in range(1, 3):
        block("lead", size(0.012, 0.03, h - 0.05), m.iron, at=P(u - w / 2 + w * k / 3, g + 0.01, z0 + (h - 0.05) / 2), bevel=0)
    for k in range(1, 6):
        block("lead", size(w, 0.03, 0.012), m.iron, at=P(u, g + 0.01, z0 + straight * k / 5), bevel=0)


def scriptorium():
    m = mats(roof="slate")
    m.ashlar = layer("brick", (0.62, 0.6, 0.57), 0.5)
    W, D, H = 10.0, 8.0, 5.2
    t = 0.65
    front = stone_wall(m, "front", -W / 2, W / 2, -D / 2, -D / 2 + t, -0.24, H, m.ashlar)
    back = stone_wall(m, "back", -W / 2, W / 2, D / 2 - t, D / 2, -0.24, H, m.ashlar)
    sides = [stone_wall(m, f"side{sx}", (sx * W / 2) - (t if sx > 0 else 0), (sx * W / 2) + (0 if sx > 0 else t), -D / 2 + t, D / 2 - t, -0.24, H, m.ashlar) for sx in (-1, 1)]
    # Sokkelen: en bredere steinrad nederst.
    block("plinth", (W + 0.3, D + 0.3, 0.6), m.stone, at=(0, 0, 0.06), bevel=0.04, segments=2)
    # Rundbuet dør midt på, med buestein og trinn.
    dw, dh = 1.3, 2.6
    cut(front, (dw, t + 0.4, dh - dw / 2), (0, -D / 2 + t / 2, 0.35 + (dh - dw / 2) / 2))
    bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=dw / 2, depth=t + 0.4, location=(0, -D / 2 + t / 2, 0.35 + dh - dw / 2), rotation=(math.pi / 2, 0, 0))
    from walls import cutter
    cutter(bpy.context.active_object, front)
    for k in range(11):
        a = math.pi * k / 10
        c = Vector((math.cos(a) * (dw / 2 + 0.15), -D / 2 - 0.04, 0.35 + dh - dw / 2 + math.sin(a) * (dw / 2 + 0.15)))
        block("door_voussoir", (0.2, 0.16, 0.34), m.stone, at=c, rot=(0, math.pi / 2 - a, 0), bevel=0.02, segments=2)
    n = 6
    for k in range(n):
        px = -dw / 2 + dw * (k + 0.5) / n
        top = 0.35 + dh - dw / 2 + math.sqrt(max(0.0, (dw / 2) ** 2 - px * px)) - 0.02
        block("door_plank", (dw / n - 0.008, 0.06, top - 0.35), m.boards, at=(px, -D / 2 + t * 0.55, 0.35 + (top - 0.35) / 2), bevel=0.005)
    for z in (0.8, 2.0):
        block("door_strap", (dw * 0.9, 0.03, 0.07), m.iron, at=(0, -D / 2 + t * 0.55 - 0.045, z), bevel=0.005)
    block("step", (dw + 0.8, 0.6, 0.2), m.stone, at=(0, -D / 2 - 0.3, 0.1), bevel=0.03, segments=2)
    block("step2", (dw + 1.2, 0.5, 0.15), m.stone, at=(0, -D / 2 - 0.65, -0.02), bevel=0.03, segments=2)
    # Høye rundbuede vinduer: to på fronten, to på hver langside.
    for x in (-3.0, 3.0):
        arched_window(m, front, x, -D / 2, -1.0, 1.4, 0.9, 2.6, t)
    for sx, side in zip((-1, 1), sides):
        for y in (-1.9, 1.9):
            arched_window(m, side, y, sx * W / 2, float(sx), 1.4, 0.8, 2.4, t, side=True)
    # Støttepilarer på hjørnene og midt på langsidene, trappet inn.
    for x, y, ax in [(-W / 2, -D / 2, 0), (W / 2, -D / 2, 0), (-W / 2, D / 2, 0), (W / 2, D / 2, 0), (-W / 2, 0, 1), (W / 2, 0, 1)]:
        for k, (hgt, depth) in enumerate([(2.2, 0.75), (1.6, 0.5), (1.0, 0.3)]):
            z0 = [0, 2.2, 3.8][k]
            if ax:
                block("buttress", (depth, 0.7, hgt), m.ashlar, at=(x + math.copysign(depth / 2, x), y, z0 + hgt / 2 - (0.24 if k == 0 else 0)), bevel=0.04, segments=2)
            else:
                block("buttress", (0.7, depth, hgt), m.ashlar, at=(x, y + math.copysign(depth / 2, y), z0 + hgt / 2 - (0.24 if k == 0 else 0)), bevel=0.04, segments=2)
    # Gesimsen under taket.
    block("cornice", (W + 0.3, D + 0.3, 0.2), m.stone, at=(0, 0, H + 0.05), bevel=0.03, segments=2)
    # Taket: bratt saltak med gavlen mot torget, gavler av hogd stein.
    rise = roof(m, W + 0.2, D, H + 0.1, 52, gable_to_street=True)
    from town import prism
    for sy in (-1, 1):
        prism("gable_stone", [(-W / 2, 0.0), (W / 2, 0.0), (0.0, rise)], t, m.ashlar, at=(0, sy * (D / 2 - t / 2), H + 0.1))
        # Et lite rundvindu i gavlen.
        g = revolve("oculus", [(0.0, -0.02), (0.35, -0.02), (0.35, 0.02), (0.0, 0.02)], m.glass, n=16)
        g.rotation_euler = (math.pi / 2, 0, 0)
        g.location = (0, sy * (D / 2 + 0.01), H + 0.1 + rise * 0.35)
        rim = revolve("oculus_rim", [(0.35, -0.08), (0.48, -0.08), (0.48, 0.08), (0.35, 0.08)], m.stone, n=16, cap=False)
        rim.rotation_euler = (math.pi / 2, 0, 0)
        rim.location = (0, sy * (D / 2 + 0.02), H + 0.1 + rise * 0.35)
    # Klokkestolen på mønet, foran: fire stolper, et lite tak og ei klokke.
    bz = H + 0.1 + rise
    for sx in (-1, 1):
        for sy in (-1, 1):
            beam("bell_post", (sx * 0.35, -D / 2 + 1.2 + sy * 0.35, bz - 0.2), (sx * 0.35, -D / 2 + 1.2 + sy * 0.35, bz + 1.1), 0.1, 0.1, m.timber)
    from walls import pyramid
    pyramid("bell_roof", 0.6, 0.6, bz + 1.1, bz + 1.8, m.roof)
    revolve("bell", [(0.0, bz + 0.85), (0.08, bz + 0.85), (0.14, bz + 0.6), (0.2, bz + 0.4), (0.0, bz + 0.42)], m.iron, n=16, at=(0, -D / 2 + 1.2, 0))
    revolve("finial", [(0.05, bz + 1.75), (0.03, bz + 2.2), (0.0, bz + 2.3)], m.iron, n=8)
    PIECES[-1].location = (0, -D / 2 + 1.2, 0)
    # Skiltet: en åpen bok i smijern på en arm ved døra.
    sx = 1.6
    beam("sign_arm", (sx, -D / 2 - 0.05, 3.3), (sx, -D / 2 - 1.0, 3.3), 0.05, 0.05, m.dark_iron)
    beam("sign_strut", (sx, -D / 2 - 0.05, 2.85), (sx, -D / 2 - 0.7, 3.28), 0.03, 0.03, m.dark_iron)
    beam("sign_chain", (sx, -D / 2 - 0.7, 3.28), (sx, -D / 2 - 0.7, 3.05), 0.015, 0.015, m.dark_iron)
    for s in (-1, 1):
        block("sign_page", (0.04, 0.32, 0.4), m.planks, at=(sx, -D / 2 - 0.7 + s * 0.17, 2.85), rot=(s * 0.25, 0, 0), bevel=0.01)
    block("sign_spine", (0.06, 0.04, 0.42), m.dark_iron, at=(sx, -D / 2 - 0.7, 2.83), bevel=0.01)
    return m


# --- Stavkirka --------------------------------------------------------------


def plank_walls(m, name, w, d, z0, z1, mat):
    """Fire vegger av stående planker rundt et rektangel, med svill og
    stavlegjer (bjelke øverst)."""
    t = 0.14
    for sy in (-1, 1):
        block(f"{name}_wall", (w, t, z1 - z0), mat, at=(0, sy * (d / 2 - t / 2), (z0 + z1) / 2), bevel=0.01)
    for sx in (-1, 1):
        block(f"{name}_wall", (t, d - 2 * t, z1 - z0), mat, at=(sx * (w / 2 - t / 2), 0, (z0 + z1) / 2), bevel=0.01)
    for z in (z0 + 0.1, z1 - 0.1):
        for sy in (-1, 1):
            beam(f"{name}_sill", (-w / 2 - 0.05, sy * (d / 2 + 0.02), z), (w / 2 + 0.05, sy * (d / 2 + 0.02), z), 0.12, 0.2, mat)
        for sx in (-1, 1):
            beam(f"{name}_sill", (sx * (w / 2 + 0.02), -d / 2 - 0.05, z), (sx * (w / 2 + 0.02), d / 2 + 0.05, z), 0.12, 0.2, mat)
    # Hjørnestavene.
    for sx in (-1, 1):
        for sy in (-1, 1):
            revolve(f"{name}_stave", [(0.16, z0), (0.16, z1)], mat, n=10, at=(sx * w / 2, sy * d / 2, 0))


def skirt_roof(m, name, w_in, d_in, w_out, d_out, z_in, z_out, mat):
    """Et lavt tak rundt et rektangel (svalgang, sideskip): fire skrå flater
    fra den indre veggen ned til den ytre kanten."""
    for sy in (-1, 1):
        run = (d_out - d_in) / 2
        tilt = math.atan2(z_in - z_out, run)
        ln = math.hypot(run, z_in - z_out)
        ob = block(f"{name}_roof", (w_out, ln, 0.12), mat, bevel=0.01)
        town._roof_uv(ob, along_y=False)
        ob.rotation_euler = (sy * tilt, 0, 0)
        ob.location = (0, sy * (d_in / 2 + run / 2), (z_in + z_out) / 2 + 0.08)
    for sx in (-1, 1):
        run = (w_out - w_in) / 2
        tilt = math.atan2(z_in - z_out, run)
        ln = math.hypot(run, z_in - z_out)
        ob = block(f"{name}_roof", (ln, d_in, 0.12), mat, bevel=0.01)
        town._roof_uv(ob, along_y=True)
        ob.rotation_euler = (0, -sx * tilt, 0)
        ob.location = (sx * (w_in / 2 + run / 2), 0, (z_in + z_out) / 2 + 0.08)


def dragon(m, x, y, z, out, mat):
    """Et drakehode på gavlspissen: en buet hals ut og opp, og et hode med
    gapende kjeft."""
    pts = [Vector((x, y, z))]
    for k in range(1, 5):
        t = k / 4
        pts.append(Vector((x, y + out * 0.9 * t, z + 0.55 * math.sin(t * math.pi * 0.7))))
    for a, b in zip(pts, pts[1:]):
        beam("dragon_neck", a, b, 0.12, 0.16, mat)
    head = pts[-1]
    block("dragon_head", (0.12, 0.38, 0.2), mat, at=head + Vector((0, out * 0.15, 0.05)), rot=(out * -0.35, 0, 0), bevel=0.03)
    block("dragon_jaw", (0.1, 0.32, 0.08), mat, at=head + Vector((0, out * 0.2, -0.1)), rot=(out * 0.3, 0, 0), bevel=0.02)
    for s in (-1, 1):
        beam("dragon_horn", head + Vector((s * 0.04, 0, 0.12)), head + Vector((s * 0.08, -out * 0.15, 0.4)), 0.04, 0.04, mat)


def stave_church():
    m = mats(roof="tiles")
    m.tar = layer("timber", (0.85, 0.85, 0.85), 0.5)
    m.shingle = layer("tiles", (0.42, 0.36, 0.3), 0.5)
    W, D = 6.0, 8.6  # sideskipet
    # Steinfundamentet.
    block("plinth", (W + 2.9, D + 2.9, 0.5), m.stone, at=(0, 0, 0.0), bevel=0.04, segments=2)
    # Svalgangen: lav plankevegg med åpne buer mellom stolpene, og tak.
    gw, gd = W + 2.6, D + 2.6
    for sy in (-1, 1):
        for k in range(9):
            x = -gw / 2 + gw * k / 8
            beam("gallery_post", (x, sy * gd / 2, 0.25), (x, sy * gd / 2, 2.0), 0.12, 0.12, m.tar, roll=math.pi / 2)
        if sy > 0:
            block("gallery_wall", (gw, 0.08, 0.9), m.tar, at=(0, sy * gd / 2, 0.7), bevel=0.01)
        else:
            for x0, x1 in ((-gw / 2, -0.9), (0.9, gw / 2)):
                block("gallery_wall", (x1 - x0, 0.08, 0.9), m.tar, at=((x0 + x1) / 2, sy * gd / 2, 0.7), bevel=0.01)
        beam("gallery_plate", (-gw / 2, sy * gd / 2, 2.0), (gw / 2, sy * gd / 2, 2.0), 0.14, 0.14, m.tar)
    for sx in (-1, 1):
        for k in range(11):
            y = -gd / 2 + gd * k / 10
            beam("gallery_post", (sx * gw / 2, y, 0.25), (sx * gw / 2, y, 2.0), 0.12, 0.12, m.tar, roll=math.pi / 2)
        block("gallery_wall", (0.08, gd, 0.9), m.tar, at=(sx * gw / 2, 0, 0.7), bevel=0.01)
        beam("gallery_plate", (sx * gw / 2, -gd / 2, 2.0), (sx * gw / 2, gd / 2, 2.0), 0.14, 0.14, m.tar)
    skirt_roof(m, "gallery", W, D, gw + 0.5, gd + 0.5, 3.0, 2.0, m.shingle)
    # Sideskipet og taket over det.
    plank_walls(m, "aisle", W, D, 0.25, 3.6, m.tar)
    NW, ND = 3.8, 6.6
    skirt_roof(m, "aisle", NW, ND, W + 0.5, D + 0.5, 4.8, 3.55, m.shingle)
    # Midtskipet: høye vegger over sideskipet, bratt tak med drakehoder.
    plank_walls(m, "nave", NW, ND, 3.6, 7.2, m.tar)
    before = len(PIECES)
    rise = roof(m, NW + 0.1, ND, 7.2, 62, overhang=0.35, gable_to_street=True)
    for ob in PIECES[before:]:
        if ob.name.startswith("roof"):
            ob.data.materials[0] = m.shingle
    for sy, ny in ((-1, -1.0), (1, 1.0)):
        gable(m, NW, 7.2, rise, sy * ND / 2, ny, random.Random(7), boards=True)
        dragon(m, 0.0, sy * (ND / 2 + 0.35), 7.2 + rise + 0.15, sy, m.tar)
    # Takrytteren midt på mønet: et lite tårn med svalganger og et spir.
    tz = 7.2 + rise - 0.6
    block("turret", (1.3, 1.3, 1.8), m.tar, at=(0, 0, tz + 0.9), bevel=0.02)
    for sy in (-1, 1):
        for k in range(3):
            block("turret_arch", (0.25, 0.04, 0.5), m.glass, at=(-0.4 + k * 0.4, sy * 0.66, tz + 1.3), bevel=0)
    from walls import pyramid
    pyramid("turret_roof", 0.9, 0.9, tz + 1.8, tz + 2.3, m.shingle)
    spire = revolve("spire", [(0.62, tz + 2.2), (0.0, tz + 5.0)], m.shingle, n=8, cap=False, uv="planar")
    spire.rotation_euler = (0, 0, math.pi / 8)
    revolve("spire_top", [(0.05, tz + 4.9), (0.04, tz + 5.6), (0.0, tz + 5.7)], m.iron, n=8)
    # Koret og apsiden bak.
    cy = D / 2 + 1.4
    before = len(PIECES)
    plank_walls(m, "choir", 3.4, 2.8, 0.25, 3.4, m.tar)
    rr = roof(m, 3.5, 2.8, 3.4, 58, overhang=0.3, gable_to_street=True)
    for ob in PIECES[before:]:
        if ob.name.startswith("roof"):
            ob.data.materials[0] = m.shingle
    moved(before, Matrix.Translation((0, cy, 0)))
    ay = cy + 1.4
    apse = revolve("apse", [(1.3, 0.25), (1.3, 3.0)], m.tar, n=16, cap=False)
    apse.location = (0, ay, 0)
    revolve("apse_roof", [(1.6, 2.9), (0.0, 4.6)], m.shingle, n=16, cap=False, uv="planar").location = (0, ay, 0)
    revolve("apse_tower", [(0.35, 4.3), (0.35, 5.2), (0.0, 6.4)], m.shingle, n=8, uv="planar").location = (0, ay, 0)
    # Portalen: rundbuet dør i vestgavlen, med søyler og en buet karm.
    py = -D / 2 - 0.08
    n = 5
    for k in range(n):
        px = -0.55 + 1.1 * (k + 0.5) / n
        top = 0.3 + 2.0 + math.sqrt(max(0.0, 0.55 ** 2 - px * px)) * 0.9
        block("portal_plank", (1.1 / n - 0.01, 0.06, top - 0.3), m.boards, at=(px, py, 0.3 + (top - 0.3) / 2), bevel=0.005)
    for sx in (-1, 1):
        revolve("portal_column", [(0.1, 0.25), (0.08, 0.4), (0.08, 2.2), (0.12, 2.35), (0.0, 2.35)], m.tar, n=10, at=(sx * 0.75, py - 0.08, 0))
        for k in range(6):
            block("carving", (0.1, 0.05, 0.12), m.tar, at=(sx * 0.95, py - 0.06, 0.6 + k * 0.3), rot=(0, sx * 0.6 * (1 if k % 2 else -1), 0), bevel=0.02)
    for k in range(9):
        a = math.pi * k / 8
        c = Vector((math.cos(a) * 0.72, py - 0.06, 2.3 + math.sin(a) * 0.6))
        block("portal_arch", (0.18, 0.1, 0.22), m.tar, at=c, rot=(0, math.pi / 2 - a, 0), bevel=0.02)
    for z in (0.8, 1.8):
        block("portal_strap", (1.0, 0.03, 0.06), m.iron, at=(0, py - 0.04, z), bevel=0.005)
    # Trinnet opp til svalgangen foran portalen.
    block("step", (1.8, 0.8, 0.25), m.stone, at=(0, -gd / 2 - 0.5, 0.0), bevel=0.03, segments=2)
    return m


def gravestone():
    m = mats()
    m.moss = layer("sod", (0.6, 0.7, 0.55), 0.5)
    # Hella med rundet topp, litt skjev, og et kors hogd ut i relieff.
    block("slab", (0.55, 0.14, 0.75), m.stone, at=(0, 0, 0.25), rot=(0.04, 0.03, 0), bevel=0.03, segments=2)
    top = revolve("slab_top", [(0.0, -0.07), (0.275, -0.07), (0.275, 0.07), (0.0, 0.07)], m.stone, n=16)
    top.rotation_euler = (math.pi / 2 + 0.04, 0, 0)
    top.location = (0, 0.03, 0.62)
    block("cross_v", (0.06, 0.03, 0.42), m.stone, at=(0, -0.08, 0.5), rot=(0.04, 0, 0), bevel=0.01)
    block("cross_h", (0.26, 0.03, 0.06), m.stone, at=(0, -0.08, 0.58), rot=(0.04, 0, 0), bevel=0.01)
    # Mose ved foten og en liten haug foran.
    block("moss", (0.6, 0.2, 0.1), m.moss, at=(0, 0, -0.07), bevel=0.04, segments=2)
    block("mound", (0.7, 1.6, 0.18), m.moss, at=(0, -0.95, -0.05), bevel=0.08, segments=3)
    return m


MODELS = {"forge": forge, "tavern": tavern, "apothecary": apothecary, "scriptorium": scriptorium, "stave_church": stave_church, "gravestone": gravestone}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in MODELS] or list(MODELS)
    for name in names:
        clear()
        PIECES.clear()
        MODELS[name]()
        finish(f"{OUT}/{name}.glb")
