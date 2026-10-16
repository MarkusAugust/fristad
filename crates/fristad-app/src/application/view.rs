//! Bildet: det rendereren får per bilde fra økta. Sola, himmelen, tåka og
//! lysene følger døgnet, været og om du er nede i krypten.

use super::session::Session;
use crate::domain::craft;
use crate::domain::scene::{Frame, Light, ModelInstance, RoomLight, MAX_LIGHTS, MAX_ROOMS, MAX_SMOKE, PALETTE};
use crate::domain::world::crypt::{self, PORTCULLIS};
use crate::domain::world::Use;
use glam::{Affine3A, Mat4, Quat, Vec3, Vec4};

impl Session {
    pub fn frame(&self, aspect: f32) -> Frame {
        self.aspect.set(aspect);
        let mut skins = vec![];
        // Kameraet sparkes og skjelver når vi treffes (bare det du ser,
        // ikke retningen du går).
        let (ky, kp) = self.combat.view_kick(self.time);
        let (vyaw, vpitch) = (self.yaw + ky, (self.pitch + kp).clamp(-1.5, 1.5));
        let view_dir = Vec3::new(-vyaw.sin() * vpitch.cos(), vpitch.sin(), -vyaw.cos() * vpitch.cos());
        let in_crypt = self.in_crypt();
        let t = self.sky.night;
        let dusk = (t * std::f32::consts::PI).sin();
        let lerp3 = |a: Vec3, b: Vec3| a.lerp(b, t);
        let w = self.sky.weather;
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
        // Nede i krypten bærer du lykta i beltet: den lyser foran deg fra
        // hofta, og dempes og blafrer når oljen går tom (`lantern.rs`). Uten
        // olje er det bare en glo, og mørket er nesten helt (eieren 13. okt.).
        if in_crypt {
            let (reach, strength) = self.hero.lamp.glow(self.time);
            let flat = Vec3::new(view_dir.x, 0.0, view_dir.z).normalize_or_zero();
            let right = flat.cross(Vec3::Y);
            lights.push(Light {
                // Litt foran deg, så lyset faller på gulvet og veggene og
                // ikke blender skjoldet og hånda.
                pos: eye_now - Vec3::Y * 1.0 + flat * 0.9 + right * 0.3,
                radius: 11.0 * self.gear().lantern.sqrt() * reach,
                color: Vec3::new(1.0, 0.7, 0.4),
                intensity: 2.4 * self.gear().lantern * strength,
                always: true,
                flicker: 0.0,
                room: None,
            });
            // Og det samme lyset nær hofta, svakt, opp på hendene og skjoldet.
            lights.push(Light {
                pos: eye_now - Vec3::Y * 0.85 + flat * 0.2 + right * 0.25,
                radius: 1.8 * reach.max(0.5),
                color: Vec3::new(1.0, 0.7, 0.4),
                intensity: 0.35 * strength,
                always: true,
                flicker: 0.0,
                room: None,
            });
            lights.extend(self.rest_lights());
        }
        lights.extend(self.portal_pose().and_then(|p| self.portal.light(p, self.time)));
        // Rommene i nivået du er i, og rommene hvert lys lyser i. Lysene som
        // ikke lyser der du står, kommer bakerst.
        let level = if in_crypt { self.crypt.level_at(eye_now - Vec3::Y * 1.6) } else { None };
        let here = level.and_then(|_| self.crypt.room_at(eye_now - Vec3::Y * 1.4)).map(|(_, r)| r);
        let mask = |l: &Light| -> u64 {
            match (l.room, level) {
                (None, _) => u64::MAX,
                (Some((lv, r)), Some(k)) if lv as usize == k => self.crypt.light_mask(k, r as usize),
                _ => 0,
            }
        };
        let rank = |l: &Light| {
            let d = l.pos.distance_squared(eye_now);
            match here {
                Some(r) if r < 64 && mask(l) & (1u64 << r) == 0 => d + 900.0,
                _ => d,
            }
        };
        // Rendereren tar de første 48: de nærmeste først.
        lights.sort_by(|a, b| rank(a).total_cmp(&rank(b)));
        lights.truncate(MAX_LIGHTS);
        let rooms = level.map(|k| RoomLight { boxes: self.crypt.levels[k].rooms.iter().take(MAX_ROOMS).map(|r| (r.min, r.max, r.y - r.sink, r.y + r.h)).collect(), masks: lights.iter().map(mask).collect() });
        // Nede i krypten: lite himmellys, mer tåke, øynene vant til mørket.
        let c = self.cellar_t;

        // Røyken fra de nærmeste pipene; tykkere om kvelden, når det fyres
        // til mat og varme. Nede i krypten ser du ingen piper.
        let mut smoke: Vec<Vec4> = if in_crypt { vec![] } else { self.smoke.iter().map(|(p, s)| p.extend(s * (0.75 + 0.25 * t))).collect() };
        smoke.sort_by(|a, b| a.truncate().distance_squared(eye_now).total_cmp(&b.truncate().distance_squared(eye_now)));
        smoke.truncate(MAX_SMOKE);
        let eye = self.eye();
        let mut instances = self.herd.instances(self.time);
        instances.extend(self.npcs.instances(&mut skins));
        instances.extend(self.mobs.instances(self.time, &mut skins));
        instances.extend(self.well_poses().into_iter().map(|(model, transform)| ModelInstance { model, transform, skin: None, flash: 0.0, tint: None }));
        instances.extend(self.portal_pose().map(|p| self.portal.poses(p, self.time)).unwrap_or_default().into_iter().map(|(model, transform)| ModelInstance {
            model,
            transform,
            skin: None,
            flash: 0.0,
            tint: None,
        }));
        instances.extend(self.gate_poses().into_iter().map(|transform| ModelInstance { model: PORTCULLIS, transform, skin: None, flash: 0.0, tint: None }));
        let posed = |model: &'static str, list: Vec<glam::Affine3A>| list.into_iter().map(move |transform| ModelInstance { model, transform, skin: None, flash: 0.0, tint: None });
        instances.extend(posed(crypt::KIT_SHORTCUT_LEAF, self.mech.shortcut_poses(self.live_crypt())));
        instances.extend(posed(crypt::CHEST_LID, self.chest_poses()));
        instances.extend(posed(crypt::KIT_LEVER_ARM, self.mech.lever_poses(self.live_crypt())));
        instances.extend(posed(crypt::KIT_SECRET, self.mech.secret_poses(self.live_crypt(), self.time)));
        instances.extend(posed(crypt::KIT_TRAPDOOR, self.mech.trapdoor_poses(self.live_crypt())));
        let (decks, chains) = self.mech.bridge_poses(self.live_crypt());
        instances.extend(posed(crypt::KIT_BRIDGE, decks));
        instances.extend(posed(crypt::KIT_CHAIN, chains));
        instances.extend(posed(crypt::KIT_WATER, self.mech.water_poses(self.live_crypt())));
        instances.extend(posed(crypt::KIT_ARROW, self.dart_poses()));
        instances.extend(posed(crypt::KIT_REST_FLAME, self.rest_flame_poses()));
        instances.extend(self.gallery_poses().into_iter().map(|(model, transform)| ModelInstance { model, transform, skin: None, flash: 0.0, tint: None }));
        if self.film.is_none() {
            instances.extend(self.combat.instances(eye, vyaw, vpitch, in_crypt, &mut skins));
        }
        let (hidden, edited) = self.edit_instances();
        instances.extend(edited);
        Frame {
            view: Mat4::look_to_rh(eye, view_dir, Vec3::Y),
            proj: Mat4::perspective_rh(self.fov().to_radians(), aspect, 0.1, 2500.0),
            camera: eye,
            time: self.time,
            sun_dir,
            sun_color,
            sun_intensity: sun_intensity * (1.0 - c * 0.85),
            night: t,
            // Under jorda er det nesten ikke noe lys utenom faklene, lysene
            // og lykta: de skal ha betydning (eieren 13. oktober).
            sky_top: sky_top * (1.0 - c * 0.999),
            sky_horizon: sky_horizon * (1.0 - c * 0.999),
            ground_bounce: (ground + snow_bounce) * (1.0 - c * 0.999),
            fog_color: grey(lerp3(Vec3::new(0.62, 0.74, 0.88), Vec3::new(0.012, 0.02, 0.04))) * (1.0 - c * 0.95),
            // Høydetåka tetner nedover. I kjelleren er det ingen tåke.
            fog_density: (0.0006 + dusk * 0.0012) * self.sky.fog_mul * (1.0 - c),
            fog_falloff: 0.03,
            exposure: (0.5 * (1.0 - t) + 2.0 * t) * (1.0 + overcast * 0.35) * (1.0 + c * 0.5),
            bloom: 0.06 + t * 0.06,
            windows_lit: 0.55,
            street_lights: 1.0,
            aurora: 1.0 - overcast,
            weather: w,
            palette: [Vec3::ONE; PALETTE],
            lights,
            smoke,
            instances,
            hidden,
            lines: self.edit_lines(),
            skins,
            hurt: self.combat.hurt,
            debug: self.debug,
            rooms,
            band: self.band(in_crypt),
        }
    }

    /// Høydene som kan synes (`Frame.band`). Oppe: alt over krypten.
    /// Nede: nivået du står i, fra det laveste gulvet til det høyeste
    /// taket.
    pub(super) fn band(&self, in_crypt: bool) -> (f32, f32) {
        if !in_crypt {
            return (crypt::TOP, f32::INFINITY);
        }
        let feet = self.body.pos - Vec3::Y * 1.6;
        match self.crypt.level_at(feet) {
            Some(k) => {
                let l = &self.crypt.levels[k];
                (l.min_y - 9.0, l.max_y + 2.0)
            }
            // Utenfor nivåene (faller): rundt deg.
            None => (feet.y - 14.0, feet.y + 10.0),
        }
    }

    /// Etter at bildet er tegnet: en flytting (`jumped`, som demper
    /// landingslyden) gjelder bare det ene bildet.
    pub fn settle(&mut self) {
        self.body.jumped = false;
    }

    /// Synsfeltet: 72°, strukket ut og tilbake når du reiser gjennom
    /// virvelen.
    pub(super) fn fov(&self) -> f32 {
        if let Some(s) = self.film {
            return s.fov;
        }
        self.portal.fov()
    }

    /// Krypten slik den står: mens du redigerer, med merkene der
    /// redigeringen har flyttet dem, så spakene, portene og lemmene følger.
    fn live_crypt(&self) -> &crate::domain::world::crypt::Crypt {
        self.edit.as_ref().map(|e| e.built_crypt()).unwrap_or(&self.crypt)
    }

    /// Det du kan bruke og hvor det står. Mens du redigerer, står det der
    /// redigeringen har satt det, så vinda og bøtta følger brønnen du drar
    /// (eieren 12. oktober), og lokket kista.
    fn target_poses(&self) -> Vec<(Use, Affine3A)> {
        match &self.edit {
            Some(e) => e.scene().targets.iter().map(|t| (t.what, t.pose)).collect(),
            None => self.targets.iter().map(|t| (t.target.what, t.target.pose)).collect(),
        }
    }

    /// Portgitrene der de står nå, skalert til bredden.
    fn gate_poses(&self) -> Vec<Affine3A> {
        self.target_poses()
            .into_iter()
            .filter_map(|(what, pose)| match what {
                Use::Gate(n) => Some(self.mech.gate_pose(self.live_crypt(), n, pose)),
                _ => None,
            })
            .collect()
    }

    /// Lokkene på kistene, dreid opp så langt de er åpnet.
    fn chest_poses(&self) -> Vec<Affine3A> {
        self.target_poses()
            .into_iter()
            .filter_map(|(what, pose)| match what {
                Use::Chest(n) => Some(self.mech.chest_lid(n, pose)),
                _ => None,
            })
            .collect()
    }

    /// Brønnvindene, tauene og bøttene der de er nå.
    fn well_poses(&self) -> Vec<(&'static str, Affine3A)> {
        self.target_poses()
            .into_iter()
            .filter_map(|(what, pose)| match what {
                Use::Well(n) => Some(self.mech.well_parts(n, pose)),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Komponentene på galleriveggen: én i hver ramme du har fylt, i
    /// rammens pose, med modellen som viser akkurat den komponenten
    /// (`craft::GALLERY_MODELS`).
    fn gallery_poses(&self) -> Vec<(&'static str, Affine3A)> {
        self.target_poses()
            .into_iter()
            .filter_map(|(what, pose)| match what {
                Use::Gallery(n) => {
                    let c = craft::GALLERY.get(n)?;
                    self.hero.found.contains(*c).then(|| (craft::GALLERY_MODELS[n], pose))
                }
                _ => None,
            })
            .collect()
    }

    /// Pilene i lufta, med spissen i farten.
    fn dart_poses(&self) -> Vec<Affine3A> {
        self.run.darts.iter().map(|d| Affine3A::from_rotation_translation(Quat::from_rotation_arc(Vec3::Z, d.vel.normalize_or(Vec3::Z)), d.pos)).collect()
    }

    /// Flammene på hvilestedene du har tent.
    pub(super) fn rest_flame_poses(&self) -> Vec<Affine3A> {
        self.live_crypt().rests.iter().zip(&self.run.rests_lit).filter(|(_, lit)| **lit).map(|(r, _)| crate::domain::world::at(r.pos, r.yaw)).collect()
    }

    /// Lysene på hvilestedene du har tent.
    fn rest_lights(&self) -> Vec<Light> {
        self.crypt
            .rests
            .iter()
            .zip(&self.run.rests_lit)
            .filter(|(_, lit)| **lit)
            .map(|(r, _)| {
                let room = self.crypt.room_at(r.pos + Vec3::Y * 0.3).map(|(l, k)| (l as u16, k as u16));
                Light {
                    pos: crate::domain::world::at(r.pos, r.yaw).transform_point3(crypt::REST_FLAME + Vec3::Y * 0.3),
                    radius: 9.0,
                    color: Vec3::new(1.0, 0.7, 0.4),
                    intensity: 2.2,
                    always: true,
                    flicker: 0.15,
                    room,
                }
            })
            .collect()
    }
}
