"""Sammensatte gjenstander: flere deler fra samme ark, hver med sin prior og sin boks i rutene, samlet til én modell
med én tekstur.

Oppslaget får `"parts": [{"name": "stump", "prior": "cylinder", "box": {"qfl": [x0, y0, x1, y1], "qbr": […]}}, …]`.
Den første delen er hoveddelen: kameraets vipp, dreining og perspektiv finnes på den, og de andre delene får dem
låst (`fix_view`), så bare priorens mål og nullpunktet (`c`, `top`) søkes per del. Nullpunktene i de to rutene gir
delens forskyvning fra hoveddelen ved minste kvadraters triangulering (`offset`); `"lock": "axis"` holder delen på
hoveddelens loddrette akse (ambolten på stubben), `"lock": "z"` eller `"x"` låser bare den ene vannrette aksen (to
motsatte ruter ser bare retningen på tvers av synslinja, så den andre må være kjent), `"on": "<del>"` setter delen oppå en annen del og `"under": "<del>"`
under den (høyden fra priorenes mål, ikke fra bildet: silhuettsentrene under vippen gir ellers luft mellom delene),
`"y_of": "<del>"` gir samme høyde, `"to_ground": true` forlenger delen ned til modellens laveste punkt (stolper), `"at": [x, y, z]` (meter fra
hoveddelen) setter stedet helt, og
`"offset": [x, y, z]` (meter) legges til for hånd.

Hver del bygges som sin egen jobb i `<arbeidsmappa>/<del>/` (seg, dybde, fuse, mesh, tekstur), teksturene pakkes i
ett atlas (`job.tex`², sidene etter silhuettarealet eller `"tex"` per del), og `lo.npz`, `albedo.png`,
`normal.png`, `frame.json` og `texture.json` skrives i hovedmappa, så `finish` og `report` virker som for én del.
Per del i oppslaget ellers: `opt` (legges over hovedoppslagets), `tris`, `tex`, `erase`, og `"own": "<navn>"` for
deler som skal bli sin egen modell med origo i sitt eget nullpunkt (brønnvinda, som økta dreier): `finish` skriver
dem som `<navn>.glb` ved siden av hovedmodellen, med samme tekstur, dreining og skala.
"""
import json
import os
import time

import numpy as np

from . import config, geom


def part_job(job, spec, tex=None):
    opt = json.loads(json.dumps(job.opt))
    opt.pop("fix_view", None)
    opt.update(spec.get("opt") or {})
    opt["prior"] = spec.get("prior", opt.get("prior", "box"))
    opt["part_box"] = spec.get("box") or {}
    if spec.get("erase"):
        opt["erase"] = spec["erase"]
    name = spec["name"]
    return config.Job(job.ref, layout=job.layout, name=f"{job.name}.{name}", views=None, scale=job.scale,
                      tris=spec.get("tris") or max(600, job.tris // max(len(job.parts), 1)), opt=opt, root=job.root,
                      work=os.path.join(job.work, name), shots=job.shots, target=job.target, limits=job.limits,
                      tex=tex or spec.get("tex") or job.tex, grid=job.grid)


def offset(job, main_fr, part_fr, fixed=None):
    """Delens nullpunkt i verden (piksler) fra nullpunktene (c, top) i rutene, med hoveddelens kamera. `fixed`
    er aksene som er gitt på forhånd ({0: x, 1: y, 2: z}: stabling, hoveddelens akse); resten trianguleres."""
    from scipy.optimize import least_squares
    keys = [k for k, v in job.views.items() if not v.mirror_of]
    fixed = fixed or {}
    free = [i for i in range(3) if i not in fixed]

    def full(x):
        d = np.zeros(3)
        for i, xi in zip(free, x):
            d[i] = xi
        for i, val in fixed.items():
            d[i] = val
        return d

    def resid(x):
        d = full(x)
        out = []
        for k in keys:
            u, v = geom.to_img(job.views[k], main_fr[k], np.array([d[0]]), np.array([d[1]]), np.array([d[2]]))
            out += [float(u[0]) - part_fr[k]["c"], float(v[0]) - part_fr[k]["top"]]
        return np.array(out)
    if not free:
        d = full([])
        return d, float(np.abs(resid([])).max())
    r = least_squares(resid, np.zeros(len(free)))
    return full(r.x), float(np.abs(r.fun).max())


def pack(sizes, tex):
    """Plasser kvadrater (navn -> side) i et tex × tex-atlas: enkel hylleplassering, sortert etter størrelse.
    Gir navn -> (px, py, side); krymper alle om de ikke får plass."""
    order = sorted(sizes, key=lambda n: -sizes[n])
    scale = 1.0
    for _ in range(40):
        pos, x, y, row = {}, 0, 0, 0
        ok = True
        for n in order:
            s = int(sizes[n] * scale) // 8 * 8
            if x + s > tex:
                x, y, row = 0, y + row, 0
            if y + s > tex:
                ok = False
                break
            pos[n] = (x, y, s)
            x += s
            row = max(row, s)
        if ok:
            return pos
        scale *= 0.94
    raise RuntimeError("teksturene får ikke plass i atlaset")


def build(job, target=None):
    """Alle delene, så atlaset og de samlede filene i hovedmappa."""
    from . import depth, fuse, mesh, sheet, texture
    from .sheet import load_mask
    t0 = time.time()
    specs = job.parts
    subs = [part_job(job, s) for s in specs]
    sheet.seg(job)  # hele silhuetten i hovedmappa (arkene i finish og rapporten)
    areas = {}
    for sub in subs:
        sheet.seg(sub)
        areas[sub.name] = sum(int(load_mask(sub, k).sum()) for k in sub.real_views())
    # teksturplass etter silhuettarealet (kvadratroten: teksler per lengde), eller `tex` per del
    tot = sum(np.sqrt(a) for a in areas.values()) + 1e-9
    sizes = {}
    for sub, spec in zip(subs, specs):
        sizes[sub.name] = int(spec.get("tex") or min(job.tex, max(256, job.tex * np.sqrt(areas[sub.name]) / tot * 1.15)))
    pos = pack(sizes, job.tex)
    main_fr = main_cal = None
    offsets, report = {}, dict(parts=[], iou={}, prior={}, photometric={}, coverage=[], sharpness=[])
    frames, reps = {}, {}
    from .shapes import extent_y
    # 1. prioren per del (hoveddelen først: de andre får kameraet derfra)
    for sub, spec in zip(subs, specs):
        sub.tex = pos[sub.name][2]
        if main_fr is not None:
            sub.opt["fix_view"] = {k: dict(pitch=main_fr[k].get("pitch", 0.0), yaw=main_fr[k].get("yaw", 0.0),
                                           persp=main_fr[k].get("persp", 0.0), sy=main_fr[k].get("sy", 1.0))
                                   for k, v in job.views.items() if not v.mirror_of}
        print("== del", sub.name, "prior", sub.opt["prior"], "tekstur", sub.tex, flush=True)
        if not sub.opt.get("prior_only", True):
            depth.depth(sub)
        fr, cal, rep = fuse.fuse(sub)
        frames[spec["name"]], reps[spec["name"]] = fr, rep
        if main_fr is None:
            main_fr, main_cal = fr, cal
    # 2. forskyvningene: triangulert, på hoveddelens akse, stablet (on/under/y_of)
    for sub, spec in zip(subs, specs):
        pname, fr, rep = spec["name"], frames[spec["name"]], reps[spec["name"]]
        if fr is main_fr:
            d, err = np.zeros(3), 0.0
        else:
            fixed = {}
            if spec.get("at"):  # stedet gitt i oppslaget (meter fra hoveddelens nullpunkt; null = fri akse)
                fixed = {i: float(x) / job.scale for i, x in enumerate(spec["at"]) if x is not None}
            lock = spec.get("lock") or []
            lock = ["x", "z"] if lock == "axis" else ([lock] if isinstance(lock, str) else list(lock))
            for ax in lock:  # to motsatte ruter ser bare én vannrett akse: lås den andre (x eller z) når den er kjent
                fixed[{"x": 0, "y": 1, "z": 2}[ax]] = 0.0
            lo_, hi_ = extent_y(rep["prior"])
            if 1 in fixed:
                pass
            elif spec.get("on"):
                fixed[1] = offsets[spec["on"]][1] + extent_y(reps[spec["on"]]["prior"])[1] - lo_
            elif spec.get("under"):
                fixed[1] = offsets[spec["under"]][1] + extent_y(reps[spec["under"]]["prior"])[0] - hi_
            elif spec.get("y_of"):
                fixed[1] = offsets[spec["y_of"]][1]
            d, err = offset(job, main_fr, fr, fixed)
        if spec.get("offset"):
            d = d + np.array(spec["offset"], float) / job.scale
        offsets[pname] = d
        report["parts"].append(dict(name=pname, offset_err_px=round(err, 1)))
    # 3. deler som skal nå bakken (stolpene): prioren forlenges ned til det laveste punktet i modellen
    ground = min(offsets[s_["name"]][1] + extent_y(reps[s_["name"]]["prior"])[0] for s_ in specs)
    for sub, spec in zip(subs, specs):
        if not spec.get("to_ground"):
            continue
        pr = reps[spec["name"]]["prior"]
        top = offsets[spec["name"]][1] + extent_y(pr)[1]
        hy = (top - ground) / 2
        key = "hy" if "hy" in pr["params"] else None
        if key:
            pr["params"][key] = round(float(hy), 2)
            offsets[spec["name"]][1] = ground + hy
            fj = json.load(open(sub.p("frame.json"))); fj["prior"]["params"][key] = pr["params"][key]
            json.dump(fj, open(sub.p("frame.json"), "w"), indent=1)
    # 4. nettet og teksturen per del
    for sub, spec, prt in zip(subs, specs, report["parts"]):
        pname = spec["name"]; rep = reps[pname]
        mesh.build(sub)
        trep = texture.build(sub)
        d = offsets[pname]
        prt.update(prior=rep.get("prior"), offset_px=[round(float(x), 1) for x in d],
                   offset_m=[round(float(x) * job.scale, 3) for x in d], tex=sub.tex,
                   tris=int(np.load(sub.p("lo.npz"))["f"].shape[0]))
        for k, v in rep.get("iou", {}).items():
            report["iou"][f"{pname}:{k}"] = v
        report["prior"][pname] = rep.get("prior")
        for k, v in trep.get("photometric", {}).items():
            report["photometric"][k.replace(sub.name, pname)] = v
        report["coverage"].append(trep.get("coverage", 0)); report["sharpness"].append(trep.get("sharpness", 0))
        print("   del %s: forskyvning %s m (feil %.1f px)" % (pname, prt["offset_m"], prt["offset_err_px"]), flush=True)
        for big in ("texels.npz", "field.npy"):  # plass på disken
            try:
                os.remove(sub.p(big))
            except OSError:
                pass
    combine(job, subs, specs, pos, offsets)
    frame = dict(frame=main_fr, cal=main_cal, iou=report["iou"], prior=report["prior"], parts=report["parts"])
    json.dump(frame, open(job.p("frame.json"), "w"), indent=1)
    json.dump(dict(coverage=round(float(np.mean(report["coverage"])), 4), photometric=report["photometric"],
                   sharpness=round(float(np.max(report["sharpness"])), 1), face_texels_cm=0.0),
              open(job.p("texture.json"), "w"), indent=1)
    job.limits["components_max"] = max(job.limits.get("components_max", 1), len(subs))
    job.time("parts", time.time() - t0)
    return report


def combine(job, subs, specs, pos, offsets):
    """Én modell (verdenspiksler) og ett atlas fra delene."""
    import cv2
    T = job.tex
    albedo = np.full((T, T, 3), 120, np.uint8)
    normal = np.full((T, T, 3), (255, 128, 128), np.uint8)  # BGR: (128, 128, 255)
    V, F, UV, HI, HN = [], [], [], [], []
    own = {}  # egne modeller (brønnvinda): navn -> (origo, [v], [f], [uv])
    n0 = 0
    for sub, spec in zip(subs, specs):
        lo = np.load(sub.p("lo.npz"))
        px, py, s = pos[sub.name]
        d = offsets[spec["name"]].astype(np.float32)
        v = lo["v"] + d
        uv = lo["uv"].copy()
        uv[..., 0] = (uv[..., 0] * s + px) / T
        uv[..., 1] = 1 - ((1 - uv[..., 1]) * s + py) / T
        if spec.get("own"):
            o = own.setdefault(spec["own"], [d.copy(), [], [], [], 0])
            o[1].append(v); o[2].append(lo["f"] + o[4]); o[3].append(uv); o[4] += len(v)
        else:
            V.append(v); F.append(lo["f"] + n0); UV.append(uv)
            n0 += len(v)
        for img, stem in ((albedo, "albedo"), (normal, "normal")):
            im = cv2.imread(sub.p(f"{stem}.png"))
            if im.shape[0] != s:
                im = cv2.resize(im, (s, s), interpolation=cv2.INTER_AREA)
            img[py:py + s, px:px + s] = im
        HI.append(np.load(sub.p("hi.npy")) + d); HN.append(np.load(sub.p("hi_n.npy")))
    np.savez(job.p("lo.npz"), v=np.concatenate(V).astype(np.float32), f=np.concatenate(F).astype(np.int32),
             uv=np.concatenate(UV).astype(np.float32))
    np.save(job.p("hi.npy"), np.concatenate(HI).astype(np.float32))
    np.save(job.p("hi_n.npy"), np.concatenate(HN).astype(np.float32))
    for name, (origin, vs, fs, uvs, _) in own.items():
        np.savez(job.p(f"own_{name}.npz"), origin=origin, v=np.concatenate(vs).astype(np.float32),
                 f=np.concatenate(fs).astype(np.int32), uv=np.concatenate(uvs).astype(np.float32))
        print("egen modell %s: %d trekanter, origo %s px" % (name, sum(len(f) for f in fs), np.round(origin, 1)))
    cv2.imwrite(job.p("albedo.png"), albedo)
    cv2.imwrite(job.p("normal.png"), normal)
    print("samlet: %d trekanter, atlas %d²" % (sum(len(f) for f in F), T))
