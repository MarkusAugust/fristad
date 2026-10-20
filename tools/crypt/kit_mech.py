"""Mekanismebitene i byggesettet (docs/nivaaredigering.md del 24 og 25):
kløfta under vindebroa, sjakta til heisen, luka til stigen og fallemmen,
kloakkens kanal, galleriet i to etasjer og muren med plastsømmer. Laget med
de samme funksjonene og målene som resten av byggesettet (`kit_cells.py`,
som tar dem inn i `build`), så de skjøtes med de andre bitene, tar steinen
fra temaet og bakes og sjekkes på samme måte.

| Bit | Slag | Hva |
|---|---|---|
| `grav_kloft` | celle | gulvet er en kløft 6 m ned mellom to murer; vindebroa (`bridge`) går over |
| `grav_sjakt`, `grav_sjakt_topp` | celle under og over | sjakta til heisen (`lift`): gulv nede, hull i hele cella oppe |
| `grav_luke`, `grav_luke_topp` | celle under og over | flatt tak med bjelker og luke nede, gulv med luke oppe (`ladder`, `trapdoor`) |
| `grav_kanal`, `grav_kanal_ende` | celle | gangen med en kanal midt i og gangbane på hver side (`water`) |
| `grav_galleri` | sammensatt 3 × 3 | sal på 10 × 10 m med galleri 5 m oppe langs nordmuren, trapp langs vestmuren og rekkverk |
| `grav_vegg_som` | halvkant | muren med magenta plastsømmer som skjærer gjennom steinen (`magic_vein`, svidd kant og plast som buler ut), variant `som` av `grav/vegg` |
"""

import math

from shapely.geometry import Polygon
from shapely.geometry import box as sbox

import kit_grav as K
from kit_common import FACE, GANG_CLEAR, GANG_R, GANG_S, HALF, ROOM_CLEAR, ROOM_R, ROOM_S, WALL_FOOT, WALL_T, WALL_TOP, gang_h, vault_ts

PIT = 6.0  # kløfta: så dypt under gulvet
STOREY = 6  # etasjen over en sjakt eller en luke (gulvskiva 0,5 m under)
HATCH = 0.8  # halve luka (1,6 × 1,6, som `kit_trapdoor`)
CH, CH_D = 0.7, 1.3  # kanalen: halve bredden og dybden
CH_END = 1.0  # kanalen slutter så langt fra midten i `kanal_ende`
GQ, GE = 5.0, 6.0  # galleriet: murflaten og kanten av kvadratet
G_S, G_R = 8.0, 3.0  # hvelvet i galleriet: vederlaget og pilhøyden
G_UP = 5  # galleriets gulv
G_EDGE = -2.6  # galleriets kant (z)
G_STAIR = 1.6  # trappa langs vestmuren
G_FOOT = 4.2  # trappefoten (z)
RAIL_T = 0.26

# Unntakene i tetthetssjekken (`kit_bake.check`): hullene i gulvet og
# hvelvene som er åpne, per bit (rektangler x0, z0, x1, z1 i bitens rom).
FLOOR_HOLES = {
    "grav/kloft": [(-HALF, -HALF, HALF, HALF)],
    "grav/sjakt_topp": [(-FACE, -FACE, FACE, FACE)],
    "grav/luke_topp": [(-HATCH, -HATCH, HATCH, HATCH)],
    "grav/kanal": [(-CH, -HALF, CH, HALF)],
    "grav/kanal_ende": [(-CH, -HALF, CH, CH_END)],
}
VAULT_HOLES = {
    "grav/sjakt": [(-FACE, -FACE, FACE, FACE)],
    "grav/luke": [(-HATCH, -HATCH, HATCH, HATCH)],
}


def _cells():
    import kit_cells

    return kit_cells


def plane(x0, z0, ux, uz, nx, nz, u0=0.0):
    return K.Plane((x0, z0), (ux, uz), (nx, nz), u0=u0)


def coped(m, surf, length, bottom, top, cope=0.3):
    """En mur fra `bottom` til `top` med et skift av hogd stein øverst (kanten
    mot gulvet over en kløft, en kanal eller et hull)."""
    K.wall(m, surf, length, bottom, top - cope, [], mat="wall", extra=0.0)
    if cope > 0:
        K.wall(m, surf, length, top - cope, top, [], mat="dressed", extra=0.0)


# --- Kløfta ----------------------------------------------------------------------


def kloft(far=False):
    """Kløfta: gulvet i cella er borte mellom murene i øst og vest, og muren
    går 6 m ned til en bunn av rasmasse; mot nord og sør (der gulvet i naboen
    slutter) er det kanten av berget, med et skift av hogd stein øverst.
    Krysshvelvet over som et romfelt."""
    c = _cells()
    m = K.MB()
    for sx in (-1, 1):
        # Murene i øst og vest fortsetter ned (halvkantene står fra gulvet).
        coped(m, plane(sx * FACE, -HALF * sx, 0.0, sx, -sx, 0.0), 2 * HALF, -PIT, WALL_FOOT, cope=0.0)
    for sz in (-1, 1):
        coped(m, plane(FACE * sz, sz * HALF, -sz, 0.0, 0.0, -sz, u0=WALL_T), 2 * FACE, -PIT, 0.0)
    K.grid_face(m, K.Horizontal(-PIT), sbox(-FACE, -HALF, FACE, HALF), "rubble", origin=(-HALF, -HALF))
    if not far:
        # Steiner som har falt ned.
        for x, z, s, h in ((-0.6, 0.9, 0.55, 0.4), (0.7, -1.0, 0.45, 0.3), (0.2, 0.1, 0.35, 0.25)):
            K.box(m, (x - s / 2, -PIT, z - s / 2), (x + s / 2, -PIT + h, z + s / 2), "wall", faces=("+y", "+x", "-x", "+z", "-z"))
    c.groin(m, ROOM_S, ROOM_R, k=1 if far else 3)
    return m


def kloft_solids():
    out = [{"box": {"center": [0.0, -PIT - 0.25, 0.0], "size": [2 * HALF, 0.5, 2 * HALF]}}]
    for sx in (-1, 1):
        out.append({"box": {"center": [sx * (FACE + HALF) / 2, (-PIT + WALL_FOOT) / 2, 0.0], "size": [HALF - FACE, WALL_FOOT + PIT, 2 * HALF]}})
    for sz in (-1, 1):
        out.append({"box": {"center": [0.0, (-PIT - 0.5) / 2, sz * (HALF + 0.2)], "size": [2 * HALF, PIT - 0.5, 0.4]}})
    return out


# --- Sjakta og luka ----------------------------------------------------------------


def spandrels(m):
    """Murene over buene på kantene (fra buen i naboens krysshvelv opp til
    gulvet i etasjen over), så hvelvet ved siden av ikke synes bakfra."""
    from kit_common import room_h

    top = STOREY - 0.5
    us = [t * HALF for t in vault_ts()]
    shape = Polygon([(u + HALF, room_h(u)) for u in us] + [(2 * HALF, top), (0.0, top)])
    for p0, ud, n in (((-HALF, -HALF), (1.0, 0.0), (0.0, 1.0)), ((HALF, -HALF), (0.0, 1.0), (-1.0, 0.0)),
                      ((HALF, HALF), (-1.0, 0.0), (0.0, -1.0)), ((-HALF, HALF), (0.0, -1.0), (1.0, 0.0))):
        K.grid_face(m, K.Plane(p0, ud, n), shape, "wall")


def sjakt(far=False):
    """Bunnen av heissjakta: gulvet, og ikke noe hvelv (sjakta går opp til
    cella over, `sjakt_topp`, der heistårnet står); murene over buene og
    taket mellom murflaten og cellekanten."""
    c = _cells()
    m = K.MB()
    c.floor(m)
    spandrels(m)
    top = STOREY - 0.5
    K.grid_face(m, K.Horizontal(top, up=False), sbox(-HALF, -HALF, HALF, HALF).difference(sbox(-FACE, -FACE, FACE, FACE)), "vault", origin=(-HALF, -HALF))
    return m


def rim(m, x0, z0, x1, z1, depth=0.5):
    """Kantene rundt et hull i gulvet (fra gulvet og `depth` ned), vendt inn
    mot hullet, med et skift av hogd stein øverst."""
    sides = [((x0, z0), (1.0, 0.0), (0.0, 1.0), x1 - x0), ((x1, z0), (0.0, 1.0), (-1.0, 0.0), z1 - z0),
             ((x1, z1), (-1.0, 0.0), (0.0, -1.0), x1 - x0), ((x0, z1), (0.0, -1.0), (1.0, 0.0), z1 - z0)]
    for p0, ud, n, L in sides:
        coped(m, K.Plane(p0, ud, n), L, -depth, 0.0, cope=min(0.3, depth))


def ring_floor(m, hole, mat="floor"):
    c = _cells()
    c.floor(m, sbox(-HALF, -HALF, HALF, HALF).difference(hole), mat=mat)


def ring_solids(x0, z0, x1, z1):
    """Gulvskiva rundt et hull (fire bokser)."""
    def bx(a, b, c, d):
        return {"box": {"center": [(a + c) / 2, -0.25, (b + d) / 2], "size": [c - a, 0.5, d - b]}}
    return [bx(-HALF, -HALF, HALF, z0), bx(-HALF, z1, HALF, HALF), bx(-HALF, z0, x0, z1), bx(x1, z0, HALF, z1)]


def ring_walk(x0, z0, x1, z1):
    return [{"rect": r, "y": [0.0, 0.0]} for r in ([-HALF, -HALF, HALF, z0], [-HALF, z1, HALF, HALF], [-HALF, z0, x0, z1], [x1, z0, HALF, z1])]


def sjakt_topp(far=False):
    """Toppen av heissjakta: hullet fyller cella mellom murflatene, kanten
    av gulvskiva rundt det, og krysshvelvet over (heistårnet står i det)."""
    c = _cells()
    m = K.MB()
    ring_floor(m, sbox(-FACE, -FACE, FACE, FACE))
    rim(m, -FACE, -FACE, FACE, FACE)
    c.groin(m, ROOM_S, ROOM_R, k=1 if far else 3)
    return m


def luke(far=False):
    """Rommet under luka: gulvet, et flatt tak av stein under gulvet i
    etasjen over (5,5 m), med tre eikebjelker og luka midt i."""
    c = _cells()
    m = K.MB()
    c.floor(m)
    spandrels(m)
    top = STOREY - 0.5
    hole = sbox(-HATCH, -HATCH, HATCH, HATCH)
    K.grid_face(m, K.Horizontal(top, up=False), sbox(-HALF, -HALF, HALF, HALF).difference(hole), "vault", origin=(-HALF, -HALF))
    for z in (-1.25, 1.25):
        K.box(m, (-FACE, top - 0.32, z - 0.14), (FACE, top, z + 0.14), "oak", faces=("-y", "+z", "-z"))
    if not far:
        # Vekslerne langs luka.
        for x in (-HATCH - 0.12, HATCH):
            K.box(m, (x, top - 0.2, -1.11), (x + 0.12, top, 1.11), "oak", faces=("-y", "+x", "-x"))
    return m


def luke_topp(far=False):
    """Rommet over luka: gulvet med luka midt i (kanten ned til taket under)
    og krysshvelvet."""
    c = _cells()
    m = K.MB()
    ring_floor(m, sbox(-HATCH, -HATCH, HATCH, HATCH))
    rim(m, -HATCH, -HATCH, HATCH, HATCH)
    c.groin(m, ROOM_S, ROOM_R, k=1 if far else 3)
    return m


# --- Kanalen -----------------------------------------------------------------------


def kanal(far=False, end=False):
    """Kloakkgangen: tønnehvelvet, gangbanen på hver side og kanalen midt i
    (1,4 m bred, 1,3 m dyp, med et skift av hogd stein langs kanten). Med
    `end` slutter kanalen 1 m fra midten mot sør, og gangen fortsetter tørr."""
    c = _cells()
    m = K.MB()
    z1 = CH_END if end else HALF
    c.floor(m, sbox(-HALF, -HALF, HALF, HALF).difference(sbox(-CH, -HALF, CH, z1)))
    for sx in (-1, 1):
        coped(m, plane(sx * CH, -HALF if sx > 0 else z1, 0.0, sx, -sx, 0.0), z1 + HALF, -CH_D, 0.0, cope=0.25)
    K.grid_face(m, K.Horizontal(-CH_D), sbox(-CH, -HALF, CH, z1), "floor", origin=(-HALF, -HALF))
    if end:
        coped(m, plane(CH, z1, -1.0, 0.0, 0.0, -1.0), 2 * CH, -CH_D, 0.0, cope=0.25)
    c.barrel(m, GANG_S, GANG_S, GANG_R, steps=1 if far else 4)
    return m


def kanal_solids(end=False):
    z1 = CH_END if end else HALF
    out = []
    for sx in (-1, 1):
        out.append({"box": {"center": [sx * (CH + HALF) / 2, -0.25, 0.0], "size": [HALF - CH, 0.5, 2 * HALF]}})
    out.append({"box": {"center": [0.0, -CH_D - 0.25, (z1 - HALF) / 2], "size": [2 * CH, 0.5, z1 + HALF]}})
    if end:
        out.append({"box": {"center": [0.0, (-CH_D - 0.5) / 2, (z1 + HALF) / 2], "size": [2 * CH, CH_D + 0.5, HALF - z1]}})
    return out


def kanal_walk(end=False):
    out = [{"rect": [-HALF, -HALF, -CH, HALF], "y": [0.0, 0.0]}, {"rect": [CH, -HALF, HALF, HALF], "y": [0.0, 0.0]}]
    if end:
        out.append({"rect": [-CH, CH_END, CH, HALF], "y": [0.0, 0.0]})
    return out


# --- Plastsømmene ------------------------------------------------------------------


def _crack(rng, u, v, ang, length, w, step=(0.11, 0.2)):
    """En sprekk som vandrer fra (u, v) i retningen `ang` (radianer, 0 opp):
    knekker brått til sidene og smalner mot enden. Gir punktene og bredden."""
    pts, ws = [(u, v)], [w * 0.5]
    run, a = 0.0, ang
    while run < length:
        d = step[0] + rng.random() * (step[1] - step[0])
        # Knekkene: oftest små, av og til et brått hakk til siden.
        a = ang + (rng.random() - 0.5) * (1.4 if rng.random() < 0.3 else 0.6)
        u, v = u + math.sin(a) * d, v + math.cos(a) * d
        run += d
        t = run / length
        pts.append((u, v))
        ws.append(w * (0.45 + 0.55 * math.sin(math.pi * min(t, 1.0)) ** 0.6) * (0.6 + 0.8 * rng.random()))
    ws[-1] = w * 0.12
    return pts, ws


def _strip(m, surf, pts, ws, d, mat, scale=1.0, joints=True):
    """Et bånd langs sprekken `d` ut fra muren, med en rute i hver knekk så
    det ikke blir glipper (`joints`)."""
    for (ua, va), (ub, vb), wa, wb in zip(pts, pts[1:], ws, ws[1:]):
        L = math.hypot(ub - ua, vb - va)
        if L < 1e-6:
            continue
        nu, nv = -(vb - va) / L, (ub - ua) / L
        ha, hb = wa * scale / 2, wb * scale / 2
        quad = [(ua - nu * ha, va - nv * ha), (ub - nu * hb, vb - nv * hb), (ub + nu * hb, vb + nv * hb), (ua + nu * ha, va + nv * ha)]
        m.poly([surf.point(a, b, d) for a, b in quad], quad, mat, facing=surf.normal(ua))
    if not joints:
        return
    for (uc, vc), wc in zip(pts[1:-1], ws[1:-1]):
        r = wc * scale / 2
        ring = [(uc + r * math.cos(k * math.pi / 2), vc + r * math.sin(k * math.pi / 2)) for k in range(4)]
        m.poly([surf.point(a, b, d * 0.98) for a, b in ring], ring, mat, facing=surf.normal(uc))


def _blob(m, surf, rng, u, v, r, h):
    """Plast som buler ut av sprekken: en ujevn, lav kul (`plastic_seam`) med en
    glødende rand der den presser seg ut av steinen."""
    n, rings = 8, 2
    rad = [r * (0.7 + 0.5 * rng.random()) for _ in range(n)]
    sq = 0.75 + 0.5 * rng.random()  # avlang
    rim = []
    for k in range(n):
        a = 2 * math.pi * k / n
        rim.append((math.cos(a) * sq, math.sin(a) / sq))

    def at(k, i):
        """Punktet på ring i (0 er randen, `rings` er toppen) i retning k."""
        t = i / rings
        rr = rad[k % n] * math.cos(t * math.pi / 2)
        hh = h * math.sin(t * math.pi / 2) ** 0.8
        cu, cv = rim[k % n]
        return (u + cu * rr, v + cv * rr, 0.004 + hh)

    top = surf.point(u, v, 0.004 + h * 1.05)
    for i in range(rings):
        for k in range(n):
            a, b = at(k, i), at(k + 1, i)
            if i == rings - 1:
                m.tri(surf.point(*a), surf.point(*b), top, "plastic_seam", facing=surf.normal(u))
            else:
                c, d = at(k + 1, i + 1), at(k, i + 1)
                m.quad(surf.point(*a), surf.point(*b), surf.point(*c), surf.point(*d), "plastic_seam", facing=surf.normal(u))
    # Randen: et smalt glødende bånd rundt foten.
    for k in range(n):
        a, b = at(k, 0), at(k + 1, 0)
        cu, cv = rim[k % n]
        du, dv = rim[(k + 1) % n]
        o = 0.025
        quad = [(a[0], a[1]), (b[0], b[1]), (b[0] + du * o, b[1] + dv * o), (a[0] + cu * o, a[1] + cv * o)]
        m.poly([surf.point(p, q, 0.009) for p, q in quad], quad, "magic_vein", facing=surf.normal(u))


SEAM_W = 0.05  # den glødende sømmen på det bredeste


def seam(m, surf, u, v0, v1, seed, w=SEAM_W, blobs=2):
    """En plastsøm som skjærer gjennom steinen (kanon: de magenta sømmene
    fra kongens rammeverk): en ujevn sprekk opp langs muren med greiner,
    svidd mørk stein langs kantene (`scorch`), magenta glød i midten
    (`magic_vein`), og plast som buler ut av sprekken (`_blob`)."""
    import random

    rng = random.Random(seed)
    main, mw = _crack(rng, u, v0, (rng.random() - 0.5) * 0.4, v1 - v0, w)
    cracks = [(main, mw)]
    # Greinene: fra et punkt på sprekken, skrått ut til en side.
    for _ in range(2 + rng.randrange(2)):
        k = rng.randrange(1, max(2, len(main) - 2))
        side = rng.choice((-1, 1))
        bu, bv = main[k]
        cracks.append(_crack(rng, bu, bv, side * (0.6 + rng.random() * 0.9), 0.25 + rng.random() * 0.6, mw[k] * 0.7, step=(0.08, 0.14)))
        # Av og til en kvist på greina.
        if rng.random() < 0.4:
            bp, bw = cracks[-1]
            j = len(bp) // 2
            cracks.append(_crack(rng, *bp[j], side * (1.2 + rng.random() * 0.6), 0.12 + rng.random() * 0.2, bw[j] * 0.7, step=(0.06, 0.1)))
    for pts, ws in cracks:
        # Hold sprekken på muren (u i 0,1–3,1, over sokkelen).
        pts[:] = [(min(2 * FACE - 0.1, max(0.1, a)), max(c_plinth() + 0.02, b)) for a, b in pts]
        _strip(m, surf, pts, ws, 0.004, "scorch", scale=3.0, joints=False)
        _strip(m, surf, pts, ws, 0.008, "magic_vein")
    for _ in range(blobs):
        k = rng.randrange(1, len(main) - 1)
        bu, bv = main[k]
        _blob(m, surf, rng, bu, bv, 0.07 + rng.random() * 0.1, 0.03 + rng.random() * 0.05)


def c_plinth():
    return _cells().PLINTH_H


def vegg_som(far=False):
    """Muren i kjernen: den vanlige halvkanten med tre plastsømmer som
    skjærer gjennom steinen (plasten som brer seg i muren)."""
    c = _cells()
    m = c.half("wall", far)
    surf = c.surf_face()
    for k, (u, v0, v1, blobs) in enumerate(((0.75, 0.35, 3.3, 1), (2.0, 1.2, 4.5, 2), (2.75, 0.4, 2.1, 1))):
        seam(m, surf, u, v0, v1, seed=17 + 5 * k, blobs=0 if far else blobs)
    return m


# --- Galleriet ---------------------------------------------------------------------


def galleri_planes():
    Q = GQ
    return [K.Plane((-Q, -Q), (1.0, 0.0), (0.0, 1.0), u0=0.0), K.Plane((Q, -Q), (0.0, 1.0), (-1.0, 0.0), u0=2 * Q),
            K.Plane((Q, Q), (-1.0, 0.0), (0.0, -1.0), u0=4 * Q), K.Plane((-Q, Q), (0.0, -1.0), (1.0, 0.0), u0=6 * Q)]


# Utgangene: (side, gulvet); vest er lukket (trappa står der).
G_EXITS = ((0, G_UP), (1, 0), (2, 0))


def galleri(far=False):
    """Galleriet 3 × 3 (origo midt i midtcella): en sal på 10 × 10 m med
    murene til 8 m og krysshvelv med ribber til 11 m, et galleri 5 m oppe
    langs nordmuren (gulv av stein på eikebjelker, med rekkverk), en trapp
    langs vestmuren opp til det, og gangene ut: i sør og øst nede, i nord fra
    galleriet."""
    c = _cells()
    m = K.MB()
    E, Q = GE, GQ
    top = G_S + G_R + 0.3
    angles = {0: -math.pi / 2, 1: 0.0, 2: math.pi / 2}  # nord, øst, sør (gangen langs +x dreid)
    # Gulvet nede og gangene ut i sør og øst.
    shape = sbox(-Q, -Q, Q, Q).union(sbox(-HALF, 0.0, HALF, E)).union(sbox(0.0, -HALF, E, HALF))
    K.grid_face(m, K.Horizontal(0.0), shape, "floor", origin=(-E, -E))
    # Murene med gangåpningene.
    ts = [t for t in vault_ts() if abs(t * HALF) <= c.ROT_GANG + 1e-9]
    for k, surf in enumerate(galleri_planes()):
        holes = []
        for side, y in G_EXITS:
            if side == k:
                holes.append(Polygon([(Q - c.ROT_GANG, y + WALL_FOOT - 0.01)] + [(Q + t * HALF, y + gang_h(t * HALF)) for t in ts] + [(Q + c.ROT_GANG, y + WALL_FOOT - 0.01)]))
        K.wall(m, surf, 2 * Q, WALL_FOOT, top, holes, mat="wall")
        if far:
            continue
        # Sokkelen langs foten: rundt gangene i sør og øst, hele nordmuren
        # (gangen ut der går fra galleriet), og vestmuren på hver side av
        # trappa (u = 0 i sørvest).
        spans = {0: [(0.0, 2 * Q, 0.0)], 3: [(0.0, Q - G_FOOT, 0.0), (Q - G_EDGE, 2 * Q, 0.0)]}.get(k, [(0.0, Q - c.ROT_GANG, 0.0), (Q + c.ROT_GANG, 2 * Q, 0.0)])
        K.plinth(m, surf, spans, h=c.PLINTH_H, proud=c.PLINTH_OUT)
    # Lysningen i gangene ut, med gulvet i nord (galleriet) og gangens hvelv.
    arc = K.ellipse_arclen(HALF, GANG_R, ts)
    for side, y in G_EXITS:
        ac = angles[side]
        cs, sn = math.cos(ac), math.sin(ac)
        R = lambda x, yy, z, cs=cs, sn=sn, y=y: (x * cs - z * sn, yy + y, x * sn + z * cs)
        x0, x1 = Q - 0.02, E - WALL_T
        for sz in (-1.0, 1.0):
            pts = [R(x0, WALL_FOOT, sz * c.ROT_GANG), R(x1, WALL_FOOT, sz * c.ROT_GANG), R(x1, gang_h(c.ROT_GANG), sz * c.ROT_GANG), R(x0, gang_h(c.ROT_GANG), sz * c.ROT_GANG)]
            m.poly(pts, [(x0, WALL_FOOT), (x1, WALL_FOOT), (x1, gang_h(c.ROT_GANG)), (x0, gang_h(c.ROT_GANG))], "wall", facing=R(0.0, -y, -sz))
        if y:
            K.grid_face(m, K.Horizontal(y), sbox(-HALF, -E, HALF, -Q), "floor", origin=(-E, -E))
        for i in range(len(ts) - 1):
            za, zb = ts[i] * HALF, ts[i + 1] * HALF
            pts = [R(x0, gang_h(za), za), R(E, gang_h(za), za), R(E, gang_h(zb), zb), R(x0, gang_h(zb), zb)]
            m.poly(pts, [(x0, arc[i]), (E, arc[i]), (E, arc[i + 1]), (x0, arc[i + 1])], "vault", facing=R(0.0, -1.0 - y, -(za + zb) / 2), smooth=True)
        if not far:
            path = [R(Q - 0.06, gang_h(t * HALF), t * HALF) for t in ts]
            K.band(m, path, 0.12, 0.2, mat="dressed", down=lambda p, R=R: K.sub(R(0.0, GANG_S, 0.0), (p[0], p[1], p[2])), stones=13 + 5 * side)
    # Hvelvet og ribbene, med konsoller i hjørnene.
    c.groin_square(m, Q, G_S, G_R, k=3 if far else 6)
    if not far:
        K.groin_ribs(m, -Q, Q, -Q, Q, G_S, G_R, width=0.32, depth=0.2, stones=31)
        for sx in (-1, 1):
            for sz in (-1, 1):
                x0, x1 = sorted((sx * Q, sx * (Q - 0.5)))
                z0, z1 = sorted((sz * Q, sz * (Q - 0.5)))
                K.box(m, (x0, G_S - 0.55, z0), (x1, G_S - 0.05, z1), "dressed", faces=("-y", "+y", "+x" if sx < 0 else "-x", "+z" if sz < 0 else "-z"))
    # Galleriet: gulvet, undersida, forkanten og bjelkene med konsoller.
    under = G_UP - 0.42
    K.grid_face(m, K.Horizontal(G_UP), sbox(-Q, -Q, Q, G_EDGE), "floor", origin=(-E, -E))
    K.grid_face(m, K.Horizontal(under, up=False), sbox(-Q, -Q, Q, G_EDGE), "oak", origin=(-E, -E))
    K.wall(m, K.Plane((-Q, G_EDGE), (1.0, 0.0), (0.0, 1.0)), 2 * Q, under, G_UP, [], mat="dressed", extra=0.0)
    for x in (-3.0, -1.0, 1.0, 3.0):
        K.box(m, (x - 0.15, under - 0.3, -Q), (x + 0.15, under, G_EDGE + 0.05), "oak", faces=("-y", "+x", "-x", "+z"))
        if not far:
            K.box(m, (x - 0.2, under - 0.75, -Q), (x + 0.2, under - 0.3, -Q + 0.45), "dressed", faces=("-y", "+x", "-x", "+z"))
    # Trappa langs vestmuren, med muren under den mot salen.
    x_in = -Q + G_STAIR
    run = G_FOOT - G_EDGE
    n = 25
    K.steps(m, (-Q + G_STAIR / 2, G_FOOT), (0.0, -1.0), (1.0, 0.0), G_STAIR, 0.0, run, 0.0, G_UP, n)
    # Sida mot salen følger trinnene (trapp på trapp, ingen glipe under inntrinnene).
    side = K.Plane((x_in, G_FOOT), (0.0, -1.0), (1.0, 0.0))
    t, r = run / n, G_UP / n
    outline = [(0.0, WALL_FOOT), (run, WALL_FOOT)]
    for k in reversed(range(n)):
        outline += [((k + 1) * t, (k + 1) * r), (k * t, (k + 1) * r)]
    K.grid_face(m, side, Polygon(outline), "wall")
    # Enden av trappa under galleriet er mur helt opp (eieren 20. oktober:
    # «ikke la det være synlig åpent under trappen»): trappa er massiv.
    end = K.Plane((x_in, G_EDGE), (-1.0, 0.0), (0.0, -1.0))
    K.grid_face(m, end, sbox(0.0, WALL_FOOT, G_STAIR, G_UP - 0.42), "wall")
    if not far:
        # Sokkelen går videre langs trappemuren mot salen og rundt enden.
        K.plinth(m, side, [(0.0, run, 0.0)], h=_cells().PLINTH_H, proud=_cells().PLINTH_OUT)
        K.plinth(m, end, [(0.0, G_STAIR, 0.0)], h=_cells().PLINTH_H, proud=_cells().PLINTH_OUT)
    # Rekkverket: opp langs trappa og videre langs galleriets kant.
    # Langs trappa: stolper av hogd stein med en håndlist av eik; langs
    # galleriet en brystning.
    rx = x_in - RAIL_T / 2
    K.parapet(m, [(rx, G_EDGE - RAIL_T / 2), (Q, G_EDGE - RAIL_T / 2)], 0.0, t=RAIL_T, base_y=[G_UP, G_UP])
    posts = 5
    for k in range(posts):
        z = G_FOOT - 0.15 - (run - 0.3) * k / (posts - 1)
        y = math.ceil((G_FOOT - z) / t) * r
        K.box(m, (rx - 0.09, y - 0.02, z - 0.09), (rx + 0.09, y + 0.95, z + 0.09), "dressed", faces=("+y", "+x", "-x", "+z", "-z"))
    K.beam(m, (rx, r + 0.98, G_FOOT - 0.15), (rx, G_UP + 0.98, G_EDGE + 0.15), 0.1, 0.08, "oak")
    return m


def galleri_solids():
    E, Q = GE, GQ
    top = G_S + G_R + 0.3
    out = [{"box": {"center": [0.0, -0.25, 0.0], "size": [2 * E, 0.5, 2 * E]}}]
    t = E - Q
    rg = _cells().ROT_GANG
    exits = dict(G_EXITS)
    for k in range(4):
        cs, sn = round(math.cos((k - 1) * math.pi / 2)), round(math.sin((k - 1) * math.pi / 2))  # k = 0: nord
        y = exits.get(k)
        for a, b in ((-E, -rg), (rg, E)) if y is not None else ((-E, E),):
            cx, cz = Q + t / 2, (a + b) / 2
            ctr = [cx * cs - cz * sn, (WALL_FOOT + top) / 2, cx * sn + cz * cs]
            size = [t, top - WALL_FOOT, b - a] if cs != 0 else [b - a, top - WALL_FOOT, t]
            out.append({"box": {"center": [round(v, 4) for v in ctr], "size": [round(v, 4) for v in size]}})
        if y is not None:
            cx = Q + t / 2
            lo, hi = y + gang_h(0.0) + 0.1, top
            out.append({"box": {"center": [round(cx * cs, 4), round((lo + hi) / 2, 4), round(cx * sn, 4)], "size": [round(v, 4) for v in ([t, hi - lo, 2 * rg] if cs != 0 else [2 * rg, hi - lo, t])]}})
            if y > 0:
                out.append({"box": {"center": [round(cx * cs, 4), round(y / 2 - 0.25, 4), round(cx * sn, 4)], "size": [round(v, 4) for v in ([t, y + 0.5, 2 * rg] if cs != 0 else [2 * rg, y + 0.5, t])]}})
    # Galleriet, trappa (rampe og massiv under) og rekkverket.
    out.append({"box": {"center": [0.0, G_UP - 0.25, (-Q + G_EDGE) / 2], "size": [2 * Q, 0.5, G_EDGE + Q]}})
    x0, x1 = -Q, -Q + G_STAIR
    pts = []
    for x in (x0, x1):
        pts += [[x, 0.0, G_FOOT], [x, G_UP, G_EDGE], [x, -0.5, G_FOOT], [x, -0.5, G_EDGE]]
    out.append({"hull": pts})
    rx = x1 - RAIL_T
    rail = []
    for x in (rx, x1):
        rail += [[x, 0.2, G_FOOT], [x, 1.15, G_FOOT], [x, G_UP, G_EDGE], [x, G_UP + 0.95, G_EDGE]]
    out.append({"hull": rail})
    out.append({"box": {"center": [(rx + Q) / 2, G_UP + 0.475, G_EDGE - RAIL_T / 2], "size": [Q - rx, 0.95, RAIL_T]}})
    return out


def galleri_walk():
    E, Q = GE, GQ
    q, g = Q - 0.4, _cells().ROT_GANG - 0.4
    x_in = -Q + G_STAIR
    out = [{"rect": [x_in + 0.4, -q, q, q], "y": [0.0, 0.0]}, {"rect": [-q, -q, x_in + 0.4, G_EDGE - 0.4], "y": [0.0, 0.0]},
           {"rect": [-g, q, g, E], "y": [0.0, 0.0]}, {"rect": [q, -g, E, g], "y": [0.0, 0.0]},
           # Trappa (gulvet stiger mot nord) og galleriet med gangen ut i nord.
           {"rect": [-q, G_EDGE, x_in - RAIL_T - 0.3, G_FOOT], "y": [float(G_UP), 0.0]},
           {"rect": [-q, G_EDGE - 0.7, x_in - RAIL_T - 0.3, G_EDGE], "y": [float(G_UP), float(G_UP)]},
           {"rect": [-q, -q, q, G_EDGE - RAIL_T - 0.4], "y": [float(G_UP), float(G_UP)]},
           {"rect": [-g, -E, g, -q], "y": [float(G_UP), float(G_UP)]}]
    return [{"rect": [round(v, 3) for v in r["rect"]], "y": r["y"]} for r in out]


def galleri_mounts():
    """Faklene: nede på murene i sør og øst og ved trappefoten, og oppe på
    nordmuren over galleriet."""
    out = []
    # (mur, u fra hjørnet, høyde): sør og øst nede, nord under og over
    # galleriet, vest over trappa (u fra sørvest) og oppe ved galleriet.
    spots = [(1, GQ - 3.2, 3.2), (1, GQ + 3.2, 3.2), (2, GQ - 3.2, 3.2), (2, GQ + 3.2, 3.2),
             (0, GQ - 3.2, 3.2), (0, GQ + 3.2, 3.2), (0, GQ - 3.2, G_UP + 2.3), (0, GQ + 3.2, G_UP + 2.3),
             (3, GQ - 3.2, 3.2), (3, GQ - 0.5, 4.6), (3, GQ + 3.6, G_UP + 2.3)]
    planes = galleri_planes()
    for k, u, y in spots:
        surf = planes[k]
        nx, _, nz = surf.normal()
        px, _, pz = surf.point(u, y)
        out.append({"kind": "fakkel", "pos": [round(px, 4), y, round(pz, 4)], "yaw": round(math.atan2(nx, nz), 6)})
    return out


# --- Manifestet ----------------------------------------------------------------------


def add(cell, model, face):
    """Bitene i manifestet (`kit_cells.build` gir `cell`, `model` og `face`)."""
    c = _cells()
    OPEN, GANG = face("open", "room"), face("gang", "gang")
    CLOSED_ROOM, CLOSED = face("closed", "room"), face("closed", "gang")
    kl = model("grav_kloft", kloft, "cell", "cell")
    cell("kloft", kl, [OPEN, CLOSED_ROOM, OPEN, CLOSED_ROOM], ROOM_CLEAR, walk=[], solids=kloft_solids())
    sj = model("grav_sjakt", sjakt, "cell", "cell")
    cell("sjakt", sj, [OPEN] * 4, ROOM_CLEAR)
    st = model("grav_sjakt_topp", sjakt_topp, "cell", "cell")
    cell("sjakt_topp", st, [OPEN] * 4, ROOM_CLEAR, walk=ring_walk(-FACE, -FACE, FACE, FACE), solids=ring_solids(-FACE, -FACE, FACE, FACE))
    lk = model("grav_luke", luke, "cell", "cell")
    cell("luke", lk, [OPEN] * 4, ROOM_CLEAR)
    lt = model("grav_luke_topp", luke_topp, "cell", "cell")
    cell("luke_topp", lt, [OPEN] * 4, ROOM_CLEAR, walk=ring_walk(-HATCH, -HATCH, HATCH, HATCH), solids=ring_solids(-HATCH, -HATCH, HATCH, HATCH))
    ka = model("grav_kanal", kanal, "gang", "cell")
    cell("kanal", ka, [GANG, CLOSED, GANG, CLOSED], GANG_CLEAR, walk=kanal_walk(), solids=kanal_solids())
    ke = model("grav_kanal_ende", lambda far: kanal(far, end=True), "gang", "cell")
    cell("kanal_ende", ke, [GANG, CLOSED, GANG, CLOSED], GANG_CLEAR, walk=kanal_walk(True), solids=kanal_solids(True))
    # Galleriet: fotavtrykket som bosshallen, men vest lukket og nord fra galleriet.
    gm = model("grav_galleri", galleri, "rotunde", "rotunde")
    foot = []
    exits = dict(G_EXITS)
    for dj in (-1, 0, 1):
        for di in (-1, 0, 1):
            fs = []
            floors = [0.0] * 4
            for w, (oi, oj) in enumerate(((0, -1), (1, 0), (0, 1), (-1, 0))):
                if -1 <= di + oi <= 1 and -1 <= dj + oj <= 1:
                    fs.append(c.INNER)
                elif (di, dj) == (oi, oj) and w in exits:
                    fs.append(GANG)
                    floors[w] = float(exits[w])
                else:
                    fs.append(CLOSED)
            fc = {"at": [di, dj], "faces": fs}
            # Bare kanten ut fra galleriet står oppe; mot resten av salen er gulvet nede.
            if any(floors):
                fc["floors"] = floors
            foot.append(fc)
    cell("galleri", gm, [c.INNER] * 4, G_S, walk=galleri_walk(), solids=galleri_solids(), mounts=galleri_mounts(), top=G_S + G_R + 0.6, extra={"footprint": foot})
