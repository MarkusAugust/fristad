"""Gatemøblene i Fristad: lykt, benk, sykkel og søppelbøtte, modellert i
Blender (bpy) og eksportert til assets/models/street/. Kjør:

    python3 tools/models/street.py [lamp|bench|bicycle|bin ...]

Materialet «paint» får fargen per instans i motoren, «glow» lyser om natta
(blir `kind::LAMP` i lasteren). Fronten/lengden ligger langs X.
"""

import math
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import box, clear, cylinder, ellipsoid, export, material, rounded_box, smooth, spin, tube  # noqa: E402

OUT = "assets/models/street"


def mats():
    return {
        "iron": material("iron", (0.045, 0.045, 0.05), 0.55),
        "paint": material("paint", (1.0, 1.0, 1.0), 0.35),
        "glow": material("glow", (1.0, 0.85, 0.6), 0.2),
        "oak": material("oak", (0.42, 0.27, 0.14), 0.6),
        "chrome": material("chrome", (0.85, 0.85, 0.88), 0.3, 1.0),
        "rubber": material("rubber", (0.03, 0.03, 0.03), 0.9),
        "leather": material("leather", (0.12, 0.08, 0.06), 0.6),
        "steel": material("steel", (0.35, 0.36, 0.38), 0.45, 1.0),
    }


def lamp(m):
    """Støpejernslykt, 3,9 m: sokkel og rillet stolpe dreid om Z, en bøyle
    med en sekskantet lyktekasse med glass, tak og knopp. Lyset i motoren
    settes nær toppen."""
    iron = m["iron"]
    profile = [(0.0, 0.0), (0.2, 0.0), (0.2, 0.05), (0.16, 0.08), (0.15, 0.3), (0.12, 0.34), (0.1, 0.38), (0.11, 0.42), (0.075, 0.5), (0.065, 1.0), (0.06, 2.4), (0.07, 2.5), (0.085, 2.55), (0.075, 2.6), (0.06, 2.62), (0.055, 3.05), (0.08, 3.08), (0.08, 3.14), (0.0, 3.16)]
    spin("post", profile, iron, 20)
    # Rillene på stolpen: tolv smale lister.
    for k in range(12):
        a = k * 2 * math.pi / 12
        box(f"flute{k}", (math.cos(a) * 0.065, math.sin(a) * 0.065, 1.5), (0.012, 0.012, 1.8), iron, rot=(0, 0, a))
    # Bøylen ut mot gata (−X) med lyktekassa.
    top = 3.12
    tube("arm", (0.0, 0.0, top), (-0.55, 0.0, top + 0.08), 0.03, iron)
    tube("brace", (0.0, 0.0, top - 0.35), (-0.4, 0.0, top + 0.02), 0.018, iron)
    ellipsoid("knob", (-0.57, 0.0, top + 0.1), (0.08, 0.08, 0.08), iron, 12, 8)
    hx, hz = -0.52, top - 0.32   # lyktekassas sentrum
    cylinder("glass", (hx, 0.0, hz), 0.2, 0.5, m["glow"], vertices=6)
    # Rammene: seks hjørnestolper, over- og underring.
    for k in range(6):
        a = k * 2 * math.pi / 6 + math.pi / 6
        box(f"corner{k}", (hx + math.cos(a) * 0.2, math.sin(a) * 0.2, hz), (0.02, 0.02, 0.52), iron, rot=(0, 0, a))
    cylinder("ring_top", (hx, 0.0, hz + 0.26), 0.22, 0.03, iron, vertices=6)
    cylinder("ring_bottom", (hx, 0.0, hz - 0.26), 0.22, 0.04, iron, vertices=6)
    bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.27, radius2=0.04, depth=0.26, location=(hx, 0.0, hz + 0.4))
    roof = bpy.context.active_object
    roof.name = "roof"
    roof.data.materials.append(iron)
    ellipsoid("finial", (hx, 0.0, hz + 0.56), (0.07, 0.07, 0.09), iron, 12, 8)
    tube("hanger", (hx, 0.0, hz + 0.5), (hx, 0.0, top + 0.08), 0.015, iron)


def bench(m):
    """Parkbenk, 1,9 m: støpte ender med armlener, fem seteplanker og tre
    ryggplanker i eik."""
    iron, oak = m["iron"], m["oak"]
    for sx in (-0.85, 0.85):
        # Forbein, bakbein og armlene som en buet stiv ramme.
        tube(f"leg_f{sx:+.0f}", (sx, 0.26, 0.0), (sx, 0.24, 0.42), 0.025, iron)
        tube(f"leg_b{sx:+.0f}", (sx, -0.24, 0.0), (sx, -0.3, 0.42), 0.025, iron)
        tube(f"seat_bar{sx:+.0f}", (sx, 0.28, 0.42), (sx, -0.32, 0.42), 0.025, iron)
        tube(f"back_bar{sx:+.0f}", (sx, -0.3, 0.42), (sx, -0.42, 0.95), 0.025, iron)
        tube(f"arm_post{sx:+.0f}", (sx, 0.22, 0.42), (sx, 0.2, 0.66), 0.022, iron)
        tube(f"arm{sx:+.0f}", (sx, 0.26, 0.66), (sx, -0.36, 0.72), 0.022, iron)
        ellipsoid(f"arm_knob{sx:+.0f}", (sx, 0.27, 0.66), (0.06, 0.06, 0.06), iron, 10, 6)
        box(f"foot_f{sx:+.0f}", (sx, 0.26, 0.015), (0.12, 0.1, 0.03), iron)
        box(f"foot_b{sx:+.0f}", (sx, -0.24, 0.015), (0.12, 0.1, 0.03), iron)
    for k in range(5):
        rounded_box(f"seat{k}", (0.0, 0.24 - k * 0.12, 0.45), (1.9, 0.1, 0.035), oak, 0.006)
    for k in range(3):
        z = 0.58 + k * 0.14
        y = -0.33 - (z - 0.42) * 0.22
        rounded_box(f"back{k}", (0.0, y, z), (1.9, 0.035, 0.11), oak, 0.006, rot=(-0.22, 0, 0))


def wheel(name, x, r, m, spokes=24):
    """Sykkelhjul i XZ-planet: dekk, felg, nav og eiker."""
    bpy.ops.mesh.primitive_torus_add(major_segments=36, minor_segments=10, major_radius=r - 0.016, minor_radius=0.016, location=(x, 0.0, r), rotation=(math.pi / 2, 0, 0))
    tyre = bpy.context.active_object
    tyre.name = name + "_tyre"
    tyre.data.materials.append(m["rubber"])
    smooth(tyre, 60)
    bpy.ops.mesh.primitive_torus_add(major_segments=36, minor_segments=6, major_radius=r - 0.03, minor_radius=0.008, location=(x, 0.0, r), rotation=(math.pi / 2, 0, 0))
    rim = bpy.context.active_object
    rim.name = name + "_rim"
    rim.data.materials.append(m["chrome"])
    smooth(rim, 60)
    cylinder(name + "_hub", (x, 0.0, r), 0.025, 0.08, m["chrome"], rot=(math.pi / 2, 0, 0), vertices=10)
    for k in range(spokes):
        a = k * 2 * math.pi / spokes
        side = 0.02 if k % 2 == 0 else -0.02
        tube(f"{name}_spoke{k}", (x, side, r), (x + math.cos(a) * (r - 0.03), 0.0, r + math.sin(a) * (r - 0.03)), 0.0015, m["chrome"], vertices=4)


def bicycle(m):
    """Klassisk sykkel, 1,75 m, hjul med 24 eiker, ramme i lakk."""
    r = 0.34
    rear, front = -0.52, 0.52
    wheel("rear", rear, r, m)
    wheel("front", front, r, m)
    paint = m["paint"]
    bb = (-0.08, 0.0, 0.3)          # krankboksen
    seat_top = (-0.2, 0.0, 0.92)
    head_top = (0.36, 0.0, 0.95)
    head_bottom = (0.44, 0.0, 0.62)
    tube("seat_tube", bb, seat_top, 0.017, paint)
    tube("down_tube", bb, head_bottom, 0.019, paint)
    tube("top_tube", (-0.17, 0.0, 0.86), (0.37, 0.0, 0.9), 0.016, paint)
    tube("head_tube", head_bottom, head_top, 0.02, paint)
    for side in (0.04, -0.04):
        tube(f"chainstay{side:+.2f}", (bb[0], side, bb[2] + 0.01), (rear, side * 0.6, r), 0.01, paint)
        tube(f"seatstay{side:+.2f}", (-0.19, side * 0.5, 0.86), (rear, side * 0.6, r), 0.009, paint)
        tube(f"fork{side:+.2f}", (head_bottom[0], side, head_bottom[2]), (front, side * 0.6, r), 0.01, paint)
    cylinder("crank_box", bb, 0.035, 0.1, paint, rot=(math.pi / 2, 0, 0), vertices=12)
    for side, a in ((0.06, 0.0), (-0.06, math.pi)):
        tube(f"crank{side:+.2f}", (bb[0], side, bb[2]), (bb[0] + math.cos(a) * 0.17, side, bb[2] + math.sin(a) * 0.17), 0.009, m["steel"])
        box(f"pedal{side:+.2f}", (bb[0] + math.cos(a) * 0.17, side + math.copysign(0.05, side), bb[2] + math.sin(a) * 0.17), (0.09, 0.06, 0.02), m["rubber"])
    cylinder("chainring", (bb[0], 0.045, bb[2]), 0.095, 0.004, m["steel"], rot=(math.pi / 2, 0, 0), vertices=24)
    cylinder("sprocket", (rear, 0.04, r), 0.035, 0.004, m["steel"], rot=(math.pi / 2, 0, 0), vertices=16)
    tube("chain_top", (bb[0], 0.045, bb[2] + 0.09), (rear, 0.04, r + 0.035), 0.004, m["steel"], vertices=6)
    tube("chain_bottom", (bb[0], 0.045, bb[2] - 0.09), (rear, 0.04, r - 0.035), 0.004, m["steel"], vertices=6)
    tube("seat_post", seat_top, (-0.23, 0.0, 1.02), 0.013, m["chrome"])
    ellipsoid("saddle", (-0.25, 0.0, 1.05), (0.27, 0.15, 0.06), m["leather"], 16, 8)
    tube("stem", head_top, (0.4, 0.0, 1.03), 0.013, m["chrome"])
    tube("handlebar", (0.42, -0.27, 1.03), (0.42, 0.27, 1.03), 0.012, m["chrome"])
    for side in (-0.27, 0.27):
        tube(f"grip{side:+.2f}", (0.42, side, 1.03), (0.3, side * 1.08, 1.0), 0.016, m["rubber"])
    # Bagasjebrett og skjermer.
    for side in (-0.05, 0.05):
        tube(f"rack_stay{side:+.2f}", (rear - 0.05, side, r + 0.02), (rear - 0.2, side, 0.8), 0.006, m["steel"])
    box("rack", (rear - 0.05, 0.0, 0.8), (0.36, 0.14, 0.015), m["steel"])
    for name, x in (("fender_r", rear), ("fender_f", front)):
        bpy.ops.mesh.primitive_torus_add(major_segments=36, minor_segments=8, major_radius=r + 0.012, minor_radius=0.03, location=(x, 0.0, r), rotation=(math.pi / 2, 0, 0))
        fe = bpy.context.active_object
        fe.name = name
        fe.scale = (1.0, 1.0, 0.33)
        fe.data.materials.append(paint)
        # Bare den øvre halvdelen av skjermen beholdes.
        import bmesh
        bm = bmesh.new()
        bm.from_mesh(fe.data)
        bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < 0.02 - (0.0 if x == rear else 0.0)], context="VERTS")
        bm.to_mesh(fe.data)
        bm.free()
        smooth(fe, 60)


def bin_(m):
    """Søppelbøtte, 1 m: stolpe, sylinder med spiler, lokk."""
    iron, paint = m["iron"], m["paint"]
    cylinder("post", (0.0, 0.0, 0.5), 0.03, 1.0, iron, vertices=10)
    cylinder("body", (0.0, 0.0, 0.62), 0.2, 0.55, paint, vertices=24)
    for k in range(16):
        a = k * 2 * math.pi / 16
        box(f"slat{k}", (math.cos(a) * 0.205, math.sin(a) * 0.205, 0.62), (0.015, 0.045, 0.5), paint, rot=(0, 0, a))
    cylinder("rim_top", (0.0, 0.0, 0.9), 0.215, 0.03, iron, vertices=24)
    cylinder("rim_bottom", (0.0, 0.0, 0.345), 0.215, 0.03, iron, vertices=24)
    bpy.ops.mesh.primitive_cone_add(vertices=24, radius1=0.24, radius2=0.1, depth=0.1, location=(0.0, 0.0, 0.96))
    lid = bpy.context.active_object
    lid.name = "lid"
    lid.data.materials.append(iron)
    smooth(lid, 45)


BUILDERS = {"lamp": lamp, "bench": bench, "bicycle": bicycle, "bin": bin_}

if __name__ == "__main__":
    kinds = [a for a in sys.argv[1:] if a in BUILDERS] or list(BUILDERS)
    import os
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        BUILDERS[k](mats())
        export(f"{OUT}/{k}.glb")
