//! Lykta i beltet (eieren 13. oktober): lyset ditt nede i krypten. Oljen
//! brenner mens du er nede og fylles opp ved hvilestedene og hos
//! Apotekeren. Når den tar slutt, blafrer lyset, krymper og dør til en
//! glo, og mørket er nesten helt.

/// Sekunder lykta brenner på full olje.
pub const FULL: f32 = 480.0;
/// Under denne delen av oljen begynner lyset å blafre og dempes.
pub const LOW: f32 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lantern {
    /// Sekunder igjen.
    pub oil: f32,
}

impl Default for Lantern {
    fn default() -> Self {
        Self { oil: FULL }
    }
}

impl Lantern {
    pub fn burn(&mut self, dt: f32) {
        self.oil = (self.oil - dt).max(0.0);
    }

    pub fn fill(&mut self) {
        self.oil = FULL;
    }

    /// Hvor mye olje som er igjen, 0..1.
    pub fn fraction(&self) -> f32 {
        (self.oil / FULL).clamp(0.0, 1.0)
    }

    /// Lyset nå: (rekkevidde, styrke) som brøk av en full lykt. Full olje
    /// gir et rolig lys som blafrer litt; under `LOW` krymper det og
    /// sputrer i rykk; tom er en svak glo rett rundt deg.
    pub fn glow(&self, time: f32) -> (f32, f32) {
        let f = self.fraction();
        if f <= 0.0 {
            let ember = 0.9 + 0.1 * (time * 3.1).sin();
            return (0.25, 0.06 * ember);
        }
        let calm = 1.0 - 0.04 * (0.5 + 0.5 * (time * 8.3).sin() * (time * 3.7).cos());
        if f >= LOW {
            return (1.0, calm);
        }
        let k = f / LOW;
        let sputter = if ((time * 5.3).sin() * (time * 2.9 + 1.3).cos()) > 0.55 - 0.5 * (1.0 - k) { 0.45 } else { 1.0 };
        (0.4 + 0.6 * k, (0.25 + 0.75 * k) * calm * sputter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lykta_brenner_ned_og_blir_en_glo() {
        let mut l = Lantern::default();
        assert_eq!(l.glow(0.0).0, 1.0);
        l.burn(FULL * (1.0 - LOW * 0.5));
        let (r, _) = l.glow(0.0);
        assert!(r < 1.0 && r > 0.25, "lav olje krymper lyset: {r}");
        l.burn(FULL);
        let (r, i) = l.glow(0.0);
        assert!(r <= 0.25 && i < 0.1, "tom lykt er en glo: {r} {i}");
        l.fill();
        assert_eq!(l.fraction(), 1.0);
    }
}
