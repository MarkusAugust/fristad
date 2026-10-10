//! Bildet: det rendereren får per bilde fra økta. Sola, himmelen, tåka og
//! lysene følger døgnet, været og om du er nede i kjelleren.

use super::session::Session;
use crate::domain::scene::{Frame, Light, ModelInstance, MAX_SMOKE, PALETTE};
use crate::domain::world::dungeon::PORTCULLIS;
use glam::{Mat4, Vec3, Vec4};

impl Session {
    pub fn frame(&self, aspect: f32) -> Frame {
        self.aspect.set(aspect);
        let mut skins = vec![];
        // Kameraet sparkes og skjelver når vi treffes (bare det du ser,
        // ikke retningen du går).
        let (ky, kp) = self.combat.view_kick(self.time);
        let (vyaw, vpitch) = (self.yaw + ky, (self.pitch + kp).clamp(-1.5, 1.5));
        let view_dir = Vec3::new(-vyaw.sin() * vpitch.cos(), vpitch.sin(), -vyaw.cos() * vpitch.cos());
        let in_cellar = self.in_cellar();
        let t = self.night;
        let dusk = (t * std::f32::consts::PI).sin();
        let lerp3 = |a: Vec3, b: Vec3| a.lerp(b, t);
        let w = self.weather;
        let overcast = w.cloud.clamp(0.0, 1.0);

        // Sola står lavt i sørvest om dagen. Mens døgnet snur går den ned,
        // og månen kommer opp i sørøst.
        let dir = |az: f32, el: f32| {
            let (az, el) = (az.to_radians(), el.to_radians());
            Vec3::new(az.sin() * el.cos(), el.sin(), az.cos() * el.cos())
        };
        let el = 27.0 * (1.0 - t) + 38.0 * t - dusk * 24.0;
        let az = -68.0 * (1.0 - t) + 42.0 * t;
        let sun_dir = dir(az, el.max(3.0));

        let day_sun = Vec3::new(1.0, 0.88, 0.68);
        let dusk_sun = Vec3::new(1.0, 0.48, 0.22);
        let moon = Vec3::new(0.55, 0.66, 1.0);
        let sun_color = lerp3(day_sun, moon).lerp(dusk_sun, dusk * 0.85);
        let sun_intensity = (3.4 * (1.0 - t) + 0.38 * t) * (1.0 - overcast * 0.8);

        let grey = |c: Vec3| c.lerp(Vec3::splat(c.dot(Vec3::new(0.3, 0.59, 0.11))), overcast * 0.6) * (1.0 - overcast * 0.25);
        let sky_top = grey(lerp3(Vec3::new(0.16, 0.34, 0.72), Vec3::new(0.008, 0.014, 0.04)).lerp(Vec3::new(0.18, 0.2, 0.42), dusk * 0.6));
        let sky_horizon = grey(lerp3(Vec3::new(0.6, 0.74, 0.88), Vec3::new(0.016, 0.028, 0.06)).lerp(Vec3::new(1.0, 0.5, 0.28), dusk * 0.7));
        let ground = lerp3(Vec3::new(0.28, 0.26, 0.22), Vec3::new(0.012, 0.013, 0.018)) * (1.0 - overcast * 0.3);
        let snow_bounce = Vec3::new(0.5, 0.52, 0.58) * w.snow_cover * (1.0 - t * 0.95);

        // Lyktene tennes i skumringen og når det er mørkt av skyer.
        let lamp = ((t - 0.35) / 0.5).clamp(0.0, 1.0).max(overcast * 0.5 * (1.0 - t)) * 0.6 + ((t - 0.35) / 0.5).clamp(0.0, 1.0) * 0.4;
        let eye_now = self.eye();
        let mut lights: Vec<Light> = self
            .lights
            .iter()
            .enumerate()
            .map(|(i, l)| {
                // Flakket: ild blafrer mykt, det som er i ferd med å ryke slukker i rykk.
                let mut k = 1.0;
                if l.flicker > 0.0 {
                    let t = self.time * 9.0 + i as f32 * 1.7;
                    let soft = 1.0 - l.flicker * 0.5 * (0.5 + 0.5 * (t.sin() * (t * 0.37).cos() + (t * 2.3).sin() * 0.5));
                    let blackout = if l.flicker > 0.4 && ((t * 0.31).sin() * (t * 0.173 + i as f32).cos()) > 0.72 { 0.1 } else { 1.0 };
                    k = soft * blackout;
                }
                Light { intensity: l.intensity * k * if l.always { 1.0 } else { lamp }, ..*l }
            })
            .filter(|l| l.intensity > 0.01)
            .collect();
        // Nede i kjelleren bærer du et lys, så du ser våpenet og det som
        // kommer mot deg.
        if in_cellar {
            lights.push(Light { pos: eye_now + view_dir * 0.4 + Vec3::Y * 0.2, radius: 6.0 * self.gear().lantern.sqrt(), color: Vec3::new(1.0, 0.8, 0.6), intensity: 0.8 * self.gear().lantern, always: true, flicker: 0.0 });
        }
        lights.extend(self.portal_light());
        // Rendereren tar de første 48: de nærmeste først.
        lights.sort_by(|a, b| a.pos.distance_squared(eye_now).total_cmp(&b.pos.distance_squared(eye_now)));
        // Nede i kjelleren: lite himmellys, mer tåke, øynene vant til mørket.
        let c = self.cellar_t;

        // Røyken fra de nærmeste pipene; tykkere om kvelden, når det fyres
        // til mat og varme. Nede i kjelleren ser du ingen piper.
        let mut smoke: Vec<Vec4> = if in_cellar {
            vec![]
        } else {
            self.smoke.iter().map(|(p, s)| p.extend(s * (0.75 + 0.25 * t))).collect()
        };
        smoke.sort_by(|a, b| a.truncate().distance_squared(eye_now).total_cmp(&b.truncate().distance_squared(eye_now)));
        smoke.truncate(MAX_SMOKE);
        let eye = self.eye();
        let mut instances = self.herd.instances(self.time);
        instances.extend(self.npcs.instances(&mut skins));
        instances.extend(self.mobs.instances(self.time, &mut skins));
        instances.extend(self.well_poses().into_iter().map(|(model, transform)| ModelInstance { model, transform, skin: None, flash: 0.0, tint: None }));
        instances.extend(self.portal_poses().into_iter().map(|(model, transform)| ModelInstance { model, transform, skin: None, flash: 0.0, tint: None }));
        instances.extend(self.gate_poses().into_iter().map(|transform| ModelInstance { model: PORTCULLIS, transform, skin: None, flash: 0.0, tint: None }));
        instances.extend(self.shortcut_poses().into_iter().map(|transform| ModelInstance { model: crate::domain::world::dungeon::SHORTCUT_LEAF, transform, skin: None, flash: 0.0, tint: None }));
        instances.extend(self.chest_poses().into_iter().map(|transform| ModelInstance { model: crate::domain::world::dungeon::CHEST_LID, transform, skin: None, flash: 0.0, tint: None }));
        instances.extend(self.gallery_poses().into_iter().map(|(transform, tint)| ModelInstance { model: crate::domain::world::GALLERY_PIECE, transform, skin: None, flash: 0.0, tint: Some(tint) }));
        instances.extend(self.combat.instances(eye, vyaw, vpitch, in_cellar, &mut skins));
        Frame {
            view: Mat4::look_to_rh(eye, view_dir, Vec3::Y),
            proj: Mat4::perspective_rh(self.fov().to_radians(), aspect, 0.1, 2500.0),
            camera: eye,
            time: self.time,
            sun_dir,
            sun_color,
            sun_intensity: sun_intensity * (1.0 - c * 0.85),
            night: t,
            sky_top: sky_top * (1.0 - c * 0.9),
            sky_horizon: sky_horizon * (1.0 - c * 0.9),
            ground_bounce: (ground + snow_bounce) * (1.0 - c * 0.8),
            fog_color: grey(lerp3(Vec3::new(0.62, 0.74, 0.88), Vec3::new(0.012, 0.02, 0.04))) * (1.0 - c * 0.95),
            // Høydetåka tetner nedover. I kjelleren er det ingen tåke.
            fog_density: (0.0006 + dusk * 0.0012) * self.fog_mul * (1.0 - c),
            fog_falloff: 0.03,
            exposure: (0.5 * (1.0 - t) + 2.0 * t) * (1.0 + overcast * 0.35) * (1.0 + c * 1.5),
            bloom: 0.06 + t * 0.06,
            windows_lit: 0.55,
            street_lights: 1.0,
            aurora: 1.0 - overcast,
            weather: w,
            palette: [Vec3::ONE; PALETTE],
            lights,
            smoke,
            instances,
            skins,
            hurt: self.combat.hurt,
            debug: self.debug,
            grid: if in_cellar { self.dungeon.grid_at(self.pos - Vec3::Y * 1.6) } else { None },
            band: self.band(in_cellar),
        }
    }

    /// Høydene som kan synes (`Frame.band`). Oppe: alt over taket i
    /// kjellerens øverste etasje. Nede: etasjen du står i, med vindeltrappa
    /// fra etasjen over (den går ned hit) og det som rager opp fra etasjen
    /// under (rotunden), men ikke rommene der.
    pub(super) fn band(&self, in_cellar: bool) -> (f32, f32) {
        use crate::domain::world::dungeon::DEPTH;
        if !in_cellar {
            return (DEPTH + 9.0, f32::INFINITY);
        }
        let feet = self.pos - Vec3::Y * 1.6;
        match self.dungeon.floor_of(feet) {
            Some(i) => {
                let y = self.dungeon.floors[i].y;
                (y - 3.0, y + 8.0)
            }
            // Mellom etasjene (i vindeltrappa): begge.
            None => (feet.y - 14.0, feet.y + 10.0),
        }
    }

    /// Etter at bildet er tegnet: en flytting (`jumped`, som demper
    /// landingslyden) gjelder bare det ene bildet.
    pub fn settle(&mut self) {
        self.jumped = false;
    }

    /// Synsfeltet: 72°, strukket ut og tilbake når du reiser gjennom
    /// virvelen.
    pub(super) fn fov(&self) -> f32 {
        let t = self.warp_t / 1.3;
        if t < 1.0 {
            72.0 + 46.0 * (t * std::f32::consts::PI).sin().powi(2) * (1.0 - t * 0.3)
        } else {
            72.0
        }
    }
}
