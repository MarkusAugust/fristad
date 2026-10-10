"""Hode Z: Buttonsmedens hode rekonstruert fra forbildet. Koden er flyttet til tools/reconstruct
(se docs/rekonstruksjon.md); dette er en tynn innpakning.

    python3.13 tools/models/hode_z.py [steg,steg,...]
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
from tools.reconstruct.__main__ import main  # noqa: E402

if __name__ == "__main__":
    args = ["smith_head", "--shots", "shots/hode_z/pakke"]
    if len(sys.argv) > 1:
        args += ["--steps", sys.argv[1]]
    main(args)
