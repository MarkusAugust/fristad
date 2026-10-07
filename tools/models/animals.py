"""Dyrene rundt Fristad: sau og ku, i deler som motoren poserer hver for
seg (kropp med hode og hale, og fire bein), så de kan gå. Eksporteres til
assets/models/animals/. Kjør:

    python3 tools/models/animals.py [sheep|cow ...]

Kroppen har origo i bakken under midten av dyret, fronten mot +X. Beina
har origo i hofte-/skulderleddet og henger nedover; beinas feste på
kroppen er LEGS i meter (x fram, y til venstre, z opp), speilet i
legs.rs ... nei, i game.rs (CREATURES)."""

import math
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
from common import box, clear, cylinder, ellipsoid, export, material, rounded_box, smooth, tube  # noqa: E402

OUT = "assets/models/animals"

# Beinfestene: (x, y, z) for sau og ku, samme rekkefølge i motoren.
SHEEP_LEGS = [(0.28, 0.14, 0.42), (0.28, -0.14, 0.42), (-0.3, 0.14, 0.42), (-0.3, -0.14, 0.42)]
COW_LEGS = [(0.65, 0.28, 0.95), (0.65, -0.28, 0.95), (-0.7, 0.28, 0.95), (-0.7, -0.28, 0.95)]


def mats():
    return {
        "wool": material("wool", (0.86, 0.84, 0.78), 0.95),
        "sheep_skin": material("sheep_skin", (0.12, 0.1, 0.09), 0.7),
        "cow_white": material("cow_white", (0.85, 0.83, 0.78), 0.75),
        "cow_black": material("cow_black", (0.06, 0.06, 0.06), 0.75),
        "pink": material("pink", (0.8, 0.55, 0.5), 0.7),
        "horn": material("horn", (0.75, 0.68, 0.55), 0.5),
        "eye": material("eye", (0.02, 0.02, 0.02), 0.2),
        "hoof": material("hoof", (0.1, 0.09, 0.08), 0.6),
    }


def sheep_body(m):
    """Ullkropp som en klump av ellipsoider, hode og hale."""
    ellipsoid("body", (0.0, 0.0, 0.62), (0.9, 0.52, 0.5), m["wool"], 16, 10)
    for k in range(10):
        a = k * 2 * math.pi / 10
        x = 0.3 * math.cos(a) * 1.2
        y = 0.2 * math.sin(a)
        ellipsoid(f"tuft{k}", (x, y, 0.62 + 0.14 * math.sin(a * 2)), (0.3, 0.26, 0.24), m["wool"], 10, 6)
    ellipsoid("neck", (0.45, 0.0, 0.7), (0.3, 0.22, 0.26), m["wool"], 10, 6)
    ellipsoid("head", (0.6, 0.0, 0.72), (0.3, 0.17, 0.2), m["sheep_skin"], 12, 8)
    ellipsoid("muzzle", (0.73, 0.0, 0.68), (0.14, 0.11, 0.1), m["sheep_skin"], 8, 6)
    for sy in (1, -1):
        ellipsoid(f"ear{sy}", (0.58, sy * 0.13, 0.76), (0.1, 0.14, 0.05), m["sheep_skin"], 8, 5)
        ellipsoid(f"eye{sy}", (0.7, sy * 0.08, 0.76), (0.04, 0.04, 0.04), m["eye"], 8, 5)
    ellipsoid("tail", (-0.48, 0.0, 0.66), (0.12, 0.1, 0.14), m["wool"], 8, 5)


def sheep_leg(m):
    cylinder("leg", (0.0, 0.0, -0.2), 0.04, 0.4, m["sheep_skin"], vertices=8)
    box("hoof", (0.0, 0.0, -0.42), (0.1, 0.08, 0.05), m["hoof"])


def cow_body(m):
    """Kuas kropp med flekker, hals, hode, horn, ører, jur og hale."""
    ellipsoid("body", (0.0, 0.0, 1.25), (1.9, 0.78, 0.9), m["cow_white"], 20, 12)
    for k, (x, y, z, s) in enumerate([(0.4, 0.3, 1.35, 0.45), (-0.5, -0.32, 1.3, 0.5), (0.1, -0.36, 1.1, 0.35), (-0.6, 0.3, 1.5, 0.4)]):
        ellipsoid(f"spot{k}", (x, y, z), (s * 1.3, s * 0.3, s), m["cow_black"], 10, 6)
    ellipsoid("neck", (0.95, 0.0, 1.35), (0.5, 0.42, 0.5), m["cow_white"], 12, 8)
    ellipsoid("head", (1.25, 0.0, 1.35), (0.55, 0.32, 0.38), m["cow_white"], 14, 10)
    ellipsoid("muzzle", (1.5, 0.0, 1.25), (0.22, 0.26, 0.2), m["pink"], 10, 6)
    for sy in (1, -1):
        ellipsoid(f"ear{sy}", (1.15, sy * 0.3, 1.5), (0.14, 0.22, 0.08), m["cow_white"], 8, 5)
        tube(f"horn{sy}", (1.2, sy * 0.14, 1.6), (1.22, sy * 0.3, 1.78), 0.03, m["horn"], vertices=8)
        ellipsoid(f"eye{sy}", (1.38, sy * 0.17, 1.42), (0.06, 0.06, 0.06), m["eye"], 8, 5)
    ellipsoid("udder", (-0.45, 0.0, 0.85), (0.4, 0.4, 0.3), m["pink"], 10, 6)
    tube("tail", (-0.9, 0.0, 1.5), (-1.0, 0.05, 0.95), 0.025, m["cow_white"], vertices=6)
    ellipsoid("tail_tuft", (-1.0, 0.05, 0.9), (0.08, 0.08, 0.14), m["cow_black"], 8, 5)


def cow_leg(m):
    cylinder("leg", (0.0, 0.0, -0.45), 0.08, 0.9, m["cow_white"], vertices=10)
    ellipsoid("knee", (0.0, 0.0, -0.45), (0.17, 0.17, 0.2), m["cow_white"], 8, 6)
    box("hoof", (0.0, 0.0, -0.93), (0.18, 0.16, 0.08), m["hoof"])


BUILDERS = {"sheep_body": sheep_body, "sheep_leg": sheep_leg, "cow_body": cow_body, "cow_leg": cow_leg}

if __name__ == "__main__":
    kinds = [a for a in sys.argv[1:] if a in BUILDERS] or list(BUILDERS)
    import os
    os.makedirs(OUT, exist_ok=True)
    for k in kinds:
        clear()
        BUILDERS[k](mats())
        export(f"{OUT}/{k}.glb")
