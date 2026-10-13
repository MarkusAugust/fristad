"""Felles for portene: et mål med terskel, og rapporten for en ting."""
from dataclasses import dataclass, field, asdict
import json
import os

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
# Det som ikke kan regnes ut om tekstur og modell, og eierens dom.
DATA = os.path.join(ROOT, "assets", "catalog")
OUT = os.path.join(ROOT, "target", "qa")


@dataclass
class Check:
    """Ett mål: hva det heter, verdien, grensen og retningen (`max`: verdien
    skal være høyst grensen, `min`: minst), og en forklaring på norsk."""
    name: str
    value: float
    limit: float
    kind: str  # "max" eller "min"
    why: str

    def __post_init__(self):
        # numpy-tall blir vanlige tall, så rapporten kan lagres som json.
        self.value = float(self.value)
        self.limit = float(self.limit)

    @property
    def ok(self):
        return bool(self.value <= self.limit if self.kind == "max" else self.value >= self.limit)

    def line(self):
        sign = "≤" if self.kind == "max" else "≥"
        mark = "ok " if self.ok else "FEIL"
        return f"  {mark} {self.name}: {self.value:.3f} (krav {sign} {self.limit}) – {self.why}"


@dataclass
class Report:
    gate: str
    item: str
    checks: list = field(default_factory=list)

    @property
    def ok(self):
        return all(c.ok for c in self.checks)

    def add(self, *a):
        self.checks.append(Check(*a))

    def failed(self):
        return [c for c in self.checks if not c.ok]

    def text(self):
        head = f"{self.gate} {self.item}: {'godkjent' if self.ok else 'STOPPET'}"
        return "\n".join([head] + [c.line() for c in self.checks])

    def save(self):
        d = os.path.join(OUT, self.gate)
        os.makedirs(d, exist_ok=True)
        data = {"gate": self.gate, "item": self.item, "ok": self.ok, "checks": [dict(asdict(c), ok=c.ok) for c in self.checks]}
        with open(os.path.join(d, f"{self.item}.json"), "w") as f:
            json.dump(data, f, indent=1, ensure_ascii=False)
