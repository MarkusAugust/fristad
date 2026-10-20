//! Riggene og klippene til hvert slag, slått opp én gang (del av `mobs.rs`).
//! Mangler slagets egen modell, står stedfortrederen (`Kind::stand_in`) der.

use super::*;

/// Det en fiende holder (beinet i riggen → modellen): bueskytterens bue
/// (`mob3d.bow_bone`, `tools/models/bow.py`). Beinets rom er modellens rom
/// slik Blender har det (z opp), og glb-fila har y opp (som lutten,
/// `Npcs::instances`). `Mobs::reserve` setter av plass til dem (uten den
/// tegnet rendereren aldri buen, og eieren så bare den på ryggen).
pub const HELD: &[(&str, &str)] = &[("bow", "bow")];

/// Det fienden holder, satt etter beinet i posen (`t` er fiendens plass).
pub(super) fn held(rig: &Rig, pose: &crate::domain::anim::Pose, clips: &Clips, t: Affine3A, flash: f32) -> Option<ModelInstance> {
    let (j, model) = clips.held?;
    let at = Affine3A::from_mat4(rig.joint_transform(pose, j)) * Affine3A::from_rotation_x(std::f32::consts::FRAC_PI_2);
    Some(ModelInstance { model, transform: t * at, skin: None, flash, tint: None, look: None })
}

pub(super) fn lookup(rigs: &[(String, Rig)]) -> (Vec<Option<Rig>>, Vec<Clips>, Vec<&'static str>) {
    let mut all_rigs = vec![];
    let mut clips = vec![];
    let mut models = vec![];
    for k in Kind::ALL {
        let mut model = k.model();
        let mut rig = rigs.iter().find(|(n, _)| n == model).map(|(_, r)| r.clone());
        if rig.is_none() {
            if let Some(s) = k.stand_in().filter(|s| rigs.iter().any(|(n, _)| n == s.model())) {
                log::info!("{} har ingen modell ennå; {} står i stedet", k.label(), s.model());
                model = s.model();
                rig = rigs.iter().find(|(n, _)| n == model).map(|(_, r)| r.clone());
            }
        }
        models.push(model);
        let c = rig
            .as_ref()
            .map(|r| {
                let find = |n: &str| r.clip(n).unwrap_or(0);
                for n in ["idle", "walk", "attack", "hit", "die"] {
                    if r.clip(n).is_none() {
                        log::warn!("Riggen til {} mangler klippet {n}", k.model());
                    }
                }
                let joint = |n: &str| r.joint(n);
                Clips {
                    idle: find("idle"),
                    walk: find("walk"),
                    attack: find("attack"),
                    hit: find("hit"),
                    die: find("die"),
                    slam: r.clip("slam"),
                    roar: r.clip("roar"),
                    alert: r.clip("alert"),
                    hit2: r.clip("hit2"),
                    stagger: r.clip("stagger"),
                    die2: r.clip("die2"),
                    climb: r.clip("climb").map(|c| (c, r.clips[c].duration)),
                    look: joint("neck").zip(joint("head")),
                    held: HELD.iter().find_map(|(b, m)| joint(b).map(|i| (i, *m))),
                    up: r.root.inverse().transform_vector3(Vec3::Y).normalize_or_zero(),
                }
            })
            .unwrap_or_default();
        if rig.is_none() {
            log::warn!("Modellen {} har ingen rigg; fienden svever", k.model());
        }
        all_rigs.push(rig);
        clips.push(c);
    }
    (all_rigs, clips, models)
}

impl Mobs {
    /// Hvor leddet `joint` står i verden nå (uten svingen og rykket etter et
    /// slag), til testene.
    pub fn joint_world(&self, i: usize, joint: &str) -> Option<Vec3> {
        let m = self.mobs.get(i)?;
        let rig = self.rigs[m.kind as usize].as_ref()?;
        let pose = m.anim.as_ref()?.pose(rig);
        let j = rig.joint_transform(&pose, rig.joint(joint)?).transform_point3(Vec3::ZERO);
        let rot = Quat::from_rotation_y(m.yaw - std::f32::consts::FRAC_PI_2);
        Some(m.pos + rot * (j * m.scale()))
    }

    /// Navnet på klippet fiende `i` spiller nå (til testene).
    pub fn clip_name(&self, i: usize) -> Option<&str> {
        let m = self.mobs.get(i)?;
        let rig = self.rigs[m.kind as usize].as_ref()?;
        rig.clips.get(m.anim.as_ref()?.clip).map(|c| c.name.as_str())
    }
}
