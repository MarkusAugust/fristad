"""Klippebordet: setter åpningsfilmen sammen til én videofil etter
`tools/film/timeline.py` (planen i `docs/film/plan-aaa.md`).

Hver innstilling blir et segment på 1920 × 1080 i 25 bilder per sekund:
Veo-klippet (`assets/film/veo/<navn>.mp4`, trimmet, saktet ned om det er
kortere enn innstillingen, uten Veos egen lyd) eller, til klippet finnes,
maleriet med en langsom glidning (`move`: midten og bredden av utsnittet
først og sist). Segmentene graderes per akt (temperatur, metning,
kontrast, vignett) og settes sammen med overtoninger (xfade). Titlene
tegnes med filmens skrifter og legges over med inn- og uttoning. Lyden er
replikkene fra fortelleren (`assets/film/voice/fortelleren.mp3` klippet
etter `lines.json`), rommene (løkkene i `assets/sounds/`), lydene med
tid (`sfx`) og musikken (`target/film/music.wav` når den er gjengitt).

  python3 tools/film/cut.py [--fast] [--out target/film/intro.mp4]

`--fast` gir 960 × 540 til å se på. Resultatet toner til svart på
slutten (`FADE_OUT`), og spillet står bak.
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, ImageFilter

sys.path.insert(0, str(Path(__file__).parent))
from timeline import SHOTS, GRADES, END, FADE_OUT  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
FILM = ROOT / "assets/film"
SOUNDS = ROOT / "assets/sounds"
FONTS = ROOT / "web/film/fonts"
OUT = ROOT / "target/film"
FPS = 25
CUT = 0.08  # et rett klipp er en overtoning på to bilder, så kjeden er lik hele veien

STYLES = {
    "heading": ("UnifrakturMaguntia-Book.ttf", 118, (243, 213, 140)),
    "title": ("UnifrakturMaguntia-Book.ttf", 170, (243, 213, 140)),
    "latin": ("UncialAntiqua-Regular.ttf", 76, (233, 199, 122)),
    "text": ("UncialAntiqua-Regular.ttf", 56, (246, 236, 214)),
}


def run(args, quiet=True):
    r = subprocess.run(["ffmpeg", "-y", "-hide_banner", "-loglevel", "error" if quiet else "info", *args], capture_output=quiet, text=True)
    if r.returncode:
        print(r.stderr[-4000:] if r.stderr else "", file=sys.stderr)
        raise SystemExit(f"ffmpeg feilet: {' '.join(args)[:300]}")


def probe(path):
    r = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height,duration", "-of", "json", str(path)], capture_output=True, text=True)
    s = json.loads(r.stdout)["streams"][0]
    return int(s["width"]), int(s["height"]), float(s.get("duration") or 0)


def grade_filter(g, W):
    """Graderingen som ffmpeg-filtre."""
    g = GRADES[g]
    temp = 6500 + g["temp"] * 6000
    return f"colortemperature=temperature={temp:.0f}:mix=0.8,eq=contrast={g['con']}:saturation={g['sat']},vignette=angle={0.35 + g['vig'] * 0.45:.2f}:mode=backward"


def still_move(src, move, dur, W, H):
    """Glidningen over et maleri: utsnittet (midten x, y og bredden som brøk)
    går fra `move[0]` til `move[1]` med myk start og slutt."""
    (ax, ay, aw), (bx, by, bw) = move
    im = Image.open(src)
    iw, ih = im.size
    # Bildet skaleres først opp, så utsnittet alltid har nok piksler.
    up = max(1.0, W / (iw * min(aw, bw)))
    sw, sh = round(iw * up), round(ih * up)
    e = f"(1-cos(PI*min(t/{dur:.3f},1)))/2"
    w = f"({aw}+({bw}-{aw})*{e})*{sw}"
    h = f"({w})*9/16"
    x = f"({ax}+({bx}-{ax})*{e})*{sw}-({w})/2"
    y = f"({ay}+({by}-{ay})*{e})*{sh}-({h})/2"
    # Utsnittet holdes inne i bildet.
    x = f"max(0,min({sw}-({w}),{x}))"
    y = f"max(0,min({sh}-({h}),{y}))"
    return f"scale={sw}:{sh}:flags=lanczos,crop=w='{w}':h='{h}':x='{x}':y='{y}',scale={W}:{H}:flags=lanczos"


def segment(shot, extra, W, H, work):
    """Ett segment: innstillingen pluss `extra` sekunder til overtoningen inn i den neste."""
    t0, t1 = shot["t"]
    dur = t1 - t0 + extra
    out = work / f"{shot['name']}.mp4"
    clip = FILM / "veo" / f"{shot.get('clip', '')}.mp4"
    vf_tail = f"{grade_filter(shot['grade'], W)},format=yuv420p,fps={FPS}"
    if shot.get("clip") and clip.exists():
        cw, ch, cd = probe(clip)
        cin = shot.get("clip_in", 0.0)
        avail = max(cd - cin - 0.6, 1.0)  # Veo løser ofte slutten med en overtoning: den siste 0,6 s brukes ikke
        slow = max(1.0, dur / avail)
        vf = f"trim=start={cin}:end={cin + min(avail, dur / slow) + 0.04},setpts=(PTS-STARTPTS)*{slow:.4f},scale={W}:{H}:flags=lanczos,{vf_tail}"
        run(["-i", str(clip), "-an", "-vf", vf, "-t", f"{dur:.3f}", "-c:v", "libx264", "-preset", "fast", "-crf", "16", str(out)])
        source = "klipp" + (f" (saktet {slow:.2f}×)" if slow > 1.01 else "")
    else:
        src = FILM / shot["still"]
        if not src.exists():
            # Maleriet mangler ennå: svart med navnet, så hullet synes.
            run(["-f", "lavfi", "-i", f"color=c=0x101014:s={W}x{H}:r={FPS}", "-vf", f"drawtext=text='{shot['name']}':fontcolor=gray:fontsize=48:x=(w-tw)/2:y=(h-th)/2,format=yuv420p", "-t", f"{dur:.3f}", "-c:v", "libx264", "-preset", "fast", "-crf", "16", str(out)])
            return out, "mangler"
        vf = f"{still_move(src, shot['move'], dur, W, H)},{vf_tail}"
        run(["-loop", "1", "-framerate", str(FPS), "-i", str(src), "-vf", vf, "-t", f"{dur:.3f}", "-c:v", "libx264", "-preset", "fast", "-crf", "16", str(out)])
        source = "stillbilde"
    return out, source


def title_png(style, text, W, work, i):
    font_file, size, color = STYLES[style]
    size = round(size * W / 1920)
    font = ImageFont.truetype(str(FONTS / font_file), size)
    pad = size
    tw = int(font.getlength(text))
    im = Image.new("RGBA", (tw + pad * 2, size * 2 + pad), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.text((pad, pad // 2), text, font=font, fill=(10, 6, 2, 230))
    shadow = im.filter(ImageFilter.GaussianBlur(size * 0.12))
    d = ImageDraw.Draw(shadow)
    d.text((pad, pad // 2), text, font=font, fill=color + (255,))
    p = work / f"title_{i}.png"
    shadow.save(p)
    return p, shadow.size


def build(fast, out_path):
    W, H = (960, 540) if fast else (1920, 1080)
    work = OUT / ("work_fast" if fast else "work")
    work.mkdir(parents=True, exist_ok=True)
    # Segmentene, hvert forlenget med overtoningen inn i det neste.
    segs = []
    for i, shot in enumerate(SHOTS):
        nxt = SHOTS[i + 1]["enter"] if i + 1 < len(SHOTS) else "cut"
        extra = CUT if nxt == "cut" else nxt[1]
        path, source = segment(shot, extra, W, H, work)
        print(f"  {shot['name']:22s} {shot['t'][0]:6.1f}–{shot['t'][1]:6.1f}  {source}")
        segs.append(path)

    # Videoen: xfade-kjeden, titlene over, og uttoningen til slutt.
    inputs = []
    for p in segs:
        inputs += ["-i", str(p)]
    f = []
    for i in range(len(segs)):
        f.append(f"[{i}:v]settb=AVTB,setpts=PTS-STARTPTS[s{i}]")
    last = "[s0]"
    offset = 0.0
    for i, shot in enumerate(SHOTS[1:], 1):
        prev_d = SHOTS[i - 1]["t"][1] - SHOTS[i - 1]["t"][0]
        enter = shot["enter"]
        offset += prev_d
        kind, d = ("fade", CUT) if enter == "cut" else ({"mix": "fade", "black": "fadeblack", "white": "fadewhite"}[enter[0]], enter[1])
        f.append(f"{last}[s{i}]xfade=transition={kind}:duration={d}:offset={offset:.3f}[v{i}]")
        last = f"[v{i}]"
    n = len(segs)
    titles = [(st, tx, a, b) for shot in SHOTS for (st, tx, a, b) in shot.get("titles", [])]
    for k, (st, tx, a, b) in enumerate(titles):
        p, (tw, th) = title_png(st, tx, W, work, k)
        inputs += ["-loop", "1", "-framerate", str(FPS), "-t", f"{b - a + 0.1:.3f}", "-i", str(p)]
        idx = n + k
        fade_in, fade_out = 0.9, 0.9
        y = "(H-h)*0.88" if st == "latin" else "(H-h)*0.2" if st == "title" else "(H-h)*0.14"
        f.append(f"[{idx}:v]format=rgba,fade=t=in:st=0:d={fade_in}:alpha=1,fade=t=out:st={b - a - fade_out:.3f}:d={fade_out}:alpha=1,setpts=PTS-STARTPTS+{a:.3f}/TB[t{k}]")
        f.append(f"{last}[t{k}]overlay=x=(W-w)/2:y={y}:eof_action=pass[u{k}]")
        last = f"[u{k}]"
    f.append(f"{last}fade=t=out:st={END - FADE_OUT:.3f}:d={FADE_OUT},trim=end={END:.3f},setpts=PTS-STARTPTS[vout]")

    # Lyden.
    voice = FILM / "voice/fortelleren.mp3"
    lines = json.loads((FILM / "voice/lines.json").read_text()) if (FILM / "voice/lines.json").exists() else []
    a_in = []
    mixes = []
    ai = len(inputs) // 2  # neste inngangsindeks regnes under
    def add_input(args):
        nonlocal inputs
        inputs += args
        return sum(1 for x in inputs if x == "-i") - 1
    if voice.exists() and lines:
        vi = add_input(["-i", str(voice)])
        for shot in SHOTS:
            for (k, at) in shot.get("voice", []):
                ln = lines[k]
                tag = f"vo{len(mixes)}"
                f.append(f"[{vi}:a]atrim=start={ln['start']}:end={ln['end'] + 0.25},asetpts=PTS-STARTPTS,afade=t=in:d=0.05,afade=t=out:st={ln['end'] - ln['start']:.3f}:d=0.25,adelay={int(at * 1000)}|{int(at * 1000)},volume=1.0[{tag}]")
                mixes.append(tag)
    manifest = json.loads((SOUNDS / "sounds.json").read_text())
    for shot in SHOTS:
        t0, t1 = shot["t"]
        for name in shot.get("amb", []):
            files = manifest.get(name, {}).get("files") or []
            if not files:
                continue
            si = add_input(["-stream_loop", "-1", "-i", str(SOUNDS / files[0])])
            gain = manifest[name].get("gain", 0.5) * 0.5
            d = t1 - t0 + 1.5
            tag = f"amb{len(mixes)}"
            f.append(f"[{si}:a]atrim=end={d:.3f},asetpts=PTS-STARTPTS,afade=t=in:d=1.2,afade=t=out:st={d - 1.5:.3f}:d=1.5,volume={gain:.3f},adelay={int(max(t0 - 0.6, 0) * 1000)}|{int(max(t0 - 0.6, 0) * 1000)}[{tag}]")
            mixes.append(tag)
        for name, at in shot.get("sfx", []):
            files = manifest.get(name, {}).get("files") or []
            if not files:
                continue
            si = add_input(["-i", str(SOUNDS / files[0])])
            tag = f"sfx{len(mixes)}"
            f.append(f"[{si}:a]volume={manifest[name].get('gain', 0.7) * 0.8:.3f},adelay={int(at * 1000)}|{int(at * 1000)}[{tag}]")
            mixes.append(tag)
    music = OUT / "music.wav"
    if music.exists():
        mi = add_input(["-i", str(music)])
        f.append(f"[{mi}:a]volume=0.55,afade=t=out:st={END - FADE_OUT - 1:.3f}:d={FADE_OUT + 1}[mus]")
        mixes.append("mus")
    if mixes:
        f.append("".join(f"[{m}]" for m in mixes) + f"amix=inputs={len(mixes)}:normalize=0:dropout_transition=0,alimiter=limit=0.95,atrim=end={END:.3f}[aout]")
        amap = ["-map", "[aout]", "-c:a", "aac", "-b:a", "192k"]
    else:
        amap = []

    graph = work / "graph.txt"
    graph.write_text(";\n".join(f))
    out_path.parent.mkdir(parents=True, exist_ok=True)
    run([*inputs, "-filter_complex_script", str(graph), "-map", "[vout]", *amap, "-c:v", "libx264", "-preset", "medium", "-crf", "21" if fast else "19", "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-t", f"{END:.3f}", str(out_path)], quiet=True)
    print(out_path)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--out", default=None)
    a = ap.parse_args()
    build(a.fast, Path(a.out) if a.out else OUT / ("intro_fast.mp4" if a.fast else "intro.mp4"))
