"""De vanlige husene i den malte stilen, bygd med byggesettet (kit.py).
Kjør:

    python3 tools/models/houses.py [house_a house_b ...]

Hvert hus er én modell (`assets/models/town/house_<x>.glb`) med vegger,
vinduer, dør, tak, gavler og pipe. Husslagene: bindingsverk med puss,
bordkledd, laftet, og steinhus med bindingsverk over."""

import math
import random
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import kit  # noqa: E402
import town  # noqa: E402
from common import clear  # noqa: E402
from kit import box, door, half_timbered, stick, window  # noqa: E402
from town import layer  # noqa: E402

OUT = "assets/models/town"
T = 0.3


def walls(p, kind, name, p0, p1, z0, z1, out, openings, rng):
    """En vegg av riktig slag: bindingsverk, bord, laft eller stein."""
    if kind == "timber":
        return half_timbered(p, name, p0, p1, z0, z1, out, openings, rng, collide_it=False)
    frame, L = kit.wall_frame(p0, p1)
    holes = [(s - w / 2, s + w / 2, za, zb) for s, za, zb, w in openings]
    mat = {"boards": p.boards, "cladding": p.cladding, "logs": layer("logs"), "stone": layer("brick")}[kind]
    kit.slab_grid(name, frame, L, z0, z1, T + (0.12 if kind in ("logs", "stone") else 0.0), holes, mat)
    # Hjørnestolper (bord) eller laftehoder (laft) og en list øverst.
    face = out * (T / 2 + 0.05)
    if kind in ("boards", "cladding"):
        for s in (0.1, L - 0.1):
            stick("corner", kit.at_wall(frame, s, face, z0), kit.at_wall(frame, s, face, z1), 0.22, 0.22, p.timber, ("timber", rng.randrange(4)), seed=rng.random() * 9)
        stick("eave_board", kit.at_wall(frame, -0.1, face, z1 - 0.12), kit.at_wall(frame, L + 0.1, face, z1 - 0.12), 0.1, 0.24, p.trim, ("trim", kit.DARK_BEAM))
    if kind == "logs":
        z = z0 + 0.15
        while z < z1:
            for s in (-0.25, L + 0.25):
                stick("log_end", kit.at_wall(frame, s - 0.2, 0, z), kit.at_wall(frame, s + 0.2, 0, z), 0.34, 0.3, layer("logs"), bevel=0.08)
            z += 0.32
    for s, za, zb, w in openings:
        if za > z0 + 0.3:
            stick("lintel", kit.at_wall(frame, s - w / 2 - 0.15, face, zb + 0.1), kit.at_wall(frame, s + w / 2 + 0.15, face, zb + 0.1), 0.16, 0.2, p.trim, ("trim", kit.DARK_BEAM))
    return frame, L


def house(spec):
    rng = random.Random(spec["seed"])
    p = kit.Paint(plaster=spec.get("plaster", "plaster"), timber=spec.get("timber", "timber"), roof=spec.get("roof", "thatch"))
    w, d = spec["w"], spec["d"]
    floors = spec.get("floors", 2)
    jetty = spec.get("jetty", 0.4)
    fh = spec.get("floor_h", 2.9)
    kinds = spec.get("kinds", ["timber"] * floors)
    f0 = 0.45
    x0, x1 = -w / 2 + T / 2, w / 2 - T / 2
    y0, y1 = -d / 2 + T / 2, d / 2 - T / 2
    # Sokkelen.
    box("plinth", (-w / 2 - 0.12, -d / 2 - 0.12, -0.25), (w / 2 + 0.12, d / 2 + 0.12, f0), p.stone, bevel=0.06)
    dx = spec.get("door_x", 0.0)
    box("step", (dx - 0.7, -d / 2 - 0.6, -0.2), (dx + 0.7, -d / 2 + 0.05, f0 - 0.15), p.stone, bevel=0.05)
    z = f0
    yy0, yy1 = y0, y1
    for k in range(floors):
        z1 = z + fh
        win = spec.get("windows", [[-1.5, 1.5]] * floors)[k]
        wz0, wz1 = z + 0.85, z + 2.05
        front = [(s - x0, wz0, wz1, 0.9) for s in win]
        if k == 0:
            front.append((dx - x0, z, z + 2.15, 1.1))
        kind = kinds[k]
        fr, _ = walls(p, kind, f"front{k}", (x0, yy0), (x1, yy0), z, z1, -1, front, rng)
        bk, _ = walls(p, kind, f"back{k}", (x0, yy1), (x1, yy1), z, z1, 1, [(0.0 - x0, wz0, wz1, 0.9)], rng)
        side = [(0.0 - yy0, wz0, wz1, 0.8)] if d > 7 else []
        lf, _ = walls(p, kind, f"left{k}", (x0, yy0), (x0, yy1), z, z1, 1, side, rng)
        rt, _ = walls(p, kind, f"right{k}", (x1, yy0), (x1, yy1), z, z1, -1, side, rng)
        th = T + (0.12 if kind in ("logs", "stone") else 0.0)
        for frame, opens, out in ((fr, front, -1), (bk, [(0.0 - x0, wz0, wz1, 0.9)], 1), (lf, side, 1), (rt, side, -1)):
            for s, za, zb, ww in opens:
                if za > z + 0.3:
                    window(p, frame, s, (za + zb) / 2, ww, zb - za, out, thick=th, rng=rng, shutters=rng.random() < 0.7, lit=False)
        if k == 0:
            door(p, fr, dx - x0, z, 1.1, 2.15, -1, thick=th, open_in=False, dark=rng.random() < 0.5)
        z = z1
        # Utkraging: neste etasje stikker ut foran og bak, med bjelkeender.
        if k < floors - 1 and jetty > 0:
            for ya, yb in ((yy0 - jetty - 0.05, yy0 + 0.2), (yy1 - 0.2, yy1 + jetty + 0.05)):
                x = x0 + 0.3
                while x < x1:
                    stick("joist_end", (x, ya, z - 0.15), (x, yb, z - 0.15), 0.22, 0.3, p.timber, ("timber", rng.randrange(4)), seed=x, bevel=0.0)
                    x += 0.8
            yy0, yy1 = yy0 - jetty, yy1 + jetty
            z += 0.3
    # Taket og gavlene.
    pitch = spec.get("pitch", 52)
    ridge = kit.gable_roof(p, x0 - T / 2, x1 + T / 2, yy0 - T / 2, yy1 + T / 2, z + 0.28, pitch)
    gable_kind = kinds[-1]
    for x, out in ((x0, -1), (x1, 1)):
        if gable_kind == "timber":
            kit.gable_wall(p, x, yy0 - T / 2, yy1 + T / 2, z, ridge, pitch, out, rng)
        else:
            gp = kit.Paint(plaster={"logs": "logs", "stone": "weatherboard", "boards": "weatherboard", "cladding": "cladding"}[gable_kind], timber=spec.get("timber", "timber"), roof=spec.get("roof", "thatch"))
            kit.gable_wall(gp, x, yy0 - T / 2, yy1 + T / 2, z, ridge, pitch, out, rng)
    # Pipa.
    cx = spec.get("chimney_x", x1 - 1.2)
    cy = spec.get("chimney_y", 1.0)
    roof_z = z + 0.28 + (yy1 + T / 2 + 0.9 - abs(cy)) * math.tan(math.radians(pitch)) - 1.2
    kit.chimney(p, cx, cy, roof_z - 1.0, roof_z + 1.6, w=0.95, smoke=0.6, lean=rng.uniform(-6, 6))
    # Vegglykta ved døra.
    town.wall_lantern(p, dx + 0.95, -d / 2 + T / 2, f0 + 2.2, -1)


HOUSES = {
    "house_a": dict(seed=1, w=6.4, d=8.0, floors=2, roof="thatch", plaster="plaster", windows=[[1.4], [-1.4, 1.4]], door_x=-1.3, pitch=55),
    "house_b": dict(seed=2, w=9.0, d=6.8, floors=2, roof="slate", plaster="plaster_white", windows=[[0.6, 2.8], [-2.8, 0.0, 2.8]], door_x=-2.3, pitch=50),
    "house_c": dict(seed=3, w=7.0, d=7.2, floors=1, roof="sod", kinds=["logs"], windows=[[-1.6]], door_x=1.4, pitch=40, jetty=0),
    "house_d": dict(seed=4, w=6.4, d=8.6, floors=3, roof="tiles", plaster="plaster_red", kinds=["stone", "timber", "timber"], windows=[[1.4], [-1.4, 1.4], [-1.4, 1.4]], door_x=-1.3, pitch=56, jetty=0.3),
    "house_e": dict(seed=5, w=9.6, d=7.0, floors=2, roof="shingles", kinds=["boards", "boards"], windows=[[-3.0, 2.6], [-3.0, 0.0, 3.0]], door_x=0.0, pitch=48, jetty=0),
    "house_f": dict(seed=6, w=7.2, d=8.2, floors=2, roof="thatch", plaster="plaster_old", timber="timber_grey", windows=[[-1.6], [-1.6, 1.6]], door_x=1.6, pitch=54),
}

if __name__ == "__main__":
    names = [a for a in sys.argv[1:] if a in HOUSES] or list(HOUSES)
    for name in names:
        clear()
        town.PIECES.clear()
        town.LIT = False
        house(HOUSES[name])
        town.finish(f"{OUT}/{name}.glb", max_edge=100.0)
