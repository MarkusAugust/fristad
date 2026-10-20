"""Godkjenningssiden: eierens dom over hver tekstur og modell.

    python3 -m tools.qa approvals review UT    # siden i mappa UT: alt, i faner, det som venter først
    python3 -m tools.qa approvals check UT     # alt som venter står på siden med bilde
    python3 -m tools.qa approvals audit        # hver dom har versjonen eieren så (testen)
    python3 -m tools.qa approvals apply SVAR   # svarene fra siden inn i approvals.json
    python3 -m tools.qa approvals assets UT MAP # bildene i UT pekes til asset-lageret (MAP: sti → /_blob/id)
    python3 -m tools.qa approvals migrate SVAR # én gang: versjonen eieren så, fra svartidene

Siden er `tools/qa/approvals.html`, bygd av `review.py`: den viser
tilstanden (det som venter, det som ligger hos meg og det godkjente), ikke
en runde. Den lagrer svarene i samlingen `dom` (dokument `t_<lag>` eller
`m_<modell>` eller `u_<tema>_<modul>`: `verdict`, `why`, `at` og `version`, versjonen kortet viste).

`apply` leser svarene (en mappe med ett json-dokument per ting, `<id>.json`,
slik ArtifactData lagrer samlingen med `out_dir`, eller en json-liste med
`{id, data}`) og skriver dem inn i `assets/catalog/approvals.json`, med
versjonen som `seen`. Eieren bestemmer; dette fører bare inn.
"""
import json
import os
import sys

from .gate import DATA

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
    fp = os.path.join(DATA, "fixes.json")
    fixes = json.load(open(fp)) if os.path.exists(fp) else {}
    n = 0
    for row in rows:
        d = row["data"]
        kind, name = row["id"].split("_", 1)
        group = {"t": "textures", "m": "models", "u": "modules"}[kind]
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
        if answered_by_fix(d, (fixes.get(name) or {}).get(d.get("version") or ""), lambda: _first_commit(fp, d["version"])):
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
    return n + judged(rows)


def judged(rows, came=None):
    """Rettingene eieren har sett: en dom over versjonen en retting gjelder,
    gitt etter at rettingen kom inn i `fixes.json`, gir rettingen `judged`
    (dommens tid), så den ikke lenger venter (20. oktober: modulene eieren
    godkjente på nytt med samme dom, ventet ellers for alltid). `came` gir
    tiden en versjon kom inn (standard: første commit med den i fixes.json)."""
    p = os.path.join(DATA, "fixes.json")
    if not os.path.exists(p):
        return 0
    with open(p) as f:
        fixes = json.load(f)
    came = came or (lambda v: _first_commit(p, v))
    n = mark_judged(rows, fixes, came)
    if n:
        with open(p, "w") as f:
            json.dump(fixes, f, indent=1, ensure_ascii=False)
            f.write("\n")
    return n


def mark_judged(rows, fixes, came):
    """Som `judged`, uten fila: gir rettingene i `fixes` `judged` og gir
    antallet. Et svar uten tid, eller gitt før rettingen kom (svaret
    rettingen svarer på, 20. oktober: riddergangen), er ingen ny dom."""
    n = 0
    for row in rows:
        d = row["data"]
        name = row["id"].split("_", 1)[1]
        ver = d.get("version") or ""
        fix = (fixes.get(name) or {}).get(ver)
        if d.get("verdict") not in VERDICTS or not fix or not fix.get("answers") or fix.get("judged"):
            continue
        if after_fix(d, came(ver)):
            fix["judged"] = d["at"]
            n += 1
    return n


def answered_by_fix(d, fix, came):
    """Svaret er det en retting av samme versjon svarer på (gitt før
    rettingen kom inn, og rettingen er ikke dømt): det er allerede ført inn
    og er ingen ny dom (20. oktober: riddergangen). `came()` gir tiden."""
    return bool(fix and fix.get("answers") and not fix.get("judged") and not after_fix(d, came() or _now()))


def _now():
    from datetime import datetime, timezone
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def after_fix(d, fixed_at):
    """Om svaret `d` er gitt etter rettingen (`fixed_at`, tiden den kom inn).
    Et svar uten gyldig tid regnes aldri som etter."""
    at = _utc(d.get("at") or "")
    return bool(fixed_at) and at is not None and (_utc(fixed_at) or at) < at


def _first_commit(path, version):
    import subprocess
    out = subprocess.run(["git", "log", "--reverse", "--format=%cI", "-S", f'"{version}"', "--", os.path.basename(path)],
                         capture_output=True, text=True, cwd=os.path.dirname(path)).stdout.split()
    return out[0] if out else None


def _utc(t):
    from datetime import datetime, timezone
    try:
        return datetime.fromisoformat(t.replace("Z", "+00:00")).astimezone(timezone.utc)
    except ValueError:
        return None


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
            print(f"{label}: {sum(it['state'] == 'venter' for it in c)} venter, {sum(it['state'] == 'hos_meg' for it in c)} hos meg, {sum(it['state'] == 'godkjent' for it in c)} godkjent" + (f", {sum(it['state'] == 'i_modul' for it in c)} bit(er) dømt gjennom modulene og kortene" if any(it['state'] == 'i_modul' for it in c) else "") + (f", {sum(it['state'] == 'utgaar' for it in c)} utgår" if any(it['state'] == 'utgaar' for it in c) else ""))
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
    if argv[:1] == ["assets"] and len(argv) == 3:
        from . import review
        with open(argv[2]) as f:
            left = review.use_assets(argv[1], json.load(f))
        print(f"{len(left)} filer igjen å publisere (siden har plass til 511 per versjon)")
        return 0 if len(left) <= 511 else 1
    if argv[:1] == ["apply"] and len(argv) == 2:
        print(f"{apply(argv[1])} dommer ført inn i assets/catalog/approvals.json")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
