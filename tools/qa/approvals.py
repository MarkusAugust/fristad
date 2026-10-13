"""Godkjenningssiden: eierens dom over hver tekstur og modell.

    python3 -m tools.qa approvals review UT    # siden i mappa UT: alt, i faner, det som venter først
    python3 -m tools.qa approvals check UT     # alt som venter står på siden med bilde
    python3 -m tools.qa approvals audit        # hver dom har versjonen eieren så (testen)
    python3 -m tools.qa approvals apply SVAR   # svarene fra siden inn i approvals.json
    python3 -m tools.qa approvals migrate SVAR # én gang: versjonen eieren så, fra svartidene

Siden er `tools/qa/approvals.html`, bygd av `review.py`: den viser
tilstanden (det som venter, det som ligger hos meg og det godkjente), ikke
en runde. Den lagrer svarene i samlingen `dom` (dokument `t_<lag>` eller
`m_<modell>`: `verdict`, `why`, `at` og `version`, versjonen kortet viste).

`apply` leser svarene (en mappe med ett json-dokument per ting, `<id>.json`,
slik ArtifactData lagrer samlingen med `out_dir`, eller en json-liste med
`{id, data}`) og skriver dem inn i `assets/catalog/approvals.json`, med
versjonen som `seen`. Eieren bestemmer; dette fører bare inn.
"""
import json
import os
import sys

from .catalog import DATA

VERDICTS = ("godkjent", "underkjent", "usikker")


def apply(path):
    """Svarene fra siden inn i approvals.json. `path` er en mappe med
    `<id>.json` (`{verdict, why, at}`) eller en json-liste med `{id, data}`."""
    if os.path.isdir(path):
        # ArtifactData med `out_dir` legger svarene i `<out_dir>/dom/`.
        if os.path.isdir(os.path.join(path, "dom")):
            path = os.path.join(path, "dom")
        rows = []
        for name in sorted(os.listdir(path)):
            if name.endswith(".json"):
                with open(os.path.join(path, name)) as f:
                    rows.append({"id": name[:-5], "data": json.load(f)})
    else:
        with open(path) as f:
            rows = json.load(f)
    if not rows:
        raise SystemExit(f"ingen svar i {path}")
    p = os.path.join(DATA, "approvals.json")
    with open(p) as f:
        ap = json.load(f)
    n = 0
    for row in rows:
        d = row["data"]
        kind, name = row["id"].split("_", 1)
        group = {"t": "textures", "m": "models"}[kind]
        if d.get("verdict") not in VERDICTS:
            # Et punkt uten dom (runde 17: «øksa holdes feil vei»): dommen
            # står, men punktet føres inn så det rettes og vises i neste runde.
            old = ap.setdefault(group, {}).get(name)
            why = d.get("why", "").strip()
            if old and why and why not in old.get("history", []):
                old["history"] = list(old.get("history", [])) + [why]
                old["why"] = why
                n += 1
            continue
        entry = {"verdict": d["verdict"], "date": d.get("at", "")[:10], "why": d.get("why", "").strip()}
        old = ap.setdefault(group, {}).get(name)
        # Versjonen dommen gjelder (siden sender den med); svar fra de gamle
        # rundesidene har den ikke, og beholder den forrige.
        if d.get("version"):
            entry["seen"] = d["version"]
        elif old and old.get("seen"):
            entry["seen"] = old["seen"]
        # Ny dom: den forrige tas vare på. Og har grunnen ikke endret seg med
        # dommen, ble den arvet fra forrige runde (feltet var fylt ut, 11.
        # okt.), så den hører til den gamle dommen, ikke den nye.
        if old and old["verdict"] != entry["verdict"]:
            if old.get("why", "") == entry["why"]:
                entry["why"] = ""
            entry["before"] = {k: old[k] for k in ("verdict", "date", "why")}
        elif old and old.get("before"):
            entry["before"] = old["before"]
        # Alle punktene eieren har skrevet om tingen, på tvers av rundene,
        # så ingen grunn blir borte når dommen står eller grunnen endres.
        seen = list((old or {}).get("history", []))
        for w in ((old or {}).get("why", ""), d.get("why", "").strip()):
            if w and w not in seen:
                seen.append(w)
        if seen:
            entry["history"] = seen
        if old != entry:
            ap[group][name] = entry
            n += 1
    with open(p, "w") as f:
        json.dump(ap, f, indent=1, ensure_ascii=False)
        f.write("\n")
    return n


def main(argv):
    if argv[:1] == ["migrate"] and len(argv) == 2:
        from . import review
        print(f"{review.migrate(argv[1])} dommer fikk versjonen eieren så")
        return 0
    if argv[:1] == ["review"] and len(argv) == 2:
        from . import review
        items, carried = review.build(argv[1])
        for key, label in review.CATEGORIES:
            c = [it for it in items if it["cat"] == key]
            print(f"{label}: {sum(it['state'] == 'venter' for it in c)} venter, {sum(it['state'] == 'hos_meg' for it in c)} hos meg, {sum(it['state'] == 'godkjent' for it in c)} godkjent")
        print(f"{carried} bygd på nytt uten å se annerledes ut (dommen står)")
        errors = review.check(argv[1])
        for e in errors:
            print("FEIL:", e)
        return 1 if errors else 0
    if argv[:1] == ["audit"]:
        from . import review
        errors = review.audit()
        for e in errors:
            print("FEIL:", e)
        return 1 if errors else 0
    if argv[:1] == ["check"] and len(argv) == 2:
        from . import review
        errors = review.check(argv[1])
        for e in errors:
            print("FEIL:", e)
        return 1 if errors else 0
    if argv[:1] == ["apply"] and len(argv) == 2:
        print(f"{apply(argv[1])} dommer ført inn i assets/catalog/approvals.json")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
