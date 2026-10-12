//! Spillerens kropp: stillingen (stående eller på huk), kapselen, øyet,
//! tyngdekraften, hoppet, skrittene og landingen. Fysikken (porten) flytter
//! kapselen; kroppen sier hvor den vil, og hører hva som skjedde.

use glam::Vec3;

/// Øyet over kapselens sentrum når du står.
pub const EYE: f32 = 0.8;
const RADIUS: f32 = 0.35;
const HALF: f32 = 0.55;
const GRAVITY: f32 = 18.0;
const JUMP: f32 = 5.4;

/// Stående eller på huk: kapselen (halv høyde, radius), øyet over
/// kapselens sentrum, og farten.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Stance {
    #[default]
    Stand,
    Crouch,
}

impl Stance {
    pub fn capsule(self) -> (f32, f32) {
        match self {
            Stance::Stand => (HALF, RADIUS),
            Stance::Crouch => (0.15, RADIUS),
        }
    }

    pub fn eye(self) -> f32 {
        match self {
            Stance::Stand => EYE,
            Stance::Crouch => 0.5,
        }
    }

    pub fn speed(self, sprint: bool) -> f32 {
        match (self, sprint) {
            (Stance::Stand, true) => 7.5,
            (Stance::Stand, false) => 4.2,
            (Stance::Crouch, _) => 2.0,
        }
    }

    /// Skrittlengden (m) og hvor høyt skrittet høres.
    fn stride(self, sprint: bool) -> (f32, f32) {
        match (self, sprint) {
            (Stance::Stand, true) => (1.25, 1.0),
            (Stance::Stand, false) => (0.85, 0.75),
            (Stance::Crouch, _) => (0.6, 0.35),
        }
    }
}

/// Det som skjedde med kroppen i ett bilde, til lydene.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Steps {
    /// Hoppet nå.
    pub jumped: bool,
    /// Landet etter et langt fall: hvor hardt (0..1).
    pub landed: Option<f32>,
    /// Tok et skritt: hvor høyt.
    pub step: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct Body {
    /// Kapselens sentrum.
    pub pos: Vec3,
    pub vel_y: f32,
    pub grounded: bool,
    pub stance: Stance,
    /// Øyet over kapselens sentrum, glir mot stillingens.
    pub eye_h: f32,
    /// Gangen (til hodet som vugger).
    pub bob: f32,
    /// Flyttet uten fysikk dette bildet (ingen landingslyd).
    pub jumped: bool,
    /// Meter gått siden forrige skritt, og den raskeste fallfarten siden
    /// du sist sto.
    stride: f32,
    fall: f32,
}

impl Body {
    /// Stående med føttene i `feet`.
    pub fn new(feet: Vec3) -> Self {
        let mut b = Self {
            pos: Vec3::ZERO,
            vel_y: 0.0,
            grounded: false,
            stance: Stance::Stand,
            eye_h: EYE,
            bob: 0.0,
            jumped: false,
            stride: 0.0,
            fall: 0.0,
        };
        b.pos = feet + Vec3::Y * (b.reach() + 0.05);
        b
    }

    /// Fra kapselens sentrum ned til sålen.
    fn reach(&self) -> f32 {
        let (half, radius) = self.stance.capsule();
        half + radius
    }

    pub fn capsule(&self) -> (f32, f32) {
        self.stance.capsule()
    }

    /// Føttene (litt under sålen, der bakken er).
    pub fn feet(&self) -> Vec3 {
        self.pos - Vec3::Y * (self.reach() + 0.05)
    }

    /// Sålen, uten slingringsmonnet.
    pub fn sole(&self) -> Vec3 {
        self.pos - Vec3::Y * self.reach()
    }

    pub fn eye(&self) -> Vec3 {
        self.pos + Vec3::Y * (self.eye_h + (self.bob * 9.0).sin() * 0.025)
    }

    /// Rett dit, uten fysikk på veien.
    pub fn teleport(&mut self, feet: Vec3) {
        self.pos = feet + Vec3::Y * (self.reach() + 0.05);
        self.vel_y = 0.0;
        self.jumped = true;
    }

    /// Bytt stilling; føttene blir der de er.
    pub fn set_stance(&mut self, stance: Stance, dt: f32) {
        if stance != self.stance {
            let before = self.reach();
            self.stance = stance;
            self.pos.y += self.reach() - before;
        }
        self.eye_h += (self.stance.eye() - self.eye_h) * (dt * 10.0).min(1.0);
    }

    /// Farten opp eller ned dette bildet: tyngden, eller et hopp fra bakken.
    /// Gir om du hoppet.
    pub fn vertical(&mut self, dt: f32, jump: bool) -> bool {
        if self.grounded {
            self.vel_y = -1.0;
            if jump && self.stance == Stance::Stand {
                self.vel_y = JUMP;
                return true;
            }
        } else {
            self.vel_y -= GRAVITY * dt;
        }
        false
    }

    /// Fysikken har flyttet kapselen: hva skjedde (hopp, landing, skritt)?
    pub fn moved(
        &mut self,
        moved: Vec3,
        grounded: bool,
        jumped: bool,
        walking: bool,
        sprint: bool,
    ) -> Steps {
        self.pos += moved;
        let was = self.grounded;
        self.grounded = grounded;
        let mut out = Steps {
            jumped,
            ..Default::default()
        };
        if jumped {
            self.fall = 0.0;
        }
        if !grounded {
            self.fall = self.fall.min(self.vel_y);
        } else if !was && !self.jumped {
            if self.fall < -4.0 {
                out.landed = Some(((-self.fall - 4.0) / 8.0).clamp(0.0, 1.0));
                self.stride = 0.0;
            }
            self.fall = 0.0;
        }
        if grounded && walking {
            self.stride += moved.with_y(0.0).length();
            let (length, gain) = self.stance.stride(sprint);
            if self.stride >= length {
                self.stride = 0.0;
                out.step = Some(gain);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foettene_blir_staende_naar_du_huker_deg() {
        let mut b = Body::new(Vec3::new(1.0, 2.0, 3.0));
        let feet = b.feet();
        b.set_stance(Stance::Crouch, 0.016);
        assert!(b.feet().distance(feet) < 1e-5);
        b.set_stance(Stance::Stand, 0.016);
        assert!(b.feet().distance(feet) < 1e-5);
    }

    #[test]
    fn hopp_bare_staende_paa_bakken() {
        let mut b = Body::new(Vec3::ZERO);
        assert!(!b.vertical(0.016, true), "i lufta");
        b.grounded = true;
        b.set_stance(Stance::Crouch, 0.016);
        assert!(!b.vertical(0.016, true), "på huk");
        b.set_stance(Stance::Stand, 0.016);
        assert!(b.vertical(0.016, true));
        assert_eq!(b.vel_y, JUMP);
    }

    #[test]
    fn langt_fall_gir_landing_og_skritt_kommer_i_takt() {
        let mut b = Body::new(Vec3::ZERO);
        b.vel_y = -10.0;
        b.moved(Vec3::ZERO, false, false, false, false);
        let s = b.moved(Vec3::ZERO, true, false, false, false);
        assert!(s.landed.is_some_and(|h| h > 0.5));
        let steps = (0..100)
            .filter(|_| {
                b.moved(Vec3::X * 0.05, true, false, true, false)
                    .step
                    .is_some()
            })
            .count();
        assert_eq!(steps, 5, "fem meter med skritt på 0,85 m");
    }
}
