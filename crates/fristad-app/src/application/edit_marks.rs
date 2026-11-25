//! Rutene og lysene i redigeringen (`Editing`): punktene på ruta til den valgte
//! innbyggeren står som stolper på bakken med en strek mellom (rundt, som
//! hen går), og de dras langs bakken; et trykk på streken legger til et
//! punkt der, og Delete sletter punktet som er tatt. Lysene i byen og i
//! krypten vises som små kors i sin egen farge, så de kan klikkes og endres.

use super::camera::Camera;
use super::editing::Editing;
use crate::domain::world::layout::Ref;
use glam::Vec3;

/// Hvor høye stolpene på ruta er.
const POST: f32 = 1.2;
/// Hvor nær en stolpe eller streken pekeren må være (brøkdel av
/// skjermhøyden).
const NEAR: f32 = 0.022;
/// Hvor langt unna lysene vises.
const LIGHTS_FAR: f32 = 40.0;
const ROUTE: Vec3 = Vec3::new(0.4, 3.0, 4.0);
const ROUTE_TAKEN: Vec3 = Vec3::new(4.0, 4.0, 4.0);

impl Editing {
    /// Den ene innbyggeren som er valgt (ruta vises bare da).
    pub(super) fn route_npc(&self) -> Option<Ref> {
        match self.editor.as_ref()?.selection.as_slice() {
            [r @ Ref::Npc(_)] => Some(r.clone()),
            _ => None,
        }
    }

    /// Stolpen under pekeren, om noe.
    pub(super) fn route_point_at(&self, at: (f32, f32), cam: &Camera) -> Option<usize> {
        let r = self.route_npc()?;
        let m = cam.point(at);
        let route = self.editor.as_ref()?.route(&r);
        route
            .iter()
            .enumerate()
            .filter_map(|(k, (p, _))| Some((k, cam.distance(m, *p, *p + Vec3::Y * POST)?)))
            .filter(|(_, d)| *d < NEAR)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(k, _)| k)
    }

    /// Streken under pekeren: punktet på bakken der et nytt punkt settes inn,
    /// og nummeret det får.
    pub(super) fn route_insert_at(&self, at: (f32, f32), cam: &Camera) -> Option<(usize, Vec3)> {
        let r = self.route_npc()?;
        let e = self.editor.as_ref()?;
        let route = e.route(&r);
        let n = route.len();
        if n < 2 {
            return None;
        }
        let m = cam.point(at);
        let lift = Vec3::Y * 0.1;
        let (k, d) = (0..n).filter_map(|k| Some((k, cam.distance(m, route[k].0 + lift, route[(k + 1) % n].0 + lift)?))).min_by(|a, b| a.1.total_cmp(&b.1))?;
        (d < NEAR).then_some(())?;
        // Der pekeren treffer bakken, lagt på streken.
        let hit = self.ground_at(Some(at), cam)?;
        let (a, b) = (route[k].0, route[(k + 1) % n].0);
        let ab = b - a;
        let t = ((hit - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
        let p = a + ab * t;
        Some((k + 1, Vec3::new(p.x, hit.y, p.z)))
    }

    /// Markeringene for ruta og lysene.
    pub(super) fn mark_lines(&self, cam: &Camera, out: &mut Vec<(Vec3, Vec3, Vec3)>) {
        let Some(e) = &self.editor else { return };
        if let Some(r) = self.route_npc() {
            let route = e.route(&r);
            let n = route.len();
            for (k, (p, _)) in route.iter().enumerate() {
                let c = if self.ui.route_point == Some(k) { ROUTE_TAKEN } else { ROUTE };
                out.push((*p, *p + Vec3::Y * POST, c));
                let top = *p + Vec3::Y * POST;
                for d in [Vec3::X, Vec3::Z] {
                    out.push((top - d * 0.15, top + d * 0.15, c));
                }
                if n > 1 {
                    let q = route[(k + 1) % n].0;
                    out.push((*p + Vec3::Y * 0.1, q + Vec3::Y * 0.1, ROUTE));
                }
            }
        }
        let eye = cam.eye;
        for (_, p, color) in e.light_marks() {
            if p.distance(eye) > LIGHTS_FAR {
                continue;
            }
            let c = color.normalize_or_zero() * 4.0;
            for d in [Vec3::X, Vec3::Y, Vec3::Z] {
                out.push((p - d * 0.2, p + d * 0.2, c));
            }
        }
    }
}
