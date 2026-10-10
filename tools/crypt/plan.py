"""Planene for kryptnivåene: data og tegning ovenfra.

Hvert nivå er en modul i `tools/crypt/levels/` med `LEVEL = Level(...)`. Planen er
både tegningen eieren godkjenner og kilden nivåskriptet i Blender bygger fra.

Koordinater i meter: x mot øst, z mot sør (som i spillet), y er gulvhøyden.

    python3 tools/crypt/plan.py            # tegner alle nivåene til docs/krypten/
    python3 tools/crypt/plan.py 1 3        # bare nivå 1 og 3
"""

import importlib.util
import math
import sys
from dataclasses import dataclass, field
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]
LEVELS = Path(__file__).resolve().parent / "levels"
OUT = ROOT / "docs" / "krypten"

# --- Data ---------------------------------------------------------------------


@dataclass
class Room:
    id: str
    name: str
    shape: tuple  # ("rect", x, z, w, d) senter og mål, ("circle", x, z, r), ("poly", [(x, z), ...])
    y: float = 0.0  # gulvhøyde
    h: float = 4.0  # takhøyde over gulvet
    secret: bool = False
    note: str = ""
    # flere gulvhøyder i samme rom: [("rect"/"ring"/..., ..., y)] tegnes oppå
    levels: list = field(default_factory=list)
    # valgfritt ankerpunkt (x, z) for forbindelser og navn, for buer og andre former
    # der snittet av hjørnene havner utenfor rommet
    at: tuple = None
    # tegnes under forbindelsene (sjøer, tomrom), så broene over synes
    backdrop: bool = False

    def center(self):
        if self.at:
            return self.at
        s = self.shape
        if s[0] in ("rect", "circle"):
            return (s[1], s[2])
        xs = [p[0] for p in s[1]]
        zs = [p[1] for p in s[1]]
        return (sum(xs) / len(xs), sum(zs) / len(zs))


# Slagene av forbindelser
DOOR, HALL, STAIRS, GATE, SHORTCUT, SECRET, LADDER, FALL, BRIDGE = (
    "door", "hall", "stairs", "gate", "shortcut", "secret", "ladder", "fall", "bridge")
LIFT = "lift"  # heis eller vinsj mellom to høyder


@dataclass
class Link:
    a: str
    b: str
    kind: str = HALL
    path: list = field(default_factory=list)  # punkter imellom (x, z)
    width: float = 3.0
    note: str = ""


# Merkene
START, REST, DOWN, UP, MOB, ELITE, BOSS, CHEST, LORE, BONUS, RARE, LEVER, TRAP, HINT = (
    "start", "rest", "down", "up", "mob", "elite", "boss", "chest", "lore", "bonus", "rare",
    "lever", "trap", "hint")


@dataclass
class Mark:
    kind: str
    x: float
    z: float
    label: str = ""
    n: int = 1  # antall fiender
    patrol: list = field(default_factory=list)  # rute (x, z) for fiender som patruljerer


@dataclass
class Level:
    n: int
    name: str
    theme: str
    intro: str
    rooms: list
    links: list
    marks: list
    main: list  # rom-id-ene langs hovedveien, i rekkefølge
    notes: list = field(default_factory=list)

    def room(self, rid):
        for r in self.rooms:
            if r.id == rid:
                return r
        raise KeyError(f"nivå {self.n}: rommet {rid} finnes ikke")

    def check(self):
        ids = [r.id for r in self.rooms]
        assert len(ids) == len(set(ids)), f"nivå {self.n}: to rom med samme id"
        for l in self.links:
            self.room(l.a), self.room(l.b)
        for a, b in zip(self.main, self.main[1:]):
            assert any({l.a, l.b} == {a, b} for l in self.links), \
                f"nivå {self.n}: hovedveien {a} → {b} har ingen forbindelse"
        kinds = [m.kind for m in self.marks]
        for k in (START, REST, DOWN):
            want = 0 if (k == DOWN and self.n == 10) else 1
            assert kinds.count(k) == want, f"nivå {self.n}: må ha {want} {k}"
        # alt skal kunne nås fra start (hemmelige rom også, gjennom sin hemmelige vei)
        seen, todo = {self.main[0]}, [self.main[0]]
        while todo:
            r = todo.pop()
            for l in self.links:
                for x, y in ((l.a, l.b), (l.b, l.a)):
                    if x == r and y not in seen and not (l.kind == FALL and x != l.a):
                        seen.add(y)
                        todo.append(y)
        lost = set(ids) - seen
        assert not lost, f"nivå {self.n}: kan ikke nås: {lost}"

    def main_length(self):
        """Lengden på hovedveien i meter, gjennom forbindelsenes punkter."""
        total = 0.0
        for a, b in zip(self.main, self.main[1:]):
            link = next(l for l in self.links if {l.a, l.b} == {a, b})
            pts = [self.room(link.a).center(), *link.path, self.room(link.b).center()]
            total += sum(math.dist(p, q) for p, q in zip(pts, pts[1:]))
        return total

    def minutes(self):
        """Grovt anslag: gange på hovedveien og tilbake i sidegrener, kamp og leting."""
        side = sum(1 for r in self.rooms if r.id not in self.main)
        fights = sum(m.n for m in self.marks if m.kind == MOB)
        t = self.main_length() / 2.5 + side * 40 + fights * 15
        t += sum({ELITE: 60, BOSS: 240, TRAP: 10, LEVER: 15}.get(m.kind, 0) for m in self.marks)
        return t / 60


# --- Tegning ------------------------------------------------------------------

PX = 7  # piksler per meter
PAD = 60
BG = (24, 26, 34)
GRID = (36, 39, 50)
INK = (236, 228, 210)
MAIN = (240, 160, 60)
SECRET_C = (190, 120, 255)

ICONS = {
    START: ("S", (120, 220, 140)), REST: ("H", (255, 210, 90)), DOWN: ("▼", (255, 120, 80)),
    UP: ("▲", (140, 200, 255)), MOB: ("•", (230, 70, 70)), ELITE: ("E", (255, 80, 80)),
    BOSS: ("B", (255, 40, 40)), CHEST: ("K", (230, 190, 90)), LORE: ("L", (180, 220, 255)),
    BONUS: ("+", (120, 255, 200)), RARE: ("★", (255, 120, 255)), LEVER: ("/", (255, 255, 255)),
    TRAP: ("!", (255, 150, 40)), HINT: ("?", (190, 120, 255)),
}
LEGEND = [
    (START, "start"), (REST, "hvilested"), (DOWN, "trapp ned"), (UP, "trapp opp"),
    (MOB, "fiende"), (ELITE, "elite"), (BOSS, "boss"), (CHEST, "kiste"), (LORE, "lore"),
    (BONUS, "varig styrke"), (RARE, "sjelden komponent"), (LEVER, "spak"), (TRAP, "felle"),
    (HINT, "hint"),
]
LINK_STYLE = {
    DOOR: ((200, 190, 170), "dør"), HALL: ((150, 145, 135), "gang"),
    STAIRS: ((120, 200, 255), "trapp"), GATE: ((255, 90, 90), "port/gitter"),
    SHORTCUT: ((120, 255, 160), "snarvei"), SECRET: (SECRET_C, "hemmelig"),
    LADDER: ((200, 200, 255), "stige"), FALL: ((255, 120, 200), "fall"),
    BRIDGE: ((210, 170, 110), "bro"),
    LIFT: ((255, 230, 120), "heis"),
}


def font(size, bold=False):
    name = "DejaVuSans-Bold.ttf" if bold else "DejaVuSans.ttf"
    for d in ("/usr/share/fonts/truetype/dejavu/",):
        try:
            return ImageFont.truetype(d + name, size)
        except OSError:
            pass
    return ImageFont.load_default()


def height_color(y, lo, hi):
    """Lavere gulv er mørkere og blåere, høyere lysere og varmere."""
    t = 0.5 if hi - lo < 0.01 else (y - lo) / (hi - lo)
    a = (52, 62, 96)
    b = (150, 140, 120)
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def bounds(level):
    xs, zs = [], []
    for r in level.rooms:
        s = r.shape
        if s[0] == "rect":
            xs += [s[1] - s[3] / 2, s[1] + s[3] / 2]
            zs += [s[2] - s[4] / 2, s[2] + s[4] / 2]
        elif s[0] == "circle":
            xs += [s[1] - s[3], s[1] + s[3]]
            zs += [s[2] - s[3], s[2] + s[3]]
        else:
            xs += [p[0] for p in s[1]]
            zs += [p[1] for p in s[1]]
    for l in level.links:
        xs += [p[0] for p in l.path]
        zs += [p[1] for p in l.path]
    return min(xs) - 6, min(zs) - 6, max(xs) + 6, max(zs) + 6


def draw(level, out):
    level.check()
    x0, z0, x1, z1 = bounds(level)
    mw, mh = int((x1 - x0) * PX), int((z1 - z0) * PX)
    side = 360
    W, H = mw + 2 * PAD + side, mh + 2 * PAD + 110
    # sidefeltet (tegnforklaring og notater) kan være høyere enn kartet
    side_h = 100 + 26 + 21 * len(LEGEND) + 6 + 19 * len(LINK_STYLE) + 22 + 30 + 24
    side_h += sum(15 * len(wrap(n, 46)) + 6 for n in level.notes) + 40
    H = max(H, side_h)
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img, "RGBA")

    def P(x, z):
        return (PAD + (x - x0) * PX, PAD + 90 + (z - z0) * PX)

    # rutenett hver 4. meter
    for gx in range(int(x0) // 4 * 4, int(x1) + 4, 4):
        d.line([P(gx, z0), P(gx, z1)], fill=GRID, width=1)
    for gz in range(int(z0) // 4 * 4, int(z1) + 4, 4):
        d.line([P(x0, gz), P(x1, gz)], fill=GRID, width=1)

    ys = [r.y for r in level.rooms] + [lv[-1] for r in level.rooms for lv in r.levels]
    lo, hi = min(ys), max(ys)

    def shape(s, fill, outline, dash=False, width=3):
        if s[0] == "rect":
            _, x, z, w, dd = s
            box = [P(x - w / 2, z - dd / 2), P(x + w / 2, z + dd / 2)]
            if dash:
                dashed(d, [box[0], (box[1][0], box[0][1]), box[1], (box[0][0], box[1][1]), box[0]], outline, width)
                d.rectangle(box, fill=fill)
            else:
                d.rectangle(box, fill=fill, outline=outline, width=width)
        elif s[0] == "circle":
            _, x, z, r = s
            box = [P(x - r, z - r), P(x + r, z + r)]
            d.ellipse(box, fill=fill, outline=outline, width=width)
        elif s[0] == "ring":  # galleri: ring mellom r0 og r1
            _, x, z, r0, r1 = s
            d.ellipse([P(x - r1, z - r1), P(x + r1, z + r1)], outline=fill, width=int((r1 - r0) * PX))
        else:
            pts = [P(*p) for p in s[1]]
            if dash:
                d.polygon(pts, fill=fill)
                dashed(d, pts + [pts[0]], outline, width)
            else:
                d.polygon(pts, fill=fill, outline=outline, width=width)

    def room(r):
        col = height_color(r.y, lo, hi)
        shape(r.shape, col, SECRET_C if r.secret else INK, dash=r.secret)
        for lv in r.levels:
            shape(lv[:-1], height_color(lv[-1], lo, hi) + (230,), INK, width=1)

    # bakgrunnen (sjøer, tomrom) under forbindelsene
    for r in level.rooms:
        if r.backdrop:
            room(r)

    # forbindelsene under rommene
    for l in level.links:
        col, _ = LINK_STYLE[l.kind]
        pts = [P(*level.room(l.a).center()), *[P(*p) for p in l.path], P(*level.room(l.b).center())]
        w = max(3, int(l.width * PX))
        fill = height_color((level.room(l.a).y + level.room(l.b).y) / 2, lo, hi)
        if l.kind in (SECRET, FALL):
            dashed(d, pts, col, max(3, w // 3))
        else:
            d.line(pts, fill=fill, width=w, joint="curve")
            d.line(pts, fill=col + (200,), width=2)

    for r in level.rooms:
        if not r.backdrop:
            room(r)

    # hovedveien
    for a, b in zip(level.main, level.main[1:]):
        link = next(l for l in level.links if {l.a, l.b} == {a, b})
        pts = [level.room(link.a).center(), *link.path, level.room(link.b).center()]
        if link.a != a:
            pts.reverse()
        d.line([P(*p) for p in pts], fill=MAIN + (170,), width=3)

    # patruljer
    for m in level.marks:
        if m.patrol:
            dashed(d, [P(*p) for p in [*m.patrol, m.patrol[0]]], (230, 70, 70), 2, seg=6)

    # navn på rommene
    f_room = font(13, True)
    f_small = font(11)
    for r in level.rooms:
        cx, _ = P(*r.center())
        sh = r.shape
        top = sh[2] - sh[4] / 2 if sh[0] == "rect" else sh[2] - sh[3] if sh[0] == "circle" else min(p[1] for p in sh[1])
        if r.at:
            top = r.at[1] - 1
        _, ty = P(0, top)
        label = r.name + ("" if r.y == 0 else f"  ({r.y:+g} m)")
        tw = d.textlength(label, font=f_room)
        d.text((cx - tw / 2, ty - 18), label, font=f_room, fill=INK, stroke_width=3, stroke_fill=BG)

    # merkene
    f_icon = font(15, True)
    for m in level.marks:
        ch, col = ICONS[m.kind]
        cx, cz = P(m.x, m.z)
        rr = 10 if m.kind != MOB else 6
        d.ellipse([cx - rr, cz - rr, cx + rr, cz + rr], fill=(0, 0, 0, 170), outline=col, width=2)
        if m.kind != MOB:
            tw = d.textlength(ch, font=f_icon)
            d.text((cx - tw / 2, cz - 9), ch, font=f_icon, fill=col)
        else:
            d.ellipse([cx - 3, cz - 3, cx + 3, cz + 3], fill=col)
            if m.n > 1:
                d.text((cx + rr + 2, cz - 16), f"×{m.n}", font=f_small, fill=col, stroke_width=2, stroke_fill=BG)
        if m.label:
            d.text((cx + rr + 3, cz - 6), m.label, font=f_small, fill=col, stroke_width=2, stroke_fill=BG)

    # tittel
    d.text((PAD, 20), f"Nivå {level.n}: {level.name}", font=font(28, True), fill=INK)
    mins = level.minutes()
    d.text((PAD, 58), f"{level.theme}   ·   hovedveien {level.main_length():.0f} m, rundt {mins:.0f} min med kamp og leting",
           font=font(14), fill=(190, 180, 160))

    # målestokk og nord
    sx, sy = PAD, H - 30
    d.line([(sx, sy), (sx + 10 * PX, sy)], fill=INK, width=3)
    d.text((sx + 10 * PX + 6, sy - 8), "10 m", font=f_small, fill=INK)
    nx = PAD + mw - 10
    d.polygon([(nx, H - 50), (nx - 8, H - 30), (nx + 8, H - 30)], fill=INK)
    d.text((nx - 4, H - 28), "N", font=f_small, fill=INK)

    # tegnforklaring og notater
    lx = PAD + mw + 30
    ly = 100
    d.text((lx, ly), "Tegnforklaring", font=font(15, True), fill=INK)
    ly += 26
    for k, name in LEGEND:
        ch, col = ICONS[k]
        d.ellipse([lx, ly, lx + 18, ly + 18], fill=(0, 0, 0), outline=col, width=2)
        tw = d.textlength(ch, font=f_small)
        d.text((lx + 9 - tw / 2, ly + 2), ch, font=f_small, fill=col)
        d.text((lx + 26, ly + 2), name, font=f_small, fill=INK)
        ly += 21
    ly += 6
    for kind, (col, name) in LINK_STYLE.items():
        if kind in (SECRET, FALL):
            dashed(d, [(lx, ly + 9), (lx + 18, ly + 9)], col, 3, seg=4)
        else:
            d.line([(lx, ly + 9), (lx + 18, ly + 9)], fill=col, width=4)
        d.text((lx + 26, ly + 2), name, font=f_small, fill=INK)
        ly += 19
    d.line([(lx, ly + 9), (lx + 18, ly + 9)], fill=MAIN, width=3)
    d.text((lx + 26, ly + 2), "hovedveien", font=f_small, fill=INK)
    ly += 22
    d.text((lx, ly), f"gulv fra {lo:+g} m (mørk) til {hi:+g} m (lys)", font=f_small, fill=(190, 180, 160))
    ly += 30
    d.text((lx, ly), "Notater", font=font(15, True), fill=INK)
    ly += 24
    for note in level.notes:
        for line in wrap(note, 46):
            d.text((lx, ly), line, font=f_small, fill=(210, 200, 185))
            ly += 15
        ly += 6

    out.parent.mkdir(parents=True, exist_ok=True)
    img.save(out)
    return out


def wrap(text, n):
    words, lines, cur = text.split(), [], ""
    for w in words:
        if len(cur) + len(w) + 1 > n:
            lines.append(cur)
            cur = w
        else:
            cur = (cur + " " + w).strip()
    if cur:
        lines.append(cur)
    return lines


def dashed(d, pts, col, width, seg=8):
    for p, q in zip(pts, pts[1:]):
        L = math.dist(p, q)
        if L < 1:
            continue
        n = max(1, int(L / seg))
        for i in range(0, n, 2):
            a = i / n
            b = min(1, (i + 1) / n)
            d.line([(p[0] + (q[0] - p[0]) * a, p[1] + (q[1] - p[1]) * a),
                    (p[0] + (q[0] - p[0]) * b, p[1] + (q[1] - p[1]) * b)], fill=col, width=width)


def load(n):
    path = LEVELS / f"level_{n:02}.py"
    spec = importlib.util.spec_from_file_location(f"level_{n:02}", path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod.LEVEL


def main():
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    ns = [int(a) for a in sys.argv[1:]] or sorted(
        int(p.stem.split("_")[1]) for p in LEVELS.glob("level_*.py"))
    for n in ns:
        level = load(n)
        out = draw(level, OUT / f"nivaa_{n:02}.png")
        print(f"nivå {n}: {len(level.rooms)} rom, hovedveien {level.main_length():.0f} m, rundt {level.minutes():.0f} min → {out.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
