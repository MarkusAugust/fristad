//! Fallskjermen: kalotten med tolv kiler i aksentfargen og hvitt, og
//! linene ned til selen. Bygges én gang som en prop uten eier; spillet
//! setter posisjonen over spilleren mens hun daler, og gjemmer den ellers.
//! Origo er selen, rett over hodet.

use super::Builder;
use crate::domain::geometry::{flag, Mat, Mesh};
use crate::domain::palette::slot;
use glam::{Affine3A, Quat, Vec2, Vec3};

pub const RADIUS: f32 = 3.2;
/// Kalottens sentrum over selen.
pub const HEIGHT: f32 = 5.4;

pub fn build(b: &mut Builder) {
    let mut m = Mesh::default();
    let blue = Mat::plain(0xffffff, 0.7).slot(slot::ACCENT_FILL).flags(flag::NO_DIRT | flag::NO_WEATHER);
    let white = Mat::plain(0xf4f1ea, 0.7).flags(flag::NO_DIRT | flag::NO_WEATHER);
    let gores = 12;
    let rings = 6;
    // Kalotten: en flattrykt halvkule, åpen nederst, synlig fra begge sider.
    let point = |phi: f32, theta: f32| Vec3::new(RADIUS * phi.sin() * theta.cos(), HEIGHT + RADIUS * 0.75 * phi.cos(), RADIUS * phi.sin() * theta.sin());
    let normal = |phi: f32, theta: f32| Vec3::new(phi.sin() * theta.cos(), 0.75 * phi.cos(), phi.sin() * theta.sin()).normalize();
    let phi_max = 1.7;
    for g in 0..gores {
        let mat = if g % 2 == 0 { &blue } else { &white };
        let t0 = g as f32 / gores as f32 * std::f32::consts::TAU;
        let t1 = (g + 1) as f32 / gores as f32 * std::f32::consts::TAU;
        for k in 0..rings {
            let p0 = k as f32 / rings as f32 * phi_max;
            let p1 = (k + 1) as f32 / rings as f32 * phi_max;
            let corners = [(p0, t0), (p0, t1), (p1, t1), (p1, t0)];
            for outside in [true, false] {
                let mut idx = [0u32; 4];
                for (i, (phi, theta)) in corners.iter().enumerate() {
                    let n = normal(*phi, *theta) * if outside { 1.0 } else { -1.0 };
                    idx[i] = m.vert(point(*phi, *theta), n, Vec2::new(*theta / std::f32::consts::TAU, *phi / phi_max), mat, 1.0);
                }
                if outside {
                    m.indices.extend_from_slice(&[idx[0], idx[2], idx[1], idx[0], idx[3], idx[2]]);
                } else {
                    m.indices.extend_from_slice(&[idx[0], idx[1], idx[2], idx[0], idx[2], idx[3]]);
                }
            }
        }
    }
    // Linene fra kanten til selen, og selen selv.
    let line = Mat::plain(0xe8e4da, 0.6).flags(flag::NO_DIRT | flag::NO_WEATHER);
    let harness = Vec3::new(0.0, 0.4, 0.0);
    for g in 0..gores {
        let theta = (g as f32 + 0.5) / gores as f32 * std::f32::consts::TAU;
        let rim = point(phi_max, theta);
        let dir = rim - harness;
        let t = Affine3A::from_rotation_translation(Quat::from_rotation_arc(Vec3::Y, dir.normalize()), harness);
        m.cylinder(&t, 0.012, 0.012, dir.length(), 4, &line, false);
    }
    // Stroppene opp fra selen, som to tynne bøyler.
    for sx in [-1.0f32, 1.0] {
        m.cuboid(&Affine3A::from_translation(Vec3::new(sx * 0.22, 0.2, 0.0)), Vec3::new(0.04, 0.4, 0.03), &Mat::plain(0x2b2d30, 0.7).flags(flag::NO_DIRT | flag::NO_WEATHER));
    }
    let _ = b.prop("parachute", m, Affine3A::from_scale(Vec3::splat(0.0001)), None);
}
