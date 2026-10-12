"""Kar til hyllene og bordene: krukke, kanne, mugge, bolle, flaske, krus,
tinnfat og små tønner. Hvert kar dreies fra en profil, og størrelsen, bredden,
fargen og formen varierer med `rng` (eieren, runde 5: «mer variasjon i form
og størrelse på det som står på hylla»). `row` fyller en hylle eller en
bordflate med en blanding.

    import vessels
    vessels.row(p, rng, x0, x1, y, z, depth, max_h)            # langs x
    vessels.row(p, rng, y0, y1, x, z, depth, max_h, along="y")  # langs y
"""
import math
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bmesh  # noqa: E402
import kit  # noqa: E402
from town import layer, plain, smooth  # noqa: E402


def lathe(name, prof, mat, n=10, at=(0.0, 0.0, 0.0), lip=0.0):
    """Et omdreiningslegeme fra profilen [(r, z), …] nedenfra og opp, med
    bunn og topp lukket (radius 0 gir en spiss); `lip` > 0 gir en åpning
    som går så langt ned i karet (mørk innside, så det ser hult ut). Uv i
    meter rundt og oppover."""
    if lip > 0:
        r_top, z_top = prof[-1]
        prof = prof + [(r_top * 0.86, z_top), (r_top * 0.8, z_top - lip)]
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.verify()
    rings = [[bm.verts.new((at[0] + r * math.cos(2 * math.pi * i / n), at[1] + r * math.sin(2 * math.pi * i / n), at[2] + z)) for i in range(n)] if r > 1e-4 else [bm.verts.new((at[0], at[1], at[2] + z))] for r, z in prof]
    circ = 2 * math.pi * max(r for r, _ in prof)
    for lo, hi in zip(rings, rings[1:]):
        for i in range(n):
            j = (i + 1) % n
            if len(lo) == 1 and len(hi) == 1:
                continue
            if len(lo) == 1:
                vs, ks = (lo[0], hi[j], hi[i]), (i + 0.5, j or n, i)
            elif len(hi) == 1:
                vs, ks = (lo[i], lo[j], hi[0]), (i, j or n, i + 0.5)
            else:
                vs, ks = (lo[i], lo[j], hi[j], hi[i]), (i, j or n, j or n, i)
            f = bm.faces.new(vs)
            for lp, k in zip(f.loops, ks):
                lp[uvl].uv = (k / n * circ, lp.vert.co.z)
    for ring, top in ((rings[0], False), (rings[-1], True)):
        if len(ring) > 1:
            f = bm.faces.new(ring if top else ring[::-1])
            for lp in f.loops:
                lp[uvl].uv = (lp.vert.co.x, lp.vert.co.y)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    return smooth(kit._object(name, bm, mat))


def clays(rng):
    """Leirfarger: brent rød, gulbrun, grå steintøy, hvit glasert, grønn og
    brun glasur."""
    # (Pusslagene med tone, til eieren har malt et eget leirlag.)
    return [layer("plaster_red", (0.85, 0.55, 0.42)), layer("plaster_red", (0.65, 0.4, 0.3)), layer("plaster", (0.95, 0.75, 0.5)), layer("plaster", (0.75, 0.72, 0.68)), layer("plaster_white", (0.95, 0.92, 0.85)), layer("plaster_white", (0.55, 0.68, 0.45)), layer("plaster_white", (0.45, 0.55, 0.75))]


def handle(x, y, z0, z1, out, r, mat, side=1.0):
    """En hank: en bue av små pinner ut fra siden av karet."""
    pts = []
    for k in range(6):
        t = k / 5
        a = math.pi * t
        pts.append((x + side * (r + out * math.sin(a)), y, z0 + (z1 - z0) * t))
    for a, b in zip(pts, pts[1:]):
        kit.stick("handle", a, b, 0.018, 0.018, mat, bevel=0)


def crock(x, y, z, r, h, mat, lid=None):
    """Krukke: rett og kraftig, med en kant; kanskje et tøy bundet over."""
    lathe("crock", [(r * 0.85, 0.0), (r, h * 0.2), (r * 0.98, h * 0.8), (r * 0.9, h * 0.92), (r * 0.95, h)], mat, at=(x, y, z), lip=0.0 if lid else h * 0.3)
    if lid:
        lathe("crock_cloth", [(r * 1.02, h - 0.02), (r * 0.9, h + 0.02), (0.0, h + 0.035)], lid, n=10, at=(x, y, z))


def jar(x, y, z, r, h, mat):
    """Bukete krukke med hals."""
    lathe("jar", [(r * 0.6, 0.0), (r, h * 0.4), (r * 0.85, h * 0.75), (r * 0.5, h * 0.88), (r * 0.6, h)], mat, at=(x, y, z), lip=h * 0.2)


def jug(x, y, z, r, h, mat):
    """Mugge: buket kropp, smal hals, utbrettet kant og en hank."""
    lathe("jug", [(r * 0.7, 0.0), (r, h * 0.35), (r * 0.8, h * 0.68), (r * 0.42, h * 0.82), (r * 0.55, h)], mat, at=(x, y, z), lip=h * 0.15)
    handle(x, y, z + h * 0.4, z + h * 0.85, r * 0.45, r * 0.75, mat)


def pitcher(x, y, z, r, h, mat):
    """Kanne: høy og slank, utvidet mot toppen."""
    lathe("pitcher", [(r * 0.75, 0.0), (r, h * 0.3), (r * 0.7, h * 0.7), (r * 0.8, h)], mat, at=(x, y, z), lip=h * 0.15)
    handle(x, y, z + h * 0.3, z + h * 0.9, r * 0.5, r * 0.75, mat)


def bowl(x, y, z, r, h, mat):
    lathe("bowl", [(r * 0.5, 0.0), (r * 0.9, h * 0.6), (r, h)], mat, n=12, at=(x, y, z), lip=h * 0.7)


def bottle(x, y, z, r, h, mat):
    """Flaske: rund kropp, lang hals og en kork."""
    lathe("bottle", [(r * 0.85, 0.0), (r, h * 0.12), (r, h * 0.5), (r * 0.35, h * 0.68), (r * 0.3, h * 0.92), (0.0, h * 0.92)], mat, n=8, at=(x, y, z))
    lathe("cork", [(r * 0.27, h * 0.9), (r * 0.3, h), (0.0, h)], layer("leather", (0.85, 0.65, 0.45)), n=6, at=(x, y, z))


def mug(x, y, z, r, h, mat):
    """Krus: rett, med hank."""
    lathe("mug", [(r * 0.95, 0.0), (r, h)], mat, n=10, at=(x, y, z), lip=h * 0.8)
    handle(x, y, z + h * 0.2, z + h * 0.85, r * 0.55, r * 0.95, mat)


def plate(x, y, z, r, mat, n=3):
    """En stabel tinnfat."""
    for k in range(n):
        lathe("plate", [(r * 0.7, 0.0), (r, 0.015), (r * 0.95, 0.02), (0.0, 0.012)], mat, n=14, at=(x, y, z + k * 0.018))


def row(p, rng, a0, a1, c, z, depth, max_h, along="x", kinds=None, gap=(0.03, 0.09)):
    """Fyll en hylle eller bordflate fra a0 til a1 (langs `along`, på tvers
    i c) med en blanding av kar i ulike former og størrelser. `kinds` velger
    hvilke (standard: alt av leire og tre), `gap` mellomrommet (tettere på
    en full hylle)."""
    clay = clays(rng)
    glass = [plain("glass_green", (0.08, 0.2, 0.1), 0.1), plain("glass_brown", (0.2, 0.1, 0.05), 0.1)]
    wood = [layer("oak_planed", (0.85, 0.7, 0.55)), layer("oak_planed", (0.6, 0.45, 0.35))]
    pewter = layer("iron", (0.85, 0.85, 0.82), 0.35)
    cloth = layer("linen", (0.9, 0.85, 0.72))
    kinds = kinds or ["crock", "jar", "jug", "pitcher", "bowl", "mug", "crock_lid", "keg"]
    u = a0 + 0.03
    last = None
    while u < a1 - 0.08:
        k = rng.choice([x for x in kinds if x != last] or kinds)
        last = k
        rmax = min(depth * 0.45, 0.16)
        r = rng.uniform(0.045, rmax)
        h = min(max_h, rng.uniform(0.1, 0.34))
        if u + 2 * r > a1:
            break
        cu = u + r
        x, y = (cu, c) if along == "x" else (c, cu)
        if k == "crock":
            crock(x, y, z, r, h * 0.8, rng.choice(clay))
        elif k == "crock_lid":
            crock(x, y, z, r, h * 0.7, rng.choice(clay), lid=cloth)
        elif k == "jar":
            jar(x, y, z, r, h, rng.choice(clay))
        elif k == "jug":
            jug(x, y, z, r * 0.85, h, rng.choice(clay))
        elif k == "pitcher":
            pitcher(x, y, z, r * 0.75, min(max_h, h * 1.2), rng.choice(clay + [pewter]))
        elif k == "bowl":
            bowl(x, y, z, r * 1.1, min(0.09, h * 0.4), rng.choice(clay + wood))
        elif k == "mug":
            mug(x, y, z, min(r, 0.05), min(0.13, h * 0.6), rng.choice(wood + [pewter]))
        elif k == "bottle":
            bottle(x, y, z, min(r, 0.055), h, rng.choice(glass))
        elif k == "plate":
            plate(x, y, z, min(r * 1.4, 0.13), pewter, n=rng.randint(2, 4))
        elif k == "keg":
            ob = kit.barrel(p, x, y, z, r=min(r, 0.1), h=min(max_h, 0.24))
        u = cu + r + rng.uniform(*gap)


def sack(x, y, z, w, h, mat, yaw=0.0, seed=0, slump=0.0):
    """Sekk: buker ut, synker sammen nederst, snørt hals med en dusk og
    folder i stoffet (eieren, runde 5: «ser ikke ut som sekker»). `slump`
    lener toppen til siden."""
    import random
    from mathutils import Matrix
    rng = random.Random(seed)
    prof = [(w * 0.82, 0.0), (w * 1.0, h * 0.18), (w * 0.98, h * 0.5), (w * 0.8, h * 0.72), (w * 0.38, h * 0.84), (w * 0.14, h * 0.88), (w * 0.2, h * 0.93), (w * 0.26, h * 0.99), (0.0, h * 0.97)]
    ob = lathe("sack", prof, mat, n=14, at=(0.0, 0.0, 0.0))
    # Flatere enn rund, med folder og litt skjevhet.
    phase = rng.uniform(0, 6)
    for v in ob.data.vertices:
        a = math.atan2(v.co.y, v.co.x)
        t = v.co.z / h
        fold = 1.0 + 0.06 * math.sin(a * 5 + phase) * math.sin(t * math.pi)
        v.co.x *= fold
        v.co.y *= fold * 0.78
        v.co.x += slump * t * t * h * 0.25
    t = 0.88
    tie = lathe("sack_tie", [(w * 0.16, h * 0.86), (w * 0.17, h * 0.88), (w * 0.16, h * 0.9)], layer("leather", (0.6, 0.45, 0.3)), n=10, at=(slump * t * t * h * 0.25, 0.0, 0.0))
    for o in (ob, tie):
        o.matrix_world = Matrix.Translation((x, y, z)) @ Matrix.Rotation(yaw, 4, "Z") @ o.matrix_world
    return ob


def horn_lantern(p, x, y, z, chain_to=None):
    """Hornlykt: bunn og tak av jern, fire hjørnestolper, ruter av horn som
    gløder, et kjegletak med ring, og kjetting opp til `chain_to` (z)."""
    iron = p.iron
    horn = plain("glow_horn", (0.3, 0.19, 0.08), 0.5)
    s = 0.11
    kit.box("lantern_base", (x - s - 0.015, y - s - 0.015, z), (x + s + 0.015, y + s + 0.015, z + 0.035), iron, bevel=0.008)
    kit.box("lantern_horn", (x - s + 0.01, y - s + 0.01, z + 0.035), (x + s - 0.01, y + s - 0.01, z + 0.29), horn, bevel=0.0)
    for sx in (-1, 1):
        for sy in (-1, 1):
            kit.box("lantern_post", (x + sx * s - 0.013, y + sy * s - 0.013, z + 0.03), (x + sx * s + 0.013, y + sy * s + 0.013, z + 0.3), iron, bevel=0.0)
    for zz in (z + 0.12, z + 0.21):
        for a, b in (((x - s, y - s), (x + s, y - s)), ((x - s, y + s), (x + s, y + s)), ((x - s, y - s), (x - s, y + s)), ((x + s, y - s), (x + s, y + s))):
            kit.stick("lantern_bar", (a[0], a[1], zz), (b[0], b[1], zz), 0.012, 0.012, iron, bevel=0)
    lathe("lantern_roof", [(s * 1.35, 0.0), (s * 1.3, 0.025), (s * 0.3, 0.13), (0.0, 0.14)], iron, n=8, at=(x, y, z + 0.3))
    lathe("lantern_ring", [(0.035, 0.0), (0.04, 0.01), (0.035, 0.05), (0.025, 0.05), (0.03, 0.01), (0.0, 0.0)], iron, n=8, at=(x, y, z + 0.44))
    if chain_to is not None and chain_to > z + 0.5:
        k, zz = 0, z + 0.49
        while zz < chain_to:
            kit.stick("chain_link", (x, y, zz), (x, y, min(chain_to, zz + 0.06)), 0.018 if k % 2 else 0.008, 0.008 if k % 2 else 0.018, iron, bevel=0)
            zz += 0.055
            k += 1


def candle_dish(p, x, y, z, h=0.14):
    """Et talglys i en jernskål med hank, med rennende talg og en tydelig
    flamme (eieren, runde 5: «jeg ser ikke at det er et lys med en flamme»)."""
    iron = p.iron
    wax = layer("plaster_white", (0.95, 0.9, 0.75))
    lathe("candle_dish", [(0.06, 0.0), (0.075, 0.012), (0.08, 0.02), (0.07, 0.02), (0.0, 0.008)], iron, n=12, at=(x, y, z))
    handle(x, y, z + 0.005, z + 0.025, 0.035, 0.07, iron)
    lathe("candle", [(0.024, 0.0), (0.027, 0.015), (0.022, h * 0.7), (0.02, h), (0.0, h)], wax, n=8, at=(x, y, z + 0.012))
    lathe("candle_drip", [(0.03, 0.0), (0.028, 0.01), (0.0, 0.012)], wax, n=8, at=(x + 0.006, y, z + 0.012))
    kit.stick("wick", (x, y, z + 0.012 + h), (x, y, z + 0.032 + h), 0.004, 0.004, layer("iron", (0.2, 0.18, 0.16)), bevel=0)
    lathe("candle_flame", [(0.0, 0.0), (0.012, 0.012), (0.014, 0.025), (0.008, 0.045), (0.0, 0.065)], p.fire, n=8, at=(x, y, z + 0.022 + h))


def bucket(p, x, y, z, r=0.2, h=0.32, fill=None, fill_depth=0.06):
    """Bøtte: staver som smalner nedover, to jernbånd, en hank i jern som
    står opp, og innholdet (`fill`) et stykke under kanten (eieren, runde 5:
    «ser ikke ut som en bøtte»)."""
    wood = layer("cladding", (0.75, 0.62, 0.5))
    iron = p.iron
    lathe("bucket", [(r * 0.8, 0.0), (r, h)], wood, n=12, at=(x, y, z), lip=h * 0.85)
    for t in (0.18, 0.78):
        rr = r * (0.8 + 0.2 * t) + 0.008
        lathe("bucket_hoop", [(rr, h * t - 0.015), (rr + 0.004, h * t), (rr, h * t + 0.015)], iron, n=12, at=(x, y, z))
    # Hanken: ører på sidene og en bue over.
    for sx in (-1, 1):
        kit.box("bucket_ear", (x + sx * r - 0.015, y - 0.02, z + h - 0.06), (x + sx * r + 0.015, y + 0.02, z + h + 0.02), iron, bevel=0)
    pts = [(x - r * math.cos(math.pi * k / 8), y, z + h + 0.01 + r * 0.9 * math.sin(math.pi * k / 8)) for k in range(9)]
    for a, b in zip(pts, pts[1:]):
        kit.stick("bucket_handle", a, b, 0.012, 0.012, iron, bevel=0)
    if fill is not None:
        lathe("bucket_fill", [(r * 0.93, 0.0), (r * 0.6, 0.03), (0.0, 0.05)], fill, n=12, at=(x, y, z + h - fill_depth))
