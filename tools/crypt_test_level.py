"""Testnivået for krypten: et lite nivå 1 i nivåformatet (docs/krypten/format.md).

Motoren bruker det til det ekte nivå 1 fra Blender finnes
(`assets/models/crypt/crypt_01.json`; build.rs velger). Det har ingen egne
glb-biter: rommene er rektangler med kollidere av bokser (gulv, murer med
døråpninger, tak, en rampe), navigasjonsnettet er firkanter 0,4 m inn fra
murene, og merkene har alt motoren kan: start, opp, hvilestedet, en spak og et
gitter, en snarveidør med slå, en falsk vegg med sprekk og trekk, en fallem ned
til et hemmelig gravkammer med lapp, varig styrke og stige opp, en trykkplate
med piler, kister og fiender som patruljerer og ligger i bakhold.

    python3 tools/crypt_test_level.py      # skriver assets/crypt_test/crypt_01.json
"""

import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "crypt_test" / "crypt_01.json"
PI = math.pi

# Rommene: id, navn, x0, x1, z0, z1, gulv, takhøyde (rektangler).
ROOMS = [
    ("kapell", "Gravkapellet", -5, 5, -4, 4, 0.0, 4.5),
    ("gang", "Gravgangen", -1.5, 1.5, 4, 16, -1.0, 4.5),
    ("krysset", "Krysset", -6, 6, 16, 28, -1.0, 5.0),
    ("sidegrav", "Sidegraven", -14, -6, 18, 26, -1.0, 3.5),
    ("dypgrav", "Den glemte graven", -14, -6, 18, 26, -7.0, 3.5, True),
    ("ostgang", "Østgangen", 6, 10, 20.5, 23.5, -1.0, 3.5),
    ("beinhus", "Beinhuset", 10, 20, 18, 26, -1.0, 6.0),
    ("soylegang", "Søylegangen", -2, 2, 28, 40, -1.0, 4.0),
    ("nedgang", "Nedgangen", -5, 5, 40, 48, -1.0, 5.0),
    ("skatt", "Skattkammeret", -10, -5, 42, 46, -1.0, 3.0, True),
    ("snarvei_b", "Snarveien", 10, 13, 26, 41, -1.0, 3.5),
    ("snarvei_a", "Snarveien", 5, 13, 41, 44, -1.0, 3.5),
]

# Døråpningene: på linja x = v eller z = v, fra lo til hi.
GAPS = [
    ("z", 4, -1.5, 1.5),
    ("z", 16, -1.5, 1.5),
    ("x", -6, 20.5, 23.5),
    ("x", 6, 20.5, 23.5),
    ("x", 10, 20.5, 23.5),
    ("z", 26, 10, 13),
    ("z", 28, -1.8, 1.8),
    ("z", 40, -2, 2),
    ("x", -5, 43, 45),
    ("x", 5, 41.7, 43.3),
    ("z", 41, 10, 13),
]

# Hullet i gulvet i sidegraven (fallemmen).
HOLE = (-12, -10, 21, 23)

# Navigasjonsnettet: (rom, port, x0, x1, z0, z1, y ved z0, y ved z1).
NAV = [
    ("kapell", None, -4.6, 4.6, -2.4, 3.6, 0, 0),
    ("gang", None, -1.1, 1.1, 3.6, 10, 0, 0),
    ("gang", None, -1.1, 1.1, 10, 16, 0, -1),
    ("gang", None, -1.1, 1.1, 16, 16.4, -1, -1),
    ("krysset", None, -5.6, 5.6, 16.4, 27.6, -1, -1),
    ("sidegrav", None, -6.4, -5.6, 20.9, 23.1, -1, -1),
    ("sidegrav", None, -9.6, -6.4, 18.4, 25.6, -1, -1),
    ("sidegrav", None, -12.4, -9.6, 18.4, 20.6, -1, -1),
    ("sidegrav", None, -12.4, -9.6, 23.4, 25.6, -1, -1),
    ("sidegrav", None, -13.6, -12.4, 18.4, 25.6, -1, -1),
    ("dypgrav", None, -13.6, -6.4, 18.4, 25.6, -7, -7),
    ("ostgang", None, 5.6, 10.4, 20.9, 23.1, -1, -1),
    ("beinhus", None, 10.4, 19.6, 18.4, 25.6, -1, -1),
    ("snarvei_b", None, 10.4, 12.6, 25.6, 41.4, -1, -1),
    ("snarvei_a", None, 5.4, 12.6, 41.4, 43.6, -1, -1),
    ("nedgang", "snarvei_1", 4.6, 5.4, 41.7, 43.3, -1, -1),
    ("soylegang", "gitter_1", -1.6, 1.6, 27.6, 28.4, -1, -1),
    ("soylegang", None, -1.6, 1.6, 28.4, 40.4, -1, -1),
    ("nedgang", None, -4.6, 4.6, 40.4, 47.6, -1, -1),
    ("skatt", "hemmelig_1", -5.4, -4.6, 43, 45, -1, -1),
    ("skatt", "hemmelig_1", -9.6, -5.4, 42.4, 45.6, -1, -1),
]

T = 0.4  # murtykkelse


def box(cx, cy, cz, sx, sy, sz):
    return {"box": {"center": [round(cx, 3), round(cy, 3), round(cz, 3)], "size": [round(sx, 3), round(sy, 3), round(sz, 3)], "yaw": 0.0}}


def wall_segments(axis, v, lo, hi):
    """Biter av muren på linja (axis = v) fra lo til hi, minus døråpningene."""
    cuts = sorted((a, b) for (ax, w, a, b) in GAPS if ax == axis and abs(w - v) < 1e-6 and b > lo and a < hi)
    out, at = [], lo
    for a, b in cuts:
        if a > at:
            out.append((at, a))
        at = max(at, b)
    if at < hi:
        out.append((at, hi))
    return out


def colliders():
    out = []
    for r in ROOMS:
        rid, _, x0, x1, z0, z1, y, h = r[:8]
        # Gulvet (med hull for fallemmen i sidegraven), og rampa i gangen.
        if rid == "sidegrav":
            hx0, hx1, hz0, hz1 = HOLE
            for (a, b, c, d) in [(x0, hx0, z0, z1), (hx1, x1, z0, z1), (hx0, hx1, z0, hz0), (hx0, hx1, hz1, z1)]:
                out.append(box((a + b) / 2, y - 0.25, (c + d) / 2, b - a, 0.5, d - c))
        elif rid == "gang":
            out.append(box(0, -0.25, 7, 3, 0.5, 6))
            out.append({"hull": [[-1.5, 0, 10], [1.5, 0, 10], [-1.5, -1, 16], [1.5, -1, 16], [-1.5, -1.6, 10], [1.5, -1.6, 10], [-1.5, -1.6, 16], [1.5, -1.6, 16]]})
        else:
            out.append(box((x0 + x1) / 2, y - 0.25, (z0 + z1) / 2, x1 - x0, 0.5, z1 - z0))
        # Taket (ikke over den glemte graven: der faller du ned).
        if rid != "dypgrav":
            out.append(box((x0 + x1) / 2, y + h + 0.25, (z0 + z1) / 2, x1 - x0 + T, 0.5, z1 - z0 + T))
        top = y + h + 0.5 if rid != "dypgrav" else -1.0
        ylo = y - 0.5
        for (axis, v, lo, hi) in [("z", z0, x0, x1), ("z", z1, x0, x1), ("x", x0, z0, z1), ("x", x1, z0, z1)]:
            for a, b in wall_segments(axis, v, lo - T / 2, hi + T / 2):
                if axis == "z":
                    out.append(box((a + b) / 2, (ylo + top) / 2, v, b - a, top - ylo, T))
                else:
                    out.append(box(v, (ylo + top) / 2, (a + b) / 2, T, top - ylo, b - a))
    return out


def nav():
    verts, index, polys, rooms, gates = [], {}, [], [], []

    def v(x, y, z):
        k = (round(x, 3), round(y, 3), round(z, 3))
        if k not in index:
            index[k] = len(verts)
            verts.append(list(k))
        return index[k]

    for room, gate, x0, x1, z0, z1, ya, yb in NAV:
        # Mot klokka sett ovenfra med x øst og z sør: (x0,z0) (x0,z1) (x1,z1) (x1,z0).
        polys.append([v(x0, ya, z0), v(x0, yb, z1), v(x1, yb, z1), v(x1, ya, z0)])
        rooms.append(room)
        gates.append(gate)
    return {"verts": verts, "polys": polys, "poly_room": rooms, "poly_gate": gates}


def lights():
    warm = [1.0, 0.62, 0.3]
    return [
        {"pos": [3.5, 1.4, -3.1], "color": warm, "range": 7, "flicker": True, "room": "kapell"},
        {"pos": [-4.2, 3.0, 0], "color": warm, "range": 8, "flicker": True, "room": "kapell"},
        {"pos": [1.1, 2.4, 9], "color": warm, "range": 7, "flicker": True, "room": "gang"},
        {"pos": [-5.6, 3.0, 22], "color": warm, "range": 10, "flicker": True, "room": "krysset"},
        {"pos": [5.6, 3.0, 18], "color": warm, "range": 9, "flicker": True, "room": "krysset"},
        {"pos": [-13.6, 1.6, 22], "color": [0.8, 0.7, 0.6], "range": 7, "flicker": True, "room": "sidegrav"},
        {"pos": [-10, -4.5, 22], "color": [0.5, 0.6, 0.9], "range": 7, "flicker": False, "room": "dypgrav"},
        {"pos": [15, 3.5, 25.6], "color": warm, "range": 10, "flicker": True, "room": "beinhus"},
        {"pos": [19.6, 2.0, 21], "color": warm, "range": 6, "flicker": True, "room": "beinhus"},
        {"pos": [0, 2.4, 34], "color": warm, "range": 8, "flicker": True, "room": "soylegang"},
        {"pos": [0, 1.2, 46], "color": [0.35, 0.55, 1.0], "range": 9, "flicker": False, "intensity": 3.0, "room": "nedgang"},
        {"pos": [4.6, 3.0, 47.6], "color": warm, "range": 8, "flicker": True, "room": "nedgang"},
        {"pos": [-8.5, 1.2, 45.6], "color": [1.0, 0.8, 0.45], "range": 5, "flicker": True, "room": "skatt"},
        {"pos": [12.6, 2.2, 34], "color": warm, "range": 7, "flicker": True, "room": "snarvei_b"},
    ]


def marks():
    return [
        {"kind": "start", "pos": [0, 0, 1.5], "yaw": 0.0},
        {"kind": "up", "pos": [0, 0, -3.6], "yaw": 0.0},
        {"kind": "rest", "pos": [3.5, 0, -3.1], "yaw": 0.0},
        {"kind": "trap", "label": "arrows", "id": "piler_1", "pos": [0, 0, 7], "size": [2.2, 1.2], "dir": [1, 0, 0]},
        {"kind": "mob", "pos": [0, -1, 14], "label": "button", "n": 1, "patrol": [[0, -1, 14], [0, -1, 24]]},
        {"kind": "mob", "pos": [3, -1, 22], "label": "tooltip", "n": 1},
        {"kind": "chest", "pos": [-13, -1, 19], "yaw": 0.0},
        {"kind": "trapdoor", "id": "fallem_1", "pos": [-11, -1, 22], "size": [2, 2], "to": "dypgrav"},
        {"kind": "hint", "label": "draft", "pos": [-11, -1, 22], "for": "fallem_1"},
        {"kind": "lore", "pos": [-8, -7, 19], "yaw": 0.0, "label": "Gravskriveren", "text": "Den som går ned, skal finne veien opp gjennom lemmen i sidegraven."},
        {"kind": "bonus", "pos": [-8, -7, 24.5], "yaw": PI, "label": "liv"},
        {"kind": "ladder", "pos": [-12, -7, 25.5], "yaw": PI, "top": [-12, -1, 24.8]},
        {"kind": "mob", "pos": [15, -1, 22], "label": "tooltip", "n": 2, "patrol": [[12, -1, 20], [18, -1, 24]]},
        {"kind": "lever", "id": "spak_1", "pos": [19.5, -1, 22], "yaw": -PI / 2, "opens": ["gitter_1"]},
        {"kind": "gate", "id": "gitter_1", "pos": [0, -1, 28], "yaw": 0.0, "width": 3.6},
        {"kind": "mob", "pos": [0, -1, 34], "label": "button", "n": 2, "patrol": [[0, -1, 30], [0, -1, 38]]},
        {"kind": "mob", "pos": [3, -1, 45], "label": "knapp i bakhold", "n": 2, "ambush": 4.0},
        {"kind": "chest", "pos": [-3.5, -1, 47.3], "yaw": PI},
        {"kind": "secret_wall", "id": "hemmelig_1", "pos": [-5, -1, 44], "yaw": PI / 2, "width": 2.0, "height": 2.6},
        {"kind": "hint", "label": "crack", "pos": [-4.75, -0.2, 45.4], "yaw": PI / 2, "for": "hemmelig_1"},
        {"kind": "rare", "pos": [-8.5, -1, 44], "yaw": PI / 2, "label": "Badge"},
        {"kind": "shortcut", "id": "snarvei_1", "pos": [5, -1, 42.5], "yaw": -PI / 2, "width": 1.6, "from": "nedgang"},
        {"kind": "down", "pos": [0, -1, 46], "yaw": PI},
    ]


def main():
    rooms = []
    for r in ROOMS:
        rid, name, x0, x1, z0, z1, y, h = r[:8]
        secret = len(r) > 8 and r[8]
        rooms.append({"id": rid, "name": name, "poly": [[x0, z0], [x1, z0], [x1, z1], [x0, z1]], "y": y, "h": h, "secret": secret})
    level = {
        "level": 1,
        "name": "Gravkjelleren under kirka (testnivå)",
        "theme": "grav",
        "pieces": [],
        "rooms": rooms,
        "colliders": colliders(),
        "nav": nav(),
        "lights": lights(),
        "marks": marks(),
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(level, ensure_ascii=False, indent=1) + "\n")
    print(OUT)


if __name__ == "__main__":
    main()
