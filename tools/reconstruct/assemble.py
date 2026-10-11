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
        for s in np.arange(0.2, 1.6, 0.002):
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


def hammer_mask(rgb, m, past=190, width=12, column=False, hand=None, kind="iron", region=None, handle=True):
    """Redskapet i én visning: det største grå jernstykket (hammerhodet) og et bånd langs skaftet fra hodet
    gjennom hånda (nærmeste hudflekk) og litt forbi."""
    import cv2
    from scipy import ndimage as ndi
    hsv = cv2.cvtColor(rgb, cv2.COLOR_RGB2HSV).astype(np.float32)
    h, s, v = hsv[..., 0] * 2, hsv[..., 1], hsv[..., 2]
    if kind == "wood":  # trekrus o.l.: gulbrunt tre (kjolen og forkleet er rødere eller lysere)
        iron = (h > 22) & (h < 45) & (s > 70) & (s < 200) & (v > 60) & (v < 215) & m
    else:
        iron = (s < 40) & (v > 35) & (v < 140) & m
    if region is not None:
        iron &= region
    lab, n = ndi.label(ndi.binary_opening(iron, iterations=2))
    if n == 0:
        return np.zeros_like(m)
    sizes = ndi.sum(np.ones_like(lab), lab, range(1, n + 1))
    blob = lab == (1 + int(np.argmax(sizes)))
    cy, cx = ndi.center_of_mass(blob)
    skin = (h > 4) & (h < 28) & (s > 50) & (s < 160) & (v > 140) & m
    out = ndi.binary_dilation(blob, iterations=4 if handle else 9)
    if not handle:
        return out & m
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
    # hvilken hånd redskapet er i: sg = 1 for høyre (x < 0), -1 for venstre
    tool_side = J.OPT.get("tool_side", "R")
    sg = 1 if tool_side == "R" else -1
    tkw = dict(kind=J.OPT.get("tool_kind", "iron"), handle=J.OPT.get("tool_handle", True))
    masks = {}
    for kk in ("front", "right"):
        rgb = cv2.imread(J.p(f"{kk}_rgb.png"))[:, :, ::-1]
        mm = _mask(body_work, kk)
        reg = None
        if kk == "front":  # figurens høyre er til venstre i bildet forfra
            reg = np.zeros_like(mm)
            c = int(fb["front"]["c"])
            if sg > 0:
                reg[:, :c] = True
            else:
                reg[:, c:] = True
        if reg is None:
            reg = np.ones_like(mm)
        o_ = fb[kk]
        reg[: int(o_["top"] + 0.42 * (o_["bot"] - o_["top"]))] = False  # under brystet (ikke pels, spenner)
        masks[kk] = hammer_mask(rgb, mm, column=(kk == "front" and tkw["handle"]), region=reg, **tkw)
    co = np.array([v.co[:] for v in body.data.vertices]) / J.SCALE
    W = np.stack([co[:, 0], co[:, 2], -co[:, 1]], -1)  # Blender -> verden
    inside = np.ones(len(W), bool)
    for kk, mm in masks.items():
        u, vv = to_img(kk, fb, W[:, 0], W[:, 1], W[:, 2])
        ui = np.clip(np.round(u).astype(int), 0, mm.shape[1] - 1); vi = np.clip(np.round(vv).astype(int), 0, mm.shape[0] - 1)
        inside &= mm[vi, ui]
    # bare ved figurens høyre hånd (forfra til venstre i bildet, x < 0) og under brystet
    H = fb["front"]["bot"] - fb["front"]["top"]
    inside &= (sg * W[:, 0] < -0.25 * np.ptp(W[:, 0]) / 2) & (W[:, 1] < -0.45 * H)
    if J.OPT.get("drop_mirrored_tool"):
        # den speilede profilen gir et «spøkelse» av redskapet i den andre hånda: fjern det som ligger i
        # redskapsmasken fra siden på figurens venstre side, utenfor neven
        hand = []
        rgb = cv2.imread(J.p("right_rgb.png"))[:, :, ::-1]
        preg = np.ones_like(_mask(body_work, "right"))
        preg[: int(fb["right"]["top"] + 0.42 * (fb["right"]["bot"] - fb["right"]["top"]))] = False
        hammer_mask(rgb, _mask(body_work, "right"), hand=hand, kind=tkw["kind"], region=preg)
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
        ghost = masks["right"][vi, ui] & (~torso | ((sg * W[:, 0] > 0.66 * (sg * W[:, 0]).max()) & (W[:, 1] > -0.7 * H))) \
            & (sg * W[:, 0] > 0.55 * (sg * W[:, 0]).max()) & (W[:, 1] < fist_y + 10) \
            & (W[:, 1] < -0.45 * H) & (np.hypot(W[:, 2] - fist_z, W[:, 1] - (fist_y - 30)) > 42)
        # venstre neve har ingen egen dybde fra siden (profilen viser høyre side): alt i hånda utenfor nevens
        # dybde er rester av høyre arm og skaftet
        lefthand = (sg * W[:, 0] > 0.66 * (sg * W[:, 0]).max()) & (W[:, 1] < fist_y + 70) & (W[:, 1] > fist_y - 140)
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
        inside &= (sg * W[:, 0] < -0.25 * np.ptp(W[:, 0]) / 2) & (W[:, 1] < -0.45 * H)
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
    inside &= (sg * W[:, 0] < -0.25 * np.ptp(W[:, 0]) / 2) & (W[:, 1] < -0.45 * H)
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

    # hullet der redskapet satt i kroppen lukkes (ellers svarte rifter ved neven)
    bm = bmesh.new(); bm.from_mesh(body.data)
    res = bmesh.ops.holes_fill(bm, edges=[e for e in bm.edges if e.is_boundary], sides=0)
    bmesh.ops.triangulate(bm, faces=res["faces"])
    bm.to_mesh(body.data); bm.free()
    print("hull etter redskapet lukket:", len(res["faces"]), "flater")
    if J.OPT.get("hips_iron"):
        mark_hips_iron(body, fb, body_work, sg)
    if tool and J.OPT.get("model_tool", "hammer") == "hammer":
        tool = model_tool(tool, fb)
    if J.OPT.get("split_arms", False):  # ga rifter i underarmene (kropp_6); av som standard
        split_arms(body, fb, body_work)
    # nevene skilt ut ga takkete sømmer ved håndleddet (kropp_6); av som standard
    hands = split_hands(body, fb, body_work) if J.OPT.get("split_hands", False) else []

    # én tekstur: kopier delene, slå dem sammen, ny uv (hodet får mer plass), bak farge og normal fra kildene
    srcs = [o for o in (body, hd, tool) if o] + hands
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
    # lokkene på nevene (der de satt fast i forkleet) får hudfargen fra midten av neven, ikke forkleet bak
    if "cap" in out.data.attributes:
        import bmesh as bm2
        b = bm2.new(); b.from_mesh(out.data)
        uv = b.loops.layers.uv.active
        cap = b.faces.layers.int.get("cap"); pl = b.faces.layers.int.get("part")
        base = 3 if tool else 2
        for k in range(len(hands)):
            own = [f for f in b.faces if f[pl] == base + k]
            skin = [f for f in own if not f[cap]]
            if not skin:
                continue
            f0 = max(skin, key=lambda f: f.calc_area())
            c = sum((l[uv].uv for l in f0.loops), type(f0.loops[0][uv].uv)((0, 0))) / len(f0.loops)
            for f in own:
                if f[cap]:
                    for l in f.loops:
                        l[uv].uv = c
        b.to_mesh(out.data); b.free()
    names = [o.name for o in srcs]
    for o in srcs:
        bpy.data.objects.remove(o)
    hands = [type("H", (), {"name": n}) for n in names if n.startswith("hand_")]
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
    # hendene skilles ut igjen (samme tekstur): stive på hand.L/hand.R i riggen
    base = 3 if tool else 2
    for k, h in enumerate(hands):
        parts = [out.data.attributes["part"].data[i].value for i in range(len(out.data.polygons))]
        _select_faces(out, [v == base + k for v in parts])
        for o in bpy.context.scene.objects:
            o.select_set(o == out)
        bpy.context.view_layer.objects.active = out
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.separate(type="SELECTED")
        bpy.ops.object.mode_set(mode="OBJECT")
        t = [o for o in bpy.context.selected_objects if o != out][0]
        t.name = h.name.replace("_src", "")
    print("trekanter", sum(len(o.data.polygons) for o in bpy.context.scene.objects if o.type == "MESH"))
    bpy.ops.wm.save_as_mainfile(filepath=J.p("assembled.blend"))


def model_tool(tool, fb):
    """Bytt den rekonstruerte hammeren (flat, fra to tynne silhuetter) med en enkel modell med volum: et rundt
    skaft og et jernhode med fas, lagt langs hovedaksen til det rekonstruerte redskapet. Fargene er malte
    (jern og tre) med litt støy, og bakes inn i den felles teksturen."""
    import bpy
    import bmesh
    from mathutils import Matrix, Vector
    from .rig import joints
    P = np.array([v.co[:] for v in tool.data.vertices])
    c = P.mean(0)
    _, _, vt = np.linalg.svd(P - c, full_matrices=False)
    a = vt[0]
    t = (P - c) @ a
    Jw = joints(fb)
    from .texture import w2b
    wrist = w2b(np.asarray(Jw["finger.R"], float) * 0.5 + np.asarray(Jw["wrist.R"], float) * 0.5)
    e0, e1 = c + a * t.min(), c + a * t.max()
    head_end, tail_end = (e0, e1) if np.linalg.norm(e0 - wrist) > np.linalg.norm(e1 - wrist) else (e1, e0)
    ax = (tail_end - head_end); L = np.linalg.norm(ax); ax /= L
    # hodets lange akse: andre hovedretning blant punktene nær hodeenden
    near = P[np.abs((P - head_end) @ ax) < 0.35 * L]
    _, _, v2 = np.linalg.svd(near - near.mean(0), full_matrices=False)
    hx = v2[0] - ax * (v2[0] @ ax); hx /= np.linalg.norm(hx) + 1e-9
    hy = np.cross(ax, hx)
    hl = float(np.clip(np.ptp((near - near.mean(0)) @ hx), 0.08, 0.13))
    hc = head_end + ax * 0.028
    L = float(np.clip(L, 0.28, 0.42))
    bm = bmesh.new()
    # jernhodet
    m = Matrix(((hx[0], hy[0], ax[0], hc[0]), (hx[1], hy[1], ax[1], hc[1]), (hx[2], hy[2], ax[2], hc[2]), (0, 0, 0, 1)))
    r = bmesh.ops.create_cube(bm, size=1.0)
    for v in r["verts"]:
        v.co = Vector((v.co.x * hl, v.co.y * 0.045, v.co.z * 0.05))
    bmesh.ops.bevel(bm, geom=list(set(e for v in r["verts"] for e in v.link_edges)), offset=0.006, segments=2,
                    affect="EDGES")
    head_faces = set(bm.faces)
    bmesh.ops.transform(bm, matrix=m, verts=list(bm.verts))
    # skaftet
    for f in bm.faces:
        f.material_index = 0
    L = L - 0.03  # skaftet begynner i hodet, stikker ikke ut på toppen
    r2 = bmesh.ops.create_cone(bm, cap_ends=True, segments=10, radius1=0.017, radius2=0.014, depth=L)
    for f in bm.faces:
        if f not in head_faces:
            f.material_index = 1
    m2 = Matrix(((hx[0], hy[0], ax[0], 0), (hx[1], hy[1], ax[1], 0), (hx[2], hy[2], ax[2], 0), (0, 0, 0, 1)))
    mid = head_end + ax * (0.03 + L / 2)
    for v in r2["verts"]:
        v.co = m2 @ v.co + Vector(mid)
    me = bpy.data.meshes.new("hammer")
    bm.to_mesh(me)
    ob = bpy.data.objects.new("tool_hammer_src", me)
    bpy.context.scene.collection.objects.link(ob)
    for name, col in (("iron", (0.10, 0.105, 0.115)), ("wood", (0.28, 0.15, 0.07))):
        mat = bpy.data.materials.new(name); mat.use_nodes = True
        nt = mat.node_tree; bsdf = nt.nodes["Principled BSDF"]
        noise = nt.nodes.new("ShaderNodeTexNoise"); noise.inputs["Scale"].default_value = 60.0
        mix = nt.nodes.new("ShaderNodeMixRGB"); mix.blend_type = "MULTIPLY"; mix.inputs[0].default_value = 0.35
        mix.inputs[1].default_value = (*col, 1)
        nt.links.new(noise.outputs["Color"], mix.inputs[2]); nt.links.new(mix.outputs[0], bsdf.inputs["Base Color"])
        me.materials.append(mat)
    bpy.data.objects.remove(tool)
    print("hammeren modellert: skaft %.2f m, hode %.2f m" % (L, hl))
    return ob


def split_arms(body, fb, work):
    """Armene er smeltet sammen med kroppen der de ligger inntil (i profilen ser ingen mellomrommet). Forfra er
    de atskilt fra armhula og ned: hjørner i armbiten forfra merkes («arm» = 1 venstre, 2 høyre), flatene som
    binder armen til kroppen under armhula fjernes, og hullene lukkes. Riggen gir armhjørnene bare armvekter."""
    import bmesh
    from .fuse import run_bounds, to_img
    mf = _mask(work, "front")
    o = fb["front"]
    c0 = int(round(o["c"]))
    # armhula: øverste rad der armene står atskilt fra kroppen forfra
    rows = [v for v in range(int(o["top"] + 0.22 * (o["bot"] - o["top"])), o["bot"]) if len(np.nonzero(np.diff(np.r_[0, mf[v].astype(int), 0]) == 1)[0]) >= 3]
    pit = rows[0] if rows else o["top"]
    lf, rf = run_bounds(mf)
    from .rig import joints
    Jw = joints(fb)
    low = int(o["top"] - min(Jw["wrist.L"][1], Jw["wrist.R"][1]) * o["sy"]) + 10  # under håndleddet: nevene er egne deler
    co = np.array([v.co[:] for v in body.data.vertices]) / J.SCALE
    W = np.stack([co[:, 0], co[:, 2], -co[:, 1]], -1)
    u, vv = to_img("front", fb, W[:, 0], W[:, 1], W[:, 2])
    ui = np.clip(np.round(u).astype(int), 0, mf.shape[1] - 1); vi = np.clip(np.round(vv).astype(int), 0, mf.shape[0] - 1)
    inmask = mf[vi, ui]
    torso_run = (lf[vi, ui] <= c0) & (rf[vi, ui] >= c0)
    arm = inmask & ~torso_run & (vi > pit + 6) & (vi < low) & near_arm(W, Jw)
    side = np.where(W[:, 0] > 0, 1, 2) * arm
    # flater som binder armen til kroppen (blandet) under armhula fjernes
    bm = bmesh.new(); bm.from_mesh(body.data)
    dead = []
    for f in bm.faces:
        ks = [side[v.index] for v in f.verts]
        if len(set(ks)) > 1 and min(vi[v.index] for v in f.verts) > pit + 6 and max(vi[v.index] for v in f.verts) < low:
            dead.append(f)
    bmesh.ops.delete(bm, geom=dead, context="FACES")
    res = bmesh.ops.holes_fill(bm, edges=[e for e in bm.edges if e.is_boundary], sides=0)
    bmesh.ops.triangulate(bm, faces=res["faces"])
    bm.to_mesh(body.data); bm.free()
    co = np.array([v.co[:] for v in body.data.vertices]) / J.SCALE
    W = np.stack([co[:, 0], co[:, 2], -co[:, 1]], -1)
    u, vv = to_img("front", fb, W[:, 0], W[:, 1], W[:, 2])
    ui = np.clip(np.round(u).astype(int), 0, mf.shape[1] - 1); vi = np.clip(np.round(vv).astype(int), 0, mf.shape[0] - 1)
    arm = mf[vi, ui] & ~((lf[vi, ui] <= c0) & (rf[vi, ui] >= c0)) & (vi > pit + 6) & (vi < low)
    arm &= near_arm(W, Jw)
    side = (np.where(W[:, 0] > 0, 1, 2) * arm).astype(int)
    a = body.data.attributes.get("arm") or body.data.attributes.new("arm", "INT", "POINT")
    a.data.foreach_set("value", side.tolist())
    print("armene skilt fra kroppen: %d bindeflater fjernet, %d armhjørner, armhula ved rad %d" % (len(dead), int(arm.sum()), pit))


def mark_hips_iron(body, fb, work, sg):
    """Jern som henger på hofta (nøkler) på siden uten redskap: merkes «hipsrigid», så riggen henger det stivt
    på hoftene i stedet for å strekke det mellom lårene."""
    import cv2
    from scipy import ndimage as ndi
    from .fuse import to_img
    rgb = cv2.imread(J.p("front_rgb.png"))[:, :, ::-1]
    hsv = cv2.cvtColor(rgb, cv2.COLOR_RGB2HSV).astype(np.float32)
    iron = (hsv[..., 1] < 45) & (hsv[..., 2] > 30) & (hsv[..., 2] < 150) & _mask(work, "front")
    c = int(fb["front"]["c"])
    reg = np.zeros_like(iron)
    if sg > 0:
        reg[:, c:] = True
    else:
        reg[:, :c] = True
    o = fb["front"]; H = o["bot"] - o["top"]
    reg[: int(o["top"] + 0.38 * H)] = False
    reg[int(o["top"] + 0.62 * H):] = False
    iron = ndi.binary_dilation(ndi.binary_opening(iron & reg, iterations=1), iterations=6)
    co = np.array([v.co[:] for v in body.data.vertices]) / J.SCALE
    W = np.stack([co[:, 0], co[:, 2], -co[:, 1]], -1)
    u, vv = to_img("front", fb, W[:, 0], W[:, 1], W[:, 2])
    ui = np.clip(np.round(u).astype(int), 0, iron.shape[1] - 1); vi = np.clip(np.round(vv).astype(int), 0, iron.shape[0] - 1)
    zfront = W[:, 2] > np.percentile(W[:, 2], 55)
    m = iron[vi, ui] & zfront
    a = body.data.attributes.get("hipsrigid") or body.data.attributes.new("hipsrigid", "INT", "POINT")
    a.data.foreach_set("value", m.astype(int).tolist())
    print("jern på hofta:", int(m.sum()), "hjørner stive på hoftene")


def near_arm(W, Jw, r=0.065):
    """Hjørner som ligger nær armbeina (skulder–albue–håndledd–fingre) i 3D, ikke bare i bildet forfra."""
    out = np.zeros(len(W), bool)
    for s in ("L", "R"):
        pts = [Jw[f"shoulder.{s}"], Jw[f"elbow.{s}"], Jw[f"wrist.{s}"]]
        for a, b in zip(pts[:-1], pts[1:]):
            a = np.asarray(a, float); b = np.asarray(b, float); ab = b - a
            t = np.clip(((W - a) @ ab) / (ab @ ab), 0, 1)
            out |= np.linalg.norm(W - (a + t[:, None] * ab), axis=1) < r / J.SCALE
    return out


def split_hands(body, fb, work):
    """Skill nevene fra kroppen (de er smeltet inn i forkleet i rekonstruksjonen): hjørner nær hånda som er hud
    forfra eller bakfra blir egne deler, og hullene lukkes, så hånda kan følge hand.* stivt uten å strekke forkleet."""
    import bpy
    import bmesh
    import cv2
    from .fuse import to_img
    from .rig import joints
    Jw = joints(fb)
    co = np.array([v.co[:] for v in body.data.vertices]) / J.SCALE
    W = np.stack([co[:, 0], co[:, 2], -co[:, 1]], -1)
    skin = np.zeros(len(W), bool)
    for k in ("front", "back"):
        rgb = cv2.imread(J.p(f"{k}_rgb.png"))[:, :, ::-1]
        hsv = cv2.cvtColor(rgb, cv2.COLOR_RGB2HSV).astype(np.float32)
        h, s_, v_ = hsv[..., 0] * 2, hsv[..., 1], hsv[..., 2]
        sk = (h > 4) & (h < 26) & (s_ > 50) & (s_ < 150) & (v_ > 150)  # forkleet er like rødbrunt, men mørkere
        u, vv = to_img(k, fb, W[:, 0], W[:, 1], W[:, 2])
        ui = np.clip(np.round(u).astype(int), 0, sk.shape[1] - 1); vi = np.clip(np.round(vv).astype(int), 0, sk.shape[0] - 1)
        skin |= sk[vi, ui]
    out = []
    for side in ("R", "L"):
        a = np.asarray(Jw[f"wrist.{side}"], float); b = np.asarray(Jw[f"finger.{side}"], float)
        b = a + (b - a) * 1.25
        ab = b - a
        t = np.clip(((W - a) @ ab) / (ab @ ab), 0, 1)
        d = np.linalg.norm(W - (a + t[:, None] * ab), axis=1)
        r = 0.065 / J.SCALE
        sel = (d < r) & skin & (t > 0.0)
        print("hånd", side, "nær", int((d < r).sum()), "hud", int(sel.sum()))
        if sel.sum() < 30:
            continue
        idx = set(np.nonzero(sel)[0].tolist())
        bm = bmesh.new(); bm.from_mesh(body.data)
        flags = [sum(v.index in idx for v in f.verts) * 3 >= 2 * len(f.verts) for f in bm.faces]
        bm.free()
        _select_faces(body, flags)
        for o in bpy.context.scene.objects:
            o.select_set(o == body)
        bpy.context.view_layer.objects.active = body
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.separate(type="SELECTED")
        bpy.ops.object.mode_set(mode="OBJECT")
        hand = [o for o in bpy.context.selected_objects if o != body][0]
        hand.name = f"hand_{side}_src"
        _drop_flakes(hand, 0.2)
        for ob in (hand,):
            bm = bmesh.new(); bm.from_mesh(ob.data)
            cap = bm.faces.layers.int.get("cap") or bm.faces.layers.int.new("cap")
            res = bmesh.ops.holes_fill(bm, edges=[e for e in bm.edges if e.is_boundary], sides=0)
            tri = bmesh.ops.triangulate(bm, faces=res["faces"])
            for f in set(res["faces"]) | set(tri["faces"]):
                if f.is_valid:
                    f[cap] = 1
            bm.to_mesh(ob.data); bm.free()
        print("hånd", side, len(hand.data.polygons), "flater")
        if len(hand.data.polygons) < 8:  # ingenting igjen av neven (redskapet eller spøkelset tok den)
            bpy.data.objects.remove(hand)
            continue
        out.append(hand)
    bm = bmesh.new(); bm.from_mesh(body.data)
    res = bmesh.ops.holes_fill(bm, edges=[e for e in bm.edges if e.is_boundary], sides=0)
    bmesh.ops.triangulate(bm, faces=res["faces"])
    bm.to_mesh(body.data); bm.free()
    return out


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
