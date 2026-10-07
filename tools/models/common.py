"""Felles hjelpere for modellene i tools/models/: materialer, primitiver,
glatting og eksport. Blender-aksene er X fram, Y til venstre, Z opp; glTF-
eksporten gjør det til Y opp, så X fram og Z mot høyre i motoren."""

import math

import bpy
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
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=False, export_animations=False, export_skins=False, export_cameras=False, export_lights=False)
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


def bake_textured(path, size=1024, keep=("glass", "glow", "mesh"), samples=16):
    """Slår sammen alle mesh-objektene (unntatt dem med materialer som
    begynner på `keep`, som beholdes som de er), UV-pakker, baker albedo × AO,
    normalkart og ruhet/metall til teksturer, bytter til teksturerte
    materialer og eksporterer glTF. Materialer som begynner på «paint» bakes
    hvite og beholder navnet, så motoren farger dem per instans; resten
    samles i «body» (metallfaktoren ligger i teksturen)."""
    import os

    import numpy as np

    def kept(ob):
        return any(m is not None and m.name.startswith(k) for m in ob.data.materials for k in keep)

    meshes = [o for o in bpy.data.objects if o.type == "MESH" and not kept(o)]
    bpy.ops.object.select_all(action="DESELECT")
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    ob = bpy.context.active_object
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=1.15, island_margin=0.003)
    bpy.ops.object.mode_set(mode="OBJECT")
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = samples
    scene.render.bake.margin = 4
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

    albedo = bake_to("albedo", "DIFFUSE", "sRGB", pass_filter={"COLOR"})
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
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=True, export_image_format="JPEG", export_jpeg_quality=82, export_animations=False, export_skins=False, export_cameras=False, export_lights=False)
    tris = sum(len(p.vertices) - 2 for o in bpy.data.objects if o.type == "MESH" for p in o.data.polygons)
    print(f"{path}: {tris} trekanter, {os.path.getsize(path) // 1024} kB")
