"""Punktene sprekkesjekken i motoren ser fra (`snapshot --leak-scan`): et
rutenett over navigasjonsnettet i et nivå, i øyehøyde over gulvet.

    python3 tools/crypt/leak_points.py N [avstand] > punkter.txt
"""
import json
import math
import os
import sys

from shapely.geometry import Point, Polygon

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EYE = 1.6


def points(n, step=2.0):
    d = json.load(open(os.path.join(ROOT, "assets", "models", "crypt", f"crypt_{n:02d}.json")))
    verts = d["nav"]["verts"]
    out = []
    for poly in d["nav"]["polys"]:
        vs = [verts[i] for i in poly]
        p = Polygon([(v[0], v[2]) for v in vs])
        y = sum(v[1] for v in vs) / len(vs)
        c = p.representative_point()
        got = [(c.x, c.y)]
        x0, z0, x1, z1 = p.bounds
        for i in range(int((x1 - x0) / step) + 1):
            for j in range(int((z1 - z0) / step) + 1):
                q = Point(x0 + step / 2 + i * step, z0 + step / 2 + j * step)
                if p.buffer(-0.3).contains(q):
                    got.append((q.x, q.y))
        out += [(x, y + EYE, z) for x, z in got]
    return out


if __name__ == "__main__":
    n = int(sys.argv[1])
    step = float(sys.argv[2]) if len(sys.argv) > 2 else 2.0
    for x, y, z in points(n, step):
        print(f"{x:.2f} {y:.2f} {z:.2f}")
