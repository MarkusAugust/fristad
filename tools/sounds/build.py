"""Lydbiblioteket: henter opptakene fra frie spill og lager assets/sounds/.

    python3 tools/sounds/build.py [--src target/sound-src]

Kildene klones fra GitHub (bare lydmappene):
- 0 A.D. (Wildfire Games), CC BY-SA 3.0
- Flare (flareteam), CC BY-SA 3.0
- Minetest Game (luanti-org), lisens per mod (CC0, CC BY 3.0, CC BY-SA 3.0)

Hver lyd blir mono mp3 (44,1 kHz), stillhet foran kuttes, og toppen
normaliseres til −1 dBFS; nivået i spillet settes med `gain` i manifestet.
Løkkene kuttes til en bit og krysstones over sømmen, så de går rundt uten
klikk. `sounds.json` sier hvilke filer hvert navn har, og om nettlaget også
skal spille den syntetiske lyden med samme navn. `CREDITS.md` krediterer
hver fil.
"""

import argparse
import glob
import json
import os
import subprocess
import sys

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "assets", "sounds")
RATE = 44100

A0 = "0ad/binaries/data/mods/public/audio"
FL = "flare-game/mods/fantasycore/soundfx"
MT = "minetest_game/mods"

REPOS = {
    "0ad": ("https://github.com/0ad/0ad", [A0]),
    "flare-game": ("https://github.com/flareteam/flare-game", None),
    "minetest_game": ("https://github.com/luanti-org/minetest_game", None),
}

LICENSES = {
    "0ad": "CC BY-SA 3.0, Wildfire Games (https://play0ad.com), https://creativecommons.org/licenses/by-sa/3.0/",
    "flare": "CC BY-SA 3.0, Flare-bidragsyterne (https://github.com/flareteam/flare-game/wiki/Credits), https://creativecommons.org/licenses/by-sa/3.0/",
}

# Minetest Game: lisensen står per fil i mods/<mod>/license.txt.
MT_LICENSES = {
    "default_dirt_footstep": "CC BY-SA 3.0, Mito551 (Minetest Game)",
    "default_snow_footstep": "CC0 1.0, Ryding (Minetest Game)",
    "default_water_footstep": "CC BY 3.0, AGFX (Minetest Game)",
    "default_chest_open": "sofar (Minetest Game), blandet av Sevin7 og Yoyodaman234 (CC0) og Percy Duke, kingsamas og bulbastre (CC BY 3.0, Freesound)",
    "doors_door_open": "CC BY 3.0, CGEffex, endret av BlockMen (Minetest Game)",
    "doors_door_close": "CC BY 3.0, bennstir (Minetest Game)",
    "doors_steel_door_close": "CC BY 3.0, HazMatt (Minetest Game, freesound.org/people/HazMattt/sounds/187283)",
    "xpanes_steel_bar_door_open": "CC BY-SA 3.0, TumeniNodes (Minetest Game)",
    "env_sounds_lava": "CC0 1.0, Yuval (Minetest Game)",
    "env_sounds_water": "CC0 1.0, Yuval (Minetest Game)",
    "fire_small": "CC BY 3.0, dobroide (Minetest Game, freesound.org/people/dobroide/sounds/4211)",
}


def S(*paths, gain=1.0, pitch=0.06, rate=1.0, synth=None, loop=None, start=0.0, max_len=None, cut=True):
    return dict(paths=list(paths), gain=gain, pitch=pitch, rate=rate, synth=synth, loop=loop, start=start, max_len=max_len, cut=cut)


# Navnet spillet bruker → kildene (globber), nivå, variasjon i tonehøyde,
# og om den syntetiske lyden med samme navn i web/sounds.js spilles i tillegg.
SOUNDS = {
    # Fottrinn etter underlaget.
    "step_grass": S(f"{A0}/actor/singlesteps/fs_grass[1-8].ogg", gain=0.45, pitch=0.08),
    "step_gravel": S(f"{A0}/actor/singlesteps/fs_gravel[1-8].ogg", gain=0.45, pitch=0.08),
    "step_rock": S(f"{A0}/actor/singlesteps/fs_rock[1-8].ogg", gain=0.42, pitch=0.08),
    "step_wood": S(f"{A0}/actor/singlesteps/fs_wood[1-8].ogg", gain=0.45, pitch=0.08),
    "step_dirt": S(f"{MT}/default/sounds/default_dirt_footstep.*.ogg", f"{A0}/actor/human/movement/hstep_dirt_MN_1*.ogg", gain=0.45, pitch=0.08),
    "step_snow": S(f"{MT}/default/sounds/default_snow_footstep.*.ogg", gain=0.45, pitch=0.08),
    "step_water": S(f"{MT}/default/sounds/default_water_footstep.*.ogg", gain=0.5, pitch=0.08),
    # Rustningen som rasler i takt med skrittene.
    "gear_mail": S(f"{FL}/steps/step_metal[1-4].ogg", gain=0.16, pitch=0.1),
    "gear_leather": S(f"{FL}/steps/step_leather[1-4].ogg", gain=0.14, pitch=0.1),
    "land": S(f"{A0}/actor/singlesteps/fs_gravel[1-4].ogg", gain=0.8, pitch=0.05, synth="land"),
    "jump": S(f"{FL}/steps/step_cloth[1-4].ogg", gain=0.35, pitch=0.08),
    # Kampen.
    "swing": S(f"{FL}/melee_attack.ogg", f"{FL}/melee_attack_2.ogg", f"{FL}/melee_attack_3.ogg", gain=0.6, pitch=0.1),
    "swing_heavy": S(f"{FL}/melee_attack.ogg", f"{FL}/melee_attack_2.ogg", f"{FL}/melee_attack_3.ogg", gain=0.85, pitch=0.05, rate=0.72),
    "hit": S(f"{A0}/attack/impact/fleshimp_1*.ogg", f"{A0}/attack/impact/fleshstab_0[12].ogg", gain=0.7, pitch=0.1, synth="hit"),
    "hit_heavy": S(f"{A0}/attack/impact/fleshstab_0[12].ogg", f"{A0}/attack/impact/fleshimp_1*.ogg", gain=0.9, pitch=0.06, synth="hit_heavy"),
    "hit_wood": S(f"{A0}/attack/impact/shield_wood_1*.ogg", gain=0.7, pitch=0.08),
    "hit_metal": S(f"{A0}/attack/weapon/swordhit_1[0-9].ogg", gain=0.55, pitch=0.06),
    "clang": S(f"{A0}/attack/impact/shield_metal_1*.ogg", gain=0.6, pitch=0.06),
    "kill": S(f"{FL}/enemies/skeleton_die.ogg", f"{FL}/enemies/skeleton_critdie.ogg", f"{FL}/enemies/grave_die.ogg", gain=0.6, pitch=0.1, synth="kill"),
    "block": S(f"{A0}/attack/impact/shield_wood_1*.ogg", gain=0.85, pitch=0.06),
    "block_metal": S(f"{A0}/attack/impact/shield_metal_1*.ogg", gain=0.75, pitch=0.06),
    "parry": S(f"{A0}/attack/weapon/swordhit_1[0-9]a.ogg", f"{A0}/attack/weapon/swordhit_1[0-9]b.ogg", gain=0.8, pitch=0.04, synth="parry"),
    "hurt": S(f"{FL}/male_hit.ogg", gain=0.65, pitch=0.08, synth="thump"),
    "death": S(f"{A0}/actor/human/death/male_death_0[1-8].ogg", gain=0.7, pitch=0.03, synth="death"),
    "dodge": S(f"{FL}/steps/step_cloth[1-4].ogg", gain=0.4, pitch=0.08, synth="dodge"),
    # Fiendene.
    "windup": S(f"{FL}/enemies/goblin_phys.ogg", f"{FL}/enemies/antlion_phys.ogg", f"{FL}/enemies/zombie_phys.ogg", gain=0.35, pitch=0.12, synth="growl"),
    "windup_big": S(f"{FL}/enemies/minotaur_phys.ogg", gain=0.55, pitch=0.06, synth="growl"),
    "notice": S(f"{FL}/enemies/wyvern_ment.ogg", f"{FL}/enemies/goblin_ment.ogg", f"{FL}/enemies/antlion_ment.ogg", gain=0.4, pitch=0.12),
    "spit": S(f"{A0}/attack/weapon/jav_attack_0*.ogg", gain=0.5, pitch=0.1, synth="spit"),
    "slam": S(f"{A0}/attack/impact/siegeprojectilehit_0*.ogg", gain=0.9, pitch=0.05, synth="slam"),
    "roar": S(f"{FL}/enemies/minotaur_hit.ogg", gain=0.8, pitch=0.03, synth="roar"),
    "boss_step": S(f"{A0}/attack/impact/siegeprojectilehit_0[34].ogg", gain=0.35, pitch=0.08, max_len=0.9),
    # Kjelleren.
    "gate": S(f"{A0}/actor/gate/stonegate_close_2*.ogg", gain=0.8, pitch=0.03),
    "locked": S(f"{MT}/doors/sounds/doors_steel_door_close.ogg", gain=0.6, pitch=0.05),
    "key": S(f"{FL}/inventory/inventory_metal.ogg", gain=0.6, pitch=0.08),
    "bolt": S(f"{MT}/xpanes/sounds/xpanes_steel_bar_door_open.ogg", gain=0.6, pitch=0.04),
    "door": S(f"{MT}/doors/sounds/doors_door_open.ogg", f"{FL}/door_open.ogg", f"{FL}/wood_open.ogg", gain=0.7, pitch=0.05, synth="door"),
    "door_close": S(f"{MT}/doors/sounds/doors_door_close.ogg", gain=0.7, pitch=0.05),
    "crypt_door": S(f"{FL}/environment/stone_open.ogg", gain=0.7, pitch=0.03, synth="door"),
    "chest": S(f"{MT}/default/sounds/default_chest_open.ogg", f"{FL}/wood_open.ogg", gain=0.6, pitch=0.05, synth="chest"),
    "loot": S(f"{FL}/flying_loot.ogg", gain=0.5, pitch=0.06),
    # Byen, handel og lageret.
    "coins": S(f"{FL}/inventory/inventory_coins.ogg", gain=0.6, pitch=0.08),
    "pickup": S(f"{FL}/inventory/inventory_coins.ogg", gain=0.35, pitch=0.12, synth="pickup"),
    "deny": S(f"{FL}/inventory/inventory_wood.ogg", gain=0.4, pitch=0.04),
    "craft": S(f"{A0}/resource/mining/mine_metal_1*.ogg", gain=0.7, pitch=0.05),
    "book": S(f"{FL}/inventory/inventory_book.ogg", gain=0.6, pitch=0.05),
    "page": S(f"{FL}/inventory/inventory_page.ogg", gain=0.5, pitch=0.1),
    "heal": S(f"{FL}/powers/heal.ogg", gain=0.45, pitch=0.03),
    "bag": S(f"{FL}/inventory/inventory_leather.ogg", gain=0.55, pitch=0.06),
    "equip_metal": S(f"{FL}/inventory/inventory_metal.ogg", f"{FL}/inventory/inventory_maille.ogg", gain=0.6, pitch=0.06),
    "equip_wood": S(f"{FL}/inventory/inventory_wood.ogg", gain=0.6, pitch=0.06),
    "equip_leather": S(f"{FL}/inventory/inventory_leather.ogg", f"{FL}/inventory/inventory_cloth.ogg", gain=0.55, pitch=0.06),
    "click": S(f"{FL}/widget_activate.ogg", gain=0.35, pitch=0.05),
    "heartbeat": S(f"{FL}/heartbeat.ogg", gain=0.55, pitch=0.0),
    "thunder": S(f"{A0}/ambient/weather/thunder_1*.ogg", gain=0.75, pitch=0.05),
    "talk": S(gain=1.0, synth="talk"),
    "well": S(f"{MT}/env_sounds/sounds/env_sounds_water.1.ogg", gain=0.25, pitch=0.0, synth="well", max_len=1.2, start=0.5),
    # Løkker (spilles av lydbildet etter hvor du står).
    "loop_rain": S(f"{A0}/ambient/weather/rain_11.ogg", gain=0.5, loop=16.0, start=20.0),
    "loop_storm": S(f"{A0}/ambient/weather/windstorm_11.ogg", gain=0.45, loop=16.0, start=20.0),
    "loop_wind": S(f"{A0}/ambient/weather/wind_11.ogg", gain=0.35, loop=16.0, start=30.0),
    "loop_market": S(f"{A0}/ambient/building/amb_trade_1[0-2].ogg", gain=0.5, loop=18.0, start=1.0),
    "loop_birds": S(f"{FL}/environment/bird_twitter_loop.ogg", gain=0.35, loop=9.0),
    "loop_owl": S(f"{FL}/environment/forest_owl_loop.ogg", gain=0.3, loop=11.0),
    "loop_forge": S(f"{FL}/environment/anvil_loop.ogg", gain=0.45, loop=22.0),
    "loop_fire": S(f"{MT}/fire/sounds/fire_small.ogg", f"{FL}/environment/open_fire_loop.ogg", gain=0.4, loop=6.0),
    "loop_drips": S(f"{FL}/environment/cave_droplets_loop.ogg", gain=0.4, loop=10.0),
    "loop_cave": S(f"{FL}/environment/cave_wind_loop.ogg", gain=0.4, loop=20.0),
    "loop_water": S(f"{MT}/env_sounds/sounds/env_sounds_water.*.ogg", gain=0.5, loop=6.0),
    "loop_lava": S(f"{MT}/env_sounds/sounds/env_sounds_lava.*.ogg", gain=0.5, loop=6.0),
    "sheep": S(f"{A0}/actor/fauna/animal/sheep_1*.ogg", gain=0.4, pitch=0.08),
    "cow": S(f"{A0}/actor/fauna/animal/cattle_0*.ogg", gain=0.45, pitch=0.06),
}


def fetch(src):
    os.makedirs(src, exist_ok=True)
    env = dict(os.environ, GIT_LFS_SKIP_SMUDGE="1")
    for name, (url, sparse) in REPOS.items():
        d = os.path.join(src, name)
        if os.path.isdir(os.path.join(d, ".git")):
            continue
        if sparse:
            subprocess.run(["git", "clone", "-q", "--depth", "1", "--filter=blob:none", "--no-checkout", url, d], check=True, env=env)
            subprocess.run(["git", "-C", d, "sparse-checkout", "set", "--no-cone", *sparse], check=True, env=env)
            subprocess.run(["git", "-C", d, "checkout", "-q"], check=True, env=env)
        else:
            subprocess.run(["git", "clone", "-q", "--depth", "1", url, d], check=True, env=env)


def decode(path):
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-ac", "1", "-ar", str(RATE), "-f", "f32le", "-"], check=True, capture_output=True).stdout
    return np.frombuffer(raw, dtype=np.float32).copy()


def encode(x, path):
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "f32le", "-ar", str(RATE), "-ac", "1", "-i", "-", "-codec:a", "libmp3lame", "-b:a", "80k", path], input=x.astype(np.float32).tobytes(), check=True)


def trim(x, spec):
    if spec["start"]:
        x = x[int(spec["start"] * RATE):]
    if spec["cut"] and not spec["loop"]:
        # Stillhet foran (under −50 dB) kuttes, med 5 ms før lyden.
        on = np.nonzero(np.abs(x) > 10 ** (-50 / 20))[0]
        if len(on):
            x = x[max(0, on[0] - int(0.005 * RATE)):]
    if spec["max_len"]:
        n = int(spec["max_len"] * RATE)
        if len(x) > n:
            fade = int(0.08 * RATE)
            x = x[:n].copy()
            x[-fade:] *= np.linspace(1, 0, fade)
    return x


def make_loop(x, seconds):
    """En bit på `seconds` sekunder der slutten krysstones inn i starten."""
    n = int(seconds * RATE)
    fade = int(min(1.5, seconds * 0.2) * RATE)
    if len(x) < n + fade:
        # Kort kilde: gjentas til den er lang nok.
        x = np.tile(x, int(np.ceil((n + fade) / max(1, len(x)))) + 1)
    body = x[:n].copy()
    tail = x[n:n + fade]
    w = np.linspace(0, 1, fade)
    # Likeeffekt-krysstoning: halen tones ut over starten, starten inn.
    body[:fade] = body[:fade] * np.sqrt(w) + tail * np.sqrt(1 - w)
    return body


def normalize(x, peak_db=-1.0):
    m = np.max(np.abs(x)) if len(x) else 0
    if m < 1e-6:
        return x
    return x * (10 ** (peak_db / 20) / m)


def credit(rel):
    if rel.startswith("0ad/"):
        return LICENSES["0ad"]
    if rel.startswith("flare-game/"):
        return LICENSES["flare"]
    base = os.path.basename(rel).split(".")[0]
    return MT_LICENSES.get(base, "CC BY-SA 3.0, Minetest Game") + ", https://github.com/luanti-org/minetest_game"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", default=os.path.join(ROOT, "target", "sound-src"))
    args = ap.parse_args()
    fetch(args.src)
    os.makedirs(OUT, exist_ok=True)
    for f in glob.glob(os.path.join(OUT, "*.mp3")):
        os.remove(f)
    manifest = {}
    credits = []
    total = 0
    done = {}
    for name, spec in SOUNDS.items():
        sources = []
        for pat in spec["paths"]:
            hits = sorted(glob.glob(os.path.join(args.src, pat)))
            if not hits:
                sys.exit(f"{name}: fant ingen filer for {pat}")
            sources += hits
        files = []
        if spec["loop"]:
            # Løkka lages av den første kilden (de andre er reserve).
            x = make_loop(trim(decode(sources[0]), spec), spec["loop"])
            out = f"{name}.mp3"
            encode(normalize(x), os.path.join(OUT, out))
            files.append(out)
            credits.append((out, os.path.relpath(sources[0], args.src), credit(os.path.relpath(sources[0], args.src))))
        else:
            for i, s in enumerate(sources):
                # Samme kilde, kuttet likt, blir samme fil.
                key = (s, spec["start"], spec["max_len"], spec["cut"])
                if key in done:
                    files.append(done[key])
                    continue
                x = normalize(trim(decode(s), spec))
                out = f"{name}_{i + 1}.mp3"
                encode(x, os.path.join(OUT, out))
                total += os.path.getsize(os.path.join(OUT, out))
                done[key] = out
                files.append(out)
                rel = os.path.relpath(s, args.src)
                credits.append((out, rel, credit(rel)))
        if spec["loop"]:
            total += os.path.getsize(os.path.join(OUT, files[0]))
        entry = {"files": files, "gain": spec["gain"], "pitch": spec["pitch"]}
        if spec["rate"] != 1.0:
            entry["rate"] = spec["rate"]
        if spec["synth"]:
            entry["synth"] = spec["synth"]
        if spec["loop"]:
            entry["loop"] = True
        manifest[name] = entry
        print(f"{name}: {len(files)} filer")
    with open(os.path.join(OUT, "sounds.json"), "w") as f:
        json.dump(manifest, f, indent=1, ensure_ascii=False)
        f.write("\n")
    with open(os.path.join(OUT, "CREDITS.md"), "w") as f:
        f.write("# Lydene\n\nOpptakene er hentet fra frie spill av `tools/sounds/build.py`, gjort om til mono mp3,\n")
        f.write("kuttet og normalisert (løkkene er krysstonet over sømmen). Lisensene følger filene; CC BY-SA-filene\n")
        f.write("(og endringene i dem) deles under samme lisens.\n\n| Fil | Kilde | Lisens og opphav |\n|---|---|---|\n")
        for out, rel, lic in credits:
            f.write(f"| `{out}` | `{rel}` | {lic} |\n")
    print(f"{len(manifest)} lyder, {total / 1e6:.1f} MB")


if __name__ == "__main__":
    main()
