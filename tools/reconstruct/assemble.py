"""Sett sammen en hel figur: kroppen fra kroppsarket og hodet fra hodearket (mye høyere oppløsning),
plassert etter silhuettene, og redskapet i hånda skilt ut som egen bit. Alt bakes til én tekstur på 2048².

    python3.13 -m tools.reconstruct smith_body --steps assemble --head smith_head
"""
import json
import os

import numpy as np

from . import job as J

HEAD_SCALE = 0.00028  # som i sheets.json for hodearkene (standard)


def _mask(work, k):
    import cv2
    return cv2.imread(os.path.join(work, f"{k}_mask.png"), 0) > 127


def _widths(m, fr, k, rows):
    """Bredden på silhuetten (verdenspiksler) i radene `rows` (verdenspiksler fra toppen)."""
    o = fr[k]
    out = []
    for y in rows:
        v = int(round(o["top"] + y * o["sy"]))
        c = np.nonzero(m[min(v, m.shape[0] - 1)])[0]
        out.append((c.max() - c.min()) / o["sy"] if len(c) else np.nan)
    return np.array(out)


def head_fit(body_work, head_work):
    """Skala (kroppspiksler per hodepiksel) og forskyvning i z, funnet ved å passe hodets silhuett forfra og fra
    siden til kroppsarket over øyehøyde (skallen: ingen skjegg, ingen ører)."""
    fb = json.load(open(os.path.join(body_work, "frame.json")))["frame"]
    fh = json.load(open(os.path.join(head_work, "frame.json")))["frame"]
    ys = np.arange(10, 200, 4.0)  # hodepiksler fra toppen: issen til brynene
    best = []
    for k in ("front", "right"):
        wh = _widths(_mask(head_work, k), fh, k, ys)
        mb = _mask(body_work, k)
        errs = []
        for s in np.arange(0.2, 0.7, 0.002):
            wb = _widths(mb, fb, k, ys * s)
            errs.append((np.nanmean((wb - s * wh) ** 2), s))
        best.append(min(errs)[1])
    s = float(np.mean(best))
    # z: midten av skallen i profilen
    def zmid(work, fr, rows):
        m = _mask(work, "right"); o = fr["right"]
        zs = []
        for y in rows:
            v = int(round(o["top"] + y * o["sy"]))
            c = np.nonzero(m[v])[0]
            zs.append(((c.min() + c.max()) / 2 - o["c"]) / o["sy"])
        return float(np.mean(zs))
    dz = zmid(body_work, fb, ys[:20] * s) - s * zmid(head_work, fh, ys[:20])
    print("hodet: skala %.3f (forfra %.3f, profil %.3f), dz %.1f" % (s, best[0], best[1], dz))
    return s, dz


def hammer_mask(rgb, m, past=190, width=12, column=False, hand=None):
    """Redskapet i én visning: det største grå jernstykket (hammerhodet) og et bånd langs skaftet fra hodet
    gjennom hånda (nærmeste hudflekk) og litt forbi."""
    import cv2
    from scipy import ndimage as ndi
    hsv = cv2.cvtColor(rgb, cv2.COLOR_RGB2HSV).astype(np.float32)
    h, s, v = hsv[..., 0] * 2, hsv[..., 1], hsv[..., 2]
    iron = (s < 40) & (v > 35) & (v < 140) & m
    lab, n = ndi.label(ndi.binary_opening(iron, iterations=2))
    if n == 0:
        return np.zeros_like(m)
    sizes = ndi.sum(np.ones_like(lab), lab, range(1, n + 1))
    blob = lab == (1 + int(np.argmax(sizes)))
    cy, cx = ndi.center_of_mass(blob)
    skin = (h > 4) & (h < 28) & (s > 50) & (s < 160) & (v > 140) & m
    out = ndi.binary_dilation(blob, iterations=4)
    if column:  # forfra står skaftet loddrett bak neven, over hammerhodet
        ys, xs = np.nonzero(blob)
        col = np.zeros_like(m)
        col[max(0, ys.min() - 260): ys.max() + 1, max(0, xs.min() - 20): xs.max() + 21] = True
        return (out | col) & m
    sy_, sx_ = np.nonzero(ndi.binary_opening(skin, iterations=2))
    if len(sx_):
        # hånda: den nærmeste hudpikselen til hammerhodet; skaftet går derfra gjennom neven og litt forbi
        i = int(np.argmin(np.hypot(sx_ - cx, sy_ - cy)))
        a = np.array([cx, cy]); hp = np.array([sx_[i], sy_[i]], float)
        u = (hp - a) / (np.linalg.norm(hp - a) + 1e-9)
        b = hp + u * past
        if hand is not None:
            hand.append(hp)
        yy, xx = np.mgrid[: m.shape[0], : m.shape[1]]
        ab = b - a
        t = np.clip(((xx - a[0]) * ab[0] + (yy - a[1]) * ab[1]) / (ab @ ab), 0, 1)
        d = np.hypot(xx - (a[0] + t * ab[0]), yy - (a[1] + t * ab[1]))
        out |= (d < width) & m
    return out


def assemble(head="smith_head", tex=2048):
    import bpy
    import cv2
    from scipy import ndimage as ndi
    from .fuse import to_img

    body_work = J.WORK
    head_work = os.path.join(J.ROOT, "target/reconstruct", head)
    s, dz = head_fit(body_work, head_work)
    fb = json.load(open(J.p("frame.json")))["frame"]

    bpy.ops.wm.open_mainfile(filepath=J.p("head.blend"))
    body = bpy.data.objects["head"]
    body.name = "body_src"
    with bpy.data.libraries.load(os.path.join(head_work, "head.blend")) as (src, dst):
        dst.objects = ["head"]
    hd = dst.objects[0]
    hd.name = "head_src"
    bpy.context.scene.collection.objects.link(hd)

    def material(name, work, stem):
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        nt = mat.node_tree
        bsdf = nt.nodes["Principled BSDF"]
        al = nt.nodes.new("ShaderNodeTexImage"); al.image = bpy.data.images.load(os.path.join(work, f"{stem}_albedo.png"))
        nm = nt.nodes.new("ShaderNodeTexImage"); nm.image = bpy.data.images.load(os.path.join(work, f"{stem}_normal.png"))
        nm.image.colorspace_settings.name = "Non-Color"
        nmap = nt.nodes.new("ShaderNodeNormalMap")
        nt.links.new(al.outputs[0], bsdf.inputs["Base Color"])
        nt.links.new(nm.outputs[0], nmap.inputs["Color"]); nt.links.new(nmap.outputs[0], bsdf.inputs["Normal"])
        return mat
    body.data.materials.clear(); body.data.materials.append(material("src_body", body_work, "head"))
    hd.data.materials.clear(); hd.data.materials.append(material("src_head", head_work, "head"))

    # trekantbudsjettet for hele figuren (rundt 14 000): hodet ~4 500, kroppen resten
    for o, n in ((hd, 4500), (body, J.TRIS - 4500)):
        if len(o.data.polygons) > n:
            for x in bpy.context.scene.objects:
                x.select_set(x == o)
            bpy.context.view_layer.objects.active = o
            md = o.modifiers.new("dec", "DECIMATE"); md.ratio = n / len(o.data.polygons)
            bpy.ops.object.modifier_apply(modifier="dec")
    # hodet inn i kroppens rom: (piksler i hodearket) * s = piksler i kroppsarket, z forskjøvet
    k = s * J.SCALE / HEAD_SCALE
    for v in hd.data.vertices:
        v.co = (v.co.x * k, v.co.y * k - dz * J.SCALE, v.co.z * k)  # w2b: Y = -z
    hd.data.update()
    hv = np.array([v.co[:] for v in hd.data.vertices])
    # kutt kroppen litt over halskuttet på hodet (bakerst), så hodets hals går ned i kragen
    back = hv[hv[:, 1] > np.median(hv[:, 1])]
    zcut = back[:, 2].min() + 0.02
    import bmesh
    bm = bmesh.new(); bm.from_mesh(body.data)
    # rundt halsen kuttes det ved halskuttet; utenfor halsen (skuldrene, kragen) først 7 cm høyere
    neck = hv[(hv[:, 2] < zcut + 0.04)]
    nc = neck[:, :2].mean(0)
    nr = np.percentile(np.linalg.norm(neck[:, :2] - nc, axis=1), 50) * 0.9  # godt innenfor hodets hals

    def drop(c):
        r = np.hypot(c.x - nc[0], c.y - nc[1])
        return c.z > zcut + 0.07 or (c.z > zcut and r < nr)
    gone = [f for f in bm.faces if drop(f.calc_center_median())]
    bmesh.ops.delete(bm, geom=gone, context="FACES")
    bm.to_mesh(body.data); bm.free()
    print("kroppen kuttet over z = %.3f m (%d flater borte)" % (zcut, len(gone)))
    # kuttflaten under halsen på hodet (malt mørk i forbildet bakfra) skal ikke stikke ut over kragen
    bm = bmesh.new(); bm.from_mesh(hd.data)
    cap = [f for f in bm.faces if f.normal.z < -0.5 and f.calc_center_median().z < zcut + 0.06]
    bmesh.ops.delete(bm, geom=cap, context="FACES")
    bm.to_mesh(hd.data); bm.free()
    print("kuttflaten på hodet fjernet:", len(cap), "flater")

    # løse flak fjernes i slutten (etter spøkelset)
    # redskapet: hjørner som ligger i redskapsmasken både forfra og fra siden
    masks = {}
    for kk in ("front", "right"):
        rgb = cv2.imread(J.p(f"{kk}_rgb.png"))[:, :, ::-1]
        mm = _mask(body_work, kk)
        masks[kk] = hammer_mask(rgb, mm, column=(kk == "front"))
    co = np.array([v.co[:] for v in body.data.vertices]) / J.SCALE
    W = np.stack([co[:, 0], co[:, 2], -co[:, 1]], -1)  # Blender -> verden
    inside = np.ones(len(W), bool)
    for kk, mm in masks.items():
        u, vv = to_img(kk, fb, W[:, 0], W[:, 1], W[:, 2])
        ui = np.clip(np.round(u).astype(int), 0, mm.shape[1] - 1); vi = np.clip(np.round(vv).astype(int), 0, mm.shape[0] - 1)
        inside &= mm[vi, ui]
    # bare ved figurens høyre hånd (forfra til venstre i bildet, x < 0) og under brystet
    H = fb["front"]["bot"] - fb["front"]["top"]
    inside &= (W[:, 0] < -0.25 * (W[:, 0].max() - W[:, 0].min()) / 2) & (W[:, 1] < -0.45 * H)
    if J.OPT.get("drop_mirrored_tool"):
        # den speilede profilen gir et «spøkelse» av redskapet i den andre hånda: fjern det som ligger i
        # redskapsmasken fra siden på figurens venstre side, utenfor neven
        hand = []
        rgb = cv2.imread(J.p("right_rgb.png"))[:, :, ::-1]
        hammer_mask(rgb, _mask(body_work, "right"), hand=hand)
        u, vv = to_img("right", fb, W[:, 0], W[:, 1], W[:, 2])
        ui = np.clip(np.round(u).astype(int), 0, masks["right"].shape[1] - 1)
        vi = np.clip(np.round(vv).astype(int), 0, masks["right"].shape[0] - 1)
        fist_z = (hand[0][0] - fb["right"]["c"]) / fb["right"]["sy"] if hand else 0.0
        fist_y = -(hand[0][1] - fb["right"]["top"]) / fb["right"]["sy"] if hand else 0.0
        # bare i armbiten forfra (ikke i biten med kroppen og forkleet)
        from .fuse import run_bounds
        mf = _mask(body_work, "front")
        lf, rf = run_bounds(mf)
        uf, vf = to_img("front", fb, W[:, 0], W[:, 1], W[:, 2])
        uf = np.clip(np.round(uf).astype(int), 0, mf.shape[1] - 1); vf = np.clip(np.round(vf).astype(int), 0, mf.shape[0] - 1)
        c0 = int(round(fb["front"]["c"]))
        torso = (lf[vf, uf] <= c0) & (rf[vf, uf] >= c0) & mf[vf, uf]
        ghost = masks["right"][vi, ui] & (~torso | (W[:, 0] > 0.66 * W[:, 0].max())) & (W[:, 0] > 0) \
            & (W[:, 1] < -0.45 * H) & (np.hypot(W[:, 2] - fist_z, W[:, 1] - (fist_y - 30)) > 42)
        # venstre neve har ingen egen dybde fra siden (profilen viser høyre side): alt i hånda utenfor nevens
        # dybde er rester av høyre arm og skaftet
        lefthand = (W[:, 0] > 0.66 * W[:, 0].max()) & (W[:, 1] < fist_y + 70) & (W[:, 1] > fist_y - 140)
        ghost |= lefthand & (np.abs(W[:, 2] - fist_z) > 48)
        gs = set(np.nonzero(ghost)[0].tolist())
        bm = bmesh.new(); bm.from_mesh(body.data)
        dead = [f for f in bm.faces if any(v.index in gs for v in f.verts)]
        bmesh.ops.delete(bm, geom=dead, context="FACES")
        bm.to_mesh(body.data); bm.free()
        print("spøkelset i venstre hånd:", len(dead), "flater fjernet")
        co = np.array([v.co[:] for v in body.data.vertices]) / J.SCALE
        W = np.stack([co[:, 0], co[:, 2], -co[:, 1]], -1)
        inside = np.ones(len(W), bool)
        for kk, mm in masks.items():
            u, vv = to_img(kk, fb, W[:, 0], W[:, 1], W[:, 2])
            ui = np.clip(np.round(u).astype(int), 0, mm.shape[1] - 1); vi = np.clip(np.round(vv).astype(int), 0, mm.shape[0] - 1)
            inside &= mm[vi, ui]
        inside &= (W[:, 0] < -0.25 * (W[:, 0].max() - W[:, 0].min()) / 2) & (W[:, 1] < -0.45 * H)
    _drop_flakes(body)
    # lukk hullene etter kuttene (halsen, venstre neve), så ingen ser inn i figuren ovenfra
    bm = bmesh.new(); bm.from_mesh(body.data)
    edges = [e for e in bm.edges if e.is_boundary]
    res = bmesh.ops.holes_fill(bm, edges=edges, sides=0)
    bmesh.ops.triangulate(bm, faces=res["faces"])
    bm.to_mesh(body.data); bm.free()
    print("hull lukket:", len(res["faces"]), "flater")
    co = np.array([v.co[:] for v in body.data.vertices]) / J.SCALE
    W = np.stack([co[:, 0], co[:, 2], -co[:, 1]], -1)
    inside = np.ones(len(W), bool)
    for kk, mm in masks.items():
        u, vv = to_img(kk, fb, W[:, 0], W[:, 1], W[:, 2])
        ui = np.clip(np.round(u).astype(int), 0, mm.shape[1] - 1); vi = np.clip(np.round(vv).astype(int), 0, mm.shape[0] - 1)
        inside &= mm[vi, ui]
    inside &= (W[:, 0] < -0.25 * (W[:, 0].max() - W[:, 0].min()) / 2) & (W[:, 1] < -0.45 * H)
    sel = set(np.nonzero(inside)[0].tolist())
    bm = bmesh.new(); bm.from_mesh(body.data)
    bm.verts.ensure_lookup_table()
    tool_faces = [f for f in bm.faces if sum(v.index in sel for v in f.verts) >= 2]
    print("redskapet:", len(tool_faces), "flater")
    tf = set(f.index for f in tool_faces)
    flags = [f.index in tf for f in bm.faces]
    bm.free()
    _select_faces(body, flags)
    for o in bpy.context.scene.objects:
        o.select_set(o == body)
    bpy.context.view_layer.objects.active = body
    tool = None
    if tool_faces:
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.separate(type="SELECTED")
        bpy.ops.object.mode_set(mode="OBJECT")
        tool = [o for o in bpy.context.selected_objects if o != body][0]
        tool.name = "tool_hammer_src"

    # én tekstur: kopier delene, slå dem sammen, ny uv (hodet får mer plass), bak farge og normal fra kildene
    srcs = [o for o in (body, hd, tool) if o]
    copies = []
    for o in srcs:
        c = o.copy(); c.data = o.data.copy(); bpy.context.scene.collection.objects.link(c); copies.append(c)
    for i, c in enumerate(copies):
        c.data.materials.clear()
        for p in c.data.polygons:
            p.material_index = 0
        c.data.attributes.new("part", "INT", "FACE")
        c.data.attributes["part"].data.foreach_set("value", [i] * len(c.data.polygons))
    for o in bpy.context.scene.objects:
        o.select_set(o in copies)
    bpy.context.view_layer.objects.active = copies[0]
    bpy.ops.object.join()
    out = copies[0]
    out.name = "body"
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=0.70, island_margin=0.003, area_weight=0.0)
    bpy.ops.object.mode_set(mode="OBJECT")
    # hodet (del 1) får 2,2 ganger så lange uv-kanter, så pakkes alt på nytt
    _scale_part_uv(out, 1, 2.2)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.pack_islands(margin=0.004, rotate=True)
    bpy.ops.object.mode_set(mode="OBJECT")

    sc = bpy.context.scene
    sc.render.engine = "CYCLES"; sc.cycles.device = "CPU"; sc.cycles.samples = 1
    mat = bpy.data.materials.new("body"); mat.use_nodes = True
    out.data.materials.append(mat)
    nt = mat.node_tree
    texn = nt.nodes.new("ShaderNodeTexImage"); nt.nodes.active = texn
    albedo = bpy.data.images.new("albedo", tex, tex, alpha=False)
    texn.image = albedo
    for o in bpy.context.scene.objects:
        o.select_set(o in srcs or o == out)
    bpy.context.view_layer.objects.active = out
    sc.render.bake.use_selected_to_active = True
    sc.render.bake.cage_extrusion = 0.01
    sc.render.bake.max_ray_distance = 0.03
    sc.render.bake.margin = 8
    bpy.ops.object.bake(type="DIFFUSE", pass_filter={"COLOR"}, margin=8)
    albedo.filepath_raw = J.p("full_albedo.png"); albedo.file_format = "PNG"; albedo.save()
    normal = bpy.data.images.new("normal", tex, tex, alpha=False)
    normal.colorspace_settings.name = "Non-Color"
    texn.image = normal
    bpy.ops.object.bake(type="NORMAL", normal_space="TANGENT", margin=8)
    normal.filepath_raw = J.p("full_normal.png"); normal.file_format = "PNG"; normal.save()
    for o in srcs:
        bpy.data.objects.remove(o)
    # verktøyet skilles ut igjen som egen bit (samme tekstur), så det kan henge stivt på hånda
    if tool:
        parts = [out.data.attributes["part"].data[i].value for i in range(len(out.data.polygons))]
        _select_faces(out, [v == 2 for v in parts])
        for o in bpy.context.scene.objects:
            o.select_set(o == out)
        bpy.context.view_layer.objects.active = out
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.separate(type="SELECTED")
        bpy.ops.object.mode_set(mode="OBJECT")
        t = [o for o in bpy.context.selected_objects if o != out][0]
        t.name = "tool_hammer"
    print("trekanter", sum(len(o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH"))
    bpy.ops.wm.save_as_mainfile(filepath=J.p("assembled.blend"))


def _drop_flakes(ob, keep_frac=0.02):
    """Fjern løse biter (flak etter kuttene) som er mindre enn en liten andel av modellen."""
    import bmesh
    bm = bmesh.new(); bm.from_mesh(ob.data)
    bm.faces.ensure_lookup_table()
    seen, parts = set(), []
    for f in bm.faces:
        if f.index in seen:
            continue
        stack, part = [f], []
        seen.add(f.index)
        while stack:
            g = stack.pop(); part.append(g)
            for e in g.edges:
                for h in e.link_faces:
                    if h.index not in seen:
                        seen.add(h.index); stack.append(h)
        parts.append(part)
    n = len(bm.faces)
    dead = [f for p in parts if len(p) < keep_frac * n for f in p]
    bmesh.ops.delete(bm, geom=dead, context="FACES")
    bm.to_mesh(ob.data); bm.free()
    print("flak fjernet:", len(dead), "flater i", sum(1 for p in parts if len(p) < keep_frac * n), "biter")


def _select_faces(ob, flags):
    """Velg nøyaktig disse flatene (og bare deres hjørner og kanter), så «separate» tar det riktige."""
    import bpy
    bpy.context.scene.tool_settings.mesh_select_mode = (False, False, True)
    me = ob.data
    for v in me.vertices:
        v.select = False
    for e in me.edges:
        e.select = False
    for p, on in zip(me.polygons, flags):
        p.select = bool(on)
        if on:
            for vi in p.vertices:
                me.vertices[vi].select = True
    me.update()


def _scale_part_uv(ob, part, k):
    """Skaler uv-øyene til en del rundt hver flates øy-midte (hele delen rundt sitt midtpunkt per øy)."""
    import bmesh
    b = bmesh.new(); b.from_mesh(ob.data)
    uv = b.loops.layers.uv.active
    pl = b.faces.layers.int.get("part")
    faces = [f for f in b.faces if f[pl] == part]
    # øyer: flater som deler uv-hjørner
    seen, islands = set(), []
    key = lambda l: (round(l[uv].uv.x, 5), round(l[uv].uv.y, 5), l.vert.index)  # noqa: E731
    by_vert = {}
    for f in faces:
        for l in f.loops:
            by_vert.setdefault(key(l), []).append(f)
    for f in faces:
        if f.index in seen:
            continue
        stack, isl = [f], []
        seen.add(f.index)
        while stack:
            g = stack.pop(); isl.append(g)
            for l in g.loops:
                for h in by_vert.get(key(l), []):
                    if h.index not in seen:
                        seen.add(h.index); stack.append(h)
        islands.append(isl)
    for isl in islands:
        pts = [l[uv].uv.copy() for f in isl for l in f.loops]
        c = sum(pts, type(pts[0])((0, 0))) / len(pts)
        for f in isl:
            for l in f.loops:
                l[uv].uv = c + (l[uv].uv - c) * k
    b.to_mesh(ob.data); b.free()
