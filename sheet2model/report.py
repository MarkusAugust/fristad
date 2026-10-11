"""Steg 9: `rapport.json` med tallene, tersklene og dommen, og de automatiske rettelsene.

Tallene: IoU per visning (formens silhuett mot forbildets), innrettingen (hva som ble flyttet og dreid),
fotometrisk feil per par av nabovisninger (teksler begge ser: snittavvik i luminans, 0–1), løse komponenter,
hull, trekanter, teksler per cm i ansiktet, skarphet (Laplace-varians i ansiktet), redskapet (funnet, hånd), det
som ble maskert, og tiden per steg.

Tersklene står i arket (`limits`) med standardene i `config.DEFAULT_LIMITS`. Under terskel prøver `cli.build`
rettelsene i `FIXES` etter tur og beholder den beste; går ikke det, feiler byggingen med `feil.png`.
"""
import json
import os

import numpy as np

# (navn, betingelse på rapporten, endring i opt, første steget som må kjøres om igjen)
FIXES = [
    ("hudlinjer", lambda r: r["layout"] == "head2x2" and (r["worst_iou"] < r["limits"]["iou_min"]
                                                           or r["worst_photo"] > r["limits"]["photo_max"]),
     dict(align_skin=True), "fuse"),
    ("mykere kant", lambda r: r["worst_photo"] > r["limits"]["photo_max"], dict(edge_fade=30.0), "texture"),
    ("bare én profil", lambda r: r["worst_photo"] > r["limits"]["photo_max"] and r["layout"] == "head2x2",
     dict(one_profile=True), "seg"),
    ("flak", lambda r: r["components"] > r["limits"]["components_max"], dict(flake_frac=0.05), "mesh"),
    ("slakkere dybde", lambda r: r["worst_iou"] < r["limits"]["iou_min"], dict(edge_slack=40.0, edge_gain=1.0), "fuse"),
]


def compute(job, assembled=False):
    from .mesh import boundary_loops, components
    fr = json.load(open(job.p("frame.json")))
    r = dict(name=job.name, layout=job.layout, limits=job.limits, iou=fr.get("iou", {}), align=fr.get("align", {}),
             cal={k: dict(a=round(v["a"], 2), residual_px=round(v["residual"], 1)) for k, v in fr.get("cal", {}).items()})
    if assembled and os.path.exists(job.p("assembled.npz")):
        d = np.load(job.p("assembled.npz"), allow_pickle=True)
        names = list(d["part_names"])
        body = d["f"][np.isin(d["part"], [i for i, n in enumerate(names) if not n.startswith("tool")])]
        r["tris"] = int(len(d["f"]))
        # kroppen og hodet er hver sin bit (hodet står inne i kragen): tell per del
        per = {n: int(components(d["v"], d["f"][d["part"] == i])) for i, n in enumerate(names) if not n.startswith("tool")}
        r["components_per_part"] = per
        r["components"] = max(per.values())
        r["holes"] = int(boundary_loops(body))
        r["parts"] = names
        t = json.load(open(job.p("assemble.json")))
        for k in ("coverage", "photometric", "sharpness", "face_texels_cm", "tool", "head_scale", "head_scale_front",
                  "head_scale_side", "ghost_faces"):
            if k in t:
                r[k] = t[k]
    else:
        d = np.load(job.p("lo.npz"))
        r["tris"] = int(len(d["f"]))
        r["components"] = int(components(d["v"], d["f"]))
        r["holes"] = int(boundary_loops(d["f"]))
        if os.path.exists(job.p("texture.json")):
            r.update(json.load(open(job.p("texture.json"))))
    if os.path.exists(job.p("maskert.json")):
        r["masked"] = json.load(open(job.p("maskert.json")))
    if os.path.exists(job.p("tid.json")):
        r["seconds"] = json.load(open(job.p("tid.json")))
        r["seconds_total"] = round(sum(r["seconds"].values()), 1)
    ious = {k: v for k, v in r["iou"].items()}
    r["worst_iou_view"] = min(ious, key=ious.get) if ious else None
    r["worst_iou"] = min(ious.values()) if ious else 1.0
    ph = r.get("photometric", {})
    r["worst_photo_pair"] = max(ph, key=ph.get) if ph else None
    r["worst_photo"] = max(ph.values()) if ph else 0.0
    r["problems"] = judge(r)
    r["ok"] = not r["problems"]
    return r


def judge(r):
    L = r["limits"]
    out = []
    if r["worst_iou"] < L["iou_min"]:
        out.append("IoU %.2f i %s (minst %.2f)" % (r["worst_iou"], r["worst_iou_view"], L["iou_min"]))
    if r["worst_photo"] > L["photo_max"]:
        out.append("fotometrisk feil %.3f i %s (høyst %.3f)" % (r["worst_photo"], r["worst_photo_pair"], L["photo_max"]))
    if r.get("components", 1) > L["components_max"]:
        out.append("%d løse komponenter (høyst %d)" % (r["components"], L["components_max"]))
    # hull: halskuttet er ett hull på et hode, og på en hel figur står hodets halsåpning inne i kragen
    allow = L["holes_max"]
    if r.get("holes", 0) > allow:
        out.append("%d hull (høyst %d)" % (r["holes"], allow))
    if r.get("face_texels_cm", 99) < L["face_texels_cm_min"]:
        out.append("%.1f teksler/cm i ansiktet (minst %.1f)" % (r["face_texels_cm"], L["face_texels_cm_min"]))
    if r.get("sharpness", 999) < L.get("sharpness_min", 0):
        out.append("skarphet %.0f (minst %.0f)" % (r["sharpness"], L["sharpness_min"]))
    t = r.get("tool")
    if isinstance(t, dict) and not t.get("found", True):
        out.append("redskapet ble ikke funnet i hånd %s" % t.get("side"))
    return out


def write(job, r, name="rapport.json"):
    path = job.p(name)
    json.dump(r, open(path, "w"), indent=1, ensure_ascii=False)
    summary = "%s: %s | IoU %s | foto %.3f | komp %d hull %d | %d tri | %s teksler/cm | skarphet %s | %ss" % (
        job.name, "OK" if r["ok"] else "FEIL: " + "; ".join(r["problems"]),
        " ".join("%s %.2f" % (k, v) for k, v in r["iou"].items()), r["worst_photo"], r.get("components", 0),
        r.get("holes", 0), r.get("tris", 0), r.get("face_texels_cm", "?"), r.get("sharpness", "?"), r.get("seconds_total", "?"))
    print(summary)
    return path


def failure_image(job, path=None):
    """Bilde av det som er galt: forbildets silhuett over formens i visningen med lavest IoU."""
    from . import align, fuse
    path = path or job.shot("feil.png")
    fr, _ = fuse.load_frame(job)
    F = np.load(job.p("field.npy"))
    grid = fuse.make_grid(job, fr)
    if grid.shape != F.shape:
        return None
    return align.worst_view_image(job, fr, grid, F > 0, path) and path
