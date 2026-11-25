//! Kameraet redigeringen ser gjennom: øyet, retningen og matrisen bildet
//! tegnes med, og regningen mellom skjermen og verden. Økta lager det
//! (`Session::camera`), og redigeringen får det som argument.

use glam::{Mat4, Vec2, Vec3, Vec4};

#[derive(Clone, Copy)]
pub struct Camera {
    pub eye: Vec3,
    pub forward: Vec3,
    pub yaw: f32,
    pub view_proj: Mat4,
    pub aspect: f32,
}

impl Camera {
    /// Strålen gjennom et punkt på skjermen (0..1 fra øverst til venstre).
    pub fn ray(&self, at: (f32, f32)) -> (Vec3, Vec3) {
        let inv = self.view_proj.inverse();
        let ndc = Vec2::new(at.0 * 2.0 - 1.0, 1.0 - at.1 * 2.0);
        let near = inv * Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
        let far = inv * Vec4::new(ndc.x, ndc.y, 1.0, 1.0);
        let (a, b) = (near.truncate() / near.w, far.truncate() / far.w);
        (self.eye, (b - a).normalize())
    }

    /// Strålen gjennom pekeren, eller rett fram uten peker.
    pub fn ray_or_forward(&self, at: Option<(f32, f32)>) -> (Vec3, Vec3) {
        match at {
            Some(at) => self.ray(at),
            None => (self.eye, self.forward),
        }
    }

    /// Et punkt i verden på skjermen, i enheter av skjermhøyden (så x og y
    /// er like lange), eller `None` bak kameraet.
    pub fn on_screen(&self, p: Vec3) -> Option<Vec2> {
        let c = self.view_proj * p.extend(1.0);
        if c.w <= 1e-4 {
            return None;
        }
        let n = c.truncate() / c.w;
        Some(Vec2::new((n.x + 1.0) * 0.5 * self.aspect, (1.0 - n.y) * 0.5))
    }

    /// Et punkt i verden på skjermen (0..1 fra øverst til venstre).
    pub fn to_screen(&self, p: Vec3) -> Option<(f32, f32)> {
        self.on_screen(p).map(|v| (v.x / self.aspect, v.y))
    }

    /// Et punkt på skjermen i enheter av skjermhøyden.
    pub fn point(&self, at: (f32, f32)) -> Vec2 {
        Vec2::new(at.0 * self.aspect, at.1)
    }

    /// Hvor langt punktet `m` på skjermen er fra streken a–b i verden.
    pub fn distance(&self, m: Vec2, a: Vec3, b: Vec3) -> Option<f32> {
        let (a, b) = (self.on_screen(a)?, self.on_screen(b)?);
        let ab = b - a;
        let t = ((m - a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
        Some((m - (a + ab * t)).length())
    }
}
