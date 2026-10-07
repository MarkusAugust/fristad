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
