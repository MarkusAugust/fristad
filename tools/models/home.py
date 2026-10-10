"""Spillerens hus på torget, i den malte stilen (kit.py). Kjør:

    python3 tools/models/home.py

Gir utsida (`home.glb`), innsida (`home_interior.glb`, tegnes bare nær) og
`home_inside.json` med kolliderne, lysene, røyken, senga og rammene på
galleriveggen, og modellen `gallery_piece` som henger i en ramme når
komponenten er funnet (lakken farges per komponent).

Huset: lavt og koselig bindingsverk med hvit puss og halmtak, blomsterkasser
under vinduene, en benk ved døra og et skilt med «Ditt hjem» i forgylte
bokstaver over døra. Inne: plankegulv med fillerye, peis med ild, seng med
lappeteppe, kiste, bord med krakker, hyller med krukker og lys, og
galleriveggen (vestveggen) med 15 rammer i tre rader.

Akser som i town.py: X til høyre, −Y fram (mot torget), Z opp. Rammene og
senga må stå der `home_inside.json` sier (det skrives herfra)."""

import json
import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import kit  # noqa: E402
import town  # noqa: E402
from common import clear  # noqa: E402
from kit import CARVED, MOULDING, ROPE, barrel, bench, box, candle, door, fire, half_timbered, planks, stick, stool, table, window  # noqa: E402
from town import collide, layer, light  # noqa: E402

OUT = "assets/models/town"

W, D, T = 8.4, 7.0, 0.3
F0 = 0.45
E = F0 + 3.2
PITCH = 50.0
X0, X1 = -W / 2 + T / 2, W / 2 - T / 2
Y0, Y1 = -D / 2 + T / 2, D / 2 - T / 2
IX0, IX1, IY0, IY1 = X0 + T / 2, X1 - T / 2, Y0 + T / 2, Y1 - T / 2
DOOR_X = 0.6
# Galleriveggen: innsida av vestveggen, 5 kolonner × 3 rader.
GAL_X = IX0 + 0.04
GAL_COLS = (-2.0, -1.0, 0.0, 1.0, 2.0)
GAL_ROWS = (2.55, 1.8, 1.05)
FRAME = 0.56
# Senga i hjørnet bak til høyre.
BED = (2.6, 2.0)


def outside(p, rng):
    sx, sy = W / 2 + 0.1, D / 2 + 0.1
    box("plinth", (-sx, -sy, -0.25), (sx, sy, F0), p.stone, bevel=0.06)
    box("step", (DOOR_X - 0.75, -sy - 0.6, -0.2), (DOOR_X + 0.75, -sy + 0.05, F0 - 0.15), p.stone, bevel=0.05)
    town.ramp(DOOR_X - 0.7, DOOR_X + 0.7, -sy - 0.8, -sy + 0.05, 0.0, F0)
    fw = (F0 + 0.95, F0 + 2.15)
    front = [(DOOR_X - X0, F0, F0 + 2.2, 1.15), (-2.5 - X0, fw[0], fw[1], 1.0), (2.8 - X0, fw[0], fw[1], 1.0)]
    fr, _ = half_timbered(p, "wall_front", (X0, Y0), (X1, Y0), F0, E, -1, front, rng)
    back = [(1.0 - X0, fw[0], fw[1], 0.9)]
    bk, _ = half_timbered(p, "wall_back", (X0, Y1), (X1, Y1), F0, E, 1, back, rng)
    lf, _ = half_timbered(p, "wall_left", (X0, Y0), (X0, Y1), F0, E, 1, [], rng)
    right = [(-1.2 - Y0, fw[0], fw[1], 0.9)]
    rt, _ = half_timbered(p, "wall_right", (X1, Y0), (X1, Y1), F0, E, -1, right, rng)
    for frame, opens, out in ((fr, front[1:], -1), (bk, back, 1), (rt, right, -1)):
        for s, za, zb, w in opens:
            window(p, frame, s, (za + zb) / 2, w, zb - za, out, rng=rng)
    door(p, fr, DOOR_X - X0, F0, 1.15, 2.2, -1, open_in=True)
    # Blomsterkasser under vinduene foran: planker med jord og blomster.
    flowers = [layer("plaster_red", (1.0, 0.75, 0.75)), layer("plaster", (1.0, 0.95, 0.4)), layer("plaster_white", (0.85, 0.8, 1.0))]
    green = layer("sod", (0.7, 1.0, 0.6))
    for x in (-2.5, 2.8):
        box("flower_box", (x - 0.6, Y0 - T / 2 - 0.32, fw[0] - 0.32), (x + 0.6, Y0 - T / 2 - 0.02, fw[0] - 0.08), p.cladding, bevel=0.02)
        for k in range(6):
            fx = x - 0.48 + k * 0.19
            box("leaves", (fx - 0.08, Y0 - T / 2 - 0.28, fw[0] - 0.1), (fx + 0.08, Y0 - T / 2 - 0.06, fw[0] + 0.06), green, bevel=0.05)
            box("bloom", (fx - 0.05, Y0 - T / 2 - 0.24, fw[0] + 0.05), (fx + 0.05, Y0 - T / 2 - 0.12, fw[0] + 0.14), flowers[(k + int(x)) % 3], bevel=0.03)
    # Taket, gavlene og pipa fra peisen.
    ridge = kit.gable_roof(p, X0 - T / 2, X1 + T / 2, Y0 - T / 2, Y1 + T / 2, E + 0.28, PITCH)
    for x, out in ((X0, -1), (X1, 1)):
        kit.gable_wall(p, x, Y0 - T / 2, Y1 + T / 2, E, ridge, PITCH, out, rng)
    roof_at = lambda y: E + 0.28 + (Y1 + T / 2 + 0.9 - abs(y)) * math.tan(math.radians(PITCH)) - 1.2
    kit.chimney(p, -0.8, 2.6, roof_at(2.6) - 0.8, roof_at(2.6) + 1.7, w=1.0, smoke=0.7, lean=-4.0)
    # Skiltet over døra: et bord med forgylte bokstaver, «Ditt hjem».
    # Skiltet: på en stolpe med galge foran huset, vendt mot torget, med
    # «Ditt hjem» i forgylte bokstaver (takskjegget henger for lavt til å
    # ha det på veggen).
    px, py = DOOR_X - 1.9, Y0 - T / 2 - 1.3
    stick("sign_post", (px, py, -0.2), (px, py, 2.9), 0.18, 0.18, p.timber, ("timber", 1))
    stick("sign_beam", (px - 0.1, py, 2.75), (px + 1.5, py, 2.75), 0.14, 0.14, p.timber, ("timber", 2))
    stick("sign_brace", (px, py, 2.2), (px + 0.5, py, 2.72), 0.1, 0.1, p.timber, ("timber", 3))
    sx0, sx1 = px + 0.2, px + 1.4
    for x in (sx0 + 0.1, sx1 - 0.1):
        stick("sign_chain", (x, py, 2.68), (x, py, 2.42), 0.02, 0.02, p.iron)
    sz = 2.15
    box("sign_board", (sx0, py - 0.04, sz - 0.27), (sx1, py + 0.04, sz + 0.27), layer("planks", (0.55, 0.4, 0.32)), bevel=0.03)
    stick("sign_rim", (sx0 - 0.03, py - 0.05, sz + 0.27), (sx1 + 0.03, py - 0.05, sz + 0.27), 0.05, 0.05, p.trim, ("trim", ROPE))
    stick("sign_rim", (sx0 - 0.03, py - 0.05, sz - 0.27), (sx1 + 0.03, py - 0.05, sz - 0.27), 0.05, 0.05, p.trim, ("trim", ROPE))
    # Malte gule bokstaver (smijernet ble for mørkt mot bordet).
    gold = layer("plaster", (1.0, 0.85, 0.3), 0.4)
    letters("Ditt", ((sx0 + sx1) / 2, py - 0.06, sz + 0.1), 0.2, gold)
    letters("hjem", ((sx0 + sx1) / 2, py - 0.06, sz - 0.12), 0.2, gold)
    # Baksida har teksten også, så den leses fra begge kanter.
    for word, dz in (("Ditt", 0.1), ("hjem", -0.12)):
        ob = letters(word, ((sx0 + sx1) / 2, py + 0.06, sz + dz), 0.2, gold)
        ob.rotation_euler = (math.pi / 2, 0, math.pi)
    collide((px, py, 1.4), (0.3, 0.3, 2.8))
    # Lykta ved døra, og en benk.
    town.wall_lantern(p, DOOR_X + 1.05, Y0 - T / 2, F0 + 2.0, -1)
    bench(p, -1.2, Y0 - T / 2 - 0.55, 0.0, 1.6)


def letters(text, at, size, mat):
    """Tekst som modell: bokstavene ekstrudert fra Blenders skrift og
    gjort om til mesh, med forsida mot −Y."""
    bpy.ops.object.text_add(location=(0, 0, 0))
    ob = bpy.context.active_object
    ob.data.body = text
    ob.data.align_x = "CENTER"
    ob.data.align_y = "CENTER"
    ob.data.size = size
    ob.data.extrude = 0.015
    ob.data.bevel_depth = 0.004
    ob.data.resolution_u = 3
    bpy.ops.object.convert(target="MESH")
    ob = bpy.context.active_object
    ob.rotation_euler = (math.pi / 2, 0, 0)
    ob.location = at
    ob.data.materials.clear()
    ob.data.materials.append(mat)
    kit.ensure_uv(ob.data)
    town.uv_box(ob)
    town.PIECES.append(ob)
    return ob


def inside(p, rng):
    # Gulvet, taket og bjelkene.
    planks(p, "floor", (IX0, IY0, F0 - 0.06), (IX1, IY1, F0 + 0.03))
    collide((0, 0, F0 - 0.15), (IX1 - IX0, IY1 - IY0, 0.3))
    box("ceiling", (IX0, IY0, E - 0.02), (IX1, IY1, E + 0.04), p.floor, bevel=0)
    x = IX0 + 0.5
    while x < IX1:
        stick("joist", (x, IY0, E - 0.14), (x, IY1, E - 0.14), 0.18, 0.24, p.timber, ("timber", rng.randrange(4)), seed=x)
        x += 1.1
    # Fillerya foran peisen.
    wool = [layer("wool", c) for c in ((0.8, 0.35, 0.28), (0.35, 0.5, 0.75), (0.85, 0.7, 0.35))]
    for k in range(5):
        box("rug", (-2.0, -0.6 + k * 0.32, F0), (0.6, -0.28 + k * 0.32, F0 + 0.02), wool[k % 3], bevel=0.01)
    # Peisen på bakveggen, med ild, og ved ved siden av.
    hx, hy = -0.8, IY1
    box("hearth_base", (hx - 1.1, hy - 0.8, F0), (hx + 1.1, hy, F0 + 0.3), p.stone, bevel=0.05)
    box("hearth_back", (hx - 0.8, hy - 0.25, F0 + 0.3), (hx + 0.8, hy, E), p.soot, bevel=0.03)
    for xx in (hx - 1.1, hx + 0.75):
        box("hearth_jamb", (xx, hy - 0.8, F0 + 0.3), (xx + 0.35, hy, F0 + 1.45), p.stone, bevel=0.05)
    box("hearth_lintel", (hx - 1.15, hy - 0.85, F0 + 1.45), (hx + 1.15, hy, F0 + 1.75), p.stone, bevel=0.05)
    stick("mantel", (hx - 1.3, hy - 0.95, F0 + 1.8), (hx + 1.3, hy - 0.95, F0 + 1.8), 0.3, 0.12, p.trim, ("trim", CARVED))
    box("hood", (hx - 0.9, hy - 0.7, F0 + 1.75), (hx + 0.9, hy, E), p.stone, bevel=0.04)
    fire(p, hx, hy - 0.4, F0 + 0.3, size=0.8, intensity=3.0, radius=9.0)
    collide((hx, hy - 0.4, F0 + 0.9), (2.3, 0.85, 1.8))
    for k in range(5):
        stick("firewood", (hx + 1.4, hy - 0.2 - 0.13 * (k % 2), F0 + 0.07 + 0.13 * (k // 2)), (hx + 2.0, hy - 0.2 - 0.13 * (k % 2), F0 + 0.07 + 0.13 * (k // 2)), 0.12, 0.12, layer("logs"), bevel=0.03)
    # Krukker og lys på hylla over peisen.
    for k, xx in enumerate((hx - 1.0, hx - 0.6, hx + 0.6, hx + 1.0)):
        box("crock", (xx - 0.08, hy - 0.98, F0 + 1.86), (xx + 0.08, hy - 0.84, F0 + 2.1), layer("plaster_red" if k % 2 else "plaster_white"), bevel=0.04)
    candle(p, hx, hy - 0.92, F0 + 1.86)

    # Senga: rammen, halmmadrass, lappeteppe, pute og en kiste ved fotenden.
    bx, by = BED
    box("bed_frame", (bx - 0.55, by - 1.05, F0), (bx + 0.55, by + 1.05, F0 + 0.38), p.timber, bevel=0.04)
    box("bed_head", (bx - 0.6, by + 0.95, F0), (bx + 0.6, by + 1.1, F0 + 1.05), p.timber, bevel=0.04)
    box("bed_straw", (bx - 0.48, by - 0.98, F0 + 0.38), (bx + 0.48, by + 0.95, F0 + 0.52), p.straw, bevel=0.05)
    for k in range(4):
        box("quilt", (bx - 0.5, by - 1.0 + k * 0.38, F0 + 0.48), (bx + 0.5, by - 0.62 + k * 0.38, F0 + 0.6), wool[k % 3], bevel=0.04)
    box("pillow", (bx - 0.36, by + 0.5, F0 + 0.5), (bx + 0.36, by + 0.9, F0 + 0.68), layer("linen"), bevel=0.07)
    collide((bx, by + 0.05, F0 + 0.3), (1.1, 1.9, 0.6))
    # Kista ved siden av senga (ikke foran, der du sikter på senga).
    cx, cy = bx - 1.05, by + 0.55
    box("chest", (cx - 0.28, cy - 0.45, F0), (cx + 0.28, cy + 0.45, F0 + 0.5), p.cladding, bevel=0.03)
    box("chest_band", (cx - 0.3, cy - 0.47, F0 + 0.34), (cx + 0.3, cy + 0.47, F0 + 0.4), p.iron, bevel=0.01)
    collide((cx, cy, F0 + 0.25), (0.56, 0.9, 0.5))

    # Bordet med krakker foran vinduet, og lys.
    table(p, 2.6, -1.6, F0, 1.4, 0.8)
    for xx, yy in ((1.75, -1.6), (3.3, -1.3)):
        stool(p, xx, yy, F0)
        collide((xx, yy, F0 + 0.25), (0.4, 0.4, 0.5))
    candle(p, 2.6, -1.6, F0 + 0.78)
    box("bread", (2.85, -1.75, F0 + 0.78), (3.1, -1.55, F0 + 0.88), layer("thatch", (0.9, 0.62, 0.35)), bevel=0.05)
    barrel(p, IX1 - 0.4, -0.2, F0, r=0.3, h=0.8)
    collide((IX1 - 0.4, -0.2, F0 + 0.4), (0.6, 0.6, 0.8))
    # Hylla på østveggen.
    for z in (F0 + 1.4, F0 + 1.9):
        box("shelf", (IX1 - 0.3, -0.9, z), (IX1, 0.5, z + 0.04), p.floor, bevel=0.01)
        for k in range(4):
            yy = -0.7 + k * 0.38
            box("jar", (IX1 - 0.24, yy - 0.08, z + 0.04), (IX1 - 0.08, yy + 0.08, z + 0.26), layer("plaster_red" if (k + int(z)) % 2 else "plaster"), bevel=0.04)

    # Galleriveggen: en list over og under, og 15 rammer med mørk bakplate.
    stick("gallery_rail", (GAL_X + 0.03, IY0 + 0.2, F0 + GAL_ROWS[0] + 0.5), (GAL_X + 0.03, IY1 - 0.2, F0 + GAL_ROWS[0] + 0.5), 0.08, 0.12, p.trim, ("trim", MOULDING))
    stick("gallery_rail", (GAL_X + 0.03, IY0 + 0.2, F0 + GAL_ROWS[-1] - 0.5), (GAL_X + 0.03, IY1 - 0.2, F0 + GAL_ROWS[-1] - 0.5), 0.08, 0.12, p.trim, ("trim", MOULDING))
    gallery = []
    for z in GAL_ROWS:
        for y in GAL_COLS:
            c = (GAL_X, y, F0 + z)
            box("frame_back", (GAL_X - 0.02, y - FRAME / 2 + 0.04, F0 + z - FRAME / 2 + 0.04), (GAL_X + 0.01, y + FRAME / 2 - 0.04, F0 + z + FRAME / 2 - 0.04), layer("wool", (0.3, 0.42, 0.36)), bevel=0)
            h = FRAME / 2
            for a, b in (((y - h, z - h), (y + h, z - h)), ((y - h, z + h), (y + h, z + h)), ((y - h, z - h), (y - h, z + h)), ((y + h, z - h), (y + h, z + h))):
                stick("frame", (GAL_X + 0.03, a[0], F0 + a[1]), (GAL_X + 0.03, b[0], F0 + b[1]), 0.07, 0.07, p.trim, ("trim", CARVED), bevel=0.015)
            # Motorens akser: (x, z, −y); rammen vender mot +X: yaw π/2.
            gallery.append([[round(c[0], 3), round(c[2], 3), round(-c[1], 3)], round(math.pi / 2, 6)])
    light((-1.5, 0.0, E - 0.7), (1.0, 0.68, 0.4), 1.2, 7.0)
    light((GAL_X + 1.6, 0.0, F0 + 2.4), (1.0, 0.75, 0.5), 0.9, 5.0)
    return gallery


def gallery_piece():
    """Komponenten i rammen: en emaljert plate med en opphøyd «knapp» og en
    liten stein, lakken farget per komponent, og en tynn gullkant."""
    enamel = bpy.data.materials.new("layer_plaster_white.paint")
    enamel.use_nodes = True
    enamel.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 0.35
    gold = layer("plaster", (1.0, 0.82, 0.35), 0.35)
    s = 0.5
    # Bygd med forsida mot −Y i Blender (motorens +Z), sentrert i origo.
    box("plate", (-s / 2, -0.02, -s / 2), (s / 2, 0.0, s / 2), enamel, bevel=0.012)
    box("button", (-0.15, -0.06, -0.08), (0.15, -0.02, 0.08), enamel, bevel=0.03)
    box("gem", (-0.035, -0.08, 0.13), (0.035, -0.02, 0.2), gold, bevel=0.02)
    for a, b in (((-s / 2, -s / 2), (s / 2, -s / 2)), ((-s / 2, s / 2), (s / 2, s / 2)), ((-s / 2, -s / 2), (-s / 2, s / 2)), ((s / 2, -s / 2), (s / 2, s / 2))):
        stick("rim", (a[0], -0.025, a[1]), (b[0], -0.025, b[1]), 0.025, 0.025, gold, bevel=0)


def build():
    rng = random.Random(1350)
    real_write = town.write_inside
    town.write_inside = lambda path: None
    clear()
    town.PIECES.clear()
    town.LIT = True
    p = kit.Paint(plaster="plaster_white", timber="timber", roof="thatch")
    outside(p, rng)
    town.finish(f"{OUT}/home.glb", max_edge=100.0)
    clear()
    town.PIECES.clear()
    p = kit.Paint(plaster="plaster_white", timber="timber", roof="thatch")
    gallery = inside(p, rng)
    town.finish(f"{OUT}/home_interior.glb", max_edge=100.0)
    town.write_inside = real_write
    real_write(f"{OUT}/home.glb")
    path = f"{OUT}/home_inside.json"
    data = json.load(open(path))
    bx, by = BED
    data["bed"] = [[bx, round(F0 + 0.3, 3), round(-by + 0.0, 3)], [0.55, 0.35, 1.05]]
    data["gallery"] = gallery
    json.dump(data, open(path, "w"))
    # Platen i rammen.
    clear()
    town.PIECES.clear()
    town.LIT = False
    gallery_piece()
    town.finish(f"{OUT}/gallery_piece.glb", max_edge=100.0)
    town.write_inside(f"{OUT}/gallery_piece.glb")


if __name__ == "__main__":
    build()
