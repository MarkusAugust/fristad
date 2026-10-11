"""Skjelettet på en rekonstruert figur: de 18 beina (som `people.game_bones`) lagt ut fra silhuettene,
automatiske vekter (varmediffusjon) med armene holdt unna kroppen, klippene fra `folk_motion.clips`,
og eksport til glb med skinn og animasjoner.

Spillets figurer ser mot +x i Blender, med føttene på z = 0 og venstre side mot +y.
"""
import json
import os
import sys

import numpy as np

from . import job as J
from .texture import w2b

# høyder over sålen som andel av høyden (stilisert figur, målt på smith_body)
FRAC = dict(ankle=0.05, knee=0.27, hip=0.49, spine03=0.56, spine01=0.68, neck=0.81, head=0.86,
            shoulder=0.80, elbow=0.635, wrist=0.495, finger=0.43)


def _masks():
    import cv2
    return {k: cv2.imread(J.p(f"{k}_mask.png"), 0) > 127 for k in J.VIEWS}


def joints(fr):
    """Leddene i verdenspiksler (x mot figurens venstre, y opp med 0 på toppen, z fram)."""
    M = _masks()
    f, r = fr["front"], fr["right"]
    top, bot = f["top"], f["bot"]
    h = bot - top
    row = lambda frac: int(round(bot - frac * h))  # noqa: E731 (rad i forfra-bildet)
    wy = lambda frac: -(row(frac) - top) / f["sy"]  # noqa: E731
    wx = lambda u: (u - f["c"]) / f["sy"]  # noqa: E731
    wz = lambda u: (u - r["c"]) / r["sy"]  # noqa: E731

    def runs(m, v):
        c = np.nonzero(m[v])[0]
        if not len(c):
            return []
        cut = np.nonzero(np.diff(c) > 1)[0]
        starts = np.r_[c[0], c[cut + 1]]
        ends = np.r_[c[cut], c[-1]]
        return list(zip(starts, ends))

    def prow(frac):  # samme høyde i profilen
        return int(round(r["top"] + (row(frac) - top) * r["sy"] / f["sy"]))

    def zc(frac, which="mid"):
        rs = runs(M["right"], prow(frac))
        if not rs:
            return 0.0
        a, b = max(rs, key=lambda t: t[1] - t[0])
        return wz({"mid": (a + b) / 2, "front": b - 0.25 * (b - a), "back": a + 0.25 * (b - a)}[which])

    J_ = {}
    torso_z = zc(FRAC["spine01"])
    # beina: midten av hver beinbit forfra ved kneet og ankelen
    for name in ("knee", "ankle"):
        frac = FRAC[name]
        while len([t for t in runs(M["front"], row(frac)) if t[1] - t[0] > 20]) < 2 and frac > 0.02:
            frac -= 0.01  # forkleet kan dekke kneet: let nedover til beina står atskilt
        rs = sorted([t for t in runs(M["front"], row(frac)) if t[1] - t[0] > 20], key=lambda t: t[1] - t[0])[-2:]
        rs = sorted(rs)
        for side, (a, b) in zip(("R", "L"), rs):  # forfra: figurens høyre er til venstre i bildet
            J_[f"{name}.{side}"] = np.array([wx((a + b) / 2), wy(FRAC[name]), zc(FRAC[name])])
    for s in ("L", "R"):
        J_[f"hip.{s}"] = np.array([J_[f"knee.{s}"][0] * 0.95, wy(FRAC["hip"]), torso_z])
        sole = -(bot - top) / f["sy"]
        toe_z = wz(max(runs(M["right"], prow(0.01)), key=lambda t: t[1] - t[0])[1]) - 10
        J_[f"toe.{s}"] = np.array([J_[f"ankle.{s}"][0], sole + 0.02 * h / f["sy"], toe_z])
    # armene: de ytterste bitene forfra ved albuen og håndleddet
    arm_w = None
    for name in ("wrist", "finger", "elbow"):
        rs = runs(M["front"], row(FRAC[name]))
        (a0, b0), (a1, b1) = rs[0], rs[-1]
        if len(rs) >= 3 and name != "elbow":
            arm_w = arm_w or 0.5 * ((b0 - a0) + (b1 - a1))
        if len(rs) < 3:  # armen ligger inntil kroppen: midten av armen et halvt armmål inn fra kanten
            w = arm_w or 0.12 * (rs[-1][1] - rs[0][0])
            a0, b0 = rs[0][0], rs[0][0] + w
            a1, b1 = rs[-1][1] - w, rs[-1][1]
        J_[f"{name}.R"] = np.array([wx((a0 + b0) / 2), wy(FRAC[name]), torso_z])
        J_[f"{name}.L"] = np.array([wx((a1 + b1) / 2), wy(FRAC[name]), torso_z])
    rs = runs(M["front"], row(FRAC["shoulder"]))
    a, b = rs[0][0], rs[-1][1]
    for s, edge in (("R", a), ("L", b)):
        inward = 1 if s == "R" else -1
        J_[f"shoulder.{s}"] = np.array([wx(edge) + inward * 0.11 * (b - a) / f["sy"], wy(FRAC["shoulder"]), torso_z])
    for name in ("spine03", "spine01", "neck", "head"):
        J_[name] = np.array([0.0, wy(FRAC[name]), zc(FRAC[name])])
    J_["pelvis"] = np.array([0.0, wy(FRAC["hip"] + 0.02), torso_z])
    J_["top"] = np.array([0.0, 0.0, J_["head"][2]])
    return J_


def to_game(p, base):
    """Verden (piksler) -> Blender i spillets retning: front mot +x, sålen på z = 0, midten i origo."""
    b = w2b(np.asarray(p, float)) - base
    return np.stack([-b[..., 1], b[..., 0], b[..., 2]], -1)  # dreid 90° om z: -Y fram blir +X


def build(out=None, style=None, grip=None):
    style = style or J.OPT.get("style", "smith")
    grip = grip or J.OPT.get("grip", "hang")
    side = J.OPT.get("tool_side", "R")
    import bpy
    sys.path.insert(0, os.path.join(J.ROOT, "tools/models"))
    from common import armature
    import folk_motion
    from . import finish

    fr = json.load(open(J.p("frame.json")))["frame"]
    tools, hands = [], []
    if os.path.exists(J.p("assembled.blend")):  # hel figur fra assemble.py (hodet fra hodearket, redskapet skilt ut)
        bpy.ops.wm.open_mainfile(filepath=J.p("assembled.blend"))
        body = bpy.data.objects["body"]
        finish.build_material(body, "full")
        tools = [o for o in bpy.data.objects if o.name.startswith("tool")]
        hands = [o for o in bpy.data.objects if o.name.startswith("hand_")]
    else:
        bpy.ops.wm.open_mainfile(filepath=J.p("head.blend"))
        body = bpy.data.objects["head"]
        body.name = "body"
        finish.build_material(body)
    body.data.materials[0].name = "body"
    # flytt figuren: sålen på 0, midten i origo, front mot +x
    vs = np.empty(len(body.data.vertices) * 3, np.float32)
    body.data.vertices.foreach_get("co", vs)
    vs = vs.reshape(-1, 3)
    base = np.array([(vs[:, 0].min() + vs[:, 0].max()) / 2, (vs[:, 1].min() + vs[:, 1].max()) / 2, vs[:, 2].min()])
    v2 = vs - base
    v2 = np.stack([-v2[:, 1], v2[:, 0], v2[:, 2]], -1)
    body.data.vertices.foreach_set("co", v2.ravel().astype(np.float32))
    body.data.update()
    for t in tools + hands:
        tv = np.array([v.co[:] for v in t.data.vertices]) - base
        tv = np.stack([-tv[:, 1], tv[:, 0], tv[:, 2]], -1)
        t.data.vertices.foreach_set("co", tv.ravel().astype(np.float32))
        t.data.update()

    Jw = joints(fr)
    G = {k: tuple(to_game(v, base)) for k, v in Jw.items()}
    bones = [
        ("root", (0, 0, 0), (0, 0, 0.1), None),
        ("hips", G["pelvis"], G["spine03"], "root"),
        ("spine", G["spine03"], G["spine01"], "hips"),
        ("chest", G["spine01"], G["neck"], "spine"),
        ("neck", G["neck"], G["head"], "chest"),
        ("head", G["head"], G["top"], "neck"),
    ]
    for s in ("L", "R"):
        bones += [
            (f"upper_arm.{s}", G[f"shoulder.{s}"], G[f"elbow.{s}"], "chest"),
            (f"forearm.{s}", G[f"elbow.{s}"], G[f"wrist.{s}"], f"upper_arm.{s}"),
            (f"hand.{s}", G[f"wrist.{s}"], G[f"finger.{s}"], f"forearm.{s}"),
        ]
    for s in ("L", "R"):
        bones += [
            (f"thigh.{s}", G[f"hip.{s}"], G[f"knee.{s}"], "hips"),
            (f"shin.{s}", G[f"knee.{s}"], G[f"ankle.{s}"], f"thigh.{s}"),
            (f"foot.{s}", G[f"ankle.{s}"], G[f"toe.{s}"], f"shin.{s}"),
        ]
    rig = armature(f"{J.NAME}_rig", bones)
    for o in bpy.context.scene.objects:
        o.select_set(False)
    body.select_set(True)
    rig.select_set(True)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    clean_weights(body, G)
    for t in tools + hands:  # redskapet og nevene henger stivt på hendene
        bone = "hand.L" if t.name.startswith("hand_L") else "hand.R" if t.name.startswith("hand_R") else f"hand.{side}"
        vg = t.vertex_groups.new(name=bone)
        vg.add(list(range(len(t.data.vertices))), 1.0, "REPLACE")
        t.parent = rig
        md = t.modifiers.new("rig", "ARMATURE"); md.object = rig
    # Armene i rekonstruksjonen henger alt et stykke ut fra kroppen (bred figur, forkle), så stilens arm_out
    # legges ikke oppå; redskapet holdes klar av beina av hvileposen selv (keep_clear ville sperret armen ut).
    sty = dict(folk_motion.style_of(style)) if isinstance(style, str) else dict(style)
    sty["arm_out"] = J.OPT.get("arm_out", -6.0)
    sty["arm"] = sty.get("arm", 15.0) * J.OPT.get("arm_swing", 1.6)  # tydeligere armsving i gangen
    st = folk_motion.clips(rig, sty, {side: grip} if grip else {}, body=body, tools=None)
    out = out or os.path.join(J.SHOTS, f"npc_{J.NAME}.glb")
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    for o in bpy.context.scene.objects:
        o.select_set(o in (body, rig) or o in tools or o in hands)
    bpy.ops.export_scene.gltf(filepath=out, export_format="GLB", use_selection=True, export_apply=False,
                              export_image_format="JPEG", export_jpeg_quality=90, export_yup=True,
                              export_animations=True, export_skins=True, export_animation_mode="NLA_TRACKS",
                              export_force_sampling=True, export_optimize_animation_size=False,
                              export_cameras=False, export_lights=False)
    import subprocess
    subprocess.run([sys.executable, os.path.join(J.ROOT, "tools/models/slim.py"), out], check=False)
    print("rigget glb", out, len(body.data.polygons), "trekanter", "fart", st.get("speed"))
    bpy.ops.wm.save_as_mainfile(filepath=J.p("rigged.blend"))
    return out


def clean_weights(body, G):
    import bpy
    """Armene skal ikke dra med seg kroppen: hjørner nærmere midtlinja enn skulderleddet (og under det) mister
    armvektene; forkleet og skjorta mellom beina deles mellom hoftene og lårene."""
    groups = {vg.name: vg for vg in body.vertex_groups}
    arm = [n for n in groups if n.split(".")[0] in ("upper_arm", "forearm", "hand")]
    from mathutils import Vector

    def seg(p, a, b):
        a, b = Vector(a), Vector(b)
        ab = b - a
        t = max(0.0, min(1.0, (p - a).dot(ab) / ab.length_squared))
        return (p - (a + ab * t)).length
    arms = {s: [(G[f"shoulder.{s}"], G[f"elbow.{s}"]), (G[f"elbow.{s}"], G[f"wrist.{s}"]), (G[f"wrist.{s}"], G[f"finger.{s}"])]
            for s in ("L", "R")}
    torso = [(G["pelvis"], G["spine03"]), (G["spine03"], G["spine01"]), (G["spine01"], G["neck"]),
             (G["hip.L"], G["knee.L"]), (G["hip.R"], G["knee.R"])]
    def apron(v, p):
        # mellom beina, over kneet: halvparten til hoftene, så forkleet ikke rives av lårene
        if abs(p.y) < 0.6 * abs(G["hip.L"][1]) and G["knee.L"][2] < p.z < G["hip.L"][2]:
            w = 0.0
            for s in ("L", "R"):
                for g in v.groups:
                    if g.group == groups[f"thigh.{s}"].index:
                        w += g.weight
                        g.weight *= 0.5
            if w > 0:
                groups["hips"].add([v.index], 0.5 * w, "ADD")

    skirt = J.OPT.get("skirt", False)
    hipsr = body.data.attributes.get("hipsrigid")
    hipsv = [d.value for d in hipsr.data] if hipsr else None
    hz, az = G["hip.L"][2], 0.5 * (G["ankle.L"][2] + G["knee.L"][2])
    hy = max(abs(G["hip.L"][1]), 0.05)

    def skirt_w(v, p):
        """Skjørtet: fra hoftene og ned blandes hoftene og lårene etter høyden, og lårene etter siden, så
        skjørtet følger beina mykt uten å rives mellom dem."""
        if not skirt or p.z > hz or p.z < G["ankle.L"][2] + 0.03:
            return False
        t = min(1.0, max(0.0, (hz - p.z) / max(hz - az, 0.05)))
        sL = min(1.0, max(0.0, 0.5 + p.y / (2.5 * hy)))
        for g in list(v.groups):
            body.vertex_groups[g.group].remove([v.index])
        groups["hips"].add([v.index], 1.0 - 0.75 * t, "REPLACE")
        groups["thigh.L"].add([v.index], 0.75 * t * sL + 1e-4, "REPLACE")
        groups["thigh.R"].add([v.index], 0.75 * t * (1 - sL) + 1e-4, "REPLACE")
        return True

    armattr = body.data.attributes.get("arm")
    armv = [d.value for d in armattr.data] if armattr else None
    for v in body.data.vertices:
        p = v.co
        if hipsv is not None and hipsv[v.index]:  # nøkler o.l. stivt på hoftene
            for g in list(v.groups):
                body.vertex_groups[g.group].remove([v.index])
            groups["hips"].add([v.index], 1.0, "REPLACE")
            continue
        if armv is not None and armv[v.index]:
            # armhjørner (merket i assemble.split_arms): bare den egne armens bein
            s_ = "L" if armv[v.index] == 1 else "R"
            own = {groups[f"{b}.{s_}"].index for b in ("upper_arm", "forearm", "hand")}
            for g in list(v.groups):
                if g.group not in own:
                    body.vertex_groups[g.group].remove([v.index])
            if not v.groups:
                best = min(range(3), key=lambda k: seg(p, *arms[s_][k]))
                groups[("upper_arm", "forearm", "hand")[best] + "." + s_].add([v.index], 1.0, "REPLACE")
            continue
        if armv is not None and p.z < G["shoulder.L"][2] - 0.12:
            for n in arm:
                groups[n].remove([v.index])
            if not v.groups:
                names = ["hips", "spine", "chest", "thigh.L", "thigh.R"]
                i = min(range(len(torso)), key=lambda k: seg(p, *torso[k]))
                groups[names[i]].add([v.index], 1.0, "REPLACE")
            if not skirt_w(v, p):
                apron(v, p)
            continue
        da = min(seg(p, a, b) for s in arms for a, b in arms[s])
        dt = min(seg(p, a, b) for a, b in torso)
        # forkleet og skjorta ved siden av hendene: nærmere kroppen enn armen, eller langt fra armbeinet
        separate_hands = any(o.name.startswith("hand_") for o in bpy.data.objects)
        below_wrist = separate_hands and p.z < max(G["wrist.L"][2], G["wrist.R"][2]) + 0.01
        if da > 0.075 or below_wrist or (dt < da and p.z < G["shoulder.L"][2] - 0.08):
            for n in arm:
                groups[n].remove([v.index])
            if not v.groups:  # bare armvekter før: gi det til nærmeste bein i kroppen
                names = ["hips", "spine", "chest", "thigh.L", "thigh.R"]
                i = min(range(len(torso)), key=lambda k: seg(p, *torso[k]))
                groups[names[i]].add([v.index], 1.0, "REPLACE")
        # armene: hjørner tett ved et armbein og langt fra kroppen følger bare armen
        if da < 0.07 and da < 0.5 * dt and not below_wrist:
            arm_idx = {groups[n].index for n in arm}
            for g in list(v.groups):
                if g.group not in arm_idx:
                    body.vertex_groups[g.group].remove([v.index])
            if not v.groups:
                best = min(((seg(p, a, b), s_, k) for s_ in arms for k, (a, b) in enumerate(arms[s_])))
                groups[("upper_arm", "forearm", "hand")[best[2]] + "." + best[1]].add([v.index], 1.0, "REPLACE")
        if not separate_hands:
            # hendene er stive: neven (smeltet sammen med forkleet i rekonstruksjonen) følger bare hånda
            for s_ in ("L", "R"):
                dh = seg(p, G[f"wrist.{s_}"], G[f"finger.{s_}"])
                if dh < 0.055:
                    for g in list(v.groups):
                        body.vertex_groups[g.group].remove([v.index])
                    groups[f"hand.{s_}"].add([v.index], 1.0, "REPLACE")
        apron(v, v.co)
