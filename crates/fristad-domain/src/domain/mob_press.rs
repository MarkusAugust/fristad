//! Den fryktinngytende Button (del av `mobs.rs`; modellen fra `mob3d.py`
//! med stilen `dread` og slaget `attack_slam`): tung, langsom og panseret
//! forfra, og når begge nevene slår i bakken foran den og treffer deg, blir
//! du trykket ned og står fast i `PRESS_SECS` (`Mobs::drag` er 0 så lenge).
//! En parade eller et kast unna går fri; blokken demper slaget, men du står
//! fast likevel.

use super::*;

/// Så lenge står du fast etter hammerslaget.
pub const PRESS_SECS: f32 = 1.2;

impl Mobs {
    /// Trykket ned av hammerslaget: står fast i `PRESS_SECS`.
    pub fn press(&mut self) {
        self.pressed = PRESS_SECS;
    }

    /// Står du fast under hammerslaget nå?
    pub fn pressed(&self) -> bool {
        self.pressed > 0.0
    }
}

/// Slaget landet: fra Den fryktinngytende Button trykker det deg ned.
pub(super) fn land(i: usize, m: &Mob, out: &mut Outcome) {
    if m.kind == Kind::DreadButton {
        out.presses.push(i);
    }
}
