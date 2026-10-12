//! Spørringene mot krypten: nivået og rommet et punkt er i, bryterne, lyset
//! som følger rommene, og resten `Crypt` svarer på underveis i spillet.

use super::*;

impl Crypt {
    /// Føttene og retningen der du kommer ned fra krypten i byen.
    pub fn start(&self) -> Option<(Vec3, f32)> {
        self.levels.first().map(|l| l.start)
    }

    /// Nivået et punkt er i, etter høyden.
    pub fn level_at(&self, p: Vec3) -> Option<usize> {
        self.levels.iter().position(|l| p.y >= l.min_y - 8.0 && p.y <= l.max_y + 4.0)
    }

    /// Bryteren med denne id-en i nivået.
    pub fn switch(&self, level: usize, id: &str) -> Option<usize> {
        self.switches.iter().position(|s| s.level == level && s.id == id)
    }

    fn open_poly<'a>(&'a self) -> impl Fn(&Poly) -> bool + 'a {
        move |p: &Poly| p.gate.is_none_or(|g| self.open.get(g).copied().unwrap_or(true))
    }

    /// Polygonen under føttene (åpen eller ikke).
    fn locate(&self, p: Vec3) -> Option<(usize, u32)> {
        let l = self.level_at(p)?;
        Some((l, self.levels[l].nav.locate(p, 0.6)?))
    }

    /// Om et punkt står på nettet (et åpent gulv der fiendene kan gå),
    /// innen en meter over eller under gulvet.
    pub fn walkable(&self, p: Vec3) -> bool {
        let Some((l, i)) = self.locate(p) else {
            return false;
        };
        let nav = &self.levels[l].nav;
        (self.open_poly())(&nav.polys[i as usize]) && (nav.height_in(i, Vec2::new(p.x, p.z)) - p.y).abs() < 1.0
    }

    /// Gulvhøyden under et punkt (absolutt y); punktets y utenfor nettet.
    pub fn height_at(&self, p: Vec3) -> f32 {
        match self.locate(p) {
            Some((l, i)) => self.levels[l].nav.height_in(i, Vec2::new(p.x, p.z)),
            None => p.y,
        }
    }

    /// Gulvet på nettet under et punkt (også bak stengte porter), om noe.
    pub fn floor_under(&self, p: Vec3) -> Option<f32> {
        let (l, i) = self.locate(p)?;
        Some(self.levels[l].nav.height_in(i, Vec2::new(p.x, p.z)))
    }

    /// Nærmeste punkt på det åpne nettet innen `reach` meter.
    pub fn snap(&self, p: Vec3, reach: f32) -> Option<Vec3> {
        let l = self.level_at(p)?;
        self.levels[l].nav.nearest(p, reach, &self.open_poly()).map(|(_, c)| c)
    }

    /// Veien fra `from` til `to` gjennom åpne polygoner: hjørnene etter
    /// startpunktet, det siste er målet.
    pub fn path(&self, from: Vec3, to: Vec3) -> Option<Vec<Vec3>> {
        let l = self.level_at(from)?;
        if self.level_at(to) != Some(l) {
            return None;
        }
        self.levels[l].nav.path(from, to, &self.open_poly())
    }

    /// Neste hjørne på veien.
    pub fn next_step(&self, from: Vec3, to: Vec3) -> Option<Vec3> {
        self.path(from, to)?.into_iter().find(|c| c.with_y(0.0).distance(from.with_y(0.0)) > 0.05)
    }

    /// Rommet et punkt står i (det minste som rommer det), i nivået.
    pub fn room_at(&self, p: Vec3) -> Option<(usize, usize)> {
        let l = self.level_at(p)?;
        let q = Vec2::new(p.x, p.z);
        self.levels[l].rooms.iter().enumerate().filter(|(_, r)| p.y >= r.y - 1.0 && p.y <= r.y + r.h && r.contains(q)).min_by(|a, b| a.1.area.total_cmp(&b.1.area)).map(|(i, _)| (l, i))
    }

    /// Rommene et lys i rom `room` lyser i: sitt eget, de med åpen
    /// forbindelse (en dør, en åpen port) og de som overlapper (galleriet
    /// over salen). Én bit per rom i nivået.
    pub fn light_mask(&self, level: usize, room: usize) -> u64 {
        let Some(l) = self.levels.get(level) else {
            return u64::MAX;
        };
        let bit = |r: usize| if r < 64 { 1u64 << r } else { 0 };
        let mut m = bit(room);
        for &(a, b, sw) in &l.links {
            let open = sw.is_none_or(|s| self.open.get(s).copied().unwrap_or(true));
            if open && (a == room || b == room) {
                m |= bit(a) | bit(b);
            }
        }
        for &(a, b) in &l.overlaps {
            if a == room || b == room {
                m |= bit(a) | bit(b);
            }
        }
        m
    }
}
