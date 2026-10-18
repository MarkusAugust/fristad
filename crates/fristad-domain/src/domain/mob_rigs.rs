//! Riggene og klippene til hvert slag, slått opp én gang (del av `mobs.rs`).
//! Mangler slagets egen modell, står stedfortrederen (`Kind::stand_in`) der.

use super::*;

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
                    look: joint("neck").zip(joint("head")),
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
