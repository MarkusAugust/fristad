"""Bygg en modell fra et forbildeark.

    python3.13 -m tools.reconstruct smith_head                       # oppslag i sheets.json
    python3.13 -m tools.reconstruct ark.jpeg --layout head2x2 --out modell.glb [--tris 7000]
    ... [--steps seg,depth,fuse,bake,project,render]
"""
import argparse
import os
import sys

from . import job as J

STEPS = ["seg", "depth", "fuse", "bake", "project", "render"]


def main(argv=None):
    ap = argparse.ArgumentParser(prog="python3.13 -m tools.reconstruct")
    ap.add_argument("sheet", help="navn i sheets.json eller sti til et ark")
    ap.add_argument("--layout", default="head2x2", choices=sorted(J.LAYOUTS))
    ap.add_argument("--out", help="glb-fila (standard: shots/reconstruct/<navn>/<navn>.glb)")
    ap.add_argument("--tris", type=int, help="trekanter etter desimering (standard 7000)")
    ap.add_argument("--steps", default=",".join(STEPS))
    ap.add_argument("--head", default=None, help="hodearket som settes på kroppen (steget assemble)")
    ap.add_argument("--rig-out", help="rigget glb (steget rig: 18 bein, vekter, klippene fra folk_motion)")
    ap.add_argument("--shots", help="mappe for ark og turnaround")
    a = ap.parse_args(argv)
    if os.path.exists(a.sheet):
        J.setup(a.sheet, a.layout, tris=a.tris, shots=a.shots)
    else:
        J.from_config(a.sheet, tris=a.tris, shots=a.shots)
    from . import assemble, depth, finish, fuse, rig, sheet, texture
    run = dict(seg=sheet.seg, depth=depth.depth, fuse=fuse.fuse, look=fuse.look, bake=texture.bake,
               project=texture.project, render=lambda: finish.render(glb=a.out),
               rig=lambda: rig.build(out=a.rig_out),
               assemble=lambda: assemble.assemble(head=a.head),
               render_full=lambda: finish.render(res=768, assembled=True))
    for s in a.steps.split(","):
        print("==", s, flush=True)
        run[s]()


if __name__ == "__main__":
    main(sys.argv[1:])
