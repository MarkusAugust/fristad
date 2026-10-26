//! Det fiendene mister: tokenene (glødende brikker i LO-TEKs farger som
//! spretter ut, faller og suges mot deg; eieren 13. oktober) og nøklene.

use crate::domain::scene::ModelInstance;
use crate::domain::world::crypt::Crypt;
use glam::{Affine3A, Quat, Vec3};

pub const TOKEN_MODEL: &str = "token";

#[derive(Clone, Debug)]
pub struct Token {
    pub pos: Vec3,
    pub age: f32,
    pub taken: bool,
    /// Farten: tokenene spretter ut når fienden dør, faller, og suges mot
    /// deg når du er nær (nøklene ligger stille).
    pub vel: Vec3,
    /// Hvilken LO-TEK-farge (`TOKEN_COLORS`).
    pub color: u8,
}

/// Fargene tokenene gløder i: fyllfargene i LO-TEKs mørke tema (0.31),
/// accent, brand 1–3, success, warning og danger.
pub const TOKEN_COLORS: [Vec3; 7] = [
    Vec3::new(0x49 as f32 / 255.0, 0x92 as f32 / 255.0, 0xe2 as f32 / 255.0),
    Vec3::new(0x45 as f32 / 255.0, 0xa3 as f32 / 255.0, 0x86 as f32 / 255.0),
    Vec3::new(0x93 as f32 / 255.0, 0x7b as f32 / 255.0, 0xe2 as f32 / 255.0),
    Vec3::new(0xd2 as f32 / 255.0, 0x6a as f32 / 255.0, 0x79 as f32 / 255.0),
    Vec3::new(0x62 as f32 / 255.0, 0xa1 as f32 / 255.0, 0x5b as f32 / 255.0),
    Vec3::new(0xae as f32 / 255.0, 0x89 as f32 / 255.0, 0x3a as f32 / 255.0),
    Vec3::new(0xdf as f32 / 255.0, 0x61 as f32 / 255.0, 0x65 as f32 / 255.0),
];
/// Innenfor denne avstanden suges tokenene mot deg.
pub const TOKEN_PULL: f32 = 3.0;

/// Flytter et token ett steg (faller, spretter, suges mot `chest` når du
/// er nær) og gir om du tok det nå.
pub fn update_token(t: &mut Token, dt: f32, chest: Vec3, in_crypt: bool, crypt: &Crypt) -> bool {
    t.age += dt;
    if t.taken {
        return false;
    }
    let to = chest - t.pos;
    let d = to.length();
    if in_crypt && t.age > 0.5 && d < TOKEN_PULL {
        // Suges mot deg, fortere jo nærmere.
        let pull = (1.0 - d / TOKEN_PULL).max(0.0);
        t.vel = t.vel.lerp(to / d.max(1e-3) * (4.0 + 10.0 * pull), (dt * 6.0).min(1.0));
        t.pos += t.vel * dt;
        if d < 0.45 {
            t.taken = true;
            return true;
        }
        return false;
    }
    // Ellers faller det og blir liggende på gulvet.
    t.vel.y -= 9.0 * dt;
    let next = t.pos + t.vel * dt;
    match crypt.floor_under(t.pos + Vec3::Y * 0.3) {
        Some(f) if next.y < f + 0.12 => {
            t.pos = Vec3::new(next.x, f + 0.12, next.z);
            t.vel = Vec3::new(t.vel.x * 0.4, (-t.vel.y * 0.3).max(0.0), t.vel.z * 0.4);
            if t.vel.y < 0.4 {
                t.vel = Vec3::ZERO;
            }
        }
        _ => t.pos = next,
    }
    false
}

/// Tokenet til rendereren: snurrer, svever litt når det ligger, og gløder i
/// sin farge.
pub fn token_instance(t: &Token, time: f32) -> ModelInstance {
    let spin = Quat::from_rotation_y(time * 2.0 + t.age * 3.0);
    let pop = (t.age * 4.0).min(1.0);
    let hover = if t.vel == Vec3::ZERO { 0.08 + (time * 3.0 + t.age).sin() * 0.04 } else { 0.0 };
    let tint = Some(TOKEN_COLORS[t.color as usize % TOKEN_COLORS.len()]);
    ModelInstance { model: TOKEN_MODEL, transform: Affine3A::from_rotation_translation(spin, t.pos + Vec3::Y * hover) * Affine3A::from_scale(Vec3::splat(pop)), skin: None, flash: 0.0, tint, look: None }
}

/// Hvor nøkkelen svever: 0,25 m over der den ligger, opp og ned.
pub fn key_at(k: &Token, time: f32) -> Vec3 {
    k.pos + Vec3::Y * (0.25 + (time * 2.5 + k.age).sin() * 0.06)
}

/// Hvor mye større nøkkelen vises der den svever enn de 0,15 m den er, så
/// den synes på avstand i mørket (som tokenene).
pub const KEY_SHOW: f32 = 1.8;

/// Nøkkelen (`key`, TRELLIS gjennom `item3d.py key`: hanken opp, foten i
/// origo) til rendereren: står og snurrer, litt på skrå, der den svever.
pub fn key_instance(k: &Token, time: f32) -> ModelInstance {
    let spin = Quat::from_rotation_y(time * 1.5 + k.age) * Quat::from_rotation_x(0.3);
    let pop = (k.age * 4.0).min(1.0) * KEY_SHOW;
    ModelInstance { model: crate::domain::mobs::KEY_MODEL, transform: Affine3A::from_rotation_translation(spin, key_at(k, time)) * Affine3A::from_scale(Vec3::splat(pop)), skin: None, flash: 0.0, tint: None, look: None }
}
