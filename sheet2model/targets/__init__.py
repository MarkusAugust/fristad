"""Målene: alt som er spesifikt for ett spill (skjelett, klipp, materialnavn, slanking, retning) bor her.

Et mål er en klasse med metodene i `base.Target`. Pakken kaller dem; den vet ingenting om spillet. Nytt prosjekt:
kopier `generic.py`, fyll inn og pek på det med `"target": "mittspill"` i arket eller `--target`.
"""
import importlib


def load(name, root=None):
    name = name or "generic"
    mod = importlib.import_module(f"{__name__}.{name}")
    return mod.Target(root=root)
