//! `.fs-skeleton`: byggeplassen ved Østre gate. Huset under bygging er
//! et bindingsverk i tre på en betongsåle, med OSB-plater på fasaden der
//! de grå plassholderne sitter og skimrer der vinduene og døra skal
//! komme, bjelkelag mellom etasjene og takstoler uten tekking. Rundt står
//! stillaset, og tomta er gjerdet inn med byggegjerde, brakke, toalett,
//! tårnkran, minigraver, sementblander, paller med lecablokker og
//! trelast (`tools/models/site.py`). R bytter variant (tekst, sirkel,
//! blokk) på den første plassholderen.

use super::signs::{self, Box2};
use super::{Component, Env, Installation, PX};
use crate::domain::city::{at, Builder, CONCRETE, IRON, OAK};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec2, Vec3};

pub const VARIANTS: [&str; 3] = ["text", "circle", "block"];

#[derive(Default)]
pub struct Skeleton {
    variant: usize,
    first: [usize; 3],
}

/// Tomta: x 42–56, z −29..−10, øst for tabs-huset, med porten mot
/// grusveien i sør. Huset står midt på med fasaden mot sør (+z); `FRONT`
/// er fasadens fotpunkt. Porten er ett gjerdepanel som står åpent.
const FRONT: Vec3 = Vec3::new(49.0, 0.0, -14.0);
const LOT: (f32, f32, f32, f32) = (42.0, 56.0, -29.0, -10.0);
const GATE: f32 = 48.0;
/// Husets mål: bredde, dybde, såletopp, etasjehøyde.
const W: f32 = 7.2;
const D: f32 = 9.0;
const SLAB: f32 = 0.3;
const STOREY: f32 = 3.0;
const STOREYS: usize = 3;

/// Bindingsverket: lyst, høvlet gran. Platene: OSB i gulbrunt.
const STUD: Mat = Mat::tex(Layer::Timber, 0xffffff);
const OSB: Mat = Mat::tex(Layer::Plywood, 0xffffff);

impl Installation for Skeleton {
    fn component(&self) -> Component {
        Component { name: "skeleton", class: ".fs-skeleton" }
    }
    fn variants(&self) -> &'static [&'static str] {
        &VARIANTS
    }
    fn variant(&self) -> usize {
        self.variant
    }
    fn set_variant(&mut self, i: usize) {
        self.variant = i % VARIANTS.len();
    }

    fn build(&mut self, b: &mut Builder) {
        let front = b.on_ground(FRONT);
        // Tomta er planert (terrain::PLOTS), så alt står på samme nivå.
        let g = front.y;
        let local = |p: Vec3| Affine3A::from_translation(front + p);
        self.house(b, &local);
        self.placeholders(b, &local);
        scaffold(b, &local);
        site(b, g);
        let _ = Quat::IDENTITY;
    }

    fn pose(&self, prop: usize, rest: Affine3A, _env: &Env) -> (Affine3A, [f32; 4]) {
        if let Some(v) = self.first.iter().position(|p| *p == prop) {
            if v == self.variant {
                return (rest, [0.0; 4]);
            }
            return (rest * Affine3A::from_scale(Vec3::splat(0.0001)), [0.0; 4]);
        }
        (rest, [0.0; 4])
    }
}

impl Skeleton {
    /// Huset: såle, bindingsverk i tre etasjer, bjelkelag, OSB på fasaden
    /// og takstoler. Lokale koordinater: x langs fasaden, z inn i huset
    /// (negativt), y opp fra bakken.
    fn house(&self, b: &mut Builder, local: &dyn Fn(Vec3) -> Affine3A) {
        let front = Vec3::from(local(Vec3::ZERO).translation);
        let m = &mut b.mesh;
        let (hw, hd) = (W * 0.5, D * 0.5);
        let zc = -hd;
        // Sålen i betong med ringmur, og gulvet (OSB) oppå.
        m.cuboid_ao(&local(Vec3::new(0.0, SLAB * 0.5, zc)), Vec3::new(W + 0.4, SLAB, D + 0.4), &CONCRETE, 1.0);
        m.cuboid(&local(Vec3::new(0.0, SLAB + 0.011, zc)), Vec3::new(W, 0.022, D), &OSB);
        let top = SLAB + STOREY * STOREYS as f32;
        // Veggene: svill, stendere hver 60 cm, topp­svill, for hver etasje.
        let walls: [(Vec3, Vec3, f32); 4] = [
            (Vec3::new(0.0, 0.0, 0.0), Vec3::X, W),      // fasaden
            (Vec3::new(0.0, 0.0, -D), Vec3::X, W),       // bakveggen
            (Vec3::new(-hw, 0.0, zc), Vec3::Z, D),       // vestveggen
            (Vec3::new(hw, 0.0, zc), Vec3::Z, D),        // østveggen
        ];
        for (c, dir, len) in walls {
            let thick = Vec3::new(0.098, 0.048, 0.098) * (Vec3::ONE - dir) + dir * len;
            for storey in 0..STOREYS {
                let y0 = SLAB + storey as f32 * STOREY;
                // Svill og dobbel toppsvill.
                m.cuboid(&local(c + Vec3::Y * (y0 + 0.024)), Vec3::new(thick.x, 0.048, thick.z), &STUD);
                m.cuboid(&local(c + Vec3::Y * (y0 + STOREY - 0.024)), Vec3::new(thick.x, 0.048, thick.z), &STUD);
                m.cuboid(&local(c + Vec3::Y * (y0 + STOREY - 0.072)), Vec3::new(thick.x, 0.048, thick.z), &STUD);
                // Stenderne, med hjørnestender i hver ende.
                let n = (len / 0.6).round() as i32;
                for k in 0..=n {
                    let s = -len * 0.5 + k as f32 * len / n as f32;
                    let p = c + dir * s + Vec3::Y * (y0 + STOREY * 0.5);
                    let size = if dir == Vec3::X { Vec3::new(0.048, STOREY - 0.12, 0.098) } else { Vec3::new(0.098, STOREY - 0.12, 0.048) };
                    m.cuboid(&local(p), size, &STUD);
                }
                // Losholter og spikerslag midt i veggen.
                m.cuboid(&local(c + Vec3::Y * (y0 + 1.1)), Vec3::new(thick.x, 0.048, thick.z) * (Vec3::ONE - dir) + dir * (len - 0.1), &STUD);
            }
        }
        // Bjelkelagene mellom etasjene: 48 × 220 hver 60 cm på tvers (langs x),
        // og gulvplater på dem: hele i andre etasje, halve i tredje.
        for storey in 1..STOREYS {
            let y = SLAB + storey as f32 * STOREY;
            let n = (D / 0.6).round() as i32;
            for k in 0..=n {
                let z = -k as f32 * D / n as f32;
                m.cuboid(&local(Vec3::new(0.0, y + 0.11, z)), Vec3::new(W - 0.2, 0.22, 0.048), &STUD);
            }
            let laid = if storey == 1 { D - 0.2 } else { D * 0.5 };
            m.cuboid(&local(Vec3::new(0.0, y + 0.231, -laid * 0.5 - 0.1)), Vec3::new(W - 0.2, 0.022, laid), &OSB);
            b.floors.push((front + Vec3::new(0.0, 0.0, -laid * 0.5 - 0.1), Vec2::new(hw - 0.1, laid * 0.5), 0.0, front.y + y + 0.242));
        }
        // OSB-platene på fasaden (1,2 × 2,4 m), nederst til øverst, som
        // plassholderne sitter på. Øverste etasje får bare de nederste.
        let m = &mut b.mesh;
        let cols = (W / 1.2).round() as i32;
        for col in 0..cols {
            let x = -hw + 0.6 + col as f32 * 1.2;
            for row in 0..4 {
                let y = SLAB + 1.2 + row as f32 * 2.4;
                if row == 3 && col % 2 == 1 {
                    continue;
                }
                m.cuboid(&local(Vec3::new(x, y, 0.06)), Vec3::new(1.19, 2.39, 0.012), &OSB);
            }
        }
        // Takstolene: sperrer fra raftene til mønet, hanebjelke, hver 1,2 m,
        // med lekter på tvers av sperrene. Ingen tekking ennå.
        let roof_h = 2.6;
        let n = (D / 1.2).round() as i32;
        let overhang = 0.5;
        let rafter_len = ((hw + overhang).powi(2) + (roof_h + overhang * roof_h / hw).powi(2)).sqrt();
        let pitch = (roof_h / hw).atan();
        for k in 0..=n {
            let z = -k as f32 * D / n as f32;
            m.cuboid(&local(Vec3::new(0.0, top + 0.1, z)), Vec3::new(W, 0.198, 0.048), &STUD);
            for side in [1.0f32, -1.0] {
                let c = Vec3::new(side * (hw + overhang) * 0.5, top + roof_h * 0.5 + 0.05, z);
                let t = local(c) * Affine3A::from_rotation_z(-side * pitch);
                m.cuboid(&t, Vec3::new(rafter_len, 0.198, 0.048), &STUD);
            }
            m.cuboid(&local(Vec3::new(0.0, top + roof_h * 0.55, z)), Vec3::new(hw * 0.9, 0.148, 0.048), &STUD);
        }
        // Mønsås og lekter.
        m.cuboid(&local(Vec3::new(0.0, top + roof_h + 0.1, zc)), Vec3::new(0.098, 0.198, D + 0.4), &STUD);
        let steps = (rafter_len / 0.36) as i32;
        for side in [1.0f32, -1.0] {
            for k in 1..steps {
                let s = k as f32 / steps as f32;
                let x = side * (hw + overhang) * (1.0 - s);
                let y = top + 0.15 + (roof_h + overhang * roof_h / hw) * s;
                m.cuboid(&local(Vec3::new(x, y + 0.03, zc)), Vec3::new(0.036, 0.048, D + 0.4), &STUD);
            }
        }
        // Kollisjon: huset som ett volum (veggene er tette nok med platene).
        b.solid_box(front + Vec3::new(0.0, (top + roof_h) * 0.5, zc), Vec3::new(W + 0.4, top + roof_h, D + 0.4), 0.0);
    }

    /// Plassholderne på fasaden, som i eksempelet: linje, sirkel og blokk
    /// i rader der vinduene skal komme. De skimrer som `.fs-skeleton`.
    fn placeholders(&mut self, b: &mut Builder, local: &dyn Fn(Vec3) -> Affine3A) {
        let mat = Mat::special(kind::SKELETON, 0xffffff, 0.8).slot(slot::NEUTRAL_SURFACE).flags(flag::NO_DIRT | flag::NO_WEATHER);
        let shape = |m: &mut Mesh, v: usize| {
            let (bw, bh, r) = match v {
                0 => (100.0, 16.0, 4.0),
                1 => (48.0, 48.0, 24.0),
                _ => (100.0, 150.0, 4.0),
            };
            m.rounded_slab(&Affine3A::IDENTITY, Vec3::ZERO, bw * PX, bh * PX, 0.04, r * PX, &mat);
            bh
        };
        for f in 0..3 {
            let y = SLAB + 1.0 + f as f32 * STOREY;
            for (k, x) in [-2.2f32, 0.0, 2.2].iter().enumerate() {
                let v = (f + k) % 3;
                if f == 0 && k == 0 {
                    // Tre varianter av den første; bare én vises.
                    for v in 0..3 {
                        let mut vm = Mesh::default();
                        let bh = shape(&mut vm, v);
                        let vt = local(Vec3::new(*x, y + bh * PX * 0.5, 0.09));
                        self.first[v] = b.prop("skeleton", vm, vt, if v == 0 { Some((Vec3::ZERO, Vec3::new(0.8, 0.8, 0.1))) } else { None });
                    }
                } else {
                    let mut pm = Mesh::default();
                    let bh = shape(&mut pm, v);
                    pm.transform(&local(Vec3::new(*x, y + bh * PX * 0.5, 0.09)));
                    b.mesh.append(&pm);
                }
            }
        }
        let _ = Box2 { w: 0.0, h: 0.0, radius: 0.0, fill: 0, border: None, depth: 0.0 };
        let _ = signs::boxed;
    }
}

/// Stillaset foran fasaden: rammestillas i to bredder med spir hver 2 m,
/// lengdebjelker, tverrbjelker, diagonaler på utsiden, plankedekker med
/// rekkverk og fotlist i tre høyder, fotplater og stige.
fn scaffold(b: &mut Builder, local: &dyn Fn(Vec3) -> Affine3A) {
    let m = &mut b.mesh;
    let r = 0.024;
    let (inner, outer) = (0.4, 1.3);
    let xs: Vec<f32> = (0..5).map(|k| -4.0 + k as f32 * 2.0).collect();
    let lifts = [2.0f32, 4.0, 6.0, 8.0];
    let top = 9.0;
    let pole = |m: &mut Mesh, p: Vec3, h: f32| {
        m.cylinder(&local(p), r, r, h, 8, &IRON, true);
    };
    let bar = |m: &mut Mesh, a: Vec3, c: Vec3, r: f32| {
        let d = c - a;
        let len = d.length();
        let t = local(a + d * 0.5) * Affine3A::from_rotation_translation(Quat::from_rotation_arc(Vec3::Y, d / len), Vec3::ZERO) * Affine3A::from_translation(Vec3::Y * -len * 0.5);
        m.cylinder(&t, r, r, len, 6, &IRON, true);
    };
    for &x in &xs {
        for z in [inner, outer] {
            pole(m, Vec3::new(x, 0.0, z), top);
            m.cuboid(&local(Vec3::new(x, 0.01, z)), Vec3::new(0.15, 0.02, 0.15), &IRON);
        }
        // Tverrbjelker ved hvert dekke og nederst.
        for &y in lifts.iter().chain([0.3f32].iter()) {
            bar(m, Vec3::new(x, y, inner), Vec3::new(x, y, outer), r * 0.8);
        }
    }
    for z in [inner, outer] {
        for &y in lifts.iter().chain([0.3f32].iter()) {
            bar(m, Vec3::new(xs[0], y, z), Vec3::new(xs[4], y, z), r);
        }
    }
    // Rekkverk og fotlist på utsiden av hvert dekke, og diagonaler.
    for &y in &lifts {
        bar(m, Vec3::new(xs[0], y + 0.5, outer), Vec3::new(xs[4], y + 0.5, outer), r * 0.8);
        bar(m, Vec3::new(xs[0], y + 1.0, outer), Vec3::new(xs[4], y + 1.0, outer), r * 0.8);
        m.cuboid(&local(Vec3::new(0.0, y + 0.13, outer + 0.03)), Vec3::new(xs[4] - xs[0], 0.15, 0.025), &OAK);
        for k in 0..3 {
            m.cuboid(&local(Vec3::new(0.0, y + 0.02, inner + 0.15 + k as f32 * 0.3)), Vec3::new(xs[4] - xs[0] + 0.3, 0.04, 0.28), &OAK);
        }
    }
    for k in 0..4 {
        let (x0, x1) = (xs[k], xs[k + 1]);
        let (y0, y1) = (0.3, lifts[k.min(3)]);
        let (a, c) = if k % 2 == 0 { (Vec3::new(x0, y0, outer + 0.05), Vec3::new(x1, y1, outer + 0.05)) } else { (Vec3::new(x1, y0, outer + 0.05), Vec3::new(x0, y1, outer + 0.05)) };
        bar(m, a, c, r * 0.7);
    }
    // Stigen fra bakken til første dekke, i ytterste bredde.
    let lx = xs[4] - 0.5;
    for dx in [-0.2f32, 0.2] {
        bar(m, Vec3::new(lx + dx, 0.0, outer - 0.3), Vec3::new(lx + dx, 2.6, inner + 0.2), r * 0.7);
    }
    for k in 1..9 {
        let s = k as f32 / 9.0;
        let p = Vec3::new(lx, 2.6 * s, outer - 0.3 + (inner + 0.2 - outer + 0.3) * s);
        bar(m, p + Vec3::X * -0.2, p + Vec3::X * 0.2, r * 0.5);
    }
    let front = Vec3::from(local(Vec3::ZERO).translation);
    b.solid_box(front + Vec3::new(0.0, top * 0.5, (inner + outer) * 0.5), Vec3::new(xs[4] - xs[0] + 0.3, top, outer - inner + 0.3), 0.0);
}

/// Tomta rundt huset: byggegjerde med port mot veien, skilt, brakke,
/// toalett, tårnkran, minigraver, sementblander, paller, trillebår og
/// trelast. `g` er tomtas høyde.
fn site(b: &mut Builder, g: f32) {
    let (x0, x1, z0, z1) = LOT;
    let panel = 3.5;
    let place = |b: &mut Builder, name: &'static str, x: f32, z: f32, yaw: f32, size: Vec3| {
        let p = Vec3::new(x, g, z);
        b.place(name, at(p, yaw));
        b.solid_box(p + Vec3::Y * size.y * 0.5, size, yaw);
    };
    // Gjerdet: paneler langs alle fire sider, jevnt fordelt så de overlapper
    // litt; porten er panelet nærmest `GATE` på sørsida, svingt 70° inn på
    // tomta om hengselet i vestenden.
    let n = ((x1 - x0) / panel).ceil() as i32;
    let gate = (0..n).min_by_key(|k| ((x0 + (x1 - x0) * (*k as f32 + 0.5) / n as f32 - GATE).abs() * 100.0) as i32).unwrap_or(0);
    for k in 0..n {
        let x = x0 + (x1 - x0) * (k as f32 + 0.5) / n as f32;
        place(b, "site_fence", x, z0, 0.0, Vec3::new(panel, 2.2, 0.1));
        if k == gate {
            let hinge = x - panel * 0.5;
            let yaw: f32 = -1.2;
            let dir = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
            let c = Vec3::new(hinge, 0.0, z1) + dir * panel * 0.5;
            place(b, "site_fence", c.x, c.z, yaw, Vec3::new(panel, 2.2, 0.1));
        } else {
            place(b, "site_fence", x, z1, 0.0, Vec3::new(panel, 2.2, 0.1));
        }
    }
    let n = ((z1 - z0) / panel).ceil() as i32;
    for k in 0..n {
        let z = z0 + (z1 - z0) * (k as f32 + 0.5) / n as f32;
        place(b, "site_fence", x0, z, std::f32::consts::FRAC_PI_2, Vec3::new(panel, 2.2, 0.1));
        place(b, "site_fence", x1, z, std::f32::consts::FRAC_PI_2, Vec3::new(panel, 2.2, 0.1));
    }
    // Skiltet på gjerdet ved porten.
    let (title, sub) = ("Byggeplass", "Adgang forbudt for uvedkommende");
    let w = (b.font.width(sub, 14.0) * PX + 0.3).max(1.0);
    b.fits("skeleton.sign", sub, 14.0, 1.0, w / PX);
    let mut sm = Mesh::default();
    signs::boxed(&mut sm, Vec3::ZERO, &Box2 { w: w / PX, h: 60.0, radius: 2.0, fill: slot::WARNING_SURFACE, border: Some(slot::WARNING_TEXT), depth: 0.02 });
    // Plata med kant har fyllet 4 mm foran senteret, så teksten må lenger fram.
    signs::text(&mut sm, b.font, title, Vec3::new(0.0, 0.06, 0.02), 22.0, slot::WARNING_TEXT);
    signs::text(&mut sm, b.font, sub, Vec3::new(0.0, -0.16, 0.02), 14.0, slot::WARNING_TEXT);
    sm.transform(&at(Vec3::new(x0 + 1.9, g + 1.5, z1 + 0.06), 0.0));
    b.mesh.append(&sm);
    // Brakka langs vestgjerdet med døra inn mot tomta, og toalettet ved porten.
    place(b, "site_container", x0 + 1.6, -22.5, std::f32::consts::FRAC_PI_2, Vec3::new(2.6, 2.7, 6.2));
    place(b, "site_toilet", x0 + 1.3, -15.5, -0.3, Vec3::new(1.2, 2.4, 1.2));
    // Krana bak huset i nordøst, med utliggeren over huset.
    place(b, "site_crane", 53.7, -26.3, std::f32::consts::PI, Vec3::new(4.4, 1.0, 4.4));
    b.solid_box(Vec3::new(53.7, g + 11.0, -26.3), Vec3::new(1.8, 22.0, 1.8), 0.0);
    // Graveren og blanderen på østsida av huset, paller og trillebår ved porten.
    place(b, "site_excavator", 54.3, -18.5, 1.6, Vec3::new(3.6, 2.6, 1.8));
    place(b, "site_mixer", 54.3, -13.5, 0.6, Vec3::new(1.2, 1.3, 1.0));
    place(b, "site_pallet", 44.5, z1 - 1.6, 0.1, Vec3::new(1.25, 0.9, 0.85));
    place(b, "site_pallet", 43.2, z1 - 1.7, -0.05, Vec3::new(1.25, 0.9, 0.85));
    place(b, "site_pallet", 51.6, z1 - 1.3, 1.4, Vec3::new(1.25, 0.9, 0.85));
    place(b, "site_wheelbarrow", 50.0, z1 - 1.5, 0.5, Vec3::new(1.4, 0.7, 0.7));
    // Trelasten langs nordgjerdet: to stabler med strø imellom.
    for (cx, cz, len, layers) in [(44.5f32, z0 + 1.2, 4.8f32, 4i32), (48.5, z0 + 1.3, 3.6, 3)] {
        let m = &mut b.mesh;
        for layer in 0..layers {
            let y = g + 0.06 + layer as f32 * 0.16;
            for k in 0..6 {
                m.cuboid(&at(Vec3::new(cx, y + 0.049, cz - 0.3 + k as f32 * 0.12), 0.0), Vec3::new(len, 0.098, 0.1), &STUD);
            }
            for sx in [-1.0f32, 1.0] {
                m.cuboid(&at(Vec3::new(cx + sx * len * 0.35, y + 0.13, cz), 0.0), Vec3::new(0.06, 0.06, 0.8), &OAK);
            }
        }
        b.solid_box(Vec3::new(cx, g + layers as f32 * 0.08, cz), Vec3::new(len, layers as f32 * 0.16, 0.8), 0.0);
    }
}
