"""Baker teksturer på bilene: UV-pakker alle delene, baker lys-uavhengig
albedo (materialfargene med AO og skitt nederst) og ruhet/metall til ett
atlas per bil, og eksporterer glTF med teksturene. Lakken bakes hvit, så
instansfargen i motoren virker. Kjør etter cars.py:

    python3 tools/models/bake_cars.py [estate hatch van] [størrelse]
"""

import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402

OUT = "assets/models/cars"


def bake(kind, size=1024):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=f"{OUT}/{kind}.glb")
    meshes = [o for o in bpy.data.objects if o.type == "MESH"]
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    # Ett objekt med alle materialene, så atlaset deles.
    bpy.ops.object.join()
    car = bpy.context.active_object
    car.name = "car"
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=1.15, island_margin=0.003)
    bpy.ops.object.mode_set(mode="OBJECT")

    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 24
    scene.render.bake.margin = 4

    # Lakken hvit (instansfargen ganges inn), litt skitt nederst på alt.
    for mat in car.data.materials:
        nodes = mat.node_tree.nodes
        bsdf = nodes["Principled BSDF"]
        if mat.name.startswith("paint"):
            bsdf.inputs["Base Color"].default_value = (1.0, 1.0, 1.0, 1.0)
        # Skitt: en gradient fra hjulhøyde og ned, mørkner fargen.
        base = bsdf.inputs["Base Color"].default_value[:]
        rgb = nodes.new("ShaderNodeRGB")
        rgb.outputs[0].default_value = base
        geo = nodes.new("ShaderNodeNewGeometry")
        sep = nodes.new("ShaderNodeSeparateXYZ")
        mat.node_tree.links.new(geo.outputs["Position"], sep.inputs[0])
        ramp = nodes.new("ShaderNodeMapRange")
        ramp.inputs["From Min"].default_value = 0.1
        ramp.inputs["From Max"].default_value = 0.55
        ramp.inputs["To Min"].default_value = 0.55
        ramp.inputs["To Max"].default_value = 1.0
        mat.node_tree.links.new(sep.outputs["Z"], ramp.inputs["Value"])
        noise = nodes.new("ShaderNodeTexNoise")
        noise.inputs["Scale"].default_value = 18.0
        mixn = nodes.new("ShaderNodeMix")
        mixn.data_type = "FLOAT"
        mixn.inputs["Factor"].default_value = 0.35
        mat.node_tree.links.new(ramp.outputs["Result"], mixn.inputs["A"])
        mat.node_tree.links.new(noise.outputs["Fac"], mixn.inputs["B"])
        mul = nodes.new("ShaderNodeMix")
        mul.data_type = "RGBA"
        mul.blend_type = "MULTIPLY"
        mul.inputs["Factor"].default_value = 1.0
        mat.node_tree.links.new(rgb.outputs[0], mul.inputs["A"])
        gray = nodes.new("ShaderNodeCombineColor")
        mat.node_tree.links.new(mixn.outputs["Result"], gray.inputs["Red"])
        mat.node_tree.links.new(mixn.outputs["Result"], gray.inputs["Green"])
        mat.node_tree.links.new(mixn.outputs["Result"], gray.inputs["Blue"])
        mat.node_tree.links.new(gray.outputs[0], mul.inputs["B"])
        mat.node_tree.links.new(mul.outputs["Result"], bsdf.inputs["Base Color"])

    def bake_to(name, bake_type, colorspace, **kw):
        img = bpy.data.images.new(name, size, size, alpha=False)
        img.colorspace_settings.name = colorspace
        for mat in car.data.materials:
            nodes = mat.node_tree.nodes
            tex = nodes.new("ShaderNodeTexImage")
            tex.image = img
            tex.name = "bake_target"
            nodes.active = tex
        bpy.ops.object.bake(type=bake_type, **kw)
        for mat in car.data.materials:
            n = mat.node_tree.nodes.get("bake_target")
            if n:
                mat.node_tree.nodes.remove(n)
        return img

    # Albedo med AO: diffus farge uten lys, ganget med AO-bake.
    albedo = bake_to("albedo", "DIFFUSE", "sRGB", pass_filter={"COLOR"}, margin=4)
    ao = bake_to("ao", "AO", "Non-Color", margin=4)
    rough = bake_to("rough", "ROUGHNESS", "Non-Color", margin=4)
    # Slå sammen: albedo × AO^0.8 i RGB, ruhet i G og metall i B i et eget bilde.
    import numpy as np
    a = np.array(albedo.pixels[:]).reshape(size, size, 4)
    o = np.array(ao.pixels[:]).reshape(size, size, 4)
    r = np.array(rough.pixels[:]).reshape(size, size, 4)
    a[:, :, :3] *= np.power(np.clip(o[:, :, :1], 0.0, 1.0), 0.8)
    albedo.pixels = a.ravel().tolist()
    # Metall per materiale: bak «emission» med metallfaktoren som farge.
    for mat in car.data.materials:
        bsdf = mat.node_tree.nodes["Principled BSDF"]
        m = bsdf.inputs["Metallic"].default_value
        bsdf.inputs["Emission Color"].default_value = (m, m, m, 1.0)
        bsdf.inputs["Emission Strength"].default_value = 1.0
    metal = bake_to("metal", "EMIT", "Non-Color", margin=4)
    mt = np.array(metal.pixels[:]).reshape(size, size, 4)
    mr = np.zeros_like(r)
    mr[:, :, 1] = r[:, :, 0]
    mr[:, :, 2] = mt[:, :, 0]
    mr[:, :, 3] = 1.0
    mr_img = bpy.data.images.new("mr", size, size, alpha=False)
    mr_img.colorspace_settings.name = "Non-Color"
    mr_img.pixels = mr.ravel().tolist()

    # Materialene byttes til ett som bruker teksturene; lakken beholder
    # navnet «paint» så flagget settes, resten samles i «body».
    tex_mat = {}
    def key_of(name):
        for k in ("paint", "glass", "chrome"):
            if name.startswith(k):
                return k
        return "body"
    for mat in car.data.materials:
        key = key_of(mat.name)
        if key not in tex_mat:
            nm = bpy.data.materials.new(key)
            nm.use_nodes = True
            nodes = nm.node_tree.nodes
            bsdf = nodes["Principled BSDF"]
            bsdf.inputs["Base Color"].default_value = (1, 1, 1, 1)
            bsdf.inputs["Emission Strength"].default_value = 0.0
            if key == "glass":
                bsdf.inputs["Roughness"].default_value = 0.05
                bsdf.inputs["Base Color"].default_value = (0.04, 0.05, 0.07, 1)
            else:
                # Ruhetsfaktoren 1: motoren ganger teksturens ruhet med den.
                bsdf.inputs["Roughness"].default_value = 1.0
                ta = nodes.new("ShaderNodeTexImage")
                ta.image = albedo
                nm.node_tree.links.new(ta.outputs[0], bsdf.inputs["Base Color"])
                tm = nodes.new("ShaderNodeTexImage")
                tm.image = mr_img
                sep = nodes.new("ShaderNodeSeparateColor")
                nm.node_tree.links.new(tm.outputs[0], sep.inputs[0])
                nm.node_tree.links.new(sep.outputs["Green"], bsdf.inputs["Roughness"])
                nm.node_tree.links.new(sep.outputs["Blue"], bsdf.inputs["Metallic"])
                bsdf.inputs["Metallic"].default_value = 1.0 if key in ("body", "chrome") else 0.0
            tex_mat[key] = nm
    for slot in car.material_slots:
        slot.material = tex_mat[key_of(slot.material.name)]
    albedo.filepath_raw = f"/tmp/{kind}_albedo.png"
    albedo.file_format = "PNG"
    albedo.save()
    mr_img.filepath_raw = f"/tmp/{kind}_mr.png"
    mr_img.file_format = "PNG"
    mr_img.save()
    path = f"{OUT}/{kind}_tex.glb"
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=True, export_image_format="JPEG", export_jpeg_quality=85, export_animations=False, export_skins=False, export_cameras=False, export_lights=False)
    print(f"{path}: {os.path.getsize(path) // 1024} kB")


if __name__ == "__main__":
    args = [a for a in sys.argv[1:]]
    kinds = [a for a in args if a in ("estate", "hatch", "van")] or ["estate", "hatch", "van"]
    size = next((int(a) for a in args if a.isdigit()), 1024)
    for k in kinds:
        bake(k, size)
