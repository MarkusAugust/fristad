//! Rutenettet (`world::Grid`) finner alt som ligger ved et punkt, i samme
//! rekkefølge som å lete gjennom alt.

use crate::domain::rng::Rng;
use crate::domain::world::Grid;
use glam::Vec2;

#[test]
fn rutenettet_finner_det_samme_som_aa_lete_gjennom_alt() {
    let mut rng = Rng(7);
    let mut f = |lo: f32, hi: f32| rng.range(lo, hi);
    // Små og store omriss, noen over mange ruter, og noen uten omriss.
    let spans: Vec<Option<(Vec2, Vec2)>> = (0..400)
        .map(|k| {
            let c = Vec2::new(f(-60.0, 60.0), f(-60.0, 60.0));
            let h = Vec2::new(f(0.1, if k % 9 == 0 { 50.0 } else { 3.0 }), f(0.1, 6.0));
            (k % 23 != 0).then_some((c - h, c + h))
        })
        .collect();
    let grid = Grid::new(spans.iter().copied());
    for _ in 0..2000 {
        let (x, z) = (f(-70.0, 70.0), f(-70.0, 70.0));
        let found: Vec<usize> = grid.at(x, z).collect();
        assert!(found.windows(2).all(|w| w[0] < w[1]), "i stigende rekkefølge: {found:?}");
        for (k, span) in spans.iter().enumerate() {
            let inside = span.is_none_or(|(lo, hi)| x >= lo.x && x <= hi.x && z >= lo.y && z <= hi.y);
            assert!(!inside || found.contains(&k), "ting {k} ligger ved ({x}, {z}), men ble ikke funnet");
        }
    }
}
