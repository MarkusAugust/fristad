"""Grensesnittet et mål må fylle. `generic.py` er en ferdig standard (humanoid med 18 bein, ingen klipp)."""
import numpy as np

# Leddene riggen legger ut fra silhuettene, som andel av høyden over sålen (stilisert figur, målt på smeden).
# Et mål kan overstyre dem for andre proporsjoner (barn, dverger).
FRAC = dict(ankle=0.05, knee=0.27, hip=0.49, spine03=0.56, spine01=0.68, neck=0.81, head=0.86,
            shoulder=0.80, elbow=0.635, wrist=0.495, finger=0.43)


class Target:
    name = "base"
    frac = FRAC
    material = "body"  # navnet på materialet i glb-fila
    jpeg_quality = 90

    def __init__(self, root=None):
        self.root = root

    # --- retning -----------------------------------------------------------------------------------------
    def to_game(self, p):
        """Blender-punkt (meter, -Y fram, Z opp, sålen på z = 0, midten i origo) -> spillets retning.
        Standard: uendret (front mot -Y)."""
        return np.asarray(p, float)

    # --- skjelettet --------------------------------------------------------------------------------------
    def bones(self, G):
        """Liste av (navn, hode, hale, forelder) fra leddene G (navn -> punkt i spillets rom). Foreldrene må komme
        før barna. Standard: 18 bein (root, hips, spine, chest, neck, head, armer og bein på hver side)."""
        bones = [
            ("root", (0, 0, 0), (0, 0, 0.1), None),
            ("hips", G["pelvis"], G["spine03"], "root"),
            ("spine", G["spine03"], G["spine01"], "hips"),
            ("chest", G["spine01"], G["neck"], "spine"),
            ("neck", G["neck"], G["head"], "chest"),
            ("head", G["head"], G["top"], "neck"),
        ]
        for s in ("L", "R"):
            bones += [
                (f"upper_arm.{s}", G[f"shoulder.{s}"], G[f"elbow.{s}"], "chest"),
                (f"forearm.{s}", G[f"elbow.{s}"], G[f"wrist.{s}"], f"upper_arm.{s}"),
                (f"hand.{s}", G[f"wrist.{s}"], G[f"finger.{s}"], f"forearm.{s}"),
            ]
        for s in ("L", "R"):
            bones += [
                (f"thigh.{s}", G[f"hip.{s}"], G[f"knee.{s}"], "hips"),
                (f"shin.{s}", G[f"knee.{s}"], G[f"ankle.{s}"], f"thigh.{s}"),
                (f"foot.{s}", G[f"ankle.{s}"], G[f"toe.{s}"], f"shin.{s}"),
            ]
        return bones

    def hand_bone(self, side):
        return f"hand.{side}"

    # --- klippene ----------------------------------------------------------------------------------------
    def clips(self, rig, body, opt, tools=()):
        """Legg animasjonene på armaturen (NLA-spor). Gir en dict med det som er verdt å rapportere (fart o.l.),
        eller None når målet ikke har klipp."""
        return None

    # --- etterarbeid -------------------------------------------------------------------------------------
    def post_export(self, path):
        """Kjøres på glb-fila etter eksport (slanking, validering)."""
        return path
