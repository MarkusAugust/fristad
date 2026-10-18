"""Steg 8: skjelettet på figuren. Leddene legges ut fra silhuettene (andeler av høyden fra målet, `Target.frac`),
beina kommer fra målet (`Target.bones`), vektene er automatiske (varmediffusjon) med armene holdt unna kroppen,
klippene fra målet (`Target.clips`), og eksporten er glb med skinn og animasjoner.

Figuren står i Blender med sålen på z = 0 og midten i origo, dreid slik målet vil (`Target.to_game`).
"""
import os
import time

import numpy as np

from . import geom
from .sheet import load_mask


def joints(job, fr, frac):
    """Leddene i verdenspiksler (x mot figurens venstre, y opp med 0 på toppen, z fram)."""
    M = {k: load_mask(job, k) for k in ("front", "right")}
    f, r = fr["front"], fr["right"]
    top, bot = f["top"], f["bot"]
    h = bot - top
    row = lambda fr_: int(round(bot - fr_ * h))  # noqa: E731 (rad i forfra-bildet)
    wy = lambda fr_: -(row(fr_) - top) / f["sy"]  # noqa: E731
    wx = lambda u: (u - f["c"]) / f["sy"]  # noqa: E731
    wz = lambda u: (u - r["c"]) / r["sy"]  # noqa: E731

    def runs(m, v):
        c = np.nonzero(m[min(max(v, 0), m.shape[0] - 1)])[0]
        if not len(c):
            return []
        cut = np.nonzero(np.diff(c) > 1)[0]
        starts = np.r_[c[0], c[cut + 1]]
        ends = np.r_[c[cut], c[-1]]
        return list(zip(starts, ends))

    def prow(fr_):  # samme høyde i profilen
        return int(round(r["top"] + (row(fr_) - top) * r["sy"] / f["sy"]))

    def zc(fr_, which="mid"):
        rs = runs(M["right"], prow(fr_))
        if not rs:
            return 0.0
        a, b = max(rs, key=lambda t: t[1] - t[0])
        return wz({"mid": (a + b) / 2, "front": b - 0.25 * (b - a), "back": a + 0.25 * (b - a)}[which])

    J_ = {}
    torso_z = zc(frac["spine01"])
    for name in ("knee", "ankle"):  # beina: midten av hver beinbit forfra ved kneet og ankelen
        fr_ = frac[name]
        while len([t for t in runs(M["front"], row(fr_)) if t[1] - t[0] > 20]) < 2 and fr_ > 0.02:
            fr_ -= 0.01  # forkleet kan dekke kneet: let nedover til beina står atskilt
        rs = sorted([t for t in runs(M["front"], row(fr_)) if t[1] - t[0] > 20], key=lambda t: t[1] - t[0])[-2:]
        rs = sorted(rs)
        if len(rs) < 2:  # beina står aldri atskilt (lang kjortel): del den ene biten på midten
            a, b = runs(M["front"], row(fr_))[0]
            rs = [(a, (a + b) // 2), ((a + b) // 2, b)]
        for side, (a, b) in zip(("R", "L"), rs):  # forfra: figurens høyre er til venstre i bildet
            J_[f"{name}.{side}"] = np.array([wx((a + b) / 2), wy(frac[name]), zc(frac[name])])
    for s in ("L", "R"):
        J_[f"hip.{s}"] = np.array([J_[f"knee.{s}"][0] * 0.95, wy(frac["hip"]), torso_z])
        sole = -(bot - top) / f["sy"]
        toe_z = wz(max(runs(M["right"], prow(0.01)), key=lambda t: t[1] - t[0])[1]) - 10
        J_[f"toe.{s}"] = np.array([J_[f"ankle.{s}"][0], sole + 0.02 * h / f["sy"], toe_z])
    arm_w = None
    for name in ("wrist", "finger", "elbow"):  # armene: de ytterste bitene forfra
        rs = runs(M["front"], row(frac[name]))
        (a0, b0), (a1, b1) = rs[0], rs[-1]
        if len(rs) >= 3 and name != "elbow":
            arm_w = arm_w or 0.5 * ((b0 - a0) + (b1 - a1))
        if len(rs) < 3:  # armen ligger inntil kroppen: midten av armen et halvt armmål inn fra kanten
            w = arm_w or 0.12 * (rs[-1][1] - rs[0][0])
            a0, b0 = rs[0][0], rs[0][0] + w
            a1, b1 = rs[-1][1] - w, rs[-1][1]
        J_[f"{name}.R"] = np.array([wx((a0 + b0) / 2), wy(frac[name]), torso_z])
        J_[f"{name}.L"] = np.array([wx((a1 + b1) / 2), wy(frac[name]), torso_z])
    rs = runs(M["front"], row(frac["shoulder"]))
    a, b = rs[0][0], rs[-1][1]
    for s, edge in (("R", a), ("L", b)):
        inward = 1 if s == "R" else -1
        J_[f"shoulder.{s}"] = np.array([wx(edge) + inward * 0.11 * (b - a) / f["sy"], wy(frac["shoulder"]), torso_z])
    for name in ("spine03", "spine01", "neck", "head"):
        J_[name] = np.array([0.0, wy(frac[name]), zc(frac[name])])
    J_["pelvis"] = np.array([0.0, wy(frac["hip"] + 0.02), torso_z])
    J_["top"] = np.array([0.0, 0.0, J_["head"][2]])
    return J_


def make_armature(name, bones):
    """En armatur fra en liste av (navn, hode, hale, forelder), med hode og hale i armaturrommet."""
    import bpy
    from mathutils import Vector
    data = bpy.data.armatures.new(name)
    arm = bpy.data.objects.new(name, data)
    bpy.context.collection.objects.link(arm)
    bpy.ops.object.select_all(action="DESELECT")
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.mode_set(mode="EDIT")
    made = {}
    for bname, head, tail, parent in bones:
        b = data.edit_bones.new(bname)
        b.head = Vector(head); b.tail = Vector(tail)
        if parent:
            b.parent = made[parent]
        made[bname] = b
    bpy.ops.object.mode_set(mode="OBJECT")
    return arm


def load_parts(job):
    """Delene fra `assembled.npz` (hel figur) eller `lo.npz` (bare én modell): (navn, v, f, uv) i verdenspiksler,
    pluss attributtene per hjørne i kroppen."""
    p = job.p("assembled.npz")
    if os.path.exists(p):
        d = np.load(p, allow_pickle=True)
        names = list(d["part_names"])
        parts, attrs = [], {}
        body_faces = np.isin(d["part"], [i for i, n in enumerate(names) if not n.startswith("tool")])
        parts.append(("body", d["v"], d["f"][body_faces], d["uv"][body_faces]))
        for i, n in enumerate(names):
            if n.startswith("tool"):
                sel = d["part"] == i
                parts.append((n, d["v"], d["f"][sel], d["uv"][sel]))
        for a in ("hipsrigid", "arm"):
            if a in d:
                attrs[a] = d[a]
        return parts, attrs, "albedo_full.png", "normal_full.png"
    d = np.load(job.p("lo.npz"))
    return [("body", d["v"], d["f"], d["uv"])], {}, "albedo.png", "normal.png"


def compact(v, f):
    """Bare hjørnene flatene bruker."""
    used, inv = np.unique(f, return_inverse=True)
    return v[used], inv.reshape(f.shape), used


def build(job, target, out=None, fr=None):
    import bpy
    from . import finish
    from .mesh import bpy_mesh, bpy_reset, bpy_select
    t0 = time.time()
    if fr is None:
        from .fuse import load_frame
        fr, _ = load_frame(job)
    parts, attrs, albedo, normal = load_parts(job)
    bpy_reset()
    # Blender-meter, sålen på 0, midten i origo, og målets retning
    allv = geom.w2b(parts[0][1], job.scale)
    base = np.array([(allv[:, 0].min() + allv[:, 0].max()) / 2, (allv[:, 1].min() + allv[:, 1].max()) / 2, allv[:, 2].min()])
    place = lambda P: target.to_game(geom.w2b(P, job.scale) - base)  # noqa: E731
    mat = finish.material(job, target.material, albedo, normal)
    objs, used = {}, {}
    for name, v, f, uv in parts:
        vv, ff, u = compact(v, f)
        ob = bpy_mesh(name, place(vv), ff, uv)
        ob.data.materials.append(mat)
        bpy_select(ob)
        bpy.ops.object.shade_smooth()
        objs[name], used[name] = ob, u
    body = objs["body"]
    Jw = joints(job, fr, target.frac)
    G = {k: tuple(place(np.asarray(v, float)[None])[0]) for k, v in Jw.items()}
    rig = make_armature(f"{job.name}_rig", target.bones(G))
    bpy_select(body, rig, active=rig)
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    clean_weights(job, body, G, {k: (a[used["body"]] if a is not None else None) for k, a in attrs.items()},
                  separate_hands=False)
    side = job.opt.get("tool_side", "R")
    tools = [o for n, o in objs.items() if n.startswith("tool")]
    for t in tools:  # redskapet henger stivt på hånda
        vg = t.vertex_groups.new(name=target.hand_bone(side))
        vg.add(list(range(len(t.data.vertices))), 1.0, "REPLACE")
        t.parent = rig
        md = t.modifiers.new("rig", "ARMATURE"); md.object = rig
    info = target.clips(rig, body, job.opt, tools=tools) or {}
    out = out or job.shot(f"npc_{job.name}.glb")
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    bpy_select(body, rig, *tools, active=rig)
    kw = dict(export_animations=True, export_skins=True, export_animation_mode="NLA_TRACKS",
              export_force_sampling=True, export_optimize_animation_size=False) if info else dict(export_skins=True)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", use_selection=True, export_apply=False,
                              export_image_format="JPEG", export_jpeg_quality=target.jpeg_quality, export_yup=True,
                              export_cameras=False, export_lights=False, **kw)
    target.post_export(out)
    try:
        bpy.ops.wm.save_as_mainfile(filepath=job.p("rigged.blend"))
    except Exception as e:  # noqa: BLE001
        print("rigged.blend ikke lagret:", e)
    job.time("rig", time.time() - t0)
    print("rigget glb", out, len(body.data.polygons), "trekanter", info)
    return out, dict(bones=len(rig.data.bones), tris=len(body.data.polygons), **info)


def clean_weights(job, body, G, attrs, separate_hands=False):
    """Armene skal ikke dra med seg kroppen: hjørner nærmere midtlinja enn skulderleddet (og under det) mister
    armvektene; forkleet og skjorta mellom beina deles mellom hoftene og lårene; nevene er stive."""
    from mathutils import Vector
    groups = {vg.name: vg for vg in body.vertex_groups}
    need = ["hips", "spine", "chest", "thigh.L", "thigh.R", "upper_arm.L", "hand.L", "hand.R"]
    if any(n not in groups for n in need):
        return  # et annet skjelett: la de automatiske vektene stå
    arm = [n for n in groups if n.split(".")[0] in ("upper_arm", "forearm", "hand")]

    def seg(p, a, b):
        a, b = Vector(a), Vector(b)
        ab = b - a
        t = max(0.0, min(1.0, (p - a).dot(ab) / max(ab.length_squared, 1e-9)))
        return (p - (a + ab * t)).length
    arms = {s: [(G[f"shoulder.{s}"], G[f"elbow.{s}"]), (G[f"elbow.{s}"], G[f"wrist.{s}"]), (G[f"wrist.{s}"], G[f"finger.{s}"])]
            for s in ("L", "R")}
    torso = [(G["pelvis"], G["spine03"]), (G["spine03"], G["spine01"]), (G["spine01"], G["neck"]),
             (G["hip.L"], G["knee.L"]), (G["hip.R"], G["knee.R"])]
    names = ["hips", "spine", "chest", "thigh.L", "thigh.R"]
    up = 2  # z opp i Blender
    side_ax = 1 if abs(G["hip.L"][1] - G["hip.R"][1]) > abs(G["hip.L"][0] - G["hip.R"][0]) else 0

    def apron(v, p):
        if abs(p[side_ax]) < 0.6 * abs(G["hip.L"][side_ax]) and G["knee.L"][up] < p[up] < G["hip.L"][up]:
            w = 0.0
            for s in ("L", "R"):
                for g in v.groups:
                    if g.group == groups[f"thigh.{s}"].index:
                        w += g.weight
                        g.weight *= 0.5
            if w > 0:
                groups["hips"].add([v.index], 0.5 * w, "ADD")

    skirt = job.opt.get("skirt", False)
    hipsv = attrs.get("hipsrigid")
    armv = attrs.get("arm")
    hz, az = G["hip.L"][up], 0.5 * (G["ankle.L"][up] + G["knee.L"][up])
    hy = max(abs(G["hip.L"][side_ax]), 0.05)

    def skirt_w(v, p):
        if not skirt or p[up] > hz or p[up] < G["ankle.L"][up] + 0.03:
            return False
        t = min(1.0, max(0.0, (hz - p[up]) / max(hz - az, 0.05)))
        sL = min(1.0, max(0.0, 0.5 + p[side_ax] / (2.5 * hy)))
        for g in list(v.groups):
            body.vertex_groups[g.group].remove([v.index])
        groups["hips"].add([v.index], 1.0 - 0.75 * t, "REPLACE")
        groups["thigh.L"].add([v.index], 0.75 * t * sL + 1e-4, "REPLACE")
        groups["thigh.R"].add([v.index], 0.75 * t * (1 - sL) + 1e-4, "REPLACE")
        return True

    for v in body.data.vertices:
        p = v.co
        if hipsv is not None and hipsv[v.index]:  # nøkler o.l. stivt på hoftene
            for g in list(v.groups):
                body.vertex_groups[g.group].remove([v.index])
            groups["hips"].add([v.index], 1.0, "REPLACE")
            continue
        if armv is not None and armv[v.index]:
            s_ = "L" if armv[v.index] == 1 else "R"
            own = {groups[f"{b}.{s_}"].index for b in ("upper_arm", "forearm", "hand")}
            for g in list(v.groups):
                if g.group not in own:
                    body.vertex_groups[g.group].remove([v.index])
            if not v.groups:
                best = min(range(3), key=lambda k: seg(p, *arms[s_][k]))
                groups[("upper_arm", "forearm", "hand")[best] + "." + s_].add([v.index], 1.0, "REPLACE")
            continue
        da = min(seg(p, a, b) for s in arms for a, b in arms[s])
        dt = min(seg(p, a, b) for a, b in torso)
        if da > 0.075 or (dt < da and p[up] < G["shoulder.L"][up] - 0.08):
            for n in arm:
                groups[n].remove([v.index])
            if not v.groups:
                i = min(range(len(torso)), key=lambda k: seg(p, *torso[k]))
                groups[names[i]].add([v.index], 1.0, "REPLACE")
        if da < 0.07 and da < 0.5 * dt:  # tett ved et armbein og langt fra kroppen: bare armen
            arm_idx = {groups[n].index for n in arm}
            for g in list(v.groups):
                if g.group not in arm_idx:
                    body.vertex_groups[g.group].remove([v.index])
            if not v.groups:
                best = min(((seg(p, a, b), s_, k) for s_ in arms for k, (a, b) in enumerate(arms[s_])))
                groups[("upper_arm", "forearm", "hand")[best[2]] + "." + best[1]].add([v.index], 1.0, "REPLACE")
        if not separate_hands:  # nevene er stive
            for s_ in ("L", "R"):
                if seg(p, G[f"wrist.{s_}"], G[f"finger.{s_}"]) < 0.055:
                    for g in list(v.groups):
                        body.vertex_groups[g.group].remove([v.index])
                    groups[f"hand.{s_}"].add([v.index], 1.0, "REPLACE")
        if not skirt_w(v, p):
            apron(v, p)
