"""Sjekker at Blender-skriptene henger sammen uten å starte Blender: hver fil
kan kompileres, og hvert `modul.navn` mot en av hjelpemodulene her
(`town.SMOKE`, `kit.door`, `common.material` …) finnes i modulen.

    python3 tools/models/check_scripts.py      # skriver feilene, kode 1 ved feil
"""
import ast
import glob
import os
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
FILES = sorted(glob.glob(os.path.join(ROOT, "tools/models/*.py")) + glob.glob(os.path.join(ROOT, "tools/crypt/*.py")))


def top_names(tree):
    names = set()
    for n in tree.body:
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            names.add(n.name)
        elif isinstance(n, (ast.Assign, ast.AugAssign, ast.AnnAssign)):
            for t in n.targets if isinstance(n, ast.Assign) else [n.target]:
                names |= {x.id for x in ast.walk(t) if isinstance(x, ast.Name)}
        elif isinstance(n, (ast.Import, ast.ImportFrom)):
            names |= {(a.asname or a.name).split(".")[0] for a in n.names}
        elif isinstance(n, (ast.If, ast.Try, ast.For, ast.With)):
            names |= top_names(ast.Module(body=getattr(n, "body", []), type_ignores=[]))
    return names


def main():
    trees, errors = {}, []
    for p in FILES:
        try:
            trees[p] = ast.parse(open(p).read(), p)
        except SyntaxError as e:
            errors.append(f"{p}: {e}")
    defs = {os.path.splitext(os.path.basename(p))[0]: top_names(t) for p, t in trees.items()}
    for p, t in trees.items():
        me = os.path.splitext(os.path.basename(p))[0]
        for x in ast.walk(t):
            if isinstance(x, ast.Attribute) and isinstance(x.value, ast.Name) and x.value.id in defs and x.value.id != me:
                if x.attr not in defs[x.value.id] and not x.attr.startswith("__"):
                    errors.append(f"{os.path.relpath(p, ROOT)}:{x.lineno}: {x.value.id}.{x.attr} finnes ikke")
    for e in sorted(set(errors)):
        print(e)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
