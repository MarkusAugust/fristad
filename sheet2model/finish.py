"""Steg 7: arkene (forbildet ved siden av modellen, og mellomvinklene), turnaround og eksport til glb.

Renderen er en enkel punktsplatting av tekslene i numpy (ortografisk, flatt lys): den er til kontroll, ikke
til presentasjon, og tar et sekund per bilde. glb-eksporten går gjennom bpy (sekunder), og målet (`targets/`)
får slanke fila etterpå.
"""
import os
import time

import numpy as np

from . import geom

GREY = 130


# ---------------------------------------------------------------- punktsplatting
class Cloud:
    """Tekslene som punktsky: posisjon (verdenspiksler), farge (sRGB 0–1) og normal."""

    def __init__(self, pos, col, nrm):
        self.pos, self.col, self.nrm = pos, col, nrm

    @classmethod
    def load(cls, job, stem="albedo", texels="texels.npz", flip=None):
        import cv2
        t = np.load(job.p(texels))
        cov = t["cov"]
        al = cv2.imread(job.p(f"{stem}.png"))[:, :, ::-1].astype(np.float32) / 255
        pos = t["pos"][cov]
        if flip is not None:
            pos = flip(pos)
        return cls(pos, al[cov], t["nrm"][cov])


def camera(yaw_deg, pitch_deg):
    """Retning mot kameraet fra yaw (0 forfra, 90 fra figurens venstre) og pitch (positiv ovenfra)."""
    yaw, pitch = np.radians(yaw_deg), np.radians(pitch_deg)
    cam = np.array([np.sin(yaw) * np.cos(pitch), np.sin(pitch), np.cos(yaw) * np.cos(pitch)])
    right = np.cross(np.array([0, 1.0, 0]), cam); right /= np.linalg.norm(right) + 1e-9
    up = np.cross(cam, right)
    return cam, right, up


def splat(cloud, cam, right, up, res, centre, extent, light=0.25):
    """Bilde (res × res, RGB uint8) med grå bakgrunn: nærmeste teksel per piksel, hull fylt fra naboene."""
    from scipy import ndimage as ndi
    P = cloud.pos - centre
    s = res / extent
    u = res / 2 + (P @ right) * s
    v = res / 2 - (P @ up) * s
    d = P @ cam
    ui = np.round(u).astype(int); vi = np.round(v).astype(int)
    ok = (ui >= 0) & (ui < res) & (vi >= 0) & (vi < res)
    ui, vi, d = ui[ok], vi[ok], d[ok]
    zb = np.full((res, res), -1e9, np.float32)
    np.maximum.at(zb, (vi, ui), d.astype(np.float32))
    front = d >= zb[vi, ui] - 1.5
    img = np.zeros((res, res, 3), np.float32)
    col = cloud.col[ok][front]
    n = cloud.nrm[ok][front]
    L = np.array([-0.3, 0.6, 0.74]); L /= np.linalg.norm(L)
    shade = (1 - light) + light * np.clip(n @ L, 0, 1)
    img[vi[front], ui[front]] = col * shade[:, None]
    hit = np.zeros((res, res), bool); hit[vi[front], ui[front]] = True
    inside = ndi.binary_closing(hit, iterations=3)
    _, (ri, ci) = ndi.distance_transform_edt(~hit, return_indices=True)
    img = np.where((inside & ~hit)[..., None], img[ri, ci], img)
    out = np.where(inside[..., None], img, GREY / 255)
    return (np.clip(out, 0, 1) * 255 + 0.5).astype(np.uint8)


def view_shot(job, cloud, k, fr, res):
    """Modellen sett som i ruta k (samme ramme), i rutas format."""
    view = job.views[k]
    o = fr[k]
    cam, right, up = geom.basis(view, o)
    # bildets midte i verden: punktet som avbildes på (TW/2, TH/2), på kameraaksen gjennom skyas midte
    a = (job.TW / 2 - o["c"]) / o["sy"]; b = -(job.TH / 2 - o["top"]) / o["sy"]
    centre = right * a + up * b
    ctr = cloud.pos.mean(0)
    centre = centre - cam * (centre @ cam) + cam * (ctr @ cam)
    extent = max(job.TW, job.TH) / o["sy"]
    r = int(round(res * max(job.TW, job.TH) / job.TH))
    img = splat(cloud, cam, right, up, r, centre, extent)
    w = int(round(r * job.TW / max(job.TW, job.TH))); h = int(round(r * job.TH / max(job.TW, job.TH)))
    return img[(r - h) // 2:(r - h) // 2 + h, (r - w) // 2:(r - w) // 2 + w]


def sheets(job, cloud, fr, res=512, tag="ark"):
    """`<tag>.png`: forbildet og modellen i samme ruter. `<tag>_mellom.png`: 45°/20° ovenfra, −40°/−10°,
    ovenfra og skrått bakfra. Ingen av forbildene viser disse vinklene, så de avslører om formen og projeksjonen
    virkelig stemmer."""
    import cv2
    from .sheet import load_rgb
    t0 = time.time()
    shown = [k for k, v in job.views.items() if not v.geo_only]
    tiles = [view_shot(job, cloud, k, fr, res) for k in shown]
    h, w = tiles[0].shape[:2]
    refs = [cv2.resize(load_rgb(job, k), (w, h), interpolation=cv2.INTER_AREA) for k in shown]
    if len(shown) == 4 and job.layout == "head2x2":
        grid = lambda ims: np.vstack([np.hstack(ims[:2]), np.hstack(ims[2:])])  # noqa: E731
        full = np.hstack([grid(refs), grid(tiles)])
    else:
        full = np.vstack([np.hstack(refs), np.hstack(tiles)])
    cv2.imwrite(job.shot(f"{tag}.png"), full[:, :, ::-1])
    ctr = (cloud.pos.min(0) + cloud.pos.max(0)) / 2
    extent = max(job.TW, job.TH) / (fr["front"] if "front" in fr else fr[shown[0]])["sy"]
    mids = []
    for yaw, pitch in ((45, 20), (-40, -10), (25, 60), (150, -5)):
        cam, right, up = camera(yaw, pitch)
        mids.append(splat(cloud, cam, right, up, res, ctr, extent))
    cv2.imwrite(job.shot(f"{tag}_mellom.png"), np.hstack(mids)[:, :, ::-1])
    job.time("render", time.time() - t0)
    return job.shot(f"{tag}.png"), job.shot(f"{tag}_mellom.png")


def turnaround(job, cloud, n=24, res=256):
    from PIL import Image
    ctr = (cloud.pos.min(0) + cloud.pos.max(0)) / 2
    extent = np.ptp(cloud.pos, axis=0).max() * 1.1
    frames = []
    for i in range(n):
        cam, right, up = camera(360 * i / n, 10)
        frames.append(Image.fromarray(splat(cloud, cam, right, up, res, ctr, extent)))
    out = job.shot(f"{job.name}_turnaround.gif")
    frames[0].save(out, save_all=True, append_images=frames[1:], duration=120, loop=0)
    return out


# ---------------------------------------------------------------- glb
def export_glb(job, path, parts, albedo="albedo.png", normal="normal.png", target=None):
    """glb med én tekstur (JPEG) og normalkart fra deler (navn, hjørner i Blender-meter, flater, uv).
    Går gjennom bpy; målet får slanke fila etterpå (`target.post_export`)."""
    import bpy
    from .mesh import bpy_mesh, bpy_reset, bpy_select
    t0 = time.time()
    bpy_reset()
    mat = material(job, "body", albedo, normal)
    obs = []
    for name, v, f, uv in parts:
        ob = bpy_mesh(name, v, f, uv)
        ob.data.materials.append(mat)
        bpy_select(ob)
        bpy.ops.object.shade_smooth()
        obs.append(ob)
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    bpy_select(*obs)
    bpy.ops.export_scene.gltf(filepath=path, export_format="GLB", use_selection=True, export_image_format="JPEG",
                              export_jpeg_quality=90, export_cameras=False, export_lights=False, export_yup=True)
    if target is not None:
        target.post_export(path)
    job.time("export", time.time() - t0)
    print("glb", path, sum(len(p[2]) for p in parts), "trekanter")
    return path


def material(job, name, albedo, normal, roughness=0.8):
    import bpy
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Specular IOR Level"].default_value = 0.15
    al = nt.nodes.new("ShaderNodeTexImage"); al.image = bpy.data.images.load(job.p(albedo))
    nt.links.new(al.outputs[0], bsdf.inputs["Base Color"])
    if normal and os.path.exists(job.p(normal)):
        nm = nt.nodes.new("ShaderNodeTexImage"); nm.image = bpy.data.images.load(job.p(normal))
        nm.image.colorspace_settings.name = "Non-Color"
        nmap = nt.nodes.new("ShaderNodeNormalMap"); nmap.inputs["Strength"].default_value = 0.8
        nt.links.new(nm.outputs[0], nmap.inputs["Color"]); nt.links.new(nmap.outputs[0], bsdf.inputs["Normal"])
    return mat


def finish(job, glb=None, gif=False, target=None):
    """Arkene og glb for én modell fra ett ark (hodet). Hele figurer eksporteres rigget av rig.py."""
    from .fuse import load_frame
    fr, _ = load_frame(job)
    lo = np.load(job.p("lo.npz"))
    if job.opt.get("render", "cycles") == "cycles":
        out = render_bpy(job, [(job.name, lo["v"], lo["f"], lo["uv"])], fr)
    else:
        out = sheets(job, Cloud.load(job), fr)
    if gif:
        turnaround(job, Cloud.load(job))
    v = geom.w2b(lo["v"], job.scale)
    glb = glb or job.shot(job.name + ".glb")
    export_glb(job, glb, [(job.name, v, lo["f"], lo["uv"])], target=target)
    return out


# ---------------------------------------------------------------- Cycles-render (til sammenligning med forbildet)
def _dir_b(d):
    """Retning i verden -> Blender (x, -z, y)."""
    return np.array([d[0], -d[2], d[1]], float)


def render_bpy(job, parts, fr, albedo="albedo.png", normal="normal.png", res=512, samples=24, tag="ark",
               flip=None):
    """Arkene rendret med Cycles (CPU, flatt lys: hvit himmel og et svakt nøkkellys), så teksturen og
    normalkartet vises slik de er, med samme skarphet som forbildet. `parts`: (navn, v i verdenspiksler, f, uv)."""
    import bpy
    import cv2
    from mathutils import Matrix, Vector
    from .mesh import bpy_mesh, bpy_reset, bpy_select
    from .sheet import load_rgb
    t0 = time.time()
    bpy_reset()
    mat = material(job, "body", albedo, normal)
    allv = []
    for name, v, f, uv in parts:
        vb = geom.w2b(v if flip is None else flip(v), job.scale)
        ob = bpy_mesh(name, vb, f, uv)
        ob.data.materials.append(mat)
        bpy_select(ob)
        bpy.ops.object.shade_smooth()
        allv.append(vb[np.unique(f)])
    allv = np.concatenate(allv)
    ctr = (allv.min(0) + allv.max(0)) / 2
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"; sc.cycles.device = "CPU"; sc.cycles.samples = samples
    sc.cycles.use_denoising = False
    sc.render.film_transparent = True
    sc.view_settings.view_transform = "Standard"
    w = bpy.data.worlds.new("w"); sc.world = w; w.use_nodes = True
    w.node_tree.nodes["Background"].inputs[0].default_value = (1, 1, 1, 1)
    w.node_tree.nodes["Background"].inputs[1].default_value = 0.85
    cam_d = bpy.data.cameras.new("cam"); cam_d.type = "ORTHO"
    cam = bpy.data.objects.new("cam", cam_d); sc.collection.objects.link(cam); sc.camera = cam
    key_d = bpy.data.lights.new("key", "AREA"); key_d.size = 0.6; key_d.energy = 1.5
    key = bpy.data.objects.new("key", key_d); sc.collection.objects.link(key)

    def shoot(cam_w, right_w, up_w, ortho, centre_b, path, w_px, h_px):
        d, r, u = _dir_b(cam_w), _dir_b(right_w), _dir_b(up_w)
        cam.location = Vector(centre_b) + Vector(d) * 1.5
        cam.matrix_world = Matrix(((r[0], u[0], d[0], cam.location.x), (r[1], u[1], d[1], cam.location.y),
                                   (r[2], u[2], d[2], cam.location.z), (0, 0, 0, 1)))
        cam_d.ortho_scale = ortho
        key.location = Vector(centre_b) + Vector(d) * 0.9 - Vector(r) * 0.5 + Vector((0, 0, 0.6))
        key.rotation_euler = (Vector(centre_b) - key.location).to_track_quat("-Z", "Y").to_euler()
        sc.render.resolution_x, sc.render.resolution_y = w_px, h_px
        sc.render.filepath = path
        bpy.ops.render.render(write_still=True)

    def flat(path):
        im = cv2.imread(path, cv2.IMREAD_UNCHANGED).astype(np.float32)
        a = im[:, :, 3:4] / 255
        return (im[:, :, :3] * a + GREY * (1 - a)).astype(np.uint8)
    shown = [k for k, v in job.views.items() if not v.geo_only]
    h_px = res; w_px = int(round(res * job.TW / job.TH / 2)) * 2
    tiles = []
    for k in shown:
        view, o = job.views[k], fr[k]
        c_w, r_w, u_w = geom.basis(view, o)
        a = (job.TW / 2 - o["c"]) / o["sy"]; b = -(job.TH / 2 - o["top"]) / o["sy"]
        centre = r_w * a + u_w * b
        cb = geom.w2b(centre, job.scale)
        db = _dir_b(c_w)
        cb = cb - db * (cb @ db) + db * (ctr @ db)
        shoot(c_w, r_w, u_w, max(job.TW, job.TH) / o["sy"] * job.scale, cb, job.p(f"r_{k}.png"), w_px, h_px)
        tiles.append(flat(job.p(f"r_{k}.png")))
    h, w_ = tiles[0].shape[:2]
    refs = [cv2.resize(load_rgb(job, k)[:, :, ::-1], (w_, h), interpolation=cv2.INTER_AREA) for k in shown]
    if len(shown) == 4 and job.layout == "head2x2":
        grid = lambda ims: np.vstack([np.hstack(ims[:2]), np.hstack(ims[2:])])  # noqa: E731
        full = np.hstack([grid(refs), grid(tiles)])
    else:
        full = np.vstack([np.hstack(refs), np.hstack(tiles)])
    cv2.imwrite(job.shot(f"{tag}.png"), full)
    mids = []
    ref = fr["front"] if "front" in fr else fr[shown[0]]
    ext = max(job.TW, job.TH) / ref["sy"] * job.scale
    for yaw, pitch in ((45, 20), (-40, -10), (25, 60), (150, -5)):
        c_w, r_w, u_w = camera(yaw, pitch)
        shoot(c_w, r_w, u_w, ext, ctr, job.p("r_mid.png"), res, res)
        mids.append(flat(job.p("r_mid.png")))
    cv2.imwrite(job.shot(f"{tag}_mellom.png"), np.hstack(mids))
    job.time("render", time.time() - t0)
    return job.shot(f"{tag}.png"), job.shot(f"{tag}_mellom.png")


def render_sharpness(path, n=4, row=0):
    """Skarpheten (Laplace-varians) i det første rendrede bildet (forfra) på et ark."""
    import cv2
    im = cv2.imread(path)
    h, w = im.shape[:2]
    if h > w // 2:  # head2x2: forbildet til venstre, modellen til høyre, 2 × 2
        tile = im[: h // 2, w // 2: w // 2 + w // 4]
    else:  # body3: forbildet øverst, modellen nederst
        tile = im[h // 2:, : w // 3]
    g = cv2.cvtColor(tile, cv2.COLOR_BGR2GRAY).astype(np.float32)
    return round(float(cv2.Laplacian(g, cv2.CV_32F).var()), 1)
