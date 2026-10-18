//! Vannet i krypten (kloakken og cisternen): det står høyt til slusa er
//! dreid, og så synker det. Det er vannet selv som holder deg ute, ikke en
//! usynlig mur (eieren 18. oktober: «man henger seg opp i noen usynlige
//! vegger»): du vader til knærne (`crypt::WADE`), men ikke dypere, og faller
//! du i, drar du deg opp der du sist sto tørt (`Run::rescue`; sjøen i
//! grottene, som aldri synker, kan du dykke i). Siktet sier hva det er.

use super::session::Session;
use crate::domain::world::crypt::WADE;
use crate::ports::driving::{Event, Prompt};
use glam::Vec3;

/// Så lenge mellom hver gang du får høre at vannet er for dypt.
const TELL_AGAIN: f32 = 4.0;

impl Session {
    /// Et skritt (`moved`, fra fysikken) som ville tatt deg dypere ut i
    /// vannet enn du kan vade, blir ikke tatt. Står du allerede dypere (du
    /// falt i), eller faller du, går det som før.
    pub(super) fn wade(&mut self, moved: Vec3, dt: f32, events: &mut Vec<Event>) -> Vec3 {
        self.wade_told += dt;
        // I et fall (hopp eller utfor kanten) går det som før; trinnene ned i en
        // trapp er ikke et fall.
        if self.crypt.waters.is_empty() || !self.in_crypt() || self.body.vel_y < -3.0 {
            return moved;
        }
        let feet = self.body.feet();
        let before = self.mech.depth_at(&self.crypt, feet);
        let after = self.mech.depth_at(&self.crypt, feet + moved);
        if after <= WADE || after <= before + 1e-4 {
            return moved;
        }
        if self.wade_told > TELL_AGAIN {
            events.push(Event::Toast("Vannet er for dypt til å vade i.".into()));
        }
        self.wade_told = 0.0;
        Vec3::new(0.0, moved.y.max(0.0), 0.0)
    }

    /// Hvor dypt føttene står i vann som slusa senker (0 på tørt). Faller du
    /// i det, drar du deg opp der du sist sto tørt; i sjøen i grottene, som
    /// aldri synker, står du på bunnen og kan dykke (nivå 6).
    pub(super) fn flood_depth(&self, feet: Vec3) -> f32 {
        match self.mech.water_at(&self.crypt, feet) {
            Some((n, y)) if self.crypt.waters[n].sinks() => y - feet.y,
            _ => 0.0,
        }
    }

    /// Siktet når du ser ned på vannflaten (innen `reach`, før det første
    /// du ville truffet), mens vannet står høyt.
    pub(super) fn water_prompt(&self, eye: Vec3, dir: Vec3, reach: f32) -> Option<Prompt> {
        if dir.y > -0.05 {
            return None;
        }
        (0..self.crypt.waters.len()).filter(|n| !self.mech.water_down(*n)).find_map(|n| {
            let y = self.mech.water_level(&self.crypt, n);
            let t = (y - eye.y) / dir.y;
            let at = eye + dir * t;
            let on = t > 0.0 && t <= reach && self.mech.water_at(&self.crypt, at - Vec3::Y * 0.01).is_some_and(|(m, _)| m == n);
            let w = &self.crypt.waters[n];
            let info = if !w.sinks() {
                "Svart, stille vann. Bunnen er langt der nede."
            } else if w.high - y > 0.01 {
                "Vannet synker."
            } else {
                "Grumsete vann, for dypt til å vade i. Kanskje noe et sted kan senke det."
            };
            on.then(|| Prompt { label: "Dypt vann".into(), verb: String::new(), trade: String::new(), info: info.into() })
        })
    }
}
