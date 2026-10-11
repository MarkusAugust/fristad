"""Kommandolinja og byggeløpet med de automatiske rettelsene.

    python3.13 -m sheet2model smith_head                      # sheets/smith_head.json (eller examples/fristad/)
    python3.13 -m sheet2model smith_body --head smith_head --rig-out assets/models/npc/npc_smith.glb
    python3.13 -m sheet2model ark.jpeg --layout head2x2 --out modell.glb
    python3.13 -m sheet2model smith_head --steps seg,depth,fuse,mesh,texture,finish,report
    python3.13 -m sheet2model smith_body --head smith_head --steps assemble,finish,rig,report

Stegene: seg, depth, fuse (med innretting), mesh, texture, finish (arkene og glb for ett ark), assemble (hodet på
kroppen, redskapet, én tekstur), rig, report. `--no-fix` slår av rettelsene.
"""
import argparse
import json
import os
import sys
import time

from . import config

HEAD_STEPS = ["seg", "depth", "fuse", "mesh", "texture", "finish", "report"]
BODY_STEPS = ["seg", "depth", "fuse", "mesh", "assemble", "finish", "rig", "report"]
PARTS_STEPS = ["parts", "finish", "report"]


def run_steps(job, steps, head=None, target=None, rig_out=None, glb=None):
    from . import assemble, depth, finish, fuse, mesh, report, rig, sheet, texture
    for s in steps:
        print("==", job.name, s, flush=True)
        t = time.time()
        if s == "seg":
            sheet.seg(job)
        elif s == "depth":
            if job.prop and job.opt.get("prior_only", True):
                print("   (geometrien er prioren: ingen dybde)")
            else:
                depth.depth(job)
        elif s == "parts":
            from . import parts
            parts.build(job, target)
        elif s == "fuse":
            fuse.fuse(job)
        elif s == "look":
            print(fuse.look(job))
        elif s == "mesh":
            mesh.build(job)
        elif s == "texture":
            texture.build(job)
        elif s == "assemble":
            assemble.assemble(job, head, target)
        elif s == "finish":
            if head is not None and os.path.exists(job.p("assembled.npz")):
                finish_full(job)
            else:
                finish.finish(job, glb=glb, target=target)
        elif s == "rig":
            rig.build(job, target, out=rig_out)
        elif s == "report":
            r = report.compute(job, assembled=head is not None)
            report.write(job, r)
        else:
            raise SystemExit("ukjent steg: " + s)
        print("   %.1f s" % (time.time() - t), flush=True)


def finish_full(job):
    """Arkene for en hel figur (kropp + hode + redskap med én tekstur)."""
    import numpy as np
    from . import finish, fuse
    fr, _ = fuse.load_frame(job)
    d = np.load(job.p("assembled.npz"), allow_pickle=True)
    parts = [("full", d["v"], d["f"], d["uv"])]
    if job.opt.get("render", "cycles") == "cycles":
        return finish.render_bpy(job, parts, fr, albedo="albedo_full.png", normal="normal_full.png", tag="ark_hel")
    return finish.sheets(job, finish.Cloud.load(job, stem="albedo_full", texels="texels_full.npz"), fr, tag="ark_hel")


def build(job, steps=None, head=None, target=None, rig_out=None, glb=None, fix=True):
    """Hele løpet med rettelsene: bygg, mål, og prøv `report.FIXES` etter tur til rapporten er OK (eller feil med
    `feil.png`)."""
    from . import report
    if job.parts:  # sammensatt gjenstand: delene bygges i parts.py, uten rettelsene
        steps = steps or PARTS_STEPS
        fix = False
    steps = steps or (BODY_STEPS if head is not None else HEAD_STEPS)
    run_steps(job, steps, head, target, rig_out, glb)
    if "report" not in steps:
        return None
    r = report.compute(job, assembled=head is not None)
    tried = []
    if fix and not r["ok"]:
        best = (score(r), r, None)
        for name, cond, change, restart in report.FIXES:
            if not cond(r):
                continue
            i = steps.index(restart) if restart in steps else 0
            print("== rettelse: %s (%s) fra steget %s" % (name, change, restart), flush=True)
            saved = dict(job.opt)
            job.opt.update(change)
            if "one_profile" in change:
                job.__init__(job.ref, layout=job.layout, name=job.name, views=None, scale=job.scale, tris=job.tris,
                             opt=job.opt, root=job.root, target=job.target, head=job.head, limits=job.limits, tex=job.tex)
            try:
                run_steps(job, [s for s in steps[i:] if s != "report"], head, target, rig_out, glb)
                r2 = report.compute(job, assembled=head is not None)
            except Exception as e:  # noqa: BLE001
                print("rettelsen feilet:", e)
                job.opt = saved
                continue
            tried.append(dict(fix=name, change=change, problems=r2["problems"]))
            if score(r2) > best[0]:
                best = (score(r2), r2, dict(job.opt))
            else:
                job.opt = saved
            r = r2
            if r["ok"]:
                break
        if best[2] is not None and not r["ok"]:
            job.opt = best[2]
            r = best[1]
    r["fixes_tried"] = tried
    report.write(job, r)
    if not r["ok"]:
        img = None
        try:
            img = report.failure_image(job)
        except Exception as e:  # noqa: BLE001
            print("feilbilde ikke laget:", e)
        raise SystemExit("FEIL: %s. Se %s" % ("; ".join(r["problems"]), img or job.p("rapport.json")))
    return r


def score(r):
    return -len(r["problems"]) * 10 + r["worst_iou"] - r["worst_photo"]


def main(argv=None):
    ap = argparse.ArgumentParser(prog="python3.13 -m sheet2model", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("sheet", help="navn på et oppslag (sheets/<navn>.json) eller sti til et ark eller en json")
    ap.add_argument("--sheets", help="mappa med oppslagene (standard: $SHEET2MODEL_SHEETS eller examples/fristad)")
    ap.add_argument("--layout", default=None, choices=sorted(config.LAYOUTS), help="for et ark uten oppslag")
    ap.add_argument("--root", help="prosjektrota (stier i oppslagene er relative til den)")
    ap.add_argument("--target", help="målet (targets/<navn>.py); standard fra oppslaget eller generic")
    ap.add_argument("--head", help="hodearket som settes på kroppen (steget assemble)")
    ap.add_argument("--out", help="glb-fila for ett ark (steget finish)")
    ap.add_argument("--rig-out", help="rigget glb (steget rig)")
    ap.add_argument("--tris", type=int)
    ap.add_argument("--steps", help="kommaskilt; standard hele løpet")
    ap.add_argument("--no-fix", action="store_true", help="ingen automatiske rettelser")
    ap.add_argument("--work", help="arbeidsmappa (standard <root>/target/sheet2model/<navn>)")
    ap.add_argument("--shots", help="mappa for arkene (standard <root>/shots/sheet2model/<navn>)")
    a = ap.parse_args(argv)
    root = a.root or os.environ.get("SHEET2MODEL_ROOT")
    if a.sheet.endswith(".json") or not os.path.exists(a.sheet):
        job = config.load_sheet(a.sheet, sheets_dir=a.sheets, root=root, tris=a.tris, work=a.work, shots=a.shots)
    else:
        job = config.Job(a.sheet, a.layout or "head2x2", tris=a.tris, root=root, work=a.work, shots=a.shots, target=a.target)
    head = None
    head_name = a.head or job.head
    if head_name:
        head = config.load_sheet(head_name, sheets_dir=a.sheets, root=root)
    from . import targets
    target = targets.load(a.target or job.target, root=job.root)
    steps = a.steps.split(",") if a.steps else None
    r = build(job, steps, head=head, target=target, rig_out=a.rig_out, glb=a.out, fix=not a.no_fix)
    if r is not None:
        print(json.dumps(r.get("seconds", {})))


if __name__ == "__main__":
    main(sys.argv[1:])
