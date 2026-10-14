"""Felles hjelpere for modellene i tools/models/: materialer, primitiver,
glatting og eksport. Blender-aksene er X fram, Y til venstre, Z opp; glTF-
eksporten gjør det til Y opp, så X fram og Z mot høyre i motoren."""

import math

import bpy
import slim
import bmesh
from mathutils import Vector


def clear():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def material(name, rgb, rough, metal=0.0):
    m = bpy.data.materials.get(name)
    if m:
        return m
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*rgb, 1.0)
    bsdf.inputs["Roughness"].default_value = rough
    bsdf.inputs["Metallic"].default_value = metal
    return m


def new_object(name, bm, mat_list):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    for m in mat_list:
        me.materials.append(m)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def smooth(ob, angle=40.0):
    for p in ob.data.polygons:
        p.use_smooth = True
    with bpy.context.temp_override(object=ob, selected_editable_objects=[ob]):
        bpy.ops.object.shade_smooth_by_angle(angle=math.radians(angle))


def apply_modifiers(ob):
    bpy.context.view_layer.objects.active = ob
    for mod in list(ob.modifiers):
        with bpy.context.temp_override(object=ob):
            bpy.ops.object.modifier_apply(modifier=mod.name)


def apply_scale(ob):
    with bpy.context.temp_override(object=ob, selected_editable_objects=[ob]):
        bpy.ops.object.transform_apply(scale=True)


def box(name, center, size, mat, rot=(0, 0, 0)):
    bpy.ops.mesh.primitive_cube_add(location=center, rotation=rot)
    ob = bpy.context.active_object
    ob.name = name
    ob.scale = (size[0] / 2, size[1] / 2, size[2] / 2)
    ob.data.materials.append(mat)
    return ob


def rounded_box(name, center, size, mat, radius=0.01, rot=(0, 0, 0), segments=2):
    ob = box(name, center, size, mat, rot)
    apply_scale(ob)
    mod = ob.modifiers.new("bevel", "BEVEL")
    mod.width = radius
    mod.segments = segments
    apply_modifiers(ob)
    smooth(ob, 50)
    return ob


def ellipsoid(name, center, size, mat, segments=20, rings=10):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=rings, location=center)
    ob = bpy.context.active_object
    ob.name = name
    ob.scale = (size[0] / 2, size[1] / 2, size[2] / 2)
    ob.data.materials.append(mat)
    smooth(ob, 80)
    return ob


def cylinder(name, center, radius, depth, mat, rot=(0, 0, 0), vertices=16, cap="NGON"):
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=center, rotation=rot, end_fill_type=cap)
    ob = bpy.context.active_object
    ob.name = name
    ob.data.materials.append(mat)
    if vertices >= 10:
        smooth(ob, 45)
    return ob


def tube(name, a, b, radius, mat, vertices=10):
    """Et rør fra punkt a til punkt b."""
    a, b = Vector(a), Vector(b)
    d = b - a
    rot = d.to_track_quat("Z", "Y").to_euler()
    return cylinder(name, (a + b) / 2, radius, d.length, mat, rot=rot, vertices=vertices)


def spin(name, profile, mat, steps=24):
    """Dreier et profil (liste av (r, z)) om Z-aksen, som en lathe."""
    bm = bmesh.new()
    verts = [bm.verts.new((r, 0.0, z)) for r, z in profile]
    edges = [bm.edges.new((verts[i], verts[i + 1])) for i in range(len(verts) - 1)]
    bmesh.ops.spin(bm, geom=verts + edges, cent=(0, 0, 0), axis=(0, 0, 1), angle=2 * math.pi, steps=steps, use_merge=True)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-4)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = new_object(name, bm, [mat])
    smooth(ob, 45)
    return ob


def export(path):
    for name in ("src", "src_side"):
        if ob.data.uv_layers.get(name) is not None:
            ob.data.uv_layers.remove(ob.data.uv_layers[name])
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=False, export_animations=armature is not None, export_skins=armature is not None, export_animation_mode="NLA_TRACKS", export_force_sampling=True, export_optimize_animation_size=False, export_cameras=False, export_lights=False)
    slim.slim(path)  # normalene og hjørnefargen pakkes (slim.py)
    tris = sum(len(p.vertices) - 2 for ob in bpy.data.objects if ob.type == "MESH" for p in ob.data.polygons)
    print(f"{path}: {tris} trekanter")


def weather(mat, dirt_from=0.0, dirt_to=0.6, strength=0.45, noise_scale=14.0):
    """Skitt og slitasje på et materiale: fargen mørkner mot bunnen (z fra
    dirt_from til dirt_to i objektets koordinater), med støy så det blir
    flekkete. Brukes før baking, så det havner i albedoen."""
    nodes, links = mat.node_tree.nodes, mat.node_tree.links
    bsdf = nodes["Principled BSDF"]
    base = bsdf.inputs["Base Color"].default_value[:]
    src = bsdf.inputs["Base Color"].links[0].from_socket if bsdf.inputs["Base Color"].is_linked else None
    if src is None:
        rgb = nodes.new("ShaderNodeRGB")
        rgb.outputs[0].default_value = base
        src = rgb.outputs[0]
    geo = nodes.new("ShaderNodeNewGeometry")
    sep = nodes.new("ShaderNodeSeparateXYZ")
    links.new(geo.outputs["Position"], sep.inputs[0])
    ramp = nodes.new("ShaderNodeMapRange")
    ramp.inputs["From Min"].default_value = dirt_from
    ramp.inputs["From Max"].default_value = dirt_to
    ramp.inputs["To Min"].default_value = 1.0 - strength
    ramp.inputs["To Max"].default_value = 1.0
    links.new(sep.outputs["Z"], ramp.inputs["Value"])
    noise = nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = noise_scale
    noise.inputs["Detail"].default_value = 4.0
    mixn = nodes.new("ShaderNodeMix")
    mixn.data_type = "FLOAT"
    mixn.inputs["Factor"].default_value = 0.4
    links.new(ramp.outputs["Result"], mixn.inputs["A"])
    noise_fac = nodes.new("ShaderNodeMapRange")
    links.new(noise.outputs["Fac"], noise_fac.inputs["Value"])
    noise_fac.inputs["To Min"].default_value = 0.75
    noise_fac.inputs["To Max"].default_value = 1.05
    links.new(noise_fac.outputs["Result"], mixn.inputs["B"])
    gray = nodes.new("ShaderNodeCombineColor")
    for ch in ("Red", "Green", "Blue"):
        links.new(mixn.outputs["Result"], gray.inputs[ch])
    mul = nodes.new("ShaderNodeMix")
    mul.data_type = "RGBA"
    mul.blend_type = "MULTIPLY"
    mul.inputs["Factor"].default_value = 1.0
    links.new(src, mul.inputs["A"])
    links.new(gray.outputs[0], mul.inputs["B"])
    links.new(mul.outputs["Result"], bsdf.inputs["Base Color"])


# Siden på flisen som de stablede øyene (tetthet 0) deler, som en del av
# kvadratroten av flaten til resten av øyene.
STACK = 0.12


def bake_textured(path, size=1024, keep=("glass", "glow", "mesh"), samples=16, armature=None, rigid=None, skin=None, density=None, jpeg_quality=82):
    """Slår sammen alle mesh-objektene (unntatt dem med materialer som
    begynner på `keep`, som beholdes som de er), UV-pakker, baker albedo × AO,
    normalkart og ruhet/metall til teksturer, bytter til teksturerte
    materialer og eksporterer glTF. Materialer som begynner på «paint» bakes
    hvite og beholder navnet, så motoren farger dem per instans; resten
    samles i «body» (metallfaktoren ligger i teksturen).

    `density` (objektnavn-prefiks → faktor) gir delene mer eller mindre
    plass i teksturen enn flatene deres tilsier, så ansiktet ikke presses
    ned til noen få piksler; 0 stabler øyene oppå hverandre i én flis
    (hårlokker og skjeggtuster). Et objekt kan
    også ha flateattributtet «uv_density» selv (ansiktet på en figur)."""
    import os

    import numpy as np

    def kept(ob):
        return any(m is not None and m.name.startswith(k) for m in ob.data.materials for k in keep)

    meshes = [o for o in bpy.data.objects if o.type == "MESH" and not kept(o)]
    kept_objects = [o for o in bpy.data.objects if o.type == "MESH" and kept(o)]
    # Faste tildelinger til bein (objektnavn-prefiks → bein) settes før
    # sammenslåingen, så gruppene følger hjørnene.
    for o in meshes + kept_objects:
        for prefix, bone in (rigid or {}).items():
            if o.name.startswith(prefix):
                rigid_group(o, bone)
    weighted = bool(density) or any("uv_density" in o.data.attributes for o in meshes)
    if weighted:
        for o in meshes:
            if "uv_density" not in o.data.attributes:
                f = next((v for k, v in (density or {}).items() if o.name.startswith(k)), 1.0)
                a = o.data.attributes.new("uv_density", "FLOAT", "FACE")
                a.data.foreach_set("value", [f] * len(o.data.polygons))
    bpy.ops.object.select_all(action="DESELECT")
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    ob = bpy.context.active_object
    with bpy.context.temp_override(object=ob, selected_editable_objects=[ob]):
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    if armature is not None:
        # Skinning: vekter etter beina, armaturmodifikator, og de beholdte
        # objektene (glødende øyne) henger på samme skjelett.
        for o in [ob] + kept_objects:
            weight_by_bones(o, armature, **(skin or {}))
            mod = o.modifiers.new("Armature", "ARMATURE")
            mod.object = armature
            o.parent = armature
    # Et uv-lag «src» er kilden til et bilde i materialet (et forbilde lagt
    # på en flate, monsters.py); atlaset bakes i det første laget, og «src»
    # tas ut før eksporten.
    src = ob.data.uv_layers.get("src")
    if src is not None:
        first = next(l for l in ob.data.uv_layers if not l.name.startswith("src"))
        ob.data.uv_layers.active = first
        first.active_render = True
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=1.15, island_margin=0.003)
    bpy.ops.object.mode_set(mode="OBJECT")
    if weighted:
        # Hver øy skaleres etter tettheten sin, og så pakkes de på nytt
        # (pakkingen beholder forholdet mellom øyene).
        import bmesh
        from bpy_extras.bmesh_utils import bmesh_linked_uv_islands

        bm = bmesh.new()
        bm.from_mesh(ob.data)
        uv = bm.loops.layers.uv.active
        dens = bm.faces.layers.float.get("uv_density")
        stacked = []
        area = 0.0
        # Hver øy flyttes til sin egen rute langt unna før pakkingen: når
        # øyene skaleres om midten sin, vokser de inn i naboene, og
        # pakkingen (som slår sammen øyer som overlapper, for de stablede)
        # ville latt dem ligge oppå hverandre, så bakingen skrev ansiktet
        # over med kroppen under klærne.
        spread = 0
        for island in bmesh_linked_uv_islands(bm, uv):
            w = sum(f[dens] for f in island) / len(island)
            loops = [l for f in island for l in f.loops]
            if w <= 0.0:
                stacked.append(loops)
                continue
            if abs(w - 1.0) > 1e-3:
                cu = sum(l[uv].uv.x for l in loops) / len(loops)
                cv = sum(l[uv].uv.y for l in loops) / len(loops)
                for l in loops:
                    l[uv].uv.x = cu + (l[uv].uv.x - cu) * w
                    l[uv].uv.y = cv + (l[uv].uv.y - cv) * w
            spread += 1
            for l in loops:
                l[uv].uv.x += 10.0 * spread
            for f in island:
                pts = [l[uv].uv for l in f.loops]
                area += abs(sum(pts[k].x * pts[k - 1].y - pts[k - 1].x * pts[k].y for k in range(len(pts)))) / 2
        # Tetthet 0: øyene legges oppå hverandre i én liten flis (hårlokker
        # og skjeggtuster, som ser like ut uansett), så tusenvis av øyer ikke
        # spiser teksturen. Flisen er en liten del av resten.
        side = STACK * area ** 0.5
        for loops in stacked:
            u0 = min(l[uv].uv.x for l in loops)
            v0 = min(l[uv].uv.y for l in loops)
            du = max(max(l[uv].uv.x for l in loops) - u0, 1e-6)
            dv = max(max(l[uv].uv.y for l in loops) - v0, 1e-6)
            for l in loops:
                l[uv].uv.x = (l[uv].uv.x - u0) / du * side
                l[uv].uv.y = (l[uv].uv.y - v0) / dv * side
        bm.to_mesh(ob.data)
        bm.free()
        bpy.ops.object.mode_set(mode="EDIT")
        bpy.ops.mesh.select_all(action="SELECT")
        bpy.ops.uv.select_all(action="SELECT")
        bpy.ops.uv.pack_islands(rotate=True, margin=0.004, merge_overlap=True)
        bpy.ops.object.mode_set(mode="OBJECT")
        ob.data.attributes.remove(ob.data.attributes["uv_density"])
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = samples
    # Kanten rundt øyene fylles godt ut, så mipmapene ikke blander inn
    # svart fra mellomrommene på avstand.
    scene.render.bake.margin = 8
    # Tomme materialplasser (fra boolean-kutt) fjernes.
    for i in reversed(range(len(ob.data.materials))):
        if ob.data.materials[i] is None:
            ob.data.materials.pop(index=i)
    for mat in ob.data.materials:
        if mat.name.startswith("paint"):
            bsdf = mat.node_tree.nodes["Principled BSDF"]
            if bsdf.inputs["Base Color"].is_linked:
                # Skitten er lagt på: bytt grunnfargen under den til hvit.
                src = bsdf.inputs["Base Color"].links[0].from_node
                for n in mat.node_tree.nodes:
                    if n.bl_idname == "ShaderNodeRGB":
                        n.outputs[0].default_value = (1.0, 1.0, 1.0, 1.0)
                del src
            else:
                bsdf.inputs["Base Color"].default_value = (1.0, 1.0, 1.0, 1.0)

    def bake_to(label, bake_type, colorspace, **kw):
        img = bpy.data.images.new(label, size, size, alpha=False)
        img.colorspace_settings.name = colorspace
        for mat in ob.data.materials:
            tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
            tex.image = img
            tex.name = "bake_target"
            mat.node_tree.nodes.active = tex
        bpy.ops.object.bake(type=bake_type, **kw)
        for mat in ob.data.materials:
            n = mat.node_tree.nodes.get("bake_target")
            if n:
                mat.node_tree.nodes.remove(n)
        return img

    # Metall har ingen diffus farge i Cycles, så albedoen bakes med
    # metallfaktoren skrudd av; metallkartet bakes for seg lenger ned.
    metals = []
    for mat in ob.data.materials:
        bsdf = mat.node_tree.nodes["Principled BSDF"]
        metals.append(bsdf.inputs["Metallic"].default_value)
        bsdf.inputs["Metallic"].default_value = 0.0
    albedo = bake_to("albedo", "DIFFUSE", "sRGB", pass_filter={"COLOR"})
    for mat, m in zip(ob.data.materials, metals):
        mat.node_tree.nodes["Principled BSDF"].inputs["Metallic"].default_value = m
    bumps = [n for mat in ob.data.materials for n in mat.node_tree.nodes if n.bl_idname == "ShaderNodeBump"]
    for n in bumps:
        n.mute = True
    ao = bake_to("ao", "AO", "Non-Color")
    for n in bumps:
        n.mute = False
    normal = bake_to("normal", "NORMAL", "Non-Color", normal_space="TANGENT")
    rough = bake_to("rough", "ROUGHNESS", "Non-Color")
    for mat in ob.data.materials:
        bsdf = mat.node_tree.nodes["Principled BSDF"]
        m = bsdf.inputs["Metallic"].default_value
        bsdf.inputs["Emission Color"].default_value = (m, m, m, 1.0)
        bsdf.inputs["Emission Strength"].default_value = 1.0
    metal = bake_to("metal", "EMIT", "Non-Color")
    a = np.array(albedo.pixels[:]).reshape(size, size, 4)
    o = np.array(ao.pixels[:]).reshape(size, size, 4)
    a[:, :, :3] *= np.power(np.clip(o[:, :, :1], 0.0, 1.0), 0.8)
    albedo.pixels = a.ravel().tolist()
    r = np.array(rough.pixels[:]).reshape(size, size, 4)
    mt = np.array(metal.pixels[:]).reshape(size, size, 4)
    mr = np.zeros_like(r)
    mr[:, :, 1] = r[:, :, 0]
    mr[:, :, 2] = mt[:, :, 0]
    mr[:, :, 3] = 1.0
    mr_img = bpy.data.images.new("mr", size, size, alpha=False)
    mr_img.colorspace_settings.name = "Non-Color"
    mr_img.pixels = mr.ravel().tolist()
    tex_mat = {}

    def key_of(name):
        return "paint" if name.startswith("paint") else "body"

    for mat in ob.data.materials:
        key = key_of(mat.name)
        if key not in tex_mat:
            nm = bpy.data.materials.new(key)
            nm.use_nodes = True
            nt = nm.node_tree
            bsdf = nt.nodes["Principled BSDF"]
            bsdf.inputs["Base Color"].default_value = (1, 1, 1, 1)
            bsdf.inputs["Roughness"].default_value = 1.0
            bsdf.inputs["Emission Strength"].default_value = 0.0
            ta = nt.nodes.new("ShaderNodeTexImage")
            ta.image = albedo
            nt.links.new(ta.outputs[0], bsdf.inputs["Base Color"])
            tn = nt.nodes.new("ShaderNodeTexImage")
            tn.image = normal
            nmap = nt.nodes.new("ShaderNodeNormalMap")
            nt.links.new(tn.outputs[0], nmap.inputs["Color"])
            nt.links.new(nmap.outputs["Normal"], bsdf.inputs["Normal"])
            tm = nt.nodes.new("ShaderNodeTexImage")
            tm.image = mr_img
            sep = nt.nodes.new("ShaderNodeSeparateColor")
            nt.links.new(tm.outputs[0], sep.inputs[0])
            nt.links.new(sep.outputs["Green"], bsdf.inputs["Roughness"])
            nt.links.new(sep.outputs["Blue"], bsdf.inputs["Metallic"])
            # Metallfaktoren 1 i materialet, så motoren leser teksturen;
            # lakken er aldri metall.
            bsdf.inputs["Metallic"].default_value = 0.0 if key == "paint" else 1.0
            tex_mat[key] = nm
    for slot in ob.material_slots:
        slot.material = tex_mat[key_of(slot.material.name)]
    for name in ("src", "src_side"):
        if ob.data.uv_layers.get(name) is not None:
            ob.data.uv_layers.remove(ob.data.uv_layers[name])
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=True, export_image_format="JPEG", export_jpeg_quality=jpeg_quality, export_animations=armature is not None, export_skins=armature is not None, export_animation_mode="NLA_TRACKS", export_force_sampling=True, export_optimize_animation_size=False, export_cameras=False, export_lights=False)
    slim.slim(path)  # normalene og hjørnefargen pakkes (slim.py)
    tris = sum(len(p.vertices) - 2 for o in bpy.data.objects if o.type == "MESH" for p in o.data.polygons)
    print(f"{path}: {tris} trekanter, {os.path.getsize(path) // 1024} kB")


# --- Rigging og animasjon -------------------------------------------------


def armature(name, bones):
    """En armatur fra en liste av (navn, hode, hale, forelder), med hode og
    hale i armaturrommet. Foreldrene må komme før barna."""
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
        b.head = Vector(head)
        b.tail = Vector(tail)
        if parent:
            b.parent = made[parent]
        made[bname] = b
    bpy.ops.object.mode_set(mode="OBJECT")
    return arm


def rigid_group(ob, bone):
    """Alle hjørnene i objektet henger stivt på ett bein (gruppa overlever
    sammenslåingen i bake_textured)."""
    vg = ob.vertex_groups.new(name=bone)
    vg.add([v.index for v in ob.data.vertices], 1.0, "REPLACE")


def weight_by_bones(ob, arm, blend=0.06, radii=None):
    """Vekter hjørnene som ikke alt har en gruppe etter avstanden til
    beina: det nærmeste beinet får mest, og det nest nærmeste blandes inn
    når avstandene er nesten like (innen `blend` meter). `radii` gir et
    bein større rekkevidde (avstanden minus radius)."""
    radii = radii or {}
    bones = [(b.name, Vector(b.head_local), Vector(b.tail_local), radii.get(b.name, 0.0)) for b in arm.data.bones]
    groups = {}
    for name, _, _, _ in bones:
        groups[name] = ob.vertex_groups.get(name) or ob.vertex_groups.new(name=name)
    mw = ob.matrix_world
    for v in ob.data.vertices:
        if len(v.groups) > 0:
            continue
        p = mw @ v.co
        ds = []
        for name, h, t, r in bones:
            ab = t - h
            l2 = ab.length_squared
            u = 0.0 if l2 < 1e-9 else max(0.0, min(1.0, (p - h).dot(ab) / l2))
            ds.append(((p - (h + ab * u)).length - r, name))
        ds.sort()
        d1, n1 = ds[0]
        if len(ds) > 1:
            d2, n2 = ds[1]
            w2 = max(0.0, 1.0 - (d2 - d1) / blend) * 0.5
        else:
            n2, w2 = n1, 0.0
        groups[n1].add([v.index], 1.0 - w2, "REPLACE")
        if w2 > 0.0:
            groups[n2].add([v.index], w2, "REPLACE")


class Clip:
    """Et klipp: en action med nøkkelbilder, lagt på et eget NLA-spor så
    glTF-eksporten tar den med som egen animasjon. Dreiningene gis i
    armaturrommet om beinets hode (x, y, z i grader, X først), så en
    klipp-definisjon ikke trenger å vite hvordan beinet er rullet."""

    def __init__(self, arm, name, length, fps=24):
        from mathutils import Matrix

        self.arm = arm
        self.name = name
        self.length = length
        self.Matrix = Matrix
        self.act = bpy.data.actions.new(name)
        arm.animation_data_create()
        arm.animation_data.action = self.act
        if hasattr(self.act, "slots") and len(self.act.slots) == 0:
            slot = self.act.slots.new(id_type="OBJECT", name=arm.name)
            arm.animation_data.action_slot = slot
        self.slot = getattr(arm.animation_data, "action_slot", None)
        for pb in arm.pose.bones:
            pb.rotation_mode = "QUATERNION"
            pb.rotation_quaternion = (1, 0, 0, 0)
            pb.location = (0, 0, 0)
            pb.scale = (1, 1, 1)
        bpy.context.scene.render.fps = fps

    def _rest(self, bone):
        return self.arm.data.bones[bone].matrix_local.to_3x3()

    def rot(self, bone, frame, x=0.0, y=0.0, z=0.0):
        M = self.Matrix
        R = M.Rotation(math.radians(z), 3, "Z") @ M.Rotation(math.radians(y), 3, "Y") @ M.Rotation(math.radians(x), 3, "X")
        rest = self._rest(bone)
        basis = rest.inverted() @ R @ rest
        pb = self.arm.pose.bones[bone]
        pb.rotation_quaternion = basis.to_quaternion()
        pb.keyframe_insert("rotation_quaternion", frame=frame)

    def rotq(self, bone, frame, q):
        """Som `rot`, men dreiningen gis som kvaternion i armaturrommet
        (fra IK-løseren i mob_motion.py)."""
        rest = self._rest(bone)
        basis = rest.inverted() @ q.to_matrix() @ rest
        pb = self.arm.pose.bones[bone]
        pb.rotation_quaternion = basis.to_quaternion()
        pb.keyframe_insert("rotation_quaternion", frame=frame)

    def loc(self, bone, frame, v):
        rest = self._rest(bone)
        pb = self.arm.pose.bones[bone]
        pb.location = rest.inverted() @ Vector(v)
        pb.keyframe_insert("location", frame=frame)

    def scale(self, bone, frame, s):
        pb = self.arm.pose.bones[bone]
        pb.scale = (s, s, s) if isinstance(s, (int, float)) else s
        pb.keyframe_insert("scale", frame=frame)

    def rest_all(self, frame):
        """Hvileposen for alle bein i dette bildet."""
        for pb in self.arm.pose.bones:
            self.rot(pb.name, frame)
            self.loc(pb.name, frame, (0, 0, 0))

    def finish(self):
        ad = self.arm.animation_data
        ad.action = None
        track = ad.nla_tracks.new()
        track.name = self.name
        strip = track.strips.new(self.name, 0, self.act)
        if self.slot is not None and hasattr(strip, "action_slot"):
            strip.action_slot = self.slot
        strip.action_frame_start = 0
        strip.action_frame_end = self.length
        strip.frame_end = self.length
        return self.act


def wave(length, period, amp, phase=0.0, step=4, offset=0.0):
    """(bilde, verdi) for en sinus over klippet, til løkker som går opp."""
    return [(f, offset + amp * math.sin(2 * math.pi * f / period + phase)) for f in range(0, length + 1, step)]
