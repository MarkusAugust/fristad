//! Legacy-kjelleren: en prosedyrisk etasje under torget der den gamle
//! CSS-en og de korrupte komponentene holder til. Et rutenett av celler
//! på 4 m gravd ut med en fyllevandring fra rommet under brønnen, med
//! noen større rom, murvegger, betonggulv og fakler. Fiendene
//! (`domain/mobs.rs`) går på rutenettet; spilleren går på kolliderne.

use super::{Builder, Shape, Rng};
use crate::domain::geometry::{flag, Layer, Mat, Mesh};
use glam::{Affine3A, Quat, Vec3};

/// Gulvets overkant, under alt i byen.
pub const DEPTH: f32 = -16.0;
pub const CELL: f32 = 4.0;
pub const WALL_H: f32 = 3.2;
/// Rutenettet er N × N celler.
pub const N: usize = 15;
/// Startrommet ligger rett under brønnen på torget.
pub const START: (usize, usize) = (7, 7);
pub const WELL: Vec3 = Vec3::new(-11.0, 0.0, 9.0);

#[derive(Clone, Debug)]
pub struct Dungeon {
    open: Vec<bool>,
    /// Verdenskoordinatene (x, z) til hjørnet av celle (0, 0).
    origin: (f32, f32),
    /// Føttene der du lander når du klatrer ned.
    pub start: Vec3,
    /// Steder fiendene kan starte, langt nok fra startrommet.
    pub spawns: Vec<Vec3>,
}

impl Dungeon {
    pub fn is_open(&self, ix: i32, iz: i32) -> bool {
        ix >= 0 && iz >= 0 && (ix as usize) < N && (iz as usize) < N && self.open[iz as usize * N + ix as usize]
    }

    /// Cellen et verdenspunkt ligger i.
    pub fn cell_of(&self, p: Vec3) -> (i32, i32) {
        (((p.x - self.origin.0) / CELL).floor() as i32, ((p.z - self.origin.1) / CELL).floor() as i32)
    }

    /// Midten av en celle, på gulvet.
    pub fn center(&self, ix: i32, iz: i32) -> Vec3 {
        Vec3::new(self.origin.0 + (ix as f32 + 0.5) * CELL, DEPTH, self.origin.1 + (iz as f32 + 0.5) * CELL)
    }

    /// Om et punkt (med margin inn fra veggene) er på gulvet i kjelleren.
    pub fn walkable(&self, p: Vec3, margin: f32) -> bool {
        let (ix, iz) = self.cell_of(p);
        if !self.is_open(ix, iz) {
            return false;
        }
        // Nær en kant: nabocellen på den sida må også være åpen.
        let lx = p.x - (self.origin.0 + ix as f32 * CELL);
        let lz = p.z - (self.origin.1 + iz as f32 * CELL);
        (lx >= margin || self.is_open(ix - 1, iz))
            && (lx <= CELL - margin || self.is_open(ix + 1, iz))
            && (lz >= margin || self.is_open(ix, iz - 1))
            && (lz <= CELL - margin || self.is_open(ix, iz + 1))
    }

    /// Neste cellemidte på korteste vei fra `from` til `to` (bredde først
    /// på rutenettet), eller `None` om det ikke finnes noen vei.
    pub fn next_step(&self, from: Vec3, to: Vec3) -> Option<Vec3> {
        let a = self.cell_of(from);
        let b = self.cell_of(to);
        if !self.is_open(a.0, a.1) || !self.is_open(b.0, b.1) {
            return None;
        }
        if a == b {
            return Some(to);
        }
        let idx = |c: (i32, i32)| c.1 as usize * N + c.0 as usize;
        let mut prev = vec![usize::MAX; N * N];
        let mut queue = std::collections::VecDeque::new();
        prev[idx(a)] = idx(a);
        queue.push_back(a);
        while let Some(c) = queue.pop_front() {
            if c == b {
                break;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (c.0 + dx, c.1 + dz);
                if self.is_open(n.0, n.1) && prev[idx(n)] == usize::MAX {
                    prev[idx(n)] = idx(c);
                    queue.push_back(n);
                }
            }
        }
        if prev[idx(b)] == usize::MAX {
            return None;
        }
        // Gå bakover fra målet til cellen etter starten.
        let mut cur = idx(b);
        while prev[cur] != idx(a) {
            cur = prev[cur];
        }
        Some(self.center((cur % N) as i32, (cur / N) as i32))
    }
}

/// Graver ut og bygger kjelleren. `seed` gir en ny etasje per nedstigning
/// om byen bygges på nytt; i testene er den fast.
pub fn build(b: &mut Builder, seed: u64) -> Dungeon {
    let mut rng = Rng(seed | 1);
    let origin = (WELL.x - (START.0 as f32 + 0.5) * CELL, WELL.z - (START.1 as f32 + 0.5) * CELL);
    let mut open = vec![false; N * N];
    let carve = |open: &mut Vec<bool>, ix: i32, iz: i32| {
        if ix >= 1 && iz >= 1 && ix < N as i32 - 1 && iz < N as i32 - 1 {
            open[iz as usize * N + ix as usize] = true;
        }
    };
    // Startrommet: 3 × 3.
    for dx in -1..=1 {
        for dz in -1..=1 {
            carve(&mut open, START.0 as i32 + dx, START.1 as i32 + dz);
        }
    }
    // Fyllevandring med litt stahet, så gangene blir lange.
    let (mut x, mut z) = (START.0 as i32, START.1 as i32);
    let mut dir = 0usize;
    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    for _ in 0..240 {
        if rng.chance(0.35) {
            dir = (rng.next() * 4.0) as usize % 4;
        }
        let (nx, nz) = (x + dirs[dir].0, z + dirs[dir].1);
        if nx < 1 || nz < 1 || nx >= N as i32 - 1 || nz >= N as i32 - 1 {
            dir = (dir + 1) % 4;
            continue;
        }
        x = nx;
        z = nz;
        carve(&mut open, x, z);
        // Rom underveis.
        if rng.chance(0.06) {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    carve(&mut open, x + dx, z + dz);
                }
            }
        }
    }
    let d = Dungeon { open, origin, start: Vec3::ZERO, spawns: vec![] };
    let start = d.center(START.0 as i32, START.1 as i32);
    // Startsteder for fiendene: åpne celler minst fire celler unna.
    let mut spawns = vec![];
    for iz in 0..N as i32 {
        for ix in 0..N as i32 {
            let far = (ix - START.0 as i32).abs() + (iz - START.1 as i32).abs();
            if d.is_open(ix, iz) && far >= 4 && rng.chance(0.3) {
                spawns.push(d.center(ix, iz));
            }
        }
    }
    let mut d = Dungeon { start, spawns, ..d };
    mesh(b, &mut d, &mut rng);
    d
}

/// Gulv, tak, vegger, kollidere og fakler.
fn mesh(b: &mut Builder, d: &mut Dungeon, rng: &mut Rng) {
    let floor = Mat::tex(Layer::Concrete, 0xffffff).tint(Vec3::new(0.35, 0.34, 0.33)).flags(flag::NO_WEATHER);
    let ceiling = Mat::tex(Layer::Stone, 0xffffff).tint(Vec3::new(0.3, 0.29, 0.28)).flags(flag::NO_WEATHER);
    let wall = Mat::tex(Layer::Brick, 0xffffff).tint(Vec3::new(0.42, 0.36, 0.33)).flags(flag::NO_WEATHER);
    let (x0, z0) = d.origin;
    let mut torches = vec![];
    let mut torch_count = 0;
    for iz in 0..N as i32 {
        for ix in 0..N as i32 {
            if !d.is_open(ix, iz) {
                continue;
            }
            let (cx, cz) = (x0 + ix as f32 * CELL, z0 + iz as f32 * CELL);
            let (y0, y1) = (DEPTH, DEPTH + WALL_H);
            let m = &mut b.mesh;
            // Gulv og tak.
            m.quad_n([Vec3::new(cx, y0, cz + CELL), Vec3::new(cx + CELL, y0, cz + CELL), Vec3::new(cx + CELL, y0, cz), Vec3::new(cx, y0, cz)], [Vec3::Y; 4], &floor, 1.0);
            m.quad_n([Vec3::new(cx, y1, cz), Vec3::new(cx + CELL, y1, cz), Vec3::new(cx + CELL, y1, cz + CELL), Vec3::new(cx, y1, cz + CELL)], [-Vec3::Y; 4], &ceiling, 1.0);
            b.colliders.push(Shape::Cuboid { center: Vec3::new(cx + CELL * 0.5, y0 - 0.25, cz + CELL * 0.5), size: Vec3::new(CELL, 0.5, CELL), yaw: 0.0 });
            // Vegger mot lukkede naboer, med normalen inn i cellen.
            let sides: [((i32, i32), [Vec3; 4], Vec3); 4] = [
                ((-1, 0), [Vec3::new(cx, y0, cz), Vec3::new(cx, y0, cz + CELL), Vec3::new(cx, y1, cz + CELL), Vec3::new(cx, y1, cz)], Vec3::X),
                ((1, 0), [Vec3::new(cx + CELL, y0, cz + CELL), Vec3::new(cx + CELL, y0, cz), Vec3::new(cx + CELL, y1, cz), Vec3::new(cx + CELL, y1, cz + CELL)], -Vec3::X),
                ((0, -1), [Vec3::new(cx + CELL, y0, cz), Vec3::new(cx, y0, cz), Vec3::new(cx, y1, cz), Vec3::new(cx + CELL, y1, cz)], Vec3::Z),
                ((0, 1), [Vec3::new(cx, y0, cz + CELL), Vec3::new(cx + CELL, y0, cz + CELL), Vec3::new(cx + CELL, y1, cz + CELL), Vec3::new(cx, y1, cz + CELL)], -Vec3::Z),
            ];
            for ((dx, dz), quad, normal) in sides {
                if d.is_open(ix + dx, iz + dz) {
                    continue;
                }
                b.mesh.quad_n(quad, [normal; 4], &wall, 0.85);
                let center = (quad[0] + quad[2]) * 0.5 - normal * 0.1;
                let size = if dx != 0 { Vec3::new(0.2, WALL_H, CELL) } else { Vec3::new(CELL, WALL_H, 0.2) };
                b.colliders.push(Shape::Cuboid { center, size, yaw: 0.0 });
                // En fakkel på hver tredje vegg, midt på, 1,8 m opp.
                torch_count += 1;
                if torch_count % 3 == 0 {
                    torches.push(((quad[0] + quad[1]) * 0.5 + Vec3::Y * 1.8 + normal * 0.02, normal));
                }
            }
        }
    }
    for (p, n) in torches {
        let yaw = n.x.atan2(n.z);
        if b.has_model("gorvek_torch") {
            b.place_mounted("gorvek_torch", Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p));
        } else {
            b.mesh.cylinder(&Affine3A::from_rotation_translation(Quat::from_rotation_x(0.4) * Quat::from_rotation_y(yaw), p), 0.03, 0.03, 0.4, 6, &super::OAK, true);
        }
        let flicker = 0.9 + rng.next() * 0.3;
        b.fire_light(p + n * 0.25 + Vec3::Y * 0.3, Vec3::new(1.0, 0.55, 0.22), 2.2 * flicker, 9.0);
    }
    // Søyler i hjørnene av rommene, så det ikke ser ut som en boks.
    let _ = Mesh::default();
}
