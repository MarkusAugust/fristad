"""Dyrene rundt Fristad, lofta av tverrsnitt etter proporsjonene til ekte
dyr og glattet med subdivision, med bakte teksturer (albedo × AO,
normalkart med hårstrå eller ullkrøller, ruhet).

Kua er ei Norsk rødt fe (mankehøyde 1,35 m, kropp 1,7 m, hode 0,55 m):
rødbrun pels med hvite tegninger på buk, bein og bles. Delene er kroppen
(med hals, hode, ører, horn, jur og hale) og to beinmodeller (fram og
bak). Sauen er en hvit norsk sau med tjukk fell, smalt hode og ett bein
for alle fire. Motoren poserer beina i hofte- og skulderleddene. Kjør:

    python3 tools/models/animals.py [cow_body|cow_front|cow_hind|sheep_body|sheep_leg ...] [størrelse]

Kroppene har origo i bakken under midten, fronten mot +X (Blender: Y
venstre, Z opp). Beina har origo i leddet og henger nedover; festene på
kroppen er LEGS/SHEEP_LEGS i meter, speilet i herd.rs."""

import math
import os
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import bpy  # noqa: E402
import bmesh  # noqa: E402
from common import apply_modifiers, clear, ellipsoid, material, new_object, smooth, tube  # noqa: E402
from mathutils import Vector  # noqa: E402

OUT = "assets/models/animals"
# Skulder- og hofteleddene (x, y, z): fram høyre, fram venstre, bak høyre, bak venstre.
LEGS = [(0.6, 0.24, 1.0), (0.6, -0.24, 1.0), (-0.62, 0.26, 1.02), (-0.62, -0.26, 1.02)]
SHEEP_LEGS = [(0.3, 0.13, 0.46), (0.3, -0.13, 0.46), (-0.32, 0.13, 0.46), (-0.32, -0.13, 0.46)]
RING = 20


def ring(c, a, up, down, axis="x", e_top=1.0, e_bot=1.0, roll=0.0):
    """En ring med RING punkter rundt sentrum c: halvbredde a (på tvers),
    up/down (opp og ned fra sentrum). Eksponentene under 1 gjør toppen
    eller bunnen flatere (som ryggen og hoftene). axis er lengdeaksen:
    'x' for kroppen (ringen i yz-planet), 'z' for beina (ringen i xy)."""
    pts = []
    for i in range(RING):
        t = 2 * math.pi * i / RING - math.pi / 2
        ct, st = math.cos(t), math.sin(t)
        e = e_top if st >= 0 else e_bot
        r = abs(ct) ** e * (1 if ct >= 0 else -1)
        s = abs(st) ** e * (1 if st >= 0 else -1)
        h = up if st >= 0 else down
        if axis == "x":
            pts.append(Vector((c[0], c[1] + a * r, c[2] + h * s)))
        else:
            # Beina: ringen i xy-planet, «opp» er fram (+x).
            pts.append(Vector((c[0] + h * s, c[1] + a * r, c[2])))
    return pts


def loft(name, rings, mat, cap_start=True, cap_end=True, creases=()):
    bm = bmesh.new()
    vr = [[bm.verts.new(p) for p in r] for r in rings]
    crease = bm.edges.layers.float.get("crease_edge") or bm.edges.layers.float.new("crease_edge")
    for i in range(len(vr) - 1):
        a, b = vr[i], vr[i + 1]
        for k in range(RING):
            k2 = (k + 1) % RING
            bm.faces.new((a[k], a[k2], b[k2], b[k]))
    if cap_start:
        bm.faces.new(list(reversed(vr[0])))
    if cap_end:
        bm.faces.new(vr[-1])
    for i, c in creases:
        r = vr[i]
        for k in range(RING):
            e = bm.edges.get((r[k], r[(k + 1) % RING]))
            if e:
                e[crease] = c
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    ob = new_object(name, bm, [mat])
    return ob


def subdivide(ob, levels=2):
    mod = ob.modifiers.new("subd", "SUBSURF")
    mod.levels = levels
    mod.render_levels = levels
    mod.use_creases = True
    apply_modifiers(ob)
    smooth(ob, 80)


# ------------------------------------------------------------------ materialer


def hide_material(part):
    """Pelsen: rødbrun NRF med hvite tegninger. Hvitt på buken (lavt), på
    beina nedenfor kneet, i blesen og i flekker fra en grov støy. Alt i
    objektkoordinater, så det er sømløst over UV-ene. Hårstrå som fin,
    strukket støy i bump."""
    mat = bpy.data.materials.new(f"hide_{part}")
    mat.use_nodes = True
    nt = mat.node_tree
    nodes, links = nt.nodes, nt.links
    bsdf = nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = 0.85
    geo = nodes.new("ShaderNodeNewGeometry")
    sep = nodes.new("ShaderNodeSeparateXYZ")
    links.new(geo.outputs["Position"], sep.inputs[0])
    coord = nodes.new("ShaderNodeTexCoord")

    def noise(scale, detail=3.0, rough=0.5, distortion=0.0, vector=None):
        n = nodes.new("ShaderNodeTexNoise")
        n.inputs["Scale"].default_value = scale
        n.inputs["Detail"].default_value = detail
        n.inputs["Roughness"].default_value = rough
        n.inputs["Distortion"].default_value = distortion
        links.new(vector or coord.outputs["Object"], n.inputs["Vector"])
        return n

    def math_node(op, a, b=None, clamp=False):
        m = nodes.new("ShaderNodeMath")
        m.operation = op
        m.use_clamp = clamp
        if hasattr(a, "is_linked"):
            links.new(a, m.inputs[0])
        else:
            m.inputs[0].default_value = a
        if b is not None:
            if hasattr(b, "is_linked"):
                links.new(b, m.inputs[1])
            else:
                m.inputs[1].default_value = b
        return m.outputs[0]

    def map_range(v, a, b, c, d):
        m = nodes.new("ShaderNodeMapRange")
        m.interpolation_type = "SMOOTHSTEP"
        links.new(v, m.inputs["Value"])
        m.inputs["From Min"].default_value = a
        m.inputs["From Max"].default_value = b
        m.inputs["To Min"].default_value = c
        m.inputs["To Max"].default_value = d
        return m.outputs["Result"]

    # Hvite tegninger: buk og bryst lavt nede, med en bølgete kant fra støy.
    edge = noise(2.5, 2.0, 0.6)
    wobble = math_node("MULTIPLY", math_node("SUBTRACT", edge.outputs["Fac"], 0.5), 0.3)
    if part == "body":
        belly_z = math_node("ADD", sep.outputs["Z"], wobble)
        belly = map_range(belly_z, 0.78, 0.68, 0.0, 1.0)
        # Blesen: et hvitt felt midt i panna og over mula.
        blaze_w = math_node("SUBTRACT", 0.07, math_node("ABSOLUTE", sep.outputs["Y"]))
        blaze_x = map_range(sep.outputs["X"], 1.3, 1.5, 0.0, 1.0)
        blaze = math_node("MULTIPLY", map_range(math_node("ADD", blaze_w, wobble), 0.0, 0.03, 0.0, 1.0), blaze_x)
        # Store flekker på sidene fra en grov, forvrengt støy.
        patch_n = noise(0.9, 2.0, 0.55, 1.2)
        patches = map_range(patch_n.outputs["Fac"], 0.6, 0.64, 0.0, 1.0)
        white = math_node("MAXIMUM", math_node("MAXIMUM", belly, blaze), patches, clamp=True)
    else:
        # Beina: hvite nedenfor kneet/hasen (z under −0.45 fra leddet).
        leg_z = math_node("ADD", sep.outputs["Z"], wobble)
        white = map_range(leg_z, -0.4, -0.5, 0.0, 1.0)
    # Pelsfargene med litt variasjon i hårene.
    red = nodes.new("ShaderNodeRGB")
    red.outputs[0].default_value = (0.34, 0.14, 0.07, 1.0)
    cream = nodes.new("ShaderNodeRGB")
    cream.outputs[0].default_value = (0.86, 0.82, 0.74, 1.0)
    mix = nodes.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    links.new(white, mix.inputs["Factor"])
    links.new(red.outputs[0], mix.inputs["A"])
    links.new(cream.outputs[0], mix.inputs["B"])
    # Mørkere rundt øyne/hode og lysere rygg: en svak gradient i z.
    mapping = nodes.new("ShaderNodeMapping")
    mapping.inputs["Scale"].default_value = (12.0, 90.0, 90.0)
    links.new(coord.outputs["Object"], mapping.inputs["Vector"])
    hair = noise(1.0, 4.0, 0.7, 0.0, mapping.outputs["Vector"])
    hair_fac = map_range(hair.outputs["Fac"], 0.3, 0.7, 0.82, 1.08)
    shade = nodes.new("ShaderNodeMix")
    shade.data_type = "RGBA"
    shade.blend_type = "MULTIPLY"
    shade.inputs["Factor"].default_value = 1.0
    links.new(mix.outputs["Result"], shade.inputs["A"])
    gray = nodes.new("ShaderNodeCombineColor")
    for ch in ("Red", "Green", "Blue"):
        links.new(hair_fac, gray.inputs[ch])
    links.new(gray.outputs[0], shade.inputs["B"])
    out_color = shade.outputs["Result"]
    if part == "body":
        # Mula: grå-rosa hud foran x = 1.76, med en myk kant.
        muzzle = map_range(sep.outputs["X"], 1.72, 1.78, 0.0, 1.0)
        pink = nodes.new("ShaderNodeRGB")
        pink.outputs[0].default_value = (0.38, 0.3, 0.3, 1.0)
        mm = nodes.new("ShaderNodeMix")
        mm.data_type = "RGBA"
        links.new(muzzle, mm.inputs["Factor"])
        links.new(out_color, mm.inputs["A"])
        links.new(pink.outputs[0], mm.inputs["B"])
        out_color = mm.outputs["Result"]
    links.new(out_color, bsdf.inputs["Base Color"])
    # Hårene i normalkartet: fin støy strukket langs kroppen.
    bump = nodes.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 0.35
    bump.inputs["Distance"].default_value = 0.004
    links.new(hair.outputs["Fac"], bump.inputs["Height"])
    links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])
    return mat


def wool_material(part):
    """Ulla: kremhvit fell med krøller fra en voronoi-støy i bump, gråere
    og skitnere nederst. Hodet og beina (part «skin») har kort, grålig
    hår uten krøller."""
    mat = bpy.data.materials.new(f"wool_{part}")
    mat.use_nodes = True
    nt = mat.node_tree
    nodes, links = nt.nodes, nt.links
    bsdf = nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = 0.95
    coord = nodes.new("ShaderNodeTexCoord")
    geo = nodes.new("ShaderNodeNewGeometry")
    sep = nodes.new("ShaderNodeSeparateXYZ")
    links.new(geo.outputs["Position"], sep.inputs[0])
    curls = nodes.new("ShaderNodeTexVoronoi")
    curls.inputs["Scale"].default_value = 22.0
    curls.inputs["Randomness"].default_value = 1.0
    links.new(coord.outputs["Object"], curls.inputs["Vector"])
    fine = nodes.new("ShaderNodeTexNoise")
    fine.inputs["Scale"].default_value = 90.0
    fine.inputs["Detail"].default_value = 3.0
    links.new(coord.outputs["Object"], fine.inputs["Vector"])
    height = nodes.new("ShaderNodeMix")
    height.data_type = "FLOAT"
    height.inputs["Factor"].default_value = 0.15
    links.new(curls.outputs["Distance"], height.inputs["A"])
    links.new(fine.outputs["Fac"], height.inputs["B"])
    bump = nodes.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = 1.0 if part == "wool" else 0.25
    bump.inputs["Distance"].default_value = 0.025 if part == "wool" else 0.003
    links.new(height.outputs["Result"], bump.inputs["Height"])
    links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])
    # Fargen: krem, mørkere i krøllenes dalsøkk, og gråbrun skitt nederst.
    cream = nodes.new("ShaderNodeRGB")
    cream.outputs[0].default_value = (0.9, 0.87, 0.78, 1.0) if part == "wool" else (0.62, 0.57, 0.5, 1.0)
    dirt = nodes.new("ShaderNodeRGB")
    dirt.outputs[0].default_value = (0.4, 0.35, 0.28, 1.0) if part == "wool" else (0.3, 0.26, 0.22, 1.0)
    grad = nodes.new("ShaderNodeMapRange")
    grad.interpolation_type = "SMOOTHSTEP"
    links.new(sep.outputs["Z"], grad.inputs["Value"])
    grad.inputs["From Min"].default_value = (0.22, -0.5)[part != "wool"]
    grad.inputs["From Max"].default_value = (0.4, -0.2)[part != "wool"]
    grad.inputs["To Min"].default_value = 0.5
    grad.inputs["To Max"].default_value = 0.0
    mix = nodes.new("ShaderNodeMix")
    mix.data_type = "RGBA"
    links.new(grad.outputs["Result"], mix.inputs["Factor"])
    links.new(cream.outputs[0], mix.inputs["A"])
    links.new(dirt.outputs[0], mix.inputs["B"])
    shade = nodes.new("ShaderNodeMix")
    shade.data_type = "RGBA"
    shade.blend_type = "MULTIPLY"
    # Bare de dypeste dalsøkkene mellom krøllene mørkner.
    valley = nodes.new("ShaderNodeMapRange")
    valley.interpolation_type = "SMOOTHSTEP"
    links.new(curls.outputs["Distance"], valley.inputs["Value"])
    valley.inputs["From Min"].default_value = 0.3
    valley.inputs["From Max"].default_value = 0.9
    valley.inputs["To Min"].default_value = 0.0
    valley.inputs["To Max"].default_value = 0.6 if part == "wool" else 0.2
    links.new(valley.outputs["Result"], shade.inputs["Factor"])
    links.new(mix.outputs["Result"], shade.inputs["A"])
    shade.inputs["B"].default_value = (0.72, 0.7, 0.66, 1.0)
    links.new(shade.outputs["Result"], bsdf.inputs["Base Color"])
    return mat


def sheep_body(m):
    """Sauen: tjukk ullkropp (lofta, med bulker fra krøllene i teksturen),
    smal hals som stikker ut foran og litt ned, smalt hode med romersk
    nese, ører til siden og kort hale."""
    S = [
        (-0.58, 0.5, 0.12, 0.14, 0.12, 0.9, 1.0),
        (-0.5, 0.5, 0.24, 0.24, 0.2, 0.8, 1.0),
        (-0.35, 0.5, 0.3, 0.29, 0.25, 0.75, 1.0),
        (-0.1, 0.5, 0.31, 0.29, 0.26, 0.8, 1.0),
        (0.15, 0.51, 0.3, 0.28, 0.26, 0.8, 1.0),
        (0.35, 0.53, 0.27, 0.26, 0.23, 0.85, 1.0),
        (0.5, 0.56, 0.22, 0.22, 0.2, 0.9, 1.0),
        (0.62, 0.6, 0.15, 0.15, 0.14, 1.0, 1.0),  # halsen i ull
        (0.68, 0.63, 0.09, 0.09, 0.09, 1.0, 1.0),
    ]
    rings = [ring((x, 0.0, zc), a, up, down, "x", et, eb) for x, zc, a, up, down, et, eb in S]
    ob = loft("fleece", rings, m["wool"], creases=[(0, 0.5)])
    subdivide(ob, 2)
    # Halsen og hodet i kort hår, lofta videre fram og opp.
    H = [
        (0.64, 0.64, 0.08, 0.085, 0.095, 1.0, 1.0),
        (0.74, 0.69, 0.08, 0.08, 0.09, 1.0, 1.0),
        (0.8, 0.72, 0.082, 0.072, 0.085, 0.7, 1.0),  # bakhodet
        (0.87, 0.71, 0.072, 0.065, 0.08, 0.7, 1.0),  # over øynene
        (0.94, 0.68, 0.055, 0.05, 0.065, 0.8, 1.0),  # neseryggen
        (0.99, 0.655, 0.04, 0.035, 0.045, 1.0, 1.0),  # mula
        (1.01, 0.65, 0.025, 0.02, 0.025, 1.0, 1.0),
    ]
    rings = [ring((x, 0.0, zc), a, up, down, "x", et, eb) for x, zc, a, up, down, et, eb in H]
    ob = loft("head", rings, m["skin"], creases=[(len(H) - 1, 0.5)])
    subdivide(ob, 2)
    for sy in (1, -1):
        e = ellipsoid(f"ear{sy}", (0.8, sy * 0.14, 0.73), (0.05, 0.16, 0.035), m["skin"], 8, 5)
        e.rotation_euler = (math.radians(-sy * 15), 0, math.radians(-sy * 10))
        ellipsoid(f"eye{sy}", (0.88, sy * 0.068, 0.715), (0.035, 0.03, 0.03), m["eye"], 8, 5)
    ellipsoid("nose", (1.01, 0.0, 0.645), (0.03, 0.035, 0.03), m["eye"], 8, 5)
    # Halen: kort, hengende ullbit.
    tail = [((-0.6, 0.0, 0.5), 0.045), ((-0.64, 0.0, 0.42), 0.04), ((-0.66, 0.0, 0.33), 0.025)]
    ob = loft("tail", [ring(c, r, r, r, "z") for c, r in tail], m["wool"])
    subdivide(ob, 1)


def sheep_leg(m):
    """Sauebeinet: tynt, med ull øverst og kort hår nedenfor kneet."""
    S = [
        (0.08, 0.0, 0.065, 0.08, 0.07),
        (-0.1, 0.0, 0.05, 0.06, 0.055),
        (-0.22, 0.0, 0.03, 0.034, 0.03),  # kneet
        (-0.35, 0.0, 0.024, 0.026, 0.024),
        (-0.41, 0.0, 0.027, 0.03, 0.026),  # koden
        (-0.44, 0.005, 0.024, 0.027, 0.022),
    ]
    rings = [ring((x, 0.0, z), a, f, b, "z") for z, x, a, f, b in S]
    ob = loft("leg", rings, m["skin"])
    subdivide(ob, 2)
    for sy in (1, -1):
        T = [(-0.435, 0.0, 0.013, 0.028, 0.022), (-0.455, 0.004, 0.014, 0.03, 0.023), (-0.47, 0.006, 0.014, 0.03, 0.022)]
        rings = [ring((xx, sy * 0.014, z), a, f, b, "z", 0.6, 0.6) for z, xx, a, f, b in T]
        ob = loft(f"toe{sy}", rings, m["hoof"], creases=[(2, 1.0)])
        subdivide(ob, 1)


# ------------------------------------------------------------------- kroppen


def body(m):
    """Kroppen fra halerota til mula: bred og dyp tønne, flat rygg med
    hoftehjørner, markert manke, hals som smalner, bred flat panne og
    kraftig mule. Verdiene er målt etter proporsjonene til ei NRF-ku."""
    S = [
        # (x, zc, halvbredde, opp, ned, e_top, e_bot)
        (-1.0, 1.08, 0.17, 0.2, 0.17, 0.9, 1.0),  # halerota / setebein
        (-0.85, 1.02, 0.3, 0.32, 0.3, 0.75, 1.0),
        (-0.62, 0.99, 0.37, 0.36, 0.37, 0.65, 1.0),  # hoftehjørnene
        (-0.35, 0.98, 0.39, 0.35, 0.4, 0.8, 1.0),  # vomma, bredest
        (-0.05, 0.99, 0.38, 0.35, 0.41, 0.85, 1.0),
        (0.25, 1.01, 0.35, 0.35, 0.39, 0.85, 1.0),
        (0.5, 1.04, 0.31, 0.35, 0.36, 0.8, 1.0),  # manken
        (0.72, 1.09, 0.26, 0.29, 0.36, 0.9, 1.1),  # bogen og brystet
        (0.9, 1.14, 0.2, 0.21, 0.3, 1.0, 1.2),  # halsrota med doglapp
        (1.08, 1.19, 0.16, 0.17, 0.23, 1.0, 1.2),
        (1.24, 1.22, 0.14, 0.14, 0.17, 1.0, 1.0),  # nakken
        (1.34, 1.22, 0.145, 0.13, 0.15, 0.6, 1.0),  # nakkekammen / panna
        (1.48, 1.17, 0.13, 0.12, 0.16, 0.6, 1.0),  # panna over øynene
        (1.62, 1.1, 0.105, 0.1, 0.14, 0.7, 1.0),  # neseryggen
        (1.74, 1.04, 0.095, 0.09, 0.12, 0.8, 1.0),
        (1.84, 1.0, 0.09, 0.08, 0.1, 0.9, 1.0),  # mula
        (1.9, 0.99, 0.06, 0.05, 0.06, 1.0, 1.0),
    ]
    rings = [ring((x, 0.0, zc), a, up, down, "x", et, eb) for x, zc, a, up, down, et, eb in S]
    ob = loft("body", rings, m["hide"], creases=[(0, 0.4), (len(S) - 1, 0.5)])
    subdivide(ob, 2)
    # Hoftehjørnene og setebeina som knokler under huden.
    # Ørene: brede, flate blad som står ut til siden og litt ned.
    for sy in (1, -1):
        e = ellipsoid(f"ear{sy}", (1.3, sy * 0.3, 1.2), (0.1, 0.32, 0.06), m["hide"], 10, 6)
        e.rotation_euler = (math.radians(-sy * 20), math.radians(10), math.radians(-sy * 15))
    # Hornene: korte, bøyd opp og fram.
    for sy in (1, -1):
        horn(m, sy)
    # Øynene på sidene av hodet, med øyelokk.
    for sy in (1, -1):
        ellipsoid(f"eye{sy}", (1.52, sy * 0.125, 1.17), (0.06, 0.05, 0.05), m["eye"], 10, 6)
        ellipsoid(f"brow{sy}", (1.5, sy * 0.13, 1.2), (0.09, 0.05, 0.03), m["hide"], 8, 5)
    # Neseborene.
    for sy in (1, -1):
        n = ellipsoid(f"nostril{sy}", (1.9, sy * 0.045, 1.0), (0.03, 0.035, 0.045), m["eye"], 8, 5)
        n.rotation_euler = (0, math.radians(30), 0)
    # Juret mellom bakbeina med fire spener.
    ellipsoid("udder", (-0.45, 0.0, 0.62), (0.46, 0.4, 0.34), m["udder"], 12, 8)
    for dx, dy in ((0.1, 0.1), (0.1, -0.1), (-0.1, 0.1), (-0.1, -0.1)):
        tube(f"teat{dx}{dy}", (-0.45 + dx, dy, 0.5), (-0.45 + dx, dy, 0.4), 0.014, m["udder"], 6)
    # Halen fra setebeina ned til hasene, med dusk.
    tail = [((-1.0, 0.0, 1.25), 0.035), ((-1.07, 0.01, 1.1), 0.03), ((-1.1, 0.02, 0.9), 0.025), ((-1.09, 0.03, 0.7), 0.02), ((-1.07, 0.03, 0.56), 0.018)]
    rings = [ring(c, r, r, r, "z") for c, r in tail]
    ob = loft("tail", rings, m["hide"])
    subdivide(ob, 1)
    tuft = [((-1.07, 0.03, 0.58), 0.02), ((-1.06, 0.03, 0.5), 0.05), ((-1.05, 0.03, 0.4), 0.045), ((-1.04, 0.03, 0.32), 0.015)]
    ob = loft("tuft", [ring(c, r, r, r, "z") for c, r in tuft], m["tuft"])
    subdivide(ob, 1)


def horn(m, sy):
    """Hornet: ett glatt, bøyd horn fra tinningen, ut til siden, opp og
    litt fram, som smalner mot spissen."""
    path = [((1.28, sy * 0.1, 1.3), 0.035), ((1.3, sy * 0.16, 1.35), 0.03), ((1.32, sy * 0.2, 1.4), 0.024), ((1.35, sy * 0.215, 1.45), 0.016), ((1.38, sy * 0.215, 1.48), 0.008), ((1.4, sy * 0.21, 1.495), 0.003)]
    rings = []
    for i, (c, r) in enumerate(path):
        rings.append(ring(c, r, r, r, "z"))
    ob = loft(f"horn{sy}", rings, m["horn"])
    subdivide(ob, 1)


def front_leg(m):
    """Frambeinet fra bogleddet: muskuløs overarm, kne, tynn pipe,
    kode og klauv. Lofta langs z nedover."""
    S = [
        # (z, x-forskyvning, halvbredde (y), fram, bak)
        (0.12, 0.0, 0.13, 0.17, 0.15),
        (-0.1, 0.0, 0.1, 0.13, 0.12),
        (-0.3, 0.01, 0.075, 0.09, 0.08),
        (-0.42, 0.01, 0.068, 0.078, 0.07),  # kneet
        (-0.5, 0.0, 0.055, 0.06, 0.055),
        (-0.75, 0.0, 0.048, 0.052, 0.048),
        (-0.86, 0.0, 0.058, 0.062, 0.058),  # koden
        (-0.93, 0.01, 0.055, 0.06, 0.05),
    ]
    rings = [ring((x, 0.0, z), a, f, b, "z") for z, x, a, f, b in S]
    ob = loft("front_leg", rings, m["hide"])
    subdivide(ob, 2)
    hoof(m, 0.015, -0.93)


def hind_leg(m):
    """Bakbeinet fra hofteleddet: bredt lår framover til kneet, legg
    bakover til hasen, og pipe rett ned."""
    S = [
        (0.12, 0.0, 0.14, 0.2, 0.17),
        (-0.15, 0.06, 0.11, 0.16, 0.13),
        (-0.32, 0.1, 0.085, 0.11, 0.09),  # kneet (stifle)
        (-0.45, 0.02, 0.065, 0.08, 0.07),
        (-0.58, -0.08, 0.06, 0.07, 0.07),  # hasen
        (-0.75, -0.09, 0.048, 0.052, 0.05),
        (-0.86, -0.09, 0.058, 0.062, 0.058),  # koden
        (-0.93, -0.08, 0.055, 0.06, 0.05),
    ]
    rings = [ring((x, 0.0, z), a, f, b, "z") for z, x, a, f, b in S]
    ob = loft("hind_leg", rings, m["hide"])
    subdivide(ob, 2)
    hoof(m, -0.07, -0.93)


def hoof(m, x, top):
    """Klauven: to tær, avrundet foran, flat under."""
    for sy in (1, -1):
        S = [
            (top + 0.01, x, 0.028, 0.06, 0.05),
            (top - 0.03, x + 0.005, 0.03, 0.068, 0.052),
            (top - 0.065, x + 0.01, 0.03, 0.07, 0.05),
        ]
        rings = [ring((xx, sy * 0.031, z), a, f, b, "z", 0.6, 0.6) for z, xx, a, f, b in S]
        ob = loft(f"toe{sy}", rings, m["hoof"], creases=[(2, 1.0)])
        subdivide(ob, 1)


def mats():
    return {
        "horn": material("horn", (0.62, 0.55, 0.42), 0.45),
        "eye": material("eye", (0.03, 0.025, 0.02), 0.15),
        "hoof": material("hoof", (0.14, 0.11, 0.09), 0.5),
        "udder": material("udder", (0.72, 0.5, 0.46), 0.7),
        "tuft": material("tuft", (0.2, 0.1, 0.06), 0.9),
    }


# ----------------------------------------------------------------------- bake


def bake_part(name, size):
    """Slå sammen delene, UV-pakk, bak albedo × AO, normalkart og ruhet,
    og bytt til ett teksturert materiale («hide», så pelsen får pelsflagget
    ingen; huden er bare diffus)."""
    meshes = [o for o in bpy.data.objects if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    ob = bpy.context.active_object
    ob.name = name
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=1.1, island_margin=0.004)
    bpy.ops.object.mode_set(mode="OBJECT")
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = 16
    scene.render.bake.margin = 6

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

    import numpy as np
    albedo = bake_to("albedo", "DIFFUSE", "sRGB", pass_filter={"COLOR"})
    # AO bakes langs materialets normal, så bumpen (hår og krøller) må være
    # avslått imens, ellers treffer strålene flaten selv og alt blir mørkt.
    bumps = [n for mat in ob.data.materials for n in mat.node_tree.nodes if n.bl_idname == "ShaderNodeBump"]
    for n in bumps:
        n.mute = True
    ao = bake_to("ao", "AO", "Non-Color")
    for n in bumps:
        n.mute = False
    normal = bake_to("normal", "NORMAL", "Non-Color", normal_space="TANGENT")
    rough = bake_to("rough", "ROUGHNESS", "Non-Color")
    a = np.array(albedo.pixels[:]).reshape(size, size, 4)
    o = np.array(ao.pixels[:]).reshape(size, size, 4)
    a[:, :, :3] *= np.power(np.clip(o[:, :, :1], 0.0, 1.0), 0.7)
    albedo.pixels = a.ravel().tolist()
    r = np.array(rough.pixels[:]).reshape(size, size, 4)
    mr = np.zeros_like(r)
    mr[:, :, 1] = r[:, :, 0]
    mr[:, :, 3] = 1.0
    mr_img = bpy.data.images.new("mr", size, size, alpha=False)
    mr_img.colorspace_settings.name = "Non-Color"
    mr_img.pixels = mr.ravel().tolist()
    tm = bpy.data.materials.new("hide_tex")
    tm.use_nodes = True
    nt = tm.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = 1.0
    ta = nt.nodes.new("ShaderNodeTexImage")
    ta.image = albedo
    nt.links.new(ta.outputs[0], bsdf.inputs["Base Color"])
    tn = nt.nodes.new("ShaderNodeTexImage")
    tn.image = normal
    nm = nt.nodes.new("ShaderNodeNormalMap")
    nt.links.new(tn.outputs[0], nm.inputs["Color"])
    nt.links.new(nm.outputs["Normal"], bsdf.inputs["Normal"])
    tr = nt.nodes.new("ShaderNodeTexImage")
    tr.image = mr_img
    sep = nt.nodes.new("ShaderNodeSeparateColor")
    nt.links.new(tr.outputs[0], sep.inputs[0])
    nt.links.new(sep.outputs["Green"], bsdf.inputs["Roughness"])
    for slot in ob.material_slots:
        slot.material = tm
    for img in (albedo, normal, mr_img, ao):
        img.filepath_raw = f"/tmp/cow_{name}_{img.name}.png"
        img.file_format = "PNG"
        img.save()
    return ob


def export(path):
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", export_apply=True, export_materials="EXPORT", export_yup=True, export_normals=True, export_texcoords=True, export_image_format="JPEG", export_jpeg_quality=82, export_animations=False, export_skins=False, export_cameras=False, export_lights=False)
    tris = sum(len(p.vertices) - 2 for ob in bpy.data.objects if ob.type == "MESH" for p in ob.data.polygons)
    print(f"{path}: {tris} trekanter, {os.path.getsize(path) // 1024} kB")


PARTS = {
    "cow_body": ("cow_body", body, 1024),
    "cow_front": ("cow_leg_front", front_leg, 512),
    "cow_hind": ("cow_leg_hind", hind_leg, 512),
    "sheep_body": ("sheep_body", sheep_body, 1024),
    "sheep_leg": ("sheep_leg", sheep_leg, 256),
}

if __name__ == "__main__":
    args = sys.argv[1:]
    parts = [a for a in args if a in PARTS] or list(PARTS)
    size = next((int(a) for a in args if a.isdigit()), None)
    os.makedirs(OUT, exist_ok=True)
    for p in parts:
        file, build, default_size = PARTS[p]
        clear()
        m = mats()
        m["hide"] = hide_material("body" if p == "cow_body" else "leg")
        m["wool"] = wool_material("wool")
        m["skin"] = wool_material("skin")
        build(m)
        bake_part(file, size or default_size)
        export(f"{OUT}/{file}.glb")
