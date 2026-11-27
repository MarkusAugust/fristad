//! Et rutenett over bakken (x og z) til å finne det som ligger ved et punkt
//! uten å lete gjennom alt: reglene i redigeringen og bakkesjekken spør om
//! tusenvis av punkter mot titusenvis av kollidere og gulv.

use glam::Vec2;
use std::collections::HashMap;

/// Sidene i rutene (m).
const CELL: f32 = 4.0;
/// Det som dekker flere ruter enn dette, står i lista som alltid er med.
const WIDE: i32 = 64;

#[derive(Default)]
pub struct Grid {
    cells: HashMap<(i32, i32), Vec<u32>>,
    /// Det som er for stort for rutene, eller ikke har noe omriss.
    everywhere: Vec<u32>,
}

fn cell(v: f32) -> i32 {
    (v / CELL).floor() as i32
}

impl Grid {
    /// Rutenettet over tingene i `spans`: omrisset (laveste og høyeste x og
    /// z) av hver, i rekkefølge. Uten omriss er tingen alltid med.
    pub fn new(spans: impl Iterator<Item = Option<(Vec2, Vec2)>>) -> Self {
        let mut g = Grid::default();
        for (k, span) in spans.enumerate() {
            let k = k as u32;
            let Some((lo, hi)) = span.filter(|(lo, hi)| lo.is_finite() && hi.is_finite()) else {
                g.everywhere.push(k);
                continue;
            };
            let (x0, x1, z0, z1) = (cell(lo.x), cell(hi.x), cell(lo.y), cell(hi.y));
            if (x1 - x0 + 1).saturating_mul(z1 - z0 + 1) > WIDE {
                g.everywhere.push(k);
                continue;
            }
            for x in x0..=x1 {
                for z in z0..=z1 {
                    g.cells.entry((x, z)).or_default().push(k);
                }
            }
        }
        g
    }

    /// Numrene på tingene som kan ligge ved punktet, i stigende rekkefølge.
    pub fn at(&self, x: f32, z: f32) -> impl Iterator<Item = usize> + '_ {
        let here = self.cells.get(&(cell(x), cell(z))).map(Vec::as_slice).unwrap_or_default();
        let (mut a, mut b) = (self.everywhere.iter().peekable(), here.iter().peekable());
        std::iter::from_fn(move || {
            let first = match (a.peek(), b.peek()) {
                (Some(x), Some(y)) => x < y,
                (Some(_), None) => true,
                _ => false,
            };
            if first { a.next() } else { b.next() }.map(|k| *k as usize)
        })
    }
}
