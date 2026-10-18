"""Målet for Fristad: de 18 beina som `people.game_bones`, klippene fra `tools/models/folk_motion.py`, figuren
vendt mot +x med sålen på z = 0, og glb slanket med `tools/models/slim.py`.

`root` er Fristad-repoet (der `tools/models/` ligger); settes fra `"root"` i arket, `SHEET2MODEL_ROOT` eller
`--root`.
"""
import os
import subprocess
import sys

import numpy as np

from .base import Target as _Base


class Target(_Base):
    name = "fristad"

    def __init__(self, root=None):
        super().__init__(root)
        self.models = os.path.join(self.root or os.getcwd(), "tools/models")
        if self.models not in sys.path:
            sys.path.insert(0, self.models)

    def to_game(self, p):
        """Spillets figurer ser mot +x: -Y fram i Blender dreies 90° om z."""
        b = np.asarray(p, float)
        return np.stack([-b[..., 1], b[..., 0], b[..., 2]], -1)

    def clips(self, rig, body, opt, tools=()):
        import folk_motion
        style = opt.get("style", "default")
        grip = opt.get("grip", "hang")
        side = opt.get("tool_side", "R")
        sty = dict(folk_motion.style_of(style)) if isinstance(style, str) else dict(style)
        # Armene i rekonstruksjonen henger alt et stykke ut fra kroppen, så stilens arm_out legges ikke oppå
        sty["arm_out"] = opt.get("arm_out", -6.0)
        sty["arm"] = sty.get("arm", 15.0) * opt.get("arm_swing", 1.6)  # tydeligere armsving i gangen
        st = folk_motion.clips(rig, sty, {side: grip} if grip else {}, body=body, tools=None)
        return dict(speed=st.get("speed"), style=style, grip=grip)

    def post_export(self, path):
        slim = os.path.join(self.models, "slim.py")
        if os.path.exists(slim):
            subprocess.run([sys.executable, slim, path], check=False)
        return path
