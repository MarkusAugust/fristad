"""Det grå pilotsettet for nivåredigeringen (docs/nivaaredigering.md, fase 1):
rundt tolv biter av dagens stein uten pynt, laget med funksjonene i
`kit_grav.py`, som kjernen setter sammen etter nivådokumentet
(`assets/levels/nivaa_*.json`, `domain/world/crypt/compose/`).

Bitene står på rutenettet på 4 × 4 m (`kit_common.py`):

| Bit | Slag | Hva |
|---|---|---|
| `grav_romfelt` | celle | gulv og krysshvelv, åpent til alle sider |
| `grav_gang` | celle | gulv og tønnehvelv langs z (gangen og blindgangen) |
| `grav_gangkryss` | celle | gulv og lavt krysshvelv (hjørne, T og kryss i gangen) |
| `grav_halvtrapp` | celle | trapp 1 m opp mot −z under et skrått tønnehvelv |
| `grav_trapp2` | celle | trapp 2 m opp mot −z (halv etasje) |
| `grav_nedgang` | celle | romfelt med hull for trappa ned (`kit_down`) |
| `grav_vegg` | halvkant | murhalvdelen |
| `grav_vegg_gang` | halvkant | overmuren over en gangåpning (gangprofilen) |
| `grav_vegg_dor` | halvkant | mur med rundbuet dør 1,6 × 2,3 |
| `grav_vegg_port` | halvkant | mur med portåpning 2,8 × 3,0 (`kit_gate`) |
| `grav_vegg_falsk` | halvkant | mur med åpning 2,0 × 2,6 til den falske veggen |
| `grav_gjordbue` | halvkant | halve gjordbuen mellom to romfelt i samme rom |
| `grav_hjorne` | kvarthjørne | muren i hjørnet |
| `grav_soyle` | søyle | frittstående søyle der fire romfelt møtes |

    python3.13 tools/crypt/kit_cells.py            # eksporterer alt og skriver manifestet
    python3.13 tools/crypt/kit_cells.py --json     # bare manifestet (uten Blender)
    python3.13 tools/crypt/kit_cells.py grav_vegg  # bare én bit
"""

import math
import os
import sys

from shapely.geometry import Polygon
from shapely.geometry import box as sbox

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import kit_grav as K  # noqa: E402
from kit_common import (  # noqa: E402
    DOOR_SPRING, DOOR_W, FACE, GANG_CLEAR, GANG_R, GANG_S, GATE_H, GATE_W, HALF, ROOM_CLEAR, ROOM_R, ROOM_S,
    SECRET_H, SECRET_W, WALL_FOOT, WALL_T, WALL_TOP, gang_h, room_h, vault_ts, write_manifest)

THEME = "grav"
VERSION = 1
OUT = os.path.join(ROOT, "assets", "models", "crypt", "kit", THEME)
MANIFEST = os.path.join(ROOT, "assets", "models", "crypt", "kit", f"{THEME}.json")
# Trappa ned (`kit_down`) står så langt fra cellemidten mot +z, med hullet
# bak seg (`kit_grav.kit_down`: 3,2 m trinn og 0,6 m sjakt bak).
DOWN_Z = 1.65
DOWN_HOLE = 1.2


# --- Flatene ---------------------------------------------------------------------


def floor(m, shape=None):
    """Gulvet over cella (eller `shape` i xz), i ruter på 1 m."""
    K.grid_face(m, K.Horizontal(0.0), shape if shape is not None else sbox(-HALF, -HALF, HALF, HALF), "floor", origin=(-HALF, -HALF))


def groin(m, S, R, mat="vault"):
    """Krysshvelvet over hele cella (som `kit_grav.groin_bay`, med prøvene i
    `vault_ts`, så buene langs kantene møter naboens)."""
    ts = vault_ts()
    a = b = HALF
    k = 3
    h = lambda t: S + R * math.sqrt(max(0.0, 1 - t * t))
    arc = K.ellipse_arclen(a, R, ts)
    for side in ("n", "s", "w", "e"):
        for i in range(len(ts) - 1):
            strips = []
            for t, ar in ((ts[i], arc[i]), (ts[i + 1], arc[i + 1])):
                y = h(t)
                if side in "ns":
                    x = a * t
                    zw = HALF if side == "s" else -HALF
                    zd = b * abs(t) * (1 if side == "s" else -1)
                    strips.append([((x, y, zw + (zd - zw) * q / k), (zw + (zd - zw) * q / k, ar)) for q in range(k + 1)])
                else:
                    z = b * t
                    xw = HALF if side == "e" else -HALF
                    xd = a * abs(t) * (1 if side == "e" else -1)
                    strips.append([((xw + (xd - xw) * q / k, y, z), (xw + (xd - xw) * q / k, ar)) for q in range(k + 1)])
            A, B = strips
            for q in range(k):
                pts = [A[q][0], B[q][0], B[q + 1][0], A[q + 1][0]]
                uvs = [A[q][1], B[q][1], B[q + 1][1], A[q + 1][1]]
                c = K.lerp(K.lerp(pts[0], pts[1], 0.5), K.lerp(pts[2], pts[3], 0.5), 0.5)
                facing = K.norm(K.sub((0.0, S - 2.0, 0.0), c))
                if q == k - 1 and math.dist(pts[2], pts[3]) < 1e-6:
                    m.poly(pts[:3], uvs[:3], mat, facing=facing, smooth=True)
                else:
                    m.poly(pts, uvs, mat, facing=facing, smooth=True)


def barrel(m, S0, S1, R, mat="vault"):
    """Tønnehvelvet langs z over hele cella, med vederlaget S0 ved +z og S1
    ved −z (skrått over trappene), i ruter på 1 m langs aksen."""
    ts = vault_ts()
    arc = K.ellipse_arclen(HALF, R, ts)
    zs = [HALF - k for k in range(5)]
    S = lambda z: S0 + (S1 - S0) * (HALF - z) / (2 * HALF)
    for i in range(len(ts) - 1):
        for q in range(4):
            pts, uvs = [], []
            for t, ar, z in ((ts[i], arc[i], zs[q]), (ts[i + 1], arc[i + 1], zs[q]), (ts[i + 1], arc[i + 1], zs[q + 1]), (ts[i], arc[i], zs[q + 1])):
                pts.append((HALF * t, S(z) + R * math.sqrt(max(0.0, 1 - t * t)), z))
                uvs.append((z, ar))
            c = K.lerp(pts[0], pts[2], 0.5)
            m.poly(pts, uvs, mat, facing=K.sub((0.0, S(c[2]) - 2.0, c[2]), c), smooth=True)


def stair(m, rise, n):
    """Trinnene fra +z (gulvet i 0) opp til −z (gulvet i `rise`), over hele
    bredden (sidene står inne i murene)."""
    t = 2 * HALF / n
    for k in range(n):
        z1 = HALF - k * t
        z0 = z1 - t
        y = rise * (k + 1) / n
        K.box(m, (-HALF, -0.5, z0), (HALF, y, z1), "dressed", faces=("+y", "+z"))


# --- Halvkantene -----------------------------------------------------------------


def opening(kind):
    """Omrisset av åpningen i halvkanten (u langs muren fra −1,6, v opp), og
    om sidene av omrisset står i endene av muren (gangen: der står hjørnene)."""
    foot = WALL_FOOT - 0.01
    if kind == "gang":
        ts = [t for t in vault_ts() if abs(t * HALF) <= FACE + 1e-9]
        return [(-FACE, foot)] + [(t * HALF, gang_h(t * HALF)) for t in ts] + [(FACE, foot)], (0.0, GANG_S)
    if kind == "door":
        return K.arch_profile(0.0, DOOR_W, foot, DOOR_SPRING - foot, n=12), (0.0, DOOR_SPRING)
    if kind == "gate":
        return K.rect_profile(0.0, GATE_W, foot, GATE_H - foot), (0.0, GATE_H / 2)
    if kind == "secret":
        return K.rect_profile(0.0, SECRET_W, foot, SECRET_H - foot), (0.0, SECRET_H / 2)
    return None, None


def half(kind):
    """Halvkanten: murflaten i z = 0,4 mot +z med åpningen skåret ut, og
    lysningen rundt åpningen inn til kantlinja (z = 0), der naboens halvdel
    tar over. Endene står mot hjørnene og toppen over hvelvet: de synes
    aldri og lages ikke."""
    m = K.MB()
    prof, centre = opening(kind)
    surf = K.Plane((-FACE, WALL_T), (1.0, 0.0), (0.0, 1.0), u0=WALL_T)
    holes = [Polygon([(u + FACE, v) for u, v in prof])] if prof else []
    K.wall(m, surf, 2 * FACE, WALL_FOOT, WALL_TOP, holes, mat="wall")
    # Baksida i kantlinja og endene mot hjørnene: de synes aldri, men står
    # bak skjøtene, så ingen sprekk viser tomrommet (`--leak-scan`).
    back = K.Plane((FACE, 0.0), (-1.0, 0.0), (0.0, -1.0), u0=WALL_T)
    K.wall(m, back, 2 * FACE, WALL_FOOT, WALL_TOP, [Polygon([(FACE - u, v) for u, v in prof])] if prof else [], mat="wall")
    low = gang_h(FACE) if kind == "gang" else WALL_FOOT
    for x, sg in ((-FACE, -1.0), (FACE, 1.0)):
        m.quad((x, low, 0.0), (x, low, WALL_T), (x, WALL_TOP + 0.05, WALL_T), (x, WALL_TOP + 0.05, 0.0), "wall", facing=(sg, 0.0, 0.0))
    if prof:
        for (ua, va), (ub, vb) in zip(prof, prof[1:]):
            if abs(ua - ub) < 1e-6 and abs(abs(ua) - FACE) < 1e-6:
                continue  # i enden av muren: hjørnets flate
            mid = ((ua + ub) / 2, (va + vb) / 2)
            facing = (centre[0] - mid[0], centre[1] - mid[1], 0.0)
            L = math.dist((ua, va), (ub, vb))
            m.poly([(ua, va, WALL_T), (ub, vb, WALL_T), (ub, vb, 0.0), (ua, va, 0.0)], [(0, 0), (L, 0), (L, WALL_T), (0, WALL_T)], "dressed", facing=facing)
    return m


def half_solids(kind):
    """Kolliderne til halvkanten (bokser i halvkantens rom)."""
    def bx(x0, x1, y0, y1):
        return {"box": {"center": [(x0 + x1) / 2, (y0 + y1) / 2, WALL_T / 2], "size": [x1 - x0, y1 - y0, WALL_T]}}
    if kind == "wall":
        return [bx(-FACE, FACE, WALL_FOOT, WALL_TOP)]
    w, top = {"door": (DOOR_W, DOOR_SPRING + DOOR_W / 2), "gate": (GATE_W, GATE_H), "secret": (SECRET_W, SECRET_H)}.get(kind, (None, None))
    if w is None:
        return []
    return [bx(-FACE, -w / 2, WALL_FOOT, WALL_TOP), bx(w / 2, FACE, WALL_FOOT, WALL_TOP), bx(-w / 2, w / 2, top, WALL_TOP)]


def arch_half():
    """Halve gjordbuen under krysshvelvene langs kantlinja (0,15 m inn i
    cella), fra hjørne til hjørne; den andre halvdelen står i naboen."""
    m = K.MB()
    path = [(HALF * t, room_h(HALF * t), 0.075) for t in vault_ts()]
    K.band(m, path, 0.15, 0.22, mat="rib", down=lambda p: (-p[0], ROOM_S - p[1], 0.0))
    return m


def corner():
    """Muren i hjørnet: de to flatene som vender inn i cella."""
    m = K.MB()
    K.box(m, (0.0, WALL_FOOT, 0.0), (WALL_T, WALL_TOP + 0.05, WALL_T), "wall")
    return m


def pillar():
    m = K.MB()
    K.pillar(m, 0.0, 0.0, 0.0, ROOM_S, size=0.7)
    return m


# --- Cellene ---------------------------------------------------------------------


def romfelt():
    m = K.MB()
    floor(m)
    groin(m, ROOM_S, ROOM_R)
    return m


def gang():
    m = K.MB()
    floor(m)
    barrel(m, GANG_S, GANG_S, GANG_R)
    return m


def gangkryss():
    m = K.MB()
    floor(m)
    groin(m, GANG_S, GANG_R)
    return m


def trapp(rise, n):
    m = K.MB()
    stair(m, rise, n)
    barrel(m, GANG_S, GANG_S + rise, GANG_R)
    return m


def nedgang():
    """Romfeltet med hullet til trappa ned. Sjakta i `kit_down` går inn under
    muren mot −z; et overligger under muren lukker toppen av den."""
    m = K.MB()
    hole = sbox(-DOWN_HOLE, -HALF - 0.01, DOWN_HOLE, DOWN_Z)
    floor(m, sbox(-HALF, -HALF, HALF, HALF).difference(hole))
    groin(m, ROOM_S, ROOM_R)
    K.box(m, (-1.5, -0.4, -HALF - 0.45), (1.5, 0.0, -FACE), "wall", faces=("-y", "+z"))
    return m


# --- Manifestet ------------------------------------------------------------------


def face(kind, profile):
    return {"kind": kind, "profile": profile}


def floor_box():
    return {"box": {"center": [0.0, -0.25, 0.0], "size": [2 * HALF, 0.5, 2 * HALF]}}


def ramp(rise):
    pts = []
    for x in (-HALF, HALF):
        pts += [[x, 0.0, HALF], [x, rise, -HALF], [x, -0.5, HALF], [x, -0.5, -HALF]]
    return {"hull": pts}


OPEN, GANG, CLOSED = face("open", "room"), face("gang", "gang"), face("closed", "gang")
FLAT = [{"rect": [-HALF, -HALF, HALF, HALF], "y": [0.0, 0.0]}]


def pieces():
    """Bitene: (manifestet, modellen)."""
    out = []

    def cell(pid, model, mb, faces, clear, floors=(0.0, 0.0, 0.0, 0.0), walk=FLAT, solids=None, mounts=(), top=WALL_TOP):
        out.append(({"id": f"{THEME}/{pid}", "model": model, "kind": "cell", "theme": THEME, "v": VERSION, "faces": faces, "floors": list(floors),
                     "clear": clear, "bottom": -0.5, "top": top, "walk": walk, "solids": solids or [floor_box()], "mounts": list(mounts), "mirror": True}, mb))

    rf = romfelt()
    cell("romfelt", "grav_romfelt", rf, [OPEN] * 4, ROOM_CLEAR)
    g = gang()
    cell("gang", "grav_gang", g, [GANG, CLOSED, GANG, CLOSED], GANG_CLEAR)
    cell("gang_blind", "grav_gang", None, [GANG, CLOSED, CLOSED, CLOSED], GANG_CLEAR)
    gk = gangkryss()
    cell("gang_hjorne", "grav_gangkryss", gk, [GANG, GANG, CLOSED, CLOSED], GANG_CLEAR)
    cell("gang_t", "grav_gangkryss", None, [GANG, GANG, CLOSED, GANG], GANG_CLEAR)
    cell("gang_kryss", "grav_gangkryss", None, [GANG] * 4, GANG_CLEAR)
    for pid, model, rise, n in (("halvtrapp", "grav_halvtrapp", 1.0, 6), ("trapp2", "grav_trapp2", 2.0, 12)):
        cell(pid, model, trapp(rise, n), [GANG, CLOSED, GANG, CLOSED], GANG_CLEAR, floors=(rise, 0.0, 0.0, 0.0),
             walk=[{"rect": [-HALF, -HALF, HALF, HALF], "y": [rise, 0.0]}], solids=[ramp(rise)], top=WALL_TOP + rise)
    hole_walk = [{"rect": [-HALF, -HALF, -DOWN_HOLE, HALF], "y": [0.0, 0.0]}, {"rect": [DOWN_HOLE, -HALF, HALF, HALF], "y": [0.0, 0.0]},
                 {"rect": [-DOWN_HOLE, DOWN_Z, DOWN_HOLE, HALF], "y": [0.0, 0.0]}]
    hole_solids = [{"box": {"center": [(-HALF - DOWN_HOLE) / 2, -0.25, 0.0], "size": [HALF - DOWN_HOLE, 0.5, 2 * HALF]}},
                   {"box": {"center": [(HALF + DOWN_HOLE) / 2, -0.25, 0.0], "size": [HALF - DOWN_HOLE, 0.5, 2 * HALF]}},
                   {"box": {"center": [0.0, -0.25, (HALF + DOWN_Z) / 2], "size": [2 * DOWN_HOLE, 0.5, HALF - DOWN_Z]}}]
    cell("nedgang", "grav_nedgang", nedgang(), [CLOSED, OPEN, OPEN, OPEN], ROOM_CLEAR, walk=hole_walk, solids=hole_solids,
         mounts=[{"kind": "down", "pos": [0.0, 0.0, DOWN_Z], "yaw": 0.0}])

    def edge(pid, model, mb, kind, width=0.0, height=0.0):
        out.append(({"id": f"{THEME}/{pid}", "model": model, "kind": "edge", "theme": THEME, "v": VERSION, "opening": kind, "width": width, "height": height,
                     "solids": half_solids(kind), "mirror": True}, mb))

    edge("vegg", "grav_vegg", half("wall"), "wall")
    edge("vegg_gang", "grav_vegg_gang", half("gang"), "gang", 2 * FACE, gang_h(0.0))
    edge("vegg_dor", "grav_vegg_dor", half("door"), "door", DOOR_W, DOOR_SPRING + DOOR_W / 2)
    edge("vegg_port", "grav_vegg_port", half("gate"), "gate", GATE_W, GATE_H)
    edge("vegg_falsk", "grav_vegg_falsk", half("secret"), "secret", SECRET_W, SECRET_H)
    edge("gjordbue", "grav_gjordbue", arch_half(), "arch")
    out.append(({"id": f"{THEME}/hjorne", "model": "grav_hjorne", "kind": "corner", "theme": THEME, "v": VERSION,
                 "solids": [{"box": {"center": [WALL_T / 2, (WALL_FOOT + WALL_TOP) / 2, WALL_T / 2], "size": [WALL_T, WALL_TOP - WALL_FOOT, WALL_T]}}], "mirror": True}, corner()))
    oct_ = [[x, y, z] for (x, z) in K.chamfered((0.0, 0.0), 0.7, 0.12) for y in (0.0, ROOM_S)]
    out.append(({"id": f"{THEME}/soyle", "model": "grav_soyle", "kind": "pillar", "theme": THEME, "v": VERSION, "radius": 0.52,
                 "solids": [{"hull": oct_}], "mirror": True}, pillar()))
    return out


def main():
    args = sys.argv[1:]
    ps = pieces()
    write_manifest(MANIFEST, THEME, VERSION, [p for p, _ in ps])
    print(f"{os.path.relpath(MANIFEST, ROOT)}: {len(ps)} biter")
    if "--json" in args:
        return
    only = [a for a in args if not a.startswith("-")]
    done = set()
    for p, mb in ps:
        if mb is None or p["model"] in done or (only and p["model"] not in only):
            continue
        done.add(p["model"])
        K.export_model(mb, os.path.join(OUT, f"{p['model']}.glb"), ao=False)


if __name__ == "__main__":
    main()
