//! Landskapet rundt byen: åsene utenfor terrenget, steinene på enga,
//! bjørkeklyngene langs veiene og kanten av verden. De andre trærne,
//! buskene og gresset kommer med de nye modellene (`docs/ny-alt.md`).

use super::{terrain, town, Builder, Shape};
use crate::domain::geometry::{Layer, Mat};
use crate::domain::rng::Rng;
use glam::{Affine3A, Vec2, Vec3};

/// Steinene utenfor muren (modellene fra TRELLIS, `item3d.py`): gruppene
/// på fem spredt på enga, og de store steinene ved veiene og på åsene.
pub const ROCK_GROUPS: usize = 40;
pub const ROCKS_BIG: usize = 14;
/// Hvor langt ned i bakken steinene står (den flate bunnen skjules).
const ROCK_SINK: f32 = 0.05;

/// Setter ut steinene, like hver gang (eget frø): aldri på veiene og
/// stiene, på tunene, kirkegården, ved portene, innenfor muren eller nær
/// noe som alt står der; loddrett på det laveste punktet under foten der
/// bakken er nesten flat, dreid og litt skalert hver for seg, med
/// kolliderne som følger formen (`extras.solids`, `Builder::place_solids`).
pub fn rocks(b: &mut Builder) {
    let gates = [town::gate(true).0, town::gate(false).0];
    // Det som alt står utenfor muren (veiskilt, brønnen, gårdene).
    let taken: Vec<Vec2> = b.placements.iter().flat_map(|p| p.transforms.iter().map(|t| Vec2::new(t.translation.x, t.translation.z))).filter(|p| terrain::town_edge(p.x, p.y) > -2.0).collect();
    let mut placed: Vec<(Vec2, f32)> = vec![];
    let mut rng = Rng(0x5713_1350_0b0c_a11e);
    // (modell, antall, skala, radius, avstand til veien (fra, til), minste normal.y)
    let kinds: [(&'static str, usize, (f32, f32), f32, (f32, f32), f32); 2] =
        [("rock_big", ROCKS_BIG, (0.8, 1.25), 1.6, (5.5, 14.0), 0.82), ("rock_group", ROCK_GROUPS, (0.75, 1.3), 1.1, (4.5, f32::MAX), 0.88)];
    for (model, count, (s0, s1), radius, (near, far), steep) in kinds {
        if !b.has_model(model) {
            b.place(model, Affine3A::IDENTITY);
            continue;
        }
        let mut n = 0;
        for _ in 0..4000 {
            if n >= count {
                break;
            }
            let a = rng.range(0.0, std::f32::consts::TAU);
            let r = rng.range(58.0, 182.0);
            let p = Vec2::new(a.cos(), a.sin()) * r;
            let (yaw, scale) = (rng.range(0.0, std::f32::consts::TAU), rng.range(s0, s1));
            let reach = radius * scale;
            let road = town::path_dist(p.x, p.y) - reach;
            let clear = road > near
                && road < far
                && town::town_dist(p.x, p.y) > 8.0 + reach
                && gates.iter().all(|g| g.distance(p) > 20.0)
                && terrain::normal(p.x, p.y).y > steep
                && taken.iter().all(|q| q.distance(p) > 6.0 + reach)
                && placed.iter().all(|(q, rq)| q.distance(p) > 5.0 + reach + rq);
            if !clear {
                continue;
            }
            // Bakken under foten: nesten flat midt under (`audit` måler hjørnene
            // ±0,6 m i modellen), og høyst en halv meter over omrisset. Steinen
            // står loddrett på det laveste punktet og går ned i bakken på
            // oversida (kolliderne er loddrette prismer, som testene leser).
            let ring = |r: f32| {
                (0..8).map(move |k| {
                    let a = k as f32 * std::f32::consts::FRAC_PI_4;
                    terrain::height(p.x + a.cos() * r, p.y + a.sin() * r)
                })
            };
            let span = |it: &mut dyn Iterator<Item = f32>| it.fold((f32::MAX, f32::MIN), |(lo, hi), h| (lo.min(h), hi.max(h)));
            let (lo_mid, hi_mid) = span(&mut ring(0.86 * scale + 0.05).chain(ring(0.45 * scale)).chain(std::iter::once(terrain::height(p.x, p.y))));
            let (lo, hi) = span(&mut ring(reach).chain(ring(0.86 * scale + 0.05)));
            if hi_mid - lo.min(lo_mid) > 0.2 || hi - lo > 0.5 {
                continue;
            }
            let t = Affine3A::from_translation(Vec3::new(p.x, lo - ROCK_SINK * scale, p.y)) * Affine3A::from_rotation_y(yaw) * Affine3A::from_scale(Vec3::splat(scale));
            b.place(model, t);
            b.place_solids(model, t);
            placed.push((p, reach));
            log::debug!("{model} ved ({:.1}, {:.1})", p.x, p.y);
            n += 1;
        }
        log::info!("Steinene: {n} av {count} {model} satt ut");
    }
}

/// Bjørkeklyngene langs veiene (TRELLIS-stammen og løvkortene fra
/// `tree3d.py`, to modeller med samme origo ved foten av stammen).
pub const BIRCH_CLUSTERS: usize = 9;
const BIRCH_TRUNK: &str = "tree_birch_trunk";
const BIRCH_LEAVES: &str = "tree_birch_leaves";
/// Hvor langt ned i bakken stammen står (rota går ned i jorda).
const BIRCH_SINK: f32 = 0.06;

/// Setter ut små klynger på 3–5 bjørker langs veiene og stiene, like hver
/// gang (eget frø): 4,5–14 m fra veien, aldri på den, aldri innen 40 m fra portene, på tunene, kirkegården,
/// ved portene eller nær noe som alt står der (steinene, gårdene). Hvert tre
/// er dreid og skalert for seg, står på det laveste punktet under foten der
/// bakken er nesten flat, og har bare stammen som kollider (`extras.solids`);
/// løvet henger på stammen (`place_mounted`).
pub fn birches(b: &mut Builder) {
    if !b.has_model(BIRCH_TRUNK) || !b.has_model(BIRCH_LEAVES) {
        b.place(BIRCH_TRUNK, Affine3A::IDENTITY);
        b.place(BIRCH_LEAVES, Affine3A::IDENTITY);
        return;
    }
    let gates = [town::gate(true).0, town::gate(false).0];
    let taken: Vec<Vec2> = b.placements.iter().flat_map(|p| p.transforms.iter().map(|t| Vec2::new(t.translation.x, t.translation.z))).filter(|p| terrain::town_edge(p.x, p.y) > -2.0).collect();
    let mut rng = Rng(0xb1a4_1350_7e3e_0b1c);
    let mut centers: Vec<Vec2> = vec![];
    let mut trees = 0;
    // Stedet for én bjørk: høyden den står i, eller ingenting.
    let spot = |p: Vec2, placed: &[Vec2]| -> Option<f32> {
        let road = town::path_dist(p.x, p.y);
        let clear = road > 4.5
            && town::town_dist(p.x, p.y) > 10.0
            && gates.iter().all(|g| g.distance(p) > 40.0)
            && terrain::normal(p.x, p.y).y > 0.9
            && taken.iter().all(|q| q.distance(p) > 4.0)
            && placed.iter().all(|q| q.distance(p) > 3.2);
        if !clear {
            return None;
        }
        // Bakken under foten (`audit` måler hjørnene ±0,6 m).
        let hs: Vec<f32> = [(0.0, 0.0), (-0.6, -0.6), (0.6, -0.6), (-0.6, 0.6), (0.6, 0.6)].iter().map(|(dx, dz)| terrain::height(p.x + dx, p.y + dz)).collect();
        let (lo, hi) = hs.iter().fold((f32::MAX, f32::MIN), |(lo, hi), h| (lo.min(*h), hi.max(*h)));
        (hi - lo < 0.2).then_some(lo)
    };
    let mut placed: Vec<Vec2> = vec![];
    for _ in 0..3000 {
        if centers.len() >= BIRCH_CLUSTERS {
            break;
        }
        let a = rng.range(0.0, std::f32::consts::TAU);
        let r = rng.range(56.0, 175.0);
        let c = Vec2::new(a.cos(), a.sin()) * r;
        let road = town::path_dist(c.x, c.y);
        if !(7.0..12.0).contains(&road) || centers.iter().any(|q| q.distance(c) < 30.0) || spot(c, &placed).is_none() {
            continue;
        }
        let want = rng.pick(&[3usize, 4, 5]);
        let mut n = 0;
        for _ in 0..60 {
            if n >= want {
                break;
            }
            let p = if n == 0 { c } else { c + Vec2::new(rng.range(-5.0, 5.0), rng.range(-5.0, 5.0)) };
            if town::path_dist(p.x, p.y) > 14.0 {
                continue;
            }
            let Some(ground) = spot(p, &placed) else { continue };
            let (yaw, scale) = (rng.range(0.0, std::f32::consts::TAU), rng.range(0.85, 1.12));
            let t = Affine3A::from_translation(Vec3::new(p.x, ground - BIRCH_SINK, p.y)) * Affine3A::from_rotation_y(yaw) * Affine3A::from_scale(Vec3::splat(scale));
            b.place(BIRCH_TRUNK, t);
            b.place_solids(BIRCH_TRUNK, t);
            b.place_mounted(BIRCH_LEAVES, t);
            placed.push(p);
            n += 1;
        }
        if n > 0 {
            centers.push(c);
            trees += n;
            log::debug!("bjørkeklynge med {n} ved ({:.1}, {:.1})", c.x, c.y);
        }
    }
    log::info!("Bjørkene: {trees} trær i {} klynger", centers.len());
}

/// Landskapet rundt terrenget: fra kanten av bakken (±200 m) stiger
/// rullende åser i ett sammenhengende polart rutenett. Lavt er det eng,
/// bratt og høyt er det berg. Ingen kollider: kanten av terrenget (`world_edge`) stopper deg
/// før du kommer dit.
pub fn landscape(b: &mut Builder) {
    let (r0, r1) = (196.0f32, 600.0f32);
    let (rings, spokes) = (64usize, 288usize);
    let grass = Mat::tex(Layer::Meadow, 0xffffff);
    // Granitten er lys i bildet; i bratte sider dempes den mot gråbrun.
    let rock = Mat::tex(Layer::Rock, 0xffffff).tint(Vec3::new(0.5, 0.47, 0.44));
    // Radiene tettere innerst, der detaljene er nær.
    let radius = |j: usize| {
        let t = j as f32 / rings as f32;
        r0 + (r1 - r0) * t * t.sqrt()
    };
    let pos = |j: usize, i: usize| {
        let a = i as f32 / spokes as f32 * std::f32::consts::TAU;
        let r = radius(j);
        let (x, z) = (a.sin() * r, a.cos() * r);
        Vec3::new(x, landscape_height(x, z), z)
    };
    let normal = |p: Vec3| {
        let d = 3.0;
        let hx = landscape_height(p.x + d, p.z) - landscape_height(p.x - d, p.z);
        let hz = landscape_height(p.x, p.z + d) - landscape_height(p.x, p.z - d);
        Vec3::new(-hx, 2.0 * d, -hz).normalize()
    };
    for j in 0..rings {
        for i in 0..spokes {
            let i2 = (i + 1) % spokes;
            let p = [pos(j, i), pos(j, i2), pos(j + 1, i2), pos(j + 1, i)];
            let n = [normal(p[0]), normal(p[1]), normal(p[2]), normal(p[3])];
            let c = (p[0] + p[1] + p[2] + p[3]) * 0.25;
            let up = (n[0] + n[1] + n[2] + n[3]).normalize().y;
            let ground = terrain::height(c.x, c.z);
            let rise = c.y - ground;
            // Bratte og høye sider er berg.
            let mat = if rise > 60.0 || up < 0.62 { &rock } else { &grass };
            b.mesh.quad_n([p[3], p[2], p[1], p[0]], [n[3], n[2], n[1], n[0]], mat, 1.0);
        }
    }
}

/// Høyden i landskapet utenfor terrenget: bakken glir over i rullende
/// åser fra 200 m, som blir høyere og mykere utover mot horisonten.
pub fn landscape_height(x: f32, z: f32) -> f32 {
    let r = (x * x + z * z).sqrt();
    let base = terrain::height(x, z);
    let smooth = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let hills = smooth((r - 200.0) / 70.0) * (terrain::noise(x * 0.012 + 20.0, z * 0.012 + 20.0) * 28.0 + terrain::noise(x * 0.03 + 40.0, z * 0.03) * 8.0);
    // Lenger ut: bare høyere, myke åser, ingen fjell.
    let far = smooth((r - 260.0) / 200.0) * (terrain::noise(x * 0.005 + 70.0, z * 0.005 + 30.0) * 60.0);
    base + hills + far
}

/// Usynlige vegger rundt kanten av terrenget (±200 m), så ingen faller
/// ut av verden eller går inn i fjellene.
pub fn world_edge(b: &mut Builder) {
    let n = 36;
    let radius = 199.0;
    for i in 0..n {
        let a = (i as f32 + 0.5) / n as f32 * std::f32::consts::TAU;
        // Fra 30 m under null (dalene mellom åsene) og opp. Kjelleren holder
        // seg innenfor ±185 m, så den går ikke inn i veggen.
        let c = Vec3::new(a.sin() * radius, 27.5, a.cos() * radius);
        b.solid(Shape::Cuboid { center: c, size: Vec3::new(2.0 * std::f32::consts::PI * radius / n as f32 + 2.0, 115.0, 2.0), yaw: a });
    }
}
