"""Sangens lydstyrke som data, til trubadurens kjeve: hvor åpen munnen er,
20 ganger i sekundet, fra et spor i `assets/music/game/`. Kjør:

    python3 tools/music/voice.py assets/music/game/tavern.mp3 assets/music/game/tavern_voice.json

Fila er `{"rate": 20, "values": [0..100, ...]}`: lydstyrken i stemmens
område (`BAND`, der lutten er svakere enn sangen) i desibel, skalert mellom
stillheten og de sterkeste partiene i sporet, med rask åpning og roligere
lukking (`ATTACK`, `RELEASE`). Nettlaget leser verdien der sangen er
(`music.voice()`) og sender den til motoren (`engine.set_voice`), som åpner
kjeven (`Npcs::set_voice`). Trenger ffmpeg og numpy."""

import json
import subprocess
import sys

import numpy as np

RATE = 20  # verdier per sekund
SR = 16000  # prøver per sekund i analysen
BAND = (250.0, 3000.0)  # stemmens område (Hz)
FLOOR, TOP = 25, 97  # persentilene som blir lukket og helt åpen munn
GATE = 0.12  # under dette er munnen lukket
ATTACK, RELEASE = 0.6, 0.25  # andel av veien per verdi når munnen åpner og lukker seg


def envelope(path):
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-ac", "1", "-ar", str(SR), "-f", "f32le", "-"], capture_output=True, check=True).stdout
    x = np.frombuffer(raw, dtype=np.float32)
    hop = SR // RATE
    win = hop * 2
    n = max(0, (len(x) - win) // hop + 1)
    freqs = np.fft.rfftfreq(win, 1.0 / SR)
    band = (freqs >= BAND[0]) & (freqs <= BAND[1])
    w = np.hanning(win).astype(np.float32)
    db = np.empty(n, dtype=np.float32)
    for i in range(n):
        spec = np.abs(np.fft.rfft(x[i * hop:i * hop + win] * w)) ** 2
        db[i] = 10 * np.log10(spec[band].sum() + 1e-9)
    lo, hi = np.percentile(db, FLOOR), np.percentile(db, TOP)
    v = np.clip((db - lo) / max(hi - lo, 1e-3), 0, 1)
    v = np.where(v < GATE, 0.0, (v - GATE) / (1 - GATE))
    out = np.empty_like(v)
    cur = 0.0
    for i, t in enumerate(v):
        cur += (t - cur) * (ATTACK if t > cur else RELEASE)
        out[i] = cur
    return out


if __name__ == "__main__":
    src, dst = sys.argv[1:3]
    v = envelope(src)
    json.dump({"rate": RATE, "values": [int(round(a * 100)) for a in v]}, open(dst, "w"), separators=(",", ":"))
    print(f"{dst}: {len(v)} verdier ({len(v) / RATE:.0f} s), snitt {v.mean():.2f}, åpen (>0,5) {np.mean(v > 0.5) * 100:.0f} %")
