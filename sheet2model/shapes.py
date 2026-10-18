"""Parametriske nett for gjenstandene: geometrien ER prioren (skarpe kanter, liten fas), ikke dybdevolumet.

`build(job, prior)` gir delene [(navn, v, f)] i verdenspiksler for prioren `props.fit` fant: `box` (boks med fas),
`chest` (kassa og lokket som to deler: boks + halvsylinder om z), `cylinder`/`barrel`/`revolve` (dreiekropp om y med
profil), `ring` (kum med hull), `cylinder_x` (liggende sylinder). `smart_uv` legger uv-kart (bpy), og
`surface_samples` gir tette punkter med normaler til z-bufferen i teksturprojeksjonen (den lave modellen alene har
for få hjørner). Fasen styres med `opt.bevel` (piksler, standard 1 % av bredden, minst 3 px).
"""
import numpy as np


def _tris_between(a, b, closed=True):
    """Trekanter mellom to ringer med samme antall hjørner (indekser). Normalene vender ut når a er nederst og
    ringene går mot klokka sett ovenfra."""
    n = len(a)
    out = []
    m = n if closed else n - 1
    for i in range(m):
        j = (i + 1) % n
        out.append((a[i], b[i], b[j]))
        out.append((a[i], b[j], a[j]))
    return out


def lathe(profile, n=24, caps=True):
    """Dreiekropp om y: profilen er [(r, y), …] nedenfra og opp. Endene lukkes med vifter når r > 0."""
    verts, faces, rings = [], [], []
    for r, y in profile:
        if r <= 1e-6:
            rings.append(None)
            verts.append((0.0, y, 0.0))
            rings[-1] = [len(verts) - 1] * n
            continue
        idx = []
        for i in range(n):
            t = 2 * np.pi * i / n
            verts.append((r * np.cos(t), y, -r * np.sin(t)))
            idx.append(len(verts) - 1)
        rings.append(idx)
    for a, b in zip(rings, rings[1:]):
        for tri in _tris_between(a, b):
            if len(set(tri)) == 3:
                faces.append(tri)
    if caps:
        for ring, up in ((rings[0], False), (rings[-1], True)):
            if len(set(ring)) < 3:
                continue
            verts.append((0.0, verts[ring[0]][1], 0.0))
            c = len(verts) - 1
            for i in range(n):
                j = (i + 1) % n
                faces.append((c, ring[j], ring[i]) if up else (c, ring[i], ring[j]))
    return np.array(verts, np.float32), np.array(faces, np.int32)


def ring_mesh(r, ri, hy, n=24):
    """Kum: yttervegg, innervegg og ringene oppe og nede."""
    v, f = [], []

    def circle(rad, y):
        start = len(v)
        for i in range(n):
            t = 2 * np.pi * i / n
            v.append((rad * np.cos(t), y, -rad * np.sin(t)))
        return list(range(start, start + n))
    ob, ot = circle(r, -hy), circle(r, hy)
    ib, it = circle(ri, -hy), circle(ri, hy)
    f += _tris_between(ob, ot)  # ytterveggen
    f += _tris_between(it, ib)  # innerveggen (vender inn)
    f += _tris_between(ot, it)  # toppringen
    f += _tris_between(ib, ob)  # bunnringen
    return np.array(v, np.float32), np.array(f, np.int32)[:, [0, 2, 1]]


def box_mesh(hx, hy, hz, bevel=0.0):
    import bpy  # noqa: F401 (bmesh trenger bpy lastet)
    import bmesh
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=2.0)
    bmesh.ops.scale(bm, vec=(hx, hy, hz), verts=bm.verts[:])
    if bevel > 0:
        bmesh.ops.bevel(bm, geom=bm.edges[:] + bm.verts[:], offset=float(min(bevel, 0.3 * min(hx, hy, hz))),
                        segments=1, affect="EDGES")
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    bm.verts.ensure_lookup_table()
    v = np.array([vv.co[:] for vv in bm.verts], np.float32)
    f = np.array([[l.vert.index for l in fc.loops] for fc in bm.faces], np.int32)
    bm.free()
    return v, f


def half_cylinder(r, hz, y0, n=16, h=None):
    """Lokket: halv (elliptisk) sylinder om z fra y0 (hengselplanet), halvakse r i x og høyde h, lukket under og
    i endene."""
    h = r if h is None else h
    v, f = [], []
    rings = []
    for z in (-hz, hz):
        idx = []
        for i in range(n + 1):
            t = np.pi * i / n
            v.append((r * np.cos(t), y0 + h * np.sin(t), z))
            idx.append(len(v) - 1)
        rings.append(idx)
    a, b = rings
    for i in range(n):  # buen
        f.append((a[i], a[i + 1], b[i + 1])); f.append((a[i], b[i + 1], b[i]))
    cs = []
    for ring, z, sign in ((a, -hz, -1), (b, hz, 1)):  # endene: vifter fra midten av hengselkanten
        v.append((0.0, y0, z)); c = len(v) - 1; cs.append(c)
        for i in range(n):
            f.append((c, ring[i + 1], ring[i]) if sign < 0 else (c, ring[i], ring[i + 1]))
    ca, cb = cs  # bunnen (ligger an mot kassa), delt ved viftesentrene så det ikke blir T-kryss
    f += [(a[0], b[0], cb), (a[0], cb, ca), (ca, cb, b[n]), (ca, b[n], a[n])]
    v, f = np.array(v, np.float32), np.array(f, np.int32)
    # normalene ut: snu de som peker inn mot midten av lokket
    c = np.array([0.0, y0 + h * 0.4, 0.0], np.float32)
    nrm = np.cross(v[f[:, 1]] - v[f[:, 0]], v[f[:, 2]] - v[f[:, 0]])
    cen = v[f].mean(1)
    flip = (nrm * (cen - c)).sum(1) < 0
    f[flip] = f[flip][:, [0, 2, 1]]
    return v, f


def shell(v, f, drop, t):
    """Et skall av et lukket nett med normalene ut: flatene der `drop(midtpunkt, normal)` er sann fjernes
    (åpningen), og resten får tykkelsen `t` innover (bmesh solidify), med kanten rundt åpningen lukket."""
    import bpy  # noqa: F401
    import bmesh
    bm = bmesh.new()
    vs = [bm.verts.new(tuple(map(float, q))) for q in v]
    for tri in f:
        try:
            bm.faces.new([vs[i] for i in tri])
        except ValueError:
            pass
    bm.normal_update()
    gone = [fc for fc in bm.faces if drop(fc.calc_center_median(), fc.normal)]
    bmesh.ops.delete(bm, geom=gone, context="FACES")
    bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=float(t))
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    bm.verts.ensure_lookup_table()
    v2 = np.array([vv.co[:] for vv in bm.verts], np.float32)
    f2 = np.array([[l.vert.index for l in fc.loops] for fc in bm.faces], np.int32)
    bm.free()
    return v2, f2


def fix_orientation(v, f, centre=None):
    """Snu trekanter som vender inn mot midten (dreiekroppene og boksen er konvekse nok til det)."""
    c = v.mean(0) if centre is None else np.asarray(centre, np.float32)
    nrm = np.cross(v[f[:, 1]] - v[f[:, 0]], v[f[:, 2]] - v[f[:, 0]])
    cen = v[f].mean(1)
    flip = (nrm * (cen - c)).sum(1) < 0
    f = f.copy()
    f[flip] = f[flip][:, [0, 2, 1]]
    return f


def revolve_profile(p, m=24):
    """Radiusprofilen til `revolve`-prioren (kontrollradier jevnt fra −hy til hy), glattet."""
    hy = p["hy"]
    keys = sorted(k for k in p if k.startswith("r") and k[1:].isdigit())
    rs = np.array([p[k] for k in keys], float)
    ys = np.linspace(-hy, hy, len(rs))
    yy = np.linspace(-hy, hy, m)
    rr = np.interp(yy, ys, rs)
    k = np.array([0.25, 0.5, 0.25])
    rr = np.concatenate([[rr[0]], np.convolve(rr, k, "valid"), [rr[-1]]])
    return list(zip(rr, yy))


def build(job, prior, names=None):
    kind, p = prior["kind"], prior["params"]
    w = 2 * max(p.get("hx", p.get("r", 50)), p.get("hz", 0))
    bevel = float(job.opt.get("bevel", max(3.0, 0.01 * w)))
    n = int(job.opt.get("segments", 24))
    if kind == "box":
        v, f = box_mesh(p["hx"], p["hy"], p["hz"], bevel)
        return [("body", v, f)]
    if kind == "gable":
        hx, hy, hz = p["hx"], p["hy"], p["hz"]
        v = np.array([(-hx, -hy, -hz), (hx, -hy, -hz), (hx, -hy, hz), (-hx, -hy, hz), (-hx, hy, 0), (hx, hy, 0)], np.float32)
        f = np.array([(0, 1, 2), (0, 2, 3),  # undersida (−y)
                      (0, 5, 1), (0, 4, 5),  # takflata mot −z
                      (3, 5, 4), (3, 2, 5),  # takflata mot +z
                      (0, 3, 4), (1, 5, 2)], np.int32)  # gavlene (−x, +x)
        return [("body", v, f)]
    if kind == "chest":
        # Kassa er hul: vegger og bunn med tykkelse `opt.wall` (meter, 3 cm), åpen topp; lokket er et skall.
        # Innsiden ser ingen rute, så den får flisen (`tiles: "auto"`, plank).
        names = names or ["body", "lid"]
        t = float(job.opt.get("wall", 0.03)) / job.scale
        bv, bf = box_mesh(p["hx"], p["hy"], p["hz"], bevel)
        bv, bf = shell(bv, bf, lambda c, n: n[1] > 0.9, t)
        lv, lf = half_cylinder(p["hx"], p["hz"], p["hy"], n=16, h=max(p.get("lid", 0.8), 0.05) * p["hx"])
        lv, lf = shell(lv, lf, lambda c, n: n[1] < -0.9, t)
        return [(names[0], bv, bf), (names[1], lv, lf)]
    if kind == "cylinder":
        v, f = lathe([(p["r"], -p["hy"]), (p["r"], p["hy"])], n)
    elif kind == "barrel":
        ys = np.linspace(-p["hy"], p["hy"], 13)
        prof = [(p["r"] * (1 + p.get("bulge", 0.12) * (1 - (y / p["hy"]) ** 2)), y) for y in ys]
        v, f = lathe(prof, n)
    elif kind == "revolve":
        v, f = lathe(revolve_profile(p), n)
    elif kind == "ring":
        v, f = ring_mesh(p["r"], p["ri"] * p["r"], p["hy"], n)
        return [("body", v, f)]
    elif kind in ("cylinder_x", "cone_x"):
        prof = [(p["r"], -p["hx"]), (p["r"], p["hx"])] if kind == "cylinder_x" else [(p["r0"], -p["hx"]), (p["r1"], p["hx"])]
        v, f = lathe(prof, n)
        v = np.stack([v[:, 1], -v[:, 0], v[:, 2]], 1).astype(np.float32)  # y-aksen -> x-aksen
    else:
        raise SystemExit("ukjent prior for parametrisk nett: " + kind)
    f = fix_orientation(v, f)
    return [("body", v, f)]


def smart_uv(v, f, angle=1.15, margin=0.006):
    """uv-kart med Smart UV (bpy). Trekantene beholder rekkefølgen (nettet er alt triangulert)."""
    import bpy
    from .mesh import bpy_mesh, bpy_read, bpy_reset, bpy_select
    bpy_reset()
    ob = bpy_mesh("prior", v, f)
    bpy_select(ob)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=angle, island_margin=margin, area_weight=0.0)
    bpy.ops.object.mode_set(mode="OBJECT")
    v2, f2, uv = bpy_read(ob)
    assert len(f2) == len(f), "trianguleringen endret flatene"
    return v2, f2, uv


def surface_samples(v, f, step=2.0, seed=1):
    """Tette punkter på flatene (ca. ett per step² piksler) med flatenormalen, til z-bufferen."""
    rng = np.random.default_rng(seed)
    e1 = v[f[:, 1]] - v[f[:, 0]]; e2 = v[f[:, 2]] - v[f[:, 0]]
    nrm = np.cross(e1, e2)
    area = 0.5 * np.linalg.norm(nrm, axis=1)
    nrm = nrm / (2 * area[:, None] + 1e-9)
    cnt = np.maximum(1, np.ceil(area / (step * step))).astype(int)
    tri = np.repeat(np.arange(len(f)), cnt)
    a = rng.random(len(tri)); b = rng.random(len(tri))
    flip = a + b > 1
    a[flip], b[flip] = 1 - a[flip], 1 - b[flip]
    P = v[f[tri, 0]] + a[:, None] * e1[tri] + b[:, None] * e2[tri]
    P = np.concatenate([P, v])  # hjørnene med, så kantene er dekket
    N = np.concatenate([nrm[tri], vertex_normals(v, f)])
    return P.astype(np.float32), N.astype(np.float32)


def vertex_normals(v, f):
    n = np.cross(v[f[:, 1]] - v[f[:, 0]], v[f[:, 2]] - v[f[:, 0]])
    vn = np.zeros_like(v)
    for i in range(3):
        np.add.at(vn, f[:, i], n)
    return vn / (np.linalg.norm(vn, axis=1, keepdims=True) + 1e-12)


def extent_y(prior):
    """Bunn og topp (y, verdenspiksler om delens nullpunkt) for prioren."""
    k, p = prior["kind"], prior["params"]
    if k == "chest":
        return -p["hy"], p["hy"] + max(p.get("lid", 0.8), 0.05) * p["hx"]
    if k in ("cylinder_x",):
        return -p["r"], p["r"]
    if k == "cone_x":
        r = max(p["r0"], p["r1"])
        return -r, r
    return -p["hy"], p["hy"]
