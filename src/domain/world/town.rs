//! Kjøpstaden Fristad rundt 1350: et torg med markedsboder og torgkors,
//! Storgata fra sørporten til nordporten, Tverrgata øst–vest, bymur med
//! tårn, og husene langs gatene: Smia, Apoteket, Skriptoriet, Vertshuset,
//! stavkirka og Gorveks øvingsplass. Nord for byen står borgruinen på
//! haugen, og under den ligger krypta, nedgangen til kjelleren.
//!
//! Alt som synes er modeller (`tools/models/town.py`, `castle.py`); byen
//! bestemmer bare hvor de står. Husmodellene har døra mot sin lokale +Z og
//! origo midt på grunnflaten i bakkehøyde, så `yaw` vender døra mot gata.

use super::{at, terrain, Builder, NpcSpec, Shape, Use};
use crate::domain::geometry::{Layer, Mat};
use glam::{Affine3A, Quat, Vec2, Vec3};
use std::f32::consts::{FRAC_PI_2, PI};

/// Torget: halve mål i x og z.
pub const SQUARE: Vec2 = Vec2::new(15.0, 12.0);
/// Storgata (nord–sør) og Tverrgata (øst–vest): halv bredde.
pub const MAIN: f32 = 3.5;
pub const CROSS: f32 = 3.0;
/// Portene i bymuren (midten av porten, i bakkehøyde fra terrenget).
pub const SOUTH_GATE: Vec2 = Vec2::new(0.0, 52.0);
pub const NORTH_GATE: Vec2 = Vec2::new(0.0, -52.0);
/// Krypta i borgruinen: midten av inngangen, og hvilken vei døra vender.
pub const CRYPT: Vec2 = Vec2::new(0.0, -128.0);
/// Åkrene utenfor muren (midte, radier).
pub const FIELDS: [(f32, f32, f32, f32); 3] = [(-95.0, 40.0, 30.0, 20.0), (90.0, 30.0, 26.0, 34.0), (70.0, -70.0, 24.0, 18.0)];

/// Det byen forteller resten av verden.
pub struct Town {
    pub spawn: Vec3,
    pub spawn_yaw: f32,
    /// Der kjelleren ligger under (x, z).
    pub crypt_anchor: Vec2,
    /// Der du står når du kommer opp fra kjelleren.
    pub crypt_exit: (Vec3, f32),
}

/// Et hus på en tomt: modell, midten av grunnflaten (x, z) og yaw.
struct Lot {
    model: &'static str,
    pos: Vec2,
    yaw: f32,
}

const W: f32 = FRAC_PI_2;
const E: f32 = -FRAC_PI_2;
const N: f32 = PI;
const S: f32 = 0.0;

/// Tomtene. `yaw` vender døra mot gata: S er døra mot sør (+z), W mot vest.
fn lots() -> Vec<Lot> {
    let l = |model, x, z, yaw| Lot { model, pos: Vec2::new(x, z), yaw };
    vec![
        // Rundt torget.
        l("forge", 22.0, 6.0, W + PI),
        l("house_b", 21.0, -6.5, W + PI),
        l("apothecary", -21.0, 6.0, W),
        l("house_a", -21.0, -6.5, W),
        l("scriptorium", 8.5, -18.5, S),
        l("house_c", -8.5, -18.0, S),
        l("tavern", -8.5, 18.5, N),
        l("house_d", 8.5, 18.0, N),
        // Storgata nordover.
        l("house_e", -10.5, -29.0, E + PI),
        l("house_a", -10.5, -40.0, E + PI),
        l("house_c", 10.5, -29.0, E),
        l("house_f", 10.5, -40.0, E),
        // Storgata sørover.
        l("house_d", -10.5, 29.0, E + PI),
        l("house_b", -10.5, 40.0, E + PI),
        l("house_f", 10.5, 29.0, E),
        l("house_e", 10.5, 40.0, E),
        // Tverrgata østover og vestover.
        l("house_a", 33.0, -10.5, S),
        l("house_d", 44.0, -10.0, S),
        l("house_c", 33.0, 10.5, N),
        l("house_b", 44.0, 10.0, N),
        l("house_f", -33.0, -10.5, S),
        l("house_e", -44.0, -10.0, S),
        l("house_a", -33.0, 10.5, N),
        l("house_c", -44.0, 10.0, N),
    ]
}

/// Hvem som står hvor, og hva de sier.
const SMITH: &[&str] = &[
    "Knappesmeden, til tjeneste. Jeg smir det du bærer opp fra krypta.",
    "Et sverd trenger en klinge og et grep: en divider og en knapp. Resten er håndlag.",
    "Kom tilbake med komponenter, så får vi se hva stålet duger til.",
];
const CLERK: &[&str] = &[
    "Skriptoriet. Ingenting smis uten et korrekt utfylt skjema.",
    "Oppskriftene er skjemaer. Fyll ut feltene med komponentene du har vunnet, og send.",
    "Bøkene her er dokumentasjonen. Les om det du bærer, så leges sårene.",
];
const APOTHECARY: &[&str] = &[
    "Apoteket. Drikker mot det meste, tokens mot alt.",
    "Les i bøkene når du er skadet. Kunnskap leger.",
    "Hold deg unna lavaen nederst i dypet. Den står ikke i noen bok.",
];
const GORVEK: &[&str] = &[
    "Gorvek. Jeg lærer deg trekk, om du har tokens og tålmodighet.",
    "Blokk i det bittet kommer. Da står de lamslått, og du hugger dobbelt.",
    "Unna med Shift. Det koster pust, men sparer blod.",
];
const INNKEEPER: &[&str] = &[
    "Velkommen til Vertshuset. Øl er det nok av, nyheter også.",
    "De sier krypta under borgen går dypere enn noen har gått.",
    "Det kommer rare ting opp fra dypet om natta. Knapper med tenner.",
];

pub fn build(b: &mut Builder) -> Town {
    for lot in lots() {
        building(b, lot.model, lot.pos, lot.yaw);
    }
    wall(b);
    square(b);
    church(b);
    yard(b);
    let crypt_exit = castle(b);
    people(b);
    let spawn = Vec3::new(0.0, 0.0, 44.0);
    Town { spawn, spawn_yaw: 0.0, crypt_anchor: CRYPT, crypt_exit }
}

/// Et hus: modellen på tomta, og en boks rundt grunnflaten så ingen går
/// gjennom veggene. Bakken under er planert inne i byen, så huset står i
/// terrenghøyden i midten; sokkelen i modellen tar resten.
fn building(b: &mut Builder, model: &'static str, pos: Vec2, yaw: f32) {
    let base = b.on_ground(Vec3::new(pos.x, 0.0, pos.y));
    if !b.place(model, at(base, yaw)) {
        return;
    }
    if let Some((lo, hi)) = b.bounds(model) {
        let size = hi - lo;
        let mid = (lo + hi) * 0.5;
        let c = at(base, yaw).transform_point3(Vec3::new(mid.x, size.y * 0.5, mid.z));
        b.solid_box(c, size, yaw);
    }
}

/// Bymuren: segmenter langs kanten av byen (`terrain::town_edge` = 0), med
/// tårn i hjørnene og porthus der Storgata går gjennom.
fn wall(b: &mut Builder) {
    let (hx, hz, r) = (44.0f32, 40.0f32, 12.0f32);
    // Gå rundt den avrundede firkanten med jevne steg.
    let mut path: Vec<Vec2> = vec![];
    let corners = [(hx, hz, 0.0f32), (-hx, hz, FRAC_PI_2), (-hx, -hz, PI), (hx, -hz, PI + FRAC_PI_2)];
    for (cx, cz, a0) in corners {
        for k in 0..=8 {
            let a = a0 + FRAC_PI_2 * k as f32 / 8.0;
            path.push(Vec2::new(cx + a.cos() * r, cz + a.sin() * r));
        }
    }
    path.push(path[0]);
    let seg_len = 6.0;
    let mut pts: Vec<Vec2> = vec![path[0]];
    let mut carry = 0.0;
    for w in path.windows(2) {
        let (a, c) = (w[0], w[1]);
        let d = (c - a).length();
        let mut t = seg_len - carry;
        while t <= d {
            pts.push(a + (c - a) * (t / d));
            t += seg_len;
        }
        carry = d - (t - seg_len);
    }
    let gates = [SOUTH_GATE, NORTH_GATE];
    for w in pts.windows(2) {
        let (a, c) = (w[0], w[1]);
        let mid = (a + c) * 0.5;
        if gates.iter().any(|g| (mid - *g).length() < 6.5) {
            continue;
        }
        let dir = (c - a).normalize();
        // Murens lokale +X går langs muren; utsida er lokal −Z.
        let yaw = (-dir.y).atan2(dir.x);
        let base = b.on_ground(Vec3::new(mid.x, 0.0, mid.y));
        if b.place("town_wall", at(base, yaw)) {
            b.solid_box(base + Vec3::Y * 3.0, Vec3::new((c - a).length() + 0.2, 6.0, 1.6), yaw);
        }
    }
    for (cx, cz, _) in corners {
        let p = Vec2::new(cx, cz) + Vec2::new(cx.signum(), cz.signum()) * r * std::f32::consts::FRAC_1_SQRT_2;
        let base = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        if b.place("wall_tower", at(base, (cz.signum()).atan2(cx.signum()))) {
            b.solid(Shape::Cylinder { center: base + Vec3::Y * 6.0, radius: 3.4, half_height: 6.0 });
        }
    }
    for (g, yaw) in [(SOUTH_GATE, 0.0), (NORTH_GATE, PI)] {
        let base = b.on_ground(Vec3::new(g.x, 0.0, g.y));
        if b.place("gatehouse", at(base, yaw)) {
            // To tårn på sidene; porten mellom er åpen.
            for sx in [-1.0f32, 1.0] {
                let side = at(base, yaw).transform_point3(Vec3::new(sx * 4.6, 5.0, 0.0));
                b.solid_box(side, Vec3::new(4.0, 10.0, 5.0), yaw);
            }
        }
    }
}

/// Torget: torgkorset midt på, markedsbodene i rekker, kjerrer, tønner og
/// kasser ved bodene.
fn square(b: &mut Builder) {
    let cross = b.on_ground(Vec3::new(-5.5, 0.0, -4.0));
    if b.place("market_cross", at(cross, 0.3)) {
        b.solid(Shape::Cylinder { center: cross + Vec3::Y * 1.5, radius: 0.9, half_height: 1.5 });
    }
    let stalls = [(-9.0, 7.0, 0.0), (-4.0, 7.5, 0.1), (6.0, 7.0, -0.05), (10.5, -6.0, PI), (6.0, -7.5, PI + 0.1)];
    for (k, (x, z, yaw)) in stalls.into_iter().enumerate() {
        let base = b.on_ground(Vec3::new(x, 0.0, z));
        let model = if k % 2 == 0 { "market_stall" } else { "market_stall_b" };
        if b.place(model, at(base, yaw)) {
            b.solid_box(base + Vec3::Y * 0.5, Vec3::new(2.6, 1.0, 1.4), yaw);
        }
    }
    let props = [("cart", -11.5, -6.0, 0.6), ("barrel", -7.4, 8.6, 0.0), ("barrel", -6.7, 9.1, 1.0), ("crate", 4.2, 8.8, 0.3), ("crate", 4.5, 9.6, 1.1), ("barrel", 12.0, -7.6, 2.0)];
    for (model, x, z, yaw) in props {
        let base = b.on_ground(Vec3::new(x, 0.0, z));
        if b.place(model, at(base, yaw)) {
            if let Some((lo, hi)) = b.bounds(model) {
                let size = hi - lo;
                b.solid_box(base + Vec3::Y * size.y * 0.5, Vec3::new(size.x, size.y, size.z), yaw);
            }
        }
    }
    // Lykter på stolper langs torget, som lyser om natta.
    for (x, z) in [(-14.0, -11.0), (14.0, -11.0), (-14.0, 11.0), (14.0, 11.0), (0.0, 30.0), (0.0, -32.0)] {
        let base = b.on_ground(Vec3::new(x, 0.0, z));
        if b.place("lantern_post", at(base, 0.0)) {
            b.solid(Shape::Cylinder { center: base + Vec3::Y * 1.4, radius: 0.15, half_height: 1.4 });
            b.light(base + Vec3::Y * 2.7, Vec3::new(1.0, 0.7, 0.4), 2.4, 11.0);
        }
    }
}

/// Stavkirka på kirkegården nordvest i byen, med steingjerde og gravsteiner.
fn church(b: &mut Builder) {
    let c = Vec2::new(-30.0, -30.0);
    let base = b.on_ground(Vec3::new(c.x, 0.0, c.y));
    if b.place("stave_church", at(base, FRAC_PI_2)) {
        b.solid_box(base + Vec3::Y * 4.0, Vec3::new(9.0, 8.0, 14.0), FRAC_PI_2);
    }
    for k in 0..7 {
        let p = Vec2::new(-22.0 + (k % 4) as f32 * 1.6, -24.0 - (k / 4) as f32 * 2.2);
        let g = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        b.place("gravestone", at(g, 0.1 * k as f32 - 0.2));
    }
}

/// Gorveks øvingsplass nordøst i byen: treningsdukker og et våpenstativ.
fn yard(b: &mut Builder) {
    for (x, z, yaw) in [(28.0, -30.0, 0.4), (33.0, -32.0, -0.3), (37.0, -28.0, 0.1)] {
        let base = b.on_ground(Vec3::new(x, 0.0, z));
        if b.place("training_dummy", at(base, yaw)) {
            b.solid(Shape::Cylinder { center: base + Vec3::Y * 0.9, radius: 0.35, half_height: 0.9 });
        }
    }
    let rack = b.on_ground(Vec3::new(32.0, 0.0, -24.0));
    if b.place("weapon_rack", at(rack, 0.0)) {
        b.solid_box(rack + Vec3::Y * 0.8, Vec3::new(2.0, 1.6, 0.6), 0.0);
    }
}

/// Borgruinen på haugen, og krypta med døra ned. Gir stedet du kommer
/// opp til fra kjelleren.
fn castle(b: &mut Builder) -> (Vec3, f32) {
    let (hx, hz, _, _) = terrain::HILL;
    let keep = b.on_ground(Vec3::new(hx, 0.0, hz));
    if b.place("castle_ruin", at(keep, 0.0)) {
        // Murene i ruinen: fire løp rundt borggården, med åpning i sør.
        for (c, size) in [(Vec3::new(0.0, 4.0, -20.0), Vec3::new(40.0, 8.0, 2.4)), (Vec3::new(-20.0, 4.0, 0.0), Vec3::new(2.4, 8.0, 40.0)), (Vec3::new(20.0, 4.0, 0.0), Vec3::new(2.4, 8.0, 40.0)), (Vec3::new(-12.5, 4.0, 20.0), Vec3::new(15.0, 8.0, 2.4)), (Vec3::new(12.5, 4.0, 20.0), Vec3::new(15.0, 8.0, 2.4))] {
            b.solid_box(keep + c, size, 0.0);
        }
    }
    // Krypta: et lite gravkapell med døra mot sør, og trappa ned bak døra.
    let crypt = b.on_ground(Vec3::new(CRYPT.x, 0.0, CRYPT.y));
    if b.place("crypt", at(crypt, 0.0)) {
        b.solid_box(crypt + Vec3::new(0.0, 2.5, -2.0), Vec3::new(6.0, 5.0, 6.0), 0.0);
    }
    // Døra: det du sikter på. Står i forkant av kapellet, vendt mot sør.
    b.target(Use::Descend, at(crypt, 0.0), Vec3::new(0.0, 1.2, 1.05), Vec3::new(0.75, 1.2, 0.2));
    b.fire_light(crypt + Vec3::new(1.3, 2.4, 1.6), Vec3::new(1.0, 0.6, 0.3), 2.0, 8.0);
    b.fire_light(crypt + Vec3::new(-1.3, 2.4, 1.6), Vec3::new(1.0, 0.6, 0.3), 2.0, 8.0);
    (crypt + Vec3::new(0.0, 0.0, 3.2), PI)
}

/// Innbyggerne ved husene sine.
fn people(b: &mut Builder) {
    let cast: [(&'static str, Vec2, f32, Vec3, &'static [&'static str]); 5] = [
        ("Knappesmeden", Vec2::new(16.4, 8.0), E, Vec3::new(0.22, 0.1, 0.05), SMITH),
        ("Skriptoriet", Vec2::new(10.0, -13.6), S, Vec3::new(0.06, 0.12, 0.42), CLERK),
        ("Apotekeren", Vec2::new(-16.4, 8.0), W, Vec3::new(0.06, 0.28, 0.1), APOTHECARY),
        ("Vertinnen", Vec2::new(-7.0, 13.6), N, Vec3::new(0.45, 0.12, 0.1), INNKEEPER),
        ("Gorvek", Vec2::new(31.0, -21.0), N + 0.3, Vec3::new(0.3, 0.1, 0.04), GORVEK),
    ];
    for (i, (name, p, yaw, tint, lines)) in cast.into_iter().enumerate() {
        let pos = b.on_ground(Vec3::new(p.x, 0.0, p.y));
        log::info!("Innbygger {name} ved ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z);
        b.npcs.push(NpcSpec { name, pos, yaw, tint, lines });
        b.target(Use::Talk(i), Affine3A::from_translation(pos), Vec3::new(0.0, 0.9, 0.0), Vec3::new(0.3, 0.9, 0.3));
    }
}

/// Hva bakken er dekket av i et punkt: brostein på torget og i Storgata,
/// tråkket jord i Tverrgata og på veiene, gress ellers.
pub fn ground_mat(x: f32, z: f32) -> Mat {
    let inside = terrain::town_edge(x, z) < -1.0;
    let in_square = x.abs() < SQUARE.x && z.abs() < SQUARE.y;
    let in_main = x.abs() < MAIN && inside;
    if in_square || in_main {
        return Mat::tex(Layer::Cobble, 0xffffff);
    }
    let edge = 1.4 + terrain::noise(x * 0.2, z * 0.2) * 1.4;
    let in_cross = z.abs() < CROSS + 0.5 && inside;
    if in_cross || path_dist(x, z) < edge {
        return Mat::tex(Layer::Gravel, 0xb8a080);
    }
    // Inne i byen er bakken tråkket rundt husene og grønn i hagene.
    if inside && terrain::noise(x * 0.15, z * 0.15) > 0.42 {
        return Mat::tex(Layer::Gravel, 0x9c8a6c);
    }
    Mat::tex(Layer::Grass, super::meadow_tint(x, z))
}

/// Avstand fra et punkt til veiene utenfor muren: sørover fra sørporten,
/// og opp til borgen fra nordporten.
pub fn path_dist(x: f32, z: f32) -> f32 {
    let mut d = f32::MAX;
    if z > SOUTH_GATE.y - 2.0 {
        let n = terrain::noise(z * 0.03, 9.0);
        d = d.min((x - (n - 0.5) * 30.0 * ((z - SOUTH_GATE.y) / 60.0).clamp(0.0, 1.0)).abs());
    }
    if z < NORTH_GATE.y + 2.0 && z > CRYPT.y - 4.0 {
        // Svinger litt mens den klatrer, og ender ved krypta.
        let t = ((NORTH_GATE.y - z) / (NORTH_GATE.y - CRYPT.y)).clamp(0.0, 1.0);
        d = d.min((x - (t * PI).sin() * 6.0).abs());
    }
    d
}

/// Avstand fra et punkt til det bygde: byen innenfor muren, og borghaugen.
/// Der holder enga, trærne og dyrene seg unna.
pub fn town_dist(x: f32, z: f32) -> f32 {
    let (hx, hz, _, _) = terrain::HILL;
    let castle = ((x - hx).powi(2) + (z - hz).powi(2)).sqrt() - terrain::KEEP_FLAT - 2.0;
    terrain::town_edge(x, z).min(castle)
}

/// Stigen opp fra startrommet i kjelleren ligger i dungeon.rs; her er bare
/// hvor torget er, til testene.
pub fn square_center() -> Vec3 {
    terrain::at(Vec3::ZERO)
}

#[allow(dead_code)]
fn turn(yaw: f32) -> Quat {
    Quat::from_rotation_y(yaw)
}
