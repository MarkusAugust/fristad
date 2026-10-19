"""Pilotsettet for gravkjelleren i den malte stilen (docs/nivaaredigering.md,
del 4.4 og fase 2): bitene kjernen setter sammen etter nivådokumentet
(`assets/levels/nivaa_*.json`, `domain/world/crypt/compose/`), laget med
funksjonene i `kit_grav.py` og de malte lagene (`crypt_wall`, `crypt_floor`,
`grave_slab`, `ossuary`), med sokkel, karmer, ribber og nisjer som i nivå 1.

Bitene står på rutenettet på 4 × 4 m (`kit_common.py`):

| Bit | Slag | Hva |
|---|---|---|
| `grav_romfelt`, `grav_romfelt_b` | celle | gulv og krysshvelv; b har gravhelle og ribber |
| `grav_gang`, `grav_gang_b` | celle | gulv og tønnehvelv langs z; b har gjordbue og gravhelle |
| `grav_gangkryss` | celle | gulv og lavt krysshvelv (hjørne, T og kryss i gangen) |
| `grav_halvtrapp`, `grav_trapp2` | celle | trapp 1 og 2 m opp mot −z under et skrått tønnehvelv |
| `grav_nedgang` | celle | romfelt med hull for trappa ned (`kit_down`) |
| `grav_rotunde` | sammensatt 3 × 3 | rundt rom med kuppel og ribber, nisjer og fire ganger ut |
| `grav_vegg`, `grav_vegg_b` | halvkant | murhalvdelen med sokkel og fakkelfeste; b med gravnisjer |
| `grav_vegg_gang` | halvkant | overmuren over en gangåpning, med buestav |
| `grav_vegg_dor` | halvkant | mur med rundbuet dør 1,6 × 2,3 og karm |
| `grav_vegg_port` | halvkant | mur med portåpning 2,8 × 3,0 (`kit_gate`) |
| `grav_vegg_falsk` | halvkant | murene rundt åpningen til den falske veggen |
| `grav_falsk` | den falske veggen | muren som synker, i flukt med muren rundt, med sokkel og sprekk |
| `grav_gjordbue` | halvkant | halve gjordbuen mellom to romfelt i samme rom |
| `grav_overmur` | halvkant | muren over et lavere hvelv, 2 m (ulike tak, fase 6) |
| `grav_rekkverk` | halvkant | brystning (ulike gulv, fase 6) |
| `grav_hjorne` | kvarthjørne | innvendig hjørne, og der muren går videre |
| `grav_hjorne_ut` | kvarthjørne | utvendig hjørne, avfaset over sokkelen |
| `grav_pilaster` | kvarthjørne | der en mur møter en gjordbue: pilaster under buen |
| `grav_soyle` | søyle | frittstående søyle der fire romfelt møtes |
| `grav_fakkel` | pynt | fakkelen i et fakkelfeste |

Hver bit har en nær- og en fjernversjon (noden `far` i glb-fila: uten
sokkel, karmer, ribber og nisjer), okklusjon bakt i sammenheng med naboene
og gjort nøytral mot skjøtene (`kit_bake.py`), festene for fakler og pynt
(`mounts`), og manifestet for biten i `extras.kit` (likt manifestet,
`manifestet_er_likt_glb_filene`). Variantene (`variants`) velges med
nivåets frø og cellens plass.

    python3.13 tools/crypt/kit_cells.py            # eksporterer alt og skriver manifestet
    python3.13 tools/crypt/kit_cells.py --json     # bare manifestet (uten Blender)
    python3.13 tools/crypt/kit_cells.py --check    # tetthetssjekken per bit (Blender)
    python3.13 tools/crypt/kit_cells.py grav_vegg  # bare én bit
"""

import math
import os
import sys

from shapely.geometry import Point, Polygon
from shapely.geometry import box as sbox

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import kit_grav as K  # noqa: E402
from kit_common import (  # noqa: E402
    DOOR_SPRING, DOOR_W, FACE, GANG_CLEAR, GANG_R, GANG_S, GATE_H, GATE_W, HALF, ROOM_CLEAR, ROOM_R, ROOM_S,
    SECRET_H, SECRET_W, WALL_FOOT, WALL_T, WALL_TOP, gang_h, room_h, vault_ts, write_manifest)

THEME = "grav"
VERSION = 2
OUT = os.path.join(ROOT, "assets", "models", "crypt", "kit", THEME)
MANIFEST = os.path.join(ROOT, "assets", "models", "crypt", "kit", f"{THEME}.json")
# Trappa ned (`kit_down`) står så langt fra cellemidten mot +z, med hullet
# bak seg (`kit_grav.kit_down`: 3,2 m trinn og 0,6 m sjakt bak).
DOWN_Z = 1.65
DOWN_HOLE = 1.2
# Sokkelen langs veggfoten (som i nivå 1).
PLINTH_H = 0.34
PLINTH_OUT = 0.07
# Fakkelfestet midt på muren, og flammen i fakkelen (`K.torch`).
TORCH_Y = 2.3
FLAME = (0.0, 0.5, 0.275)
# Pilasteren under gjordbuen: halv bredde i hver celle og hvor langt ut.
PIL_W, PIL_OUT = 0.2, 0.15
# Rotunden: indre radius, muren rundt og kuppelen.
ROT_R = 5.0
ROT_S, ROT_CROWN = 3.4, 7.6
ROT_GANG = 1.6  # halve gangbredden ut fra rotunden (som murflaten i gangen)


def surf_face():
    """Murflaten i halvkanten (z = 0,4 mot +z), u fra −1,6; uv som i nivået."""
    return K.Plane((-FACE, WALL_T), (1.0, 0.0), (0.0, 1.0), u0=WALL_T)


def surf_mid():
    """Samme flate med u = 0 midt på muren (karmene)."""
    return K.Plane((0.0, WALL_T), (1.0, 0.0), (0.0, 1.0), u0=HALF)


# --- Flatene ---------------------------------------------------------------------


def floor(m, shape=None, mat="floor"):
    """Gulvet over cella (eller `shape` i xz), i ruter på 1 m."""
    K.grid_face(m, K.Horizontal(0.0), shape if shape is not None else sbox(-HALF, -HALF, HALF, HALF), mat, origin=(-HALF, -HALF))


def slab_floor(m, w, d, cx=0.0, cz=0.0, hole=None):
    """Gulvet med et felt av gravheller (w × d) i flukt midt i: feltet er egen
    flate med gravhelle-laget fra hjørnet av bildet (2 × 2 m gir hele bildet
    med de fire hellene), gulvet rundt har hullet."""
    s = sbox(cx - w / 2, cz - d / 2, cx + w / 2, cz + d / 2)
    rest = sbox(-HALF, -HALF, HALF, HALF).difference(s)
    if hole is not None:
        rest = rest.difference(hole)
    floor(m, rest)
    # Uv i meter fra hjørnet av feltet, så hellene i bildet står hele (glTF
    # har v nedover, v = 1 − v her: v fra −1 gir radene 0..d i bildet), i
    # ruter på 1 m som gulvet rundt, så kantene deler hjørner (ingen T-kryss).

    class Field(K.Horizontal):
        def uv(self, u, v):
            return (u - (cx - w / 2), v - (cz - d / 2) - 1.0)

    K.grid_face(m, Field(0.0), s, "slab", origin=(-HALF, -HALF))


def groin(m, S, R, mat="vault", k=3):
    """Krysshvelvet over hele cella (som `kit_grav.groin_bay`, med prøvene i
    `vault_ts`, så buene langs kantene møter naboens). `k` er antall ruter
    fra kanten inn til graten (fjernversjonen har færre)."""
    ts = vault_ts()
    a = b = HALF
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


def barrel(m, S0, S1, R, mat="vault", steps=4):
    """Tønnehvelvet langs z over hele cella, med vederlaget S0 ved +z og S1
    ved −z (skrått over trappene), i `steps` ruter langs aksen."""
    ts = vault_ts()
    arc = K.ellipse_arclen(HALF, R, ts)
    zs = [HALF - 2 * HALF * k / steps for k in range(steps + 1)]
    S = lambda z: S0 + (S1 - S0) * (HALF - z) / (2 * HALF)
    for i in range(len(ts) - 1):
        for q in range(steps):
            pts, uvs = [], []
            for t, ar, z in ((ts[i], arc[i], zs[q]), (ts[i + 1], arc[i + 1], zs[q]), (ts[i + 1], arc[i + 1], zs[q + 1]), (ts[i], arc[i], zs[q + 1])):
                pts.append((HALF * t, S(z) + R * math.sqrt(max(0.0, 1 - t * t)), z))
                uvs.append((z, ar))
            c = K.lerp(pts[0], pts[2], 0.5)
            m.poly(pts, uvs, mat, facing=K.sub((0.0, S(c[2]) - 2.0, c[2]), c), smooth=True)


def stone_column(m, surf, u0, u1, v0, v1, proud, seed=0, mat="dressed"):
    """En stabel hogde steiner på murflaten (pilasteren, hjørnesteinene):
    skift annenhver 0,42 og 0,30 m høye, hver med én hel stein fra murbildet
    (`K.stone_uv`) og mørtel bak fugene, som karmene."""
    g = 0.008
    q = [(u0 + 0.004, v0), (u1 - 0.004, v0), (u1 - 0.004, v1), (u0 + 0.004, v1)]
    m.poly([surf.point(u, v, proud * 0.35) for u, v in q], [surf.uv(u, v) for u, v in q], "mortar", facing=surf.normal(0.0))
    v, j, k = v0, 0, seed
    while v < v1 - 0.05:
        h = min(0.42 if j % 2 == 0 else 0.3, v1 - v)
        pts = [(u0, v + g), (u1, v + g), (u1, v + h - g), (u0, v + h - g)]
        K._trim_block(m, surf, pts, proud, mat, K.stone_uv(k, pts))
        v += h
        j += 1
        k += 3


def sealed_niche(m, surf, u, v0, w=1.7, h=0.44):
    """En gravnisje lukket med en gravhelle: et grunt innhugg med den store
    hella fra gravhelle-bildet på bunnen (hele hella over flaten, ikke flisen
    gjentatt). Gir omrisset."""
    prof = [(u - w / 2, v0), (u + w / 2, v0), (u + w / 2, v0 + h)]
    for k in range(1, 6):
        a = k / 6
        prof.append((u + w / 2 - w * a, v0 + h + 0.1 * math.sin(math.pi * a)))
    prof.append((u - w / 2, v0 + h))
    depth = 0.07
    K.recess(m, surf, prof, depth, "dressed", "slab", back=False)
    # Den store, glatte hella i bildet: kolonnene 476–884 og radene 612–824
    # av 1024 (2 m), med glTF-ets v nedover (v = 1 − rad i meter).
    c0, c1, r0, r1 = 476 / 512, 884 / 512, 612 / 512, 824 / 512
    top = v0 + h + 0.1
    uvs = [(c0 + (pu - (u - w / 2)) / w * (c1 - c0), 1.0 - (r1 - (pv - v0) / (top - v0) * (r1 - r0))) for pu, pv in prof]
    m.poly([surf.point(pu, pv, -depth) for pu, pv in prof], uvs, "slab", facing=surf.normal(u))
    return prof


def stair(m, rise, n):
    """Trinnene fra +z (gulvet i 0) opp til −z (gulvet i `rise`), over hele
    bredden (sidene står inne i murene)."""
    t = 2 * HALF / n
    xs = [-HALF + q for q in range(int(2 * HALF) + 1)]
    for k in range(n):
        z1 = HALF - k * t
        z0 = z1 - t
        y = rise * (k + 1) / n
        # Inntrinn og opptrinn i ruter på 1 m på tvers, så den øverste kanten
        # deler hjørner med gulvet i cella over (ingen T-kryss i skjøten).
        for xa, xb in zip(xs, xs[1:]):
            m.quad((xa, y, z0), (xb, y, z0), (xb, y, z1), (xa, y, z1), "dressed", facing=(0, 1, 0))
            m.quad((xa, -0.5, z1), (xb, -0.5, z1), (xb, y, z1), (xa, y, z1), "dressed", facing=(0, 0, 1))


def wall_rib(m, z, S, R, with_consoles=True):
    """Gjordbuen på tvers av gangen i z (over tønnehvelvet), som dør inn i
    murene, med en konsoll under på hver side."""
    K.barrel_rib(m, "z", z, 0.0, S, R, HALF, width=0.32, depth=0.15)
    if with_consoles:
        y = gang_h(FACE)
        for sx in (-1.0, 1.0):
            x0, x1 = sorted((sx * FACE, sx * (FACE - 0.16)))
            K.box(m, (x0, y - 0.32, z - 0.2), (x1, y - 0.02, z + 0.2), "dressed", faces=("-y", "+z", "-z", "+x" if sx < 0 else "-x"))


# --- Halvkantene -----------------------------------------------------------------


def opening(kind):
    """Omrisset av åpningen i halvkanten (u langs muren fra −1,6, v opp), og
    midten av den (til lysningens retning)."""
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


def half(kind, far=False, niches=False):
    """Halvkanten: murflaten i z = 0,4 mot +z med åpningen skåret ut, og
    lysningen rundt åpningen inn til kantlinja (z = 0), der naboens halvdel
    tar over. Sokkelen langs foten og karmen rundt døra og porten (ikke i
    fjernversjonen). `niches`: to rader gravnisjer (varianten b)."""
    m = K.MB()
    prof, centre = opening(kind)
    surf = surf_face()
    holes = [Polygon([(u + FACE, v) for u, v in prof])] if prof else []
    extra = K.MB()
    if niches and not far:
        # To rader gravnisjer: den nederste åpen med knokler, den øverste
        # lukket med en helle (som katakombene).
        (u1, a1, _, d1, w1), (u2, a2, _, _, w2) = NICHES
        n1 = K.niche(extra, surf, FACE + u1, a1, w=w1, h=0.48, depth=d1)
        n2 = sealed_niche(extra, surf, FACE + u2, a2, w=w2, h=0.44)
        holes += [Polygon(n1), Polygon(n2)]
    K.wall(m, surf, 2 * FACE, WALL_FOOT, WALL_TOP, holes, mat="wall")
    m.faces += extra.faces
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
    if far:
        return m
    # Sokkelen langs foten, mellom åpningene som går ned til gulvet.
    if kind != "gang":
        spans = K.free_spans(2 * FACE, holes[:1] if prof else [], 0.0, PLINTH_H)
        K.plinth(m, surf, spans, h=PLINTH_H, proud=PLINTH_OUT)
    # Karmene.
    if kind == "door":
        K.arch_trim(m, surf_mid(), DOOR_W, DOOR_SPRING, v0=0.0, band=0.3, proud=0.1, seed=3)
    elif kind == "gate":
        K.portal(m, surf_mid(), K.rect_profile(0.0, GATE_W, 0.0, GATE_H), mat="dressed", band=0.14, depth=0.1, rect=True)
    elif kind == "gang":
        # Buestaven rundt gangåpningen på romsida: på murflaten langs
        # profilen, ut fra åpningen, og innenfor muren i endene.
        ts = [t for t in vault_ts() if abs(t * HALF) <= FACE - 0.1]
        path = []
        for t in ts:
            u, v = t * HALF, gang_h(t * HALF)
            nu, nv = u / HALF ** 2, (v - GANG_S) / GANG_R ** 2  # ut fra ellipsen
            L = math.hypot(nu, nv)
            path.append((u + 0.11 * nu / L, v + 0.11 * nv / L, WALL_T))
        K.band(m, path, 0.2, 0.12, mat="dressed", down=lambda p: (0.0, 0.0, 1.0))
    return m


def half_solids(kind):
    """Kolliderne til halvkanten (bokser i halvkantens rom)."""
    def bx(x0, x1, y0, y1, z0=0.0, z1=WALL_T):
        return {"box": {"center": [(x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2], "size": [x1 - x0, y1 - y0, z1 - z0]}}
    if kind == "wall":
        return [bx(-FACE, FACE, WALL_FOOT, WALL_TOP)]
    if kind == "overmur":
        return [bx(-FACE, FACE, WALL_TOP - 0.05, WALL_TOP + 2.0)]
    if kind == "rail":
        return [bx(-FACE, FACE, -0.05, 1.0, 0.05, 0.35)]
    w, top = {"door": (DOOR_W, DOOR_SPRING + DOOR_W / 2), "gate": (GATE_W, GATE_H), "secret": (SECRET_W, SECRET_H)}.get(kind, (None, None))
    if w is None:
        return []
    return [bx(-FACE, -w / 2, WALL_FOOT, WALL_TOP), bx(w / 2, FACE, WALL_FOOT, WALL_TOP), bx(-w / 2, w / 2, top, WALL_TOP)]


# Gravnisjene i varianten b av muren (u midt i nisjen, fra, til, dybden).
NICHES = [(0.0, 0.55, 0.55 + 0.48 + 0.1, 0.3, 1.7), (0.0, 1.42, 1.42 + 0.44 + 0.1, 0.07, 1.7)]


def niche_solids():
    """Kolliderne til muren med gravnisjer: muren rundt, og bak hver nisje bare
    det som står igjen bak bunnen av den (ingen usynlig vegg foran nisjen)."""
    def bx(x0, x1, y0, y1, z0=0.0, z1=WALL_T):
        return {"box": {"center": [(x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2], "size": [x1 - x0, y1 - y0, z1 - z0]}}
    w = NICHES[0][4] / 2
    out = [bx(-FACE, -w, WALL_FOOT, WALL_TOP), bx(w, FACE, WALL_FOOT, WALL_TOP)]
    y = WALL_FOOT
    for _, v0, v1, depth, _ in NICHES:
        out.append(bx(-w, w, y, v0))
        out.append(bx(-w, w, v0, v1, 0.0, WALL_T - depth))
        y = v1
    out.append(bx(-w, w, y, WALL_TOP))
    return out


def arch_half(far=False):
    """Halve gjordbuen under krysshvelvene langs kantlinja (0,15 m inn i
    cella), fra hjørne til hjørne; den andre halvdelen står i naboen."""
    m = K.MB()
    ts = vault_ts() if not far else [t for k, t in enumerate(vault_ts()) if k % 2 == 0 or abs(t) > 0.99]
    path = [(HALF * t, room_h(HALF * t), 0.075) for t in ts]
    K.band(m, path, 0.15, 0.22, mat="rib", down=lambda p: (-p[0], ROOM_S - p[1], 0.0))
    return m


def overmur(far=False):
    """Muren over et lavere hvelv: fra murtoppen og 2 m opp (der takene på
    hver side er ulike; tas i bruk i fase 6)."""
    m = K.MB()
    surf = surf_face()
    K.wall(m, surf, 2 * FACE, WALL_TOP - 0.05, WALL_TOP + 2.0, [], mat="wall")
    back = K.Plane((FACE, 0.0), (-1.0, 0.0), (0.0, -1.0), u0=WALL_T)
    K.wall(m, back, 2 * FACE, WALL_TOP - 0.05, WALL_TOP + 2.0, [], mat="wall")
    m.quad((-FACE, WALL_TOP - 0.05, 0.0), (FACE, WALL_TOP - 0.05, 0.0), (FACE, WALL_TOP - 0.05, WALL_T), (-FACE, WALL_TOP - 0.05, WALL_T), "wall", facing=(0, -1, 0))
    return m


def rail(far=False):
    """Brystningen langs kanten (der gulvet på andre siden er lavere)."""
    m = K.MB()
    K.parapet(m, [(-FACE, 0.2), (FACE, 0.2)], 0.0, h=0.95, t=0.28, mat="dressed", cap="slab", ends=False)
    return m


def false_leaf():
    """Den falske veggen (`grav_falsk`): muren i åpningen, i flukt med
    murflaten rundt (fronten i z = 0, mot +z; motoren setter den i
    murflaten), med murens uv (`x` som i halvkanten, bildet gjentas hver 2. m),
    sokkelen over hele bredden og sprekken i fugene. Den synker i gulvet."""
    m = K.MB()
    w, h, t = SECRET_W, SECRET_H, WALL_T
    surf = K.Plane((-w / 2, 0.0), (1.0, 0.0), (0.0, 1.0), u0=HALF - w / 2)
    K.wall(m, surf, w, 0.0, h, [], mat="wall", extra=0.0)
    K.box(m, (-w / 2, 0.0, -t), (w / 2, h, 0.0), "wall", faces=("+x", "-x", "+y", "-z"))
    K.plinth(m, surf, [(0.0, w, 0.0)], h=PLINTH_H, proud=PLINTH_OUT)
    K.mortar_crack(m, 0.0, 11, x0=0.2, half=w / 2, top=1.9)
    return m


# --- Hjørnene --------------------------------------------------------------------


def corner(kind="plain", far=False):
    """Kvarthjørnet (origo i hjørnet, fyller ruta 0,4 × 0,4 mot +x og +z): de
    to flatene som vender inn i cella, med sokkelen. `out`: utvendig hjørne,
    avfaset over sokkelen, med sokkelen rundt hjørnet; `pilaster`: pilasteren under
    gjordbuen ved begge flatene (den som står mot en mur, synes ikke)."""
    m = K.MB()
    top = WALL_TOP + 0.05
    # Flatene deles i de samme høydene som murene ved siden av (rutene på
    # 1 m), så skjøten ikke får T-kryss som slipper gjennom en piksel.
    ys = [WALL_FOOT] + [float(k) for k in range(0, int(top) + 1)] + [top]
    if kind == "out":
        # Det utvendige hjørnet er avfaset over sokkelen (en hogd kant).
        c = 0.07
        y0 = PLINTH_H - 0.035
        m.quad((0.0, WALL_FOOT, WALL_T), (WALL_T, WALL_FOOT, WALL_T), (WALL_T, y0, WALL_T), (0.0, y0, WALL_T), "wall", facing=(0, 0, 1))
        m.quad((WALL_T, WALL_FOOT, 0.0), (WALL_T, WALL_FOOT, WALL_T), (WALL_T, y0, WALL_T), (WALL_T, y0, 0.0), "wall", facing=(1, 0, 0))
        for ya, yb in zip([y0] + [y for y in ys if y > y0], [y for y in ys if y > y0]):
            m.quad((0.0, ya, WALL_T), (WALL_T - c, ya, WALL_T), (WALL_T - c, yb, WALL_T), (0.0, yb, WALL_T), "wall", facing=(0, 0, 1))
            m.quad((WALL_T, ya, 0.0), (WALL_T, ya, WALL_T - c), (WALL_T, yb, WALL_T - c), (WALL_T, yb, 0.0), "wall", facing=(1, 0, 0))
            q = [(WALL_T - c, ya, WALL_T), (WALL_T, ya, WALL_T - c), (WALL_T, yb, WALL_T - c), (WALL_T - c, yb, WALL_T)]
            m.poly(q, [(0.0, ya), (c * 1.414, ya), (c * 1.414, yb), (0.0, yb)], "dressed", facing=(1, 0, 1))
        m.poly([(WALL_T - c, y0, WALL_T), (WALL_T, y0, WALL_T), (WALL_T, y0, WALL_T - c)], [(0, 0), (c, 0), (c, c)], "dressed", facing=(0, 1, 0))
    else:
        for y0, y1 in zip(ys, ys[1:]):
            m.quad((0.0, y0, WALL_T), (WALL_T, y0, WALL_T), (WALL_T, y1, WALL_T), (0.0, y1, WALL_T), "wall", facing=(0, 0, 1))
            m.quad((WALL_T, y0, 0.0), (WALL_T, y0, WALL_T), (WALL_T, y1, WALL_T), (WALL_T, y1, 0.0), "wall", facing=(1, 0, 0))
    if far:
        return m
    zf = K.Plane((0.0, WALL_T), (1.0, 0.0), (0.0, 1.0))  # flaten mot +z, u = x
    xf = K.Plane((WALL_T, 0.0), (0.0, 1.0), (1.0, 0.0))  # flaten mot +x, u = z
    if kind == "pilaster":
        for s in (zf, xf):
            K.plinth(m, s, [(PIL_W, WALL_T, 0.0)], h=PLINTH_H, proud=PLINTH_OUT)
            # Skaftet av hogde steiner, basen og kapitélet (impost) under buen.
            stone_column(m, s, 0.0, PIL_W, PLINTH_H + 0.06, ROOM_S - 0.25, PIL_OUT, seed=7 if s is zf else 11)
            pil = K.MB()
            K.box(pil, (0.0, WALL_FOOT, 0.0), (PIL_W + 0.06, PLINTH_H + 0.06, PIL_OUT + 0.08), "dressed", faces=("+z", "+x", "+y"))
            K.box(pil, (0.0, ROOM_S - 0.25, 0.0), (PIL_W + 0.08, ROOM_S, PIL_OUT + 0.1), "dressed", faces=("+z", "+x", "-y", "+y"))
            o = s.point(0.0, 0.0, 0.0)
            for pts, uvs, mat, smooth in pil.faces:
                q = [s.point(p[0], p[1], p[2]) for p in pts]
                f = K.sub(s.point(*K.newell(pts)), o)  # normalen gjennom samme avbildning
                m.poly(q, [K.auto_uv(p, f) for p in q], mat, facing=f, smooth=smooth)
    else:
        K.plinth(m, zf, [(0.0, WALL_T, 0.0)], h=PLINTH_H, proud=PLINTH_OUT)
        K.plinth(m, xf, [(0.0, WALL_T, 0.0)], h=PLINTH_H, proud=PLINTH_OUT)
    if kind == "out":
        # Sokkelen rundt hjørnet.
        K.box(m, (WALL_T, WALL_FOOT + 0.02, WALL_T), (WALL_T + PLINTH_OUT, PLINTH_H - 0.035, WALL_T + PLINTH_OUT), "dressed", faces=("+x", "+z", "+y"))
    return m


def pillar(far=False):
    m = K.MB()
    K.pillar(m, 0.0, 0.0, 0.0, ROOM_S, size=0.7)
    return m


def torch_piece():
    """Fakkelen i et fakkelfeste: origo i murflaten, ut mot +z."""
    m = K.MB()
    K.torch(m, (0.0, 0.0, 0.0), (0.0, 1.0))
    return m


# --- Cellene ---------------------------------------------------------------------


def romfelt(far=False, b=False):
    m = K.MB()
    if b and not far:
        slab_floor(m, 2.0, 2.0)
    else:
        floor(m)
    groin(m, ROOM_S, ROOM_R, k=1 if far else 3)
    if b and not far:
        K.groin_ribs(m, -HALF, HALF, -HALF, HALF, ROOM_S, ROOM_R, width=0.22, depth=0.15)
    return m


def gang(far=False, b=False):
    m = K.MB()
    if b and not far:
        slab_floor(m, 2.0, 2.0)
    else:
        floor(m)
    barrel(m, GANG_S, GANG_S, GANG_R, steps=1 if far else 4)
    if b and not far:
        wall_rib(m, 0.0, GANG_S, GANG_R)
    return m


def gangkryss(far=False):
    m = K.MB()
    floor(m)
    groin(m, GANG_S, GANG_R, k=1 if far else 3)
    return m


def trapp(rise, n, far=False):
    m = K.MB()
    stair(m, rise, n)
    barrel(m, GANG_S, GANG_S + rise, GANG_R, steps=2 if far else 4)
    return m


def nedgang(far=False):
    """Romfeltet med hullet til trappa ned. Sjakta i `kit_down` går inn under
    muren mot −z; et overligger under muren lukker toppen av den."""
    m = K.MB()
    hole = sbox(-DOWN_HOLE, -HALF - 0.01, DOWN_HOLE, DOWN_Z)
    floor(m, sbox(-HALF, -HALF, HALF, HALF).difference(hole))
    groin(m, ROOM_S, ROOM_R, k=1 if far else 3)
    K.box(m, (-1.5, -0.4, -HALF - 0.45), (1.5, 0.0, -FACE), "wall", faces=("-y", "+z"))
    return m


# --- Rotunden (sammensatt 3 × 3) -------------------------------------------------


def rot_walk_rects():
    """Det gangbare i rotunden som rektangler på 0,2 m (i rotundens rom),
    holdt 0,4 m fra muren, og gangene ut (midt på hver side)."""
    lim = ROT_R - 0.45
    disc = Point(0.0, 0.0).buffer(lim, 128)
    out = []
    step = 0.2
    n = int(round(6.0 / step))
    for gz in range(-n, n):
        row = []
        for gx in range(-n, n):
            x0, z0 = round(gx * step, 6), round(gz * step, 6)
            sq = sbox(x0 + 1e-6, z0 + 1e-6, x0 + step - 1e-6, z0 + step - 1e-6)
            inside = disc.contains(sq)
            for sx, sz in ((0, -1), (1, 0), (0, 1), (-1, 0)):
                # Gangene ut: 2,4 m brede (murflaten 1,6 minus 0,4 på hver side).
                if sx == 0:
                    g = sbox(-ROT_GANG + 0.4, min(0, 6 * sz), ROT_GANG - 0.4, max(0, 6 * sz))
                else:
                    g = sbox(min(0, 6 * sx), -ROT_GANG + 0.4, max(0, 6 * sx), ROT_GANG - 0.4)
                inside = inside or g.contains(sq)
            row.append(inside)
        # Slå sammen langs x.
        gx = 0
        while gx < len(row):
            if row[gx]:
                s = gx
                while gx < len(row) and row[gx]:
                    gx += 1
                out.append([(-n + s) * step, gz * step, (-n + gx) * step, (gz + 1) * step])
            else:
                gx += 1
    # Slå sammen rader med samme spenn.
    merged = []
    for r in out:
        for q in merged:
            if abs(q[0] - r[0]) < 1e-6 and abs(q[2] - r[2]) < 1e-6 and abs(q[3] - r[1]) < 1e-6:
                q[3] = r[3]
                break
        else:
            merged.append(list(r))
    return [[round(v, 3) for v in r] for r in merged]


def rotunde(far=False):
    """Rotunden 3 × 3 (origo midt i midtcella): et rundt rom med radius 5 m,
    muren med sokkel og åtte nisjer, kuppelen med ribber og sluttstein, og en
    gang ut midt på hver side til kvadratets kant (gangprofilen, så gangen
    eller overmuren på naboen passer). Mellom sirkelen og kvadratet er fjell."""
    m = K.MB()
    E = 3 * HALF  # 6: kanten av kvadratet
    seg = 32 if far else 64
    # Gulvet: sirkelen og gangene.
    # Gulvet i gangene går under murene til cellekanten (|x| ≤ 2), så kanten
    # mot naboen har de samme hjørnene som naboens gulv (ingen T-kryss).
    disc = Point(0.0, 0.0).buffer(ROT_R, seg // 4)
    shape = disc
    for sx, sz in ((0, -1), (1, 0), (0, 1), (-1, 0)):
        if sx == 0:
            shape = shape.union(sbox(-HALF, min(0, E * sz), HALF, max(0, E * sz)))
        else:
            shape = shape.union(sbox(min(0, E * sx), -HALF, max(0, E * sx), HALF))
    K.grid_face(m, K.Horizontal(0.0), shape, "floor", origin=(-E, -E))
    # Muren rundt: sylinderen innenfra, med gangåpningene skåret ut.
    circ = 2 * math.pi * ROT_R
    gang_ang = math.asin(ROT_GANG / ROT_R)
    a0 = -math.pi / 4  # sømmen midt mellom to ganger
    cyl = K.Cyl((0.0, 0.0), ROT_R, a0=a0, inward=True)
    holes = []
    for k in range(4):
        ac = k * math.pi / 2  # +x, +z, −x, −z
        u = ((ac - a0) % (2 * math.pi)) * ROT_R
        prof = [(u - gang_ang * ROT_R, WALL_FOOT - 0.01)]
        for t in vault_ts():
            if abs(t * HALF) <= ROT_GANG + 1e-9:
                prof.append((u + math.asin(max(-1, min(1, t * HALF / ROT_R))) * ROT_R, gang_h(t * HALF)))
        prof.append((u + gang_ang * ROT_R, WALL_FOOT - 0.01))
        holes.append(Polygon(prof))
    niches = []
    if not far:
        for k in range(4):
            u = ((k * math.pi / 2 + math.pi / 4 - a0) % (2 * math.pi)) * ROT_R
            tmp = K.MB()
            niches.append((tmp, K.niche(tmp, cyl, u, 0.6, w=1.6, h=0.5, depth=0.32, sealed=k % 2 == 1)))
    # Rutene i muren følger ringen i kuppelen (samme hjørner langs toppen),
    # så det blir ingen sprekk der muren møter kuppelen.
    K.wall(m, cyl, circ, WALL_FOOT, ROT_S + 0.3, holes + [Polygon(p) for _, p in niches], mat="wall", cell=circ / seg)
    for tmp, _ in niches:
        m.faces += tmp.faces
    # Lysningen i gangene: fra sirkelen ut til kanten av kvadratet, med
    # gangens tønnehvelv.
    for k in range(4):
        ac = k * math.pi / 2
        c, s = math.cos(ac), math.sin(ac)
        R = lambda x, y, z: (x * c - z * s, y, x * s + z * c)  # gangen langs +x dreid til ac
        x0 = math.sqrt(ROT_R ** 2 - ROT_GANG ** 2) - 0.02
        # Murene langs gangen går til kvarthjørnene, som kompilatoren setter
        # i de siste 0,4 m (som i en vanlig gang), med sokkelen.
        x1 = E - WALL_T
        for sz in (-1.0, 1.0):
            pts = [R(x0, WALL_FOOT, sz * ROT_GANG), R(x1, WALL_FOOT, sz * ROT_GANG), R(x1, gang_h(ROT_GANG), sz * ROT_GANG), R(x0, gang_h(ROT_GANG), sz * ROT_GANG)]
            m.poly(pts, [(x0, WALL_FOOT), (x1, WALL_FOOT), (x1, gang_h(ROT_GANG)), (x0, gang_h(ROT_GANG))], "wall", facing=R(0.0, 0.0, -sz))
            if not far:
                tmp = K.MB()
                K.plinth(tmp, K.Plane((x0, sz * ROT_GANG), (1.0, 0.0), (0.0, -sz)), [(0.0, x1 - x0, 0.0)], h=PLINTH_H, proud=PLINTH_OUT)
                for pts_, uvs_, mat_, sm_ in tmp.faces:
                    m.poly([R(*p_) for p_ in pts_], uvs_, mat_, facing=R(*K.newell(pts_)), smooth=sm_)
        ts = [t for t in vault_ts() if abs(t * HALF) <= ROT_GANG + 1e-9]
        arc = K.ellipse_arclen(HALF, GANG_R, ts)
        for i in range(len(ts) - 1):
            za, zb = ts[i] * HALF, ts[i + 1] * HALF
            # Hvelvet begynner der sirkelen skjærer gangprofilen.
            xa = math.sqrt(max(0.0, ROT_R ** 2 - za ** 2)) - 0.02
            xb = math.sqrt(max(0.0, ROT_R ** 2 - zb ** 2)) - 0.02
            pts = [R(xa, gang_h(za), za), R(E, gang_h(za), za), R(E, gang_h(zb), zb), R(xb, gang_h(zb), zb)]
            m.poly(pts, [(xa, arc[i]), (E, arc[i]), (E, arc[i + 1]), (xb, arc[i + 1])], "vault", facing=R(0.0, -1.0, -(za + zb) / 2), smooth=True)
        if not far:
            # Buestaven rundt gangåpningen i rotunden.
            path = [R(math.sqrt(max(0.0, ROT_R ** 2 - (t * HALF) ** 2)) - 0.06, gang_h(t * HALF), t * HALF) for t in ts]
            K.band(m, path, 0.12, 0.2, mat="dressed", down=lambda p, R=R: K.sub(R(0.0, GANG_S, 0.0), (p[0], p[1], p[2])))
    # Sokkelen langs muren mellom gangene.
    if not far:
        spans = []
        for k in range(4):
            u0 = ((k * math.pi / 2 - a0) % (2 * math.pi)) * ROT_R + gang_ang * ROT_R
            u1 = (((k + 1) * math.pi / 2 - a0) % (2 * math.pi) or 2 * math.pi) * ROT_R - gang_ang * ROT_R
            if u1 < u0:
                u1 += circ
            spans.append((u0, u1))
        for u0, u1 in spans:
            n = max(2, int((u1 - u0) / 0.5))
            for q in range(n):
                a, b = u0 + (u1 - u0) * q / n, u0 + (u1 - u0) * (q + 1) / n
                K.plinth(m, cyl, [(a, b, 0.0)], h=PLINTH_H, proud=PLINTH_OUT)
    # Kuppelen over muren, med en list ved foten, ribber og sluttstein.
    h = K.dome(m, (0.0, 0.0), ROT_R, ROT_S + 0.3, ROT_CROWN, rings=6 if far else 10, seg=seg)
    if not far:
        K.dome_ribs(m, (0.0, 0.0), ROT_R, h, count=8, a0=math.pi / 4)
        # Gesimsen der kuppelen begynner.
        ring_in = K.circle((0.0, 0.0), ROT_R - 0.14, seg)
        ring_out = K.circle((0.0, 0.0), ROT_R + 0.01, seg)
        y0, y1 = ROT_S + 0.12, ROT_S + 0.3
        for i in range(seg):
            j = (i + 1) % seg
            a, b = ring_in[i], ring_in[j]
            m.poly([(a[0], y0, a[1]), (b[0], y0, b[1]), (b[0], y1, b[1]), (a[0], y1, a[1])], [(i * 0.5, y0), ((i + 1) * 0.5, y0), ((i + 1) * 0.5, y1), (i * 0.5, y1)], "dressed", facing=(-(a[0] + b[0]), 0, -(a[1] + b[1])))
            c, d = ring_out[i], ring_out[j]
            m.poly([(c[0], y0, c[1]), (d[0], y0, d[1]), (b[0], y0, b[1]), (a[0], y0, a[1])], [(c[0], c[1]), (d[0], d[1]), (b[0], b[1]), (a[0], a[1])], "dressed", facing=(0, -1, 0))
    # Toppen av murene over gangene og hvelvet ut mot kanten: fylles av
    # overmuren (`grav_vegg_gang`) eller gangen på naboens side.
    return m


def rot_solids():
    """Kolliderne i rotunden: gulvet, muren rundt (bokser langs sirkelen mellom
    gangene) og fjellet i hjørnene."""
    E = 3 * HALF
    out = [{"box": {"center": [0.0, -0.25, 0.0], "size": [2 * E, 0.5, 2 * E]}}]
    n = 48
    for k in range(n):
        a = 2 * math.pi * (k + 0.5) / n
        c, s = math.cos(a), math.sin(a)
        # Hopp over gangene (midt på hver side).
        if min(abs(((a - q * math.pi / 2 + math.pi) % (2 * math.pi)) - math.pi) for q in range(4)) < math.asin(ROT_GANG / ROT_R) + math.pi / n:
            continue
        L = 2 * math.pi * (ROT_R + 0.3) / n + 0.05
        # Bak nisjene (på skrå mellom gangene) står muren bare bak bunnen.
        niche = min(abs(((a - math.pi / 4 - q * math.pi / 2 + math.pi) % (2 * math.pi)) - math.pi) for q in range(4)) < 0.8 / ROT_R + math.pi / n
        parts = [(WALL_FOOT, 0.6, ROT_R), (0.6, 1.2, ROT_R + 0.32), (1.2, 4.05, ROT_R)] if niche else [(WALL_FOOT, 4.05, ROT_R)]
        for y0, y1, r0 in parts:
            rc = (r0 + ROT_R + 0.6) / 2
            out.append({"box": {"center": [round(c * rc, 4), round((y0 + y1) / 2, 4), round(s * rc, 4)], "size": [round(ROT_R + 0.6 - r0, 4), round(y1 - y0, 4), round(L, 4)], "yaw": round(-a, 5)}})
    # Murene langs gangene ut, fra sirkelen til kanten.
    x0 = math.sqrt(ROT_R ** 2 - ROT_GANG ** 2) - 0.1
    for k in range(4):
        c, s = round(math.cos(k * math.pi / 2)), round(math.sin(k * math.pi / 2))
        for sz in (-1, 1):
            cx, cz = (x0 + E) / 2, sz * (ROT_GANG + 0.5)
            sx_, sz_ = E - x0, 1.0
            ctr = [cx * c - cz * s, 2.0, cx * s + cz * c]
            size = [sx_, 4.1, sz_] if c != 0 else [sz_, 4.1, sx_]
            out.append({"box": {"center": [round(v, 4) for v in ctr], "size": [round(v, 4) for v in size]}})
    return out


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


OPEN, GANG, CLOSED, INNER = face("open", "room"), face("gang", "gang"), face("closed", "gang"), face("inner", "room")
FLAT = [{"rect": [-HALF, -HALF, HALF, HALF], "y": [0.0, 0.0]}]
# Pyntefestene på gulvet i et romfelt (hjørnene, et stykke inn) og i nisjene.
FLOOR_SLOTS = [{"kind": "pynt_gulv", "pos": [sx * 1.05, 0.0, sz * 1.05], "yaw": 0.0} for sx in (-1, 1) for sz in (-1, 1)]
TORCH_MOUNT = {"kind": "fakkel", "pos": [0.0, TORCH_Y, WALL_T], "yaw": 0.0}


class Model:
    """En modell i byggesettet: nær- og fjernversjonen, og omgivelsene den
    bakes i (`kit_bake.context`)."""

    def __init__(self, name, near, far, ctx, seams):
        self.name, self.near, self.far, self.ctx, self.seams = name, near, far, ctx, seams


def build():
    """(manifestet, modellene)."""
    pieces, models = [], {}

    def model(name, fn, ctx, seams, **kw):
        if name not in models:
            models[name] = Model(name, fn(far=False, **kw), fn(far=True, **kw), ctx, seams)
        return name

    def variants(*vs):
        out = []
        for v in vs:
            vid, m, w = v[:3]
            d = {"id": vid, "model": m, "weight": w}
            if len(v) > 3:
                d["solids"] = v[3]  # en variant med en annen form har egne kollidere
            out.append(d)
        return out

    def cell(pid, mdl, faces, clear, floors=(0.0, 0.0, 0.0, 0.0), walk=FLAT, solids=None, mounts=(), top=WALL_TOP, var=None, extra=None):
        p = {"id": f"{THEME}/{pid}", "model": mdl, "kind": "cell", "theme": THEME, "v": VERSION, "faces": faces, "floors": list(floors),
             "clear": clear, "bottom": -0.5, "top": top, "walk": walk, "solids": solids or [floor_box()], "mounts": list(mounts), "mirror": True}
        if var:
            p["variants"] = var
        if extra:
            p.update(extra)
        pieces.append(p)

    cellseams = "cell"
    rf = model("grav_romfelt", romfelt, "cell", cellseams)
    rfb = model("grav_romfelt_b", lambda far: romfelt(far, b=True), "cell", cellseams)
    cell("romfelt", rf, [OPEN] * 4, ROOM_CLEAR, mounts=FLOOR_SLOTS, var=variants(("a", rf, 3.0), ("b", rfb, 1.0)))
    g = model("grav_gang", gang, "gang", cellseams)
    gb = model("grav_gang_b", lambda far: gang(far, b=True), "gang", cellseams)
    gvar = variants(("a", g, 3.0), ("b", gb, 1.0))
    cell("gang", g, [GANG, CLOSED, GANG, CLOSED], GANG_CLEAR, var=gvar)
    cell("gang_blind", g, [GANG, CLOSED, CLOSED, CLOSED], GANG_CLEAR, var=gvar)
    gk = model("grav_gangkryss", gangkryss, "gangkryss", cellseams)
    cell("gang_hjorne", gk, [GANG, GANG, CLOSED, CLOSED], GANG_CLEAR)
    cell("gang_t", gk, [GANG, GANG, CLOSED, GANG], GANG_CLEAR)
    cell("gang_kryss", gk, [GANG] * 4, GANG_CLEAR)
    for pid, name, rise, n in (("halvtrapp", "grav_halvtrapp", 1.0, 6), ("trapp2", "grav_trapp2", 2.0, 12)):
        mdl = model(name, lambda far, rise=rise, n=n: trapp(rise, n, far), "gang", cellseams)
        cell(pid, mdl, [GANG, CLOSED, GANG, CLOSED], GANG_CLEAR, floors=(rise, 0.0, 0.0, 0.0),
             walk=[{"rect": [-HALF, -HALF, HALF, HALF], "y": [rise, 0.0]}], solids=[ramp(rise)], top=WALL_TOP + rise)
    hole_walk = [{"rect": [-HALF, -HALF, -DOWN_HOLE, HALF], "y": [0.0, 0.0]}, {"rect": [DOWN_HOLE, -HALF, HALF, HALF], "y": [0.0, 0.0]},
                 {"rect": [-DOWN_HOLE, DOWN_Z, DOWN_HOLE, HALF], "y": [0.0, 0.0]}]
    hole_solids = [{"box": {"center": [(-HALF - DOWN_HOLE) / 2, -0.25, 0.0], "size": [HALF - DOWN_HOLE, 0.5, 2 * HALF]}},
                   {"box": {"center": [(HALF + DOWN_HOLE) / 2, -0.25, 0.0], "size": [HALF - DOWN_HOLE, 0.5, 2 * HALF]}},
                   {"box": {"center": [0.0, -0.25, (HALF + DOWN_Z) / 2], "size": [2 * DOWN_HOLE, 0.5, HALF - DOWN_Z]}}]
    ng = model("grav_nedgang", nedgang, "cell", cellseams)
    cell("nedgang", ng, [CLOSED, OPEN, OPEN, OPEN], ROOM_CLEAR, walk=hole_walk, solids=hole_solids,
         mounts=[{"kind": "down", "pos": [0.0, 0.0, DOWN_Z], "yaw": 0.0}])
    # Rotunden: ni celler; midtcella bærer modellen, kolliderne og det
    # gangbare for hele rotunden (i midtcellas rom).
    rot = model("grav_rotunde", rotunde, "rotunde", "rotunde")
    foot = []
    for dj in (-1, 0, 1):
        for di in (-1, 0, 1):
            fs = []
            for w, (oi, oj) in enumerate(((0, -1), (1, 0), (0, 1), (-1, 0))):
                if -1 <= di + oi <= 1 and -1 <= dj + oj <= 1:
                    fs.append(INNER)
                elif (di, dj) == (oi, oj) or (di == 0 and oj != 0 and dj == oj) or (dj == 0 and oi != 0 and di == oi):
                    fs.append(GANG)
                else:
                    fs.append(CLOSED)
            foot.append({"at": [di, dj], "faces": fs})
    # Fakkelfestene over nisjene (på skrå mellom gangene), mot midten.
    rot_mounts = []
    for k in range(4):
        a = k * math.pi / 2 + math.pi / 4
        rot_mounts.append({"kind": "fakkel", "pos": [ROT_R * math.cos(a), TORCH_Y, ROT_R * math.sin(a)], "yaw": math.atan2(-math.cos(a), -math.sin(a))})
    cell("rotunde", rot, [INNER] * 4, ROT_S - 0.4, walk=[{"rect": r, "y": [0.0, 0.0]} for r in rot_walk_rects()], solids=rot_solids(),
         mounts=rot_mounts, top=ROT_CROWN + 0.6, extra={"footprint": foot})

    def edge(pid, mdl, kind, width=0.0, height=0.0, mounts=(), var=None):
        p = {"id": f"{THEME}/{pid}", "model": mdl, "kind": "edge", "theme": THEME, "v": VERSION, "opening": kind, "width": width, "height": height,
             "solids": half_solids(kind), "mounts": list(mounts), "mirror": True}
        if var:
            p["variants"] = var
        pieces.append(p)

    v = model("grav_vegg", lambda far: half("wall", far), "edge", "edge")
    vb = model("grav_vegg_b", lambda far: half("wall", far, niches=True), "edge", "edge")
    # Festene er felles for variantene: fakkelen midt på muren, og pynt
    # (sarkofag, bein, kister) langs foten.
    edge("vegg", v, "wall", mounts=[TORCH_MOUNT, {"kind": "pynt_vegg", "pos": [0.0, 0.0, WALL_T + 0.6], "yaw": 0.0}], var=variants(("a", v, 2.0), ("b", vb, 1.0, niche_solids())))
    edge("vegg_gang", model("grav_vegg_gang", lambda far: half("gang", far), "edge", "edge"), "gang", 2 * FACE, gang_h(0.0))
    edge("vegg_dor", model("grav_vegg_dor", lambda far: half("door", far), "edge", "edge"), "door", DOOR_W, DOOR_SPRING + DOOR_W / 2)
    edge("vegg_port", model("grav_vegg_port", lambda far: half("gate", far), "edge", "edge"), "gate", GATE_W, GATE_H)
    edge("vegg_falsk", model("grav_vegg_falsk", lambda far: half("secret", far), "edge", "edge"), "secret", SECRET_W, SECRET_H)
    edge("gjordbue", model("grav_gjordbue", arch_half, "arch", "edge"), "arch")
    edge("overmur", model("grav_overmur", overmur, "edge", "edge"), "overmur", 0.0, 2.0)
    edge("rekkverk", model("grav_rekkverk", rail, "rail", "edge"), "rail", 0.0, 1.0)
    pieces.append({"id": f"{THEME}/falsk", "model": model("grav_falsk", lambda far: false_leaf(), "leaf", "none"), "kind": "leaf", "theme": THEME, "v": VERSION,
                   "opening": "secret", "width": SECRET_W, "height": SECRET_H, "mirror": True})
    csol = [{"box": {"center": [WALL_T / 2, (WALL_FOOT + WALL_TOP) / 2, WALL_T / 2], "size": [WALL_T, WALL_TOP - WALL_FOOT, WALL_T]}}]
    for pid, name, kind, fits in (("hjorne", "grav_hjorne", "plain", ["inner", "wall"]), ("hjorne_ut", "grav_hjorne_ut", "out", ["outer"]),
                                  ("pilaster", "grav_pilaster", "pilaster", ["pilaster"])):
        sol = list(csol)
        if kind == "pilaster":
            sol += [{"box": {"center": [PIL_W / 2, ROOM_S / 2, WALL_T + PIL_OUT / 2], "size": [PIL_W, ROOM_S, PIL_OUT]}},
                    {"box": {"center": [WALL_T + PIL_OUT / 2, ROOM_S / 2, PIL_W / 2], "size": [PIL_OUT, ROOM_S, PIL_W]}}]
        pieces.append({"id": f"{THEME}/{pid}", "model": model(name, lambda far, kind=kind: corner(kind, far), "corner", "corner"), "kind": "corner", "theme": THEME,
                       "v": VERSION, "fits": fits, "solids": sol, "mirror": True})
    oct_ = [[x, y, z] for (x, z) in K.chamfered((0.0, 0.0), 0.7, 0.12) for y in (0.0, ROOM_S)]
    pieces.append({"id": f"{THEME}/soyle", "model": model("grav_soyle", pillar, "pillar", "none"), "kind": "pillar", "theme": THEME, "v": VERSION, "radius": 0.52,
                   "solids": [{"hull": oct_}], "mirror": True})
    pieces.append({"id": f"{THEME}/fakkel", "model": model("grav_fakkel", lambda far: torch_piece(), "torch", "none"), "kind": "prop", "theme": THEME, "v": VERSION,
                   "mounts": [{"kind": "lys", "pos": list(FLAME), "yaw": 0.0}], "mirror": True})
    return pieces, models


def kit_extras(pieces, name):
    """`extras.kit` i glb-fila: manifestet for bitene som bruker modellen
    (som modell eller variant)."""
    from kit_common import rounded

    return [rounded(p) for p in pieces if p["model"] == name or any(v["model"] == name for v in p.get("variants", []))]


def main():
    args = sys.argv[1:]
    pieces, models = build()
    write_manifest(MANIFEST, THEME, VERSION, pieces)
    print(f"{os.path.relpath(MANIFEST, ROOT)}: {len(pieces)} biter, {len(models)} modeller")
    for name, md in models.items():
        print(f"  {name}: {md.near.tris()} trekanter nær, {md.far.tris()} fjern")
    if "--json" in args:
        return
    import kit_bake

    only = [a for a in args if not a.startswith("-")]
    if "--check" in args:
        bad = kit_bake.check_all(pieces, models, only)
        raise SystemExit(1 if bad else 0)
    kit_bake.export_all(pieces, models, OUT, only)


if __name__ == "__main__":
    main()
