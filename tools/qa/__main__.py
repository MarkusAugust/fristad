import sys

from . import texture


def main(argv):
    """python3 -m tools.qa texture [lag …] | catalog"""
    if argv[:1] == ["catalog"]:
        from . import catalog
        m, t, r = catalog.build()
        print(f"docs/katalog.md: {len(t)} teksturer, {len(m)} modeller, {len(r)} forbilder")
        return 0
    if not argv or argv[0] != "texture":
        print(main.__doc__)
        return 2
    items = argv[1:] or texture.names()
    bad = 0
    for name in items:
        r = texture.check(name)
        r.save()
        print(r.text())
        bad += not r.ok
    print(f"\n{len(items) - bad} av {len(items)} godkjent")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
