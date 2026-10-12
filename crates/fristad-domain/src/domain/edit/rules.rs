//! Reglene for det som står i byen, som rene funksjoner over det som er
//! satt ut (`Builder`) og dataene (`Layout`): de samme som `cargo test`
//! sjekker, så det redigeringen lar deg lagre, alltid går gjennom testene
//! (`docs/redigering.md`, «Reglene»).
//!
//! - svever eller står nede i bakken (`Builder::ground_findings`),
//! - står inni en vegg eller i noe annet,
//! - husene holder avstand til hverandre, muren, gatene og torget,
//! - alt stort har kollider,
//! - modellen er godkjent av eieren (bare det som er endret).

use crate::domain::world::layout::{Layout, Ref};
use crate::domain::world::{terrain, town, Builder, Shape};
use glam::{Affine3A, Vec2, Vec3};
use std::collections::{HashMap, HashSet};

/// En regel som er brutt: tingen og hvorfor.
#[derive(Clone, Debug, PartialEq)]
pub struct Breach {
    pub r: Ref,
    pub why: String,
}

/// Takskjegget: veggene er så mye mindre enn hele husmodellen. Veggene
/// holder avstand til gata og torget; hele huset med taket og litt luft
/// holder avstand til nabohusene og muren (CLAUDE.md, «Naturlig og
/// organisk»).
pub const EAVES: f32 = 0.7;
pub const AIR: f32 = 0.3;

/// Et rektangel på bakken: midte, halve mål langs husets x og z, og yaw.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub c: Vec2,
    pub h: Vec2,
    pub yaw: f32,
}

impl Rect {
    fn axes(&self) -> (Vec2, Vec2) {
        let (s, c) = self.yaw.sin_cos();
        (Vec2::new(c, -s), Vec2::new(s, c))
    }

    pub fn corners(&self) -> [Vec2; 4] {
        let (ax, az) = self.axes();
        let (x, z) = (ax * self.h.x, az * self.h.y);
        [
            self.c - x - z,
            self.c + x - z,
            self.c + x + z,
            self.c - x + z,
        ]
    }

    /// Hjørnene, kantmidtene og midten.
    pub fn samples(&self) -> [Vec2; 9] {
        let k = self.corners();
        [
            k[0],
            k[1],
            k[2],
            k[3],
            (k[0] + k[1]) * 0.5,
            (k[1] + k[2]) * 0.5,
            (k[2] + k[3]) * 0.5,
            (k[3] + k[0]) * 0.5,
            self.c,
        ]
    }

    /// Skiller en akse de to rektanglene (de overlapper ikke)?
    pub fn overlaps(&self, o: &Rect) -> bool {
        let (a1, a2) = self.axes();
        let (b1, b2) = o.axes();
        let (ka, kb) = (self.corners(), o.corners());
        for axis in [a1, a2, b1, b2] {
            let (amin, amax) = ka.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
                (lo.min(p.dot(axis)), hi.max(p.dot(axis)))
            });
            let (bmin, bmax) = kb.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
                (lo.min(p.dot(axis)), hi.max(p.dot(axis)))
            });
            if amax <= bmin || bmax <= amin {
                return false;
            }
        }
        true
    }

    /// Hele huset med takskjegg og luft rundt veggene.
    pub fn full(&self) -> Rect {
        Rect {
            h: self.h + Vec2::splat(EAVES + AIR),
            ..*self
        }
    }
}

/// Veggene til et hus (modellens mål minus takskjegget) satt ut med `t`.
pub fn walls(lo: Vec3, hi: Vec3, t: &Affine3A) -> Rect {
    let yaw = crate::domain::world::layout::Pose::of(*t).yaw();
    let mid = Vec2::new((lo.x + hi.x) * 0.5, (lo.z + hi.z) * 0.5);
    let half =
        (Vec2::new(hi.x - lo.x, hi.z - lo.z) * 0.5 - Vec2::splat(EAVES)).max(Vec2::splat(1.0));
    let r = Rect {
        c: Vec2::new(t.translation.x, t.translation.z),
        h: half,
        yaw,
    };
    let (ax, az) = r.axes();
    Rect {
        c: r.c + ax * mid.x + az * mid.y,
        ..r
    }
}

/// Biter som skjøtes: gjerdene og murdelene går inn i hverandre og inn i
/// husveggene de møter, med vilje.
pub fn joins(model: &str) -> bool {
    model.ends_with("_fence") || matches!(model, "town_wall" | "wall_tower" | "gatehouse")
}

/// Bitene av kryptnivåene og innsidemodellene er rom, ikke ting.
fn is_room(model: &str) -> bool {
    model.ends_with("_interior")
        || model
            .strip_prefix("crypt_")
            .is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()))
}

/// Alle reglene for det som er satt ut fra `layout`. `approved` er
/// modellene eieren har godkjent; de sjekkes bare for tingene i `touched`
/// (det som er endret i redigeringen), siden byen står med modeller som
/// venter på ny runde.
pub fn check(
    b: &Builder,
    layout: &Layout,
    approved: Option<&HashSet<String>>,
    touched: &HashSet<Ref>,
) -> Vec<Breach> {
    let mut out = vec![];
    let origin: HashMap<(&str, usize), &Ref> =
        b.origins.iter().map(|(m, n, r)| ((*m, *n), r)).collect();
    // Svever eller står nede.
    for (model, n, why) in b.ground_findings() {
        if let Some(r) = origin.get(&(model, n)) {
            out.push(Breach {
                r: (*r).clone(),
                why,
            });
        }
    }
    collider_breaches(b, &mut out);
    inside_breaches(b, layout, &mut out);
    house_breaches(b, layout, &mut out);
    if let Some(ok) = approved {
        for (model, _, r) in &b.origins {
            if touched.contains(r) && !ok.contains(*model) {
                out.push(Breach {
                    r: r.clone(),
                    why: format!("{model} er ikke godkjent ennå"),
                });
            }
        }
    }
    out.sort_by(|a, b| a.r.cmp(&b.r).then(a.why.cmp(&b.why)));
    out.dedup();
    out
}

/// Verdens mål for en instans: hjørnene av modellens boks satt ut med `t`.
pub fn world_box(lo: Vec3, hi: Vec3, t: &Affine3A) -> (Vec3, Vec3) {
    let mut wlo = Vec3::splat(f32::MAX);
    let mut whi = Vec3::splat(f32::MIN);
    for k in 0..8 {
        let c = Vec3::new(
            if k & 1 == 0 { lo.x } else { hi.x },
            if k & 2 == 0 { lo.y } else { hi.y },
            if k & 4 == 0 { lo.z } else { hi.z },
        );
        let w = t.transform_point3(c);
        wlo = wlo.min(w);
        whi = whi.max(w);
    }
    (wlo, whi)
}

/// Alt stort har kollider: hver instans høyere enn 60 cm og bredere enn
/// 30 cm har en kollider ved foten, eller høyere opp på midtlinja for det
/// som er åpent nede (portrommet). Samme regel som `alt_stort_har_kollider`.
pub fn lacks_collider(colliders: &[Shape], lo: Vec3, hi: Vec3, t: &Affine3A) -> Option<Vec3> {
    let (wlo, whi) = world_box(lo, hi, t);
    let size = whi - wlo;
    if size.y < 0.6 || size.x.max(size.z) < 0.3 {
        return None;
    }
    let o = Vec3::from(t.translation);
    let q = Vec3::new(o.x, wlo.y + 0.5, o.z);
    let steps = ((whi.y - wlo.y - 1.0) / 0.5).max(0.0) as usize;
    let covered = (0..=steps).any(|k| {
        colliders
            .iter()
            .any(|c| c.covers(q + Vec3::Y * (k as f32 * 0.5), 0.35))
    });
    (!covered).then_some(q)
}

fn collider_breaches(b: &Builder, out: &mut Vec<Breach>) {
    for (model, n, r) in &b.origins {
        if is_room(model) {
            continue;
        }
        let (Some((lo, hi)), Some(t)) = (
            b.bounds(model),
            b.placements
                .iter()
                .find(|p| p.model == *model)
                .and_then(|p| p.transforms.get(*n)),
        ) else {
            continue;
        };
        if let Some(q) = lacks_collider(&b.colliders, lo, hi, t) {
            out.push(Breach {
                r: r.clone(),
                why: format!("{model} ved ({:.1}, {:.1}) har ingen kollider", q.x, q.z),
            });
        }
    }
}

/// Står inni en vegg eller i noe annet: punkter inne i tingen (et rutenett
/// 4 cm innenfor kantene, på tre høyder) ligger i en kollider som hører til
/// en annen ting. Det som henger på en vegg, husene selv, bitene som skjøtes
/// (`joins`), og det som ikke har modell (usynlige vegger, som gjerdene står
/// i), er unntatt.
fn inside_breaches(b: &Builder, layout: &Layout, out: &mut Vec<Breach>) {
    let item_of = |r: &Ref| {
        let it = match r {
            Ref::Town(i) => layout.items.get(*i),
            Ref::Inside { house, i } => b.contents.get(house).and_then(|v| v.get(*i)),
            Ref::Npc(_) | Ref::Crypt { .. } | Ref::CryptLight { .. } => None,
        }?;
        Some((it.model.as_deref().is_some_and(|m| !joins(m)), it.building))
    };
    let solid: Vec<(usize, &Ref)> = b
        .solid_origins
        .iter()
        .enumerate()
        .filter_map(|(k, o)| o.as_ref().map(|r| (k, r)))
        .filter(|(_, r)| item_of(r).is_some_and(|(m, _)| m))
        .collect();
    for (model, n, r) in &b.origins {
        if is_room(model)
            || joins(model)
            || b.mounted.contains(&(*model, *n))
            || item_of(r).is_none_or(|(_, building)| building)
        {
            continue;
        }
        let (Some((lo, hi)), Some(t)) = (
            b.bounds(model),
            b.placements
                .iter()
                .find(|p| p.model == *model)
                .and_then(|p| p.transforms.get(*n)),
        ) else {
            continue;
        };
        let inset = Vec3::new(0.04, 0.0, 0.04).min((hi - lo) * 0.25);
        let (a, z) = (lo + inset, hi - inset);
        let hit = (0..27).find_map(|k| {
            let f = |i: usize, lo: f32, hi: f32| lo + (hi - lo) * [0.0, 0.5, 1.0][i];
            let fy = [0.25, 0.5, 0.75][(k / 3) % 3];
            let p = t.transform_point3(Vec3::new(
                f(k % 3, a.x, z.x),
                lo.y + (hi.y - lo.y) * fy,
                f(k / 9, a.z, z.z),
            ));
            solid
                .iter()
                .find(|(c, o)| *o != r && b.colliders[*c].covers(p, -0.02))
                .map(|(_, o)| (*o).clone())
        });
        if let Some(o) = hit {
            out.push(Breach {
                r: r.clone(),
                why: format!("{model} står inni {}", name_of(b, layout, &o)),
            });
        }
    }
}

fn name_of(b: &Builder, layout: &Layout, r: &Ref) -> String {
    let m = match r {
        Ref::Town(i) => layout.items.get(*i).and_then(|it| it.model.clone()),
        Ref::Inside { house, i } => b
            .contents
            .get(house)
            .and_then(|v| v.get(*i))
            .and_then(|it| it.model.clone()),
        Ref::Npc(i) => layout.npcs.get(*i).map(|n| n.name.clone()),
        Ref::Crypt { .. } => Some("et merke i krypten".into()),
        Ref::CryptLight { .. } => Some("et lys i krypten".into()),
    };
    m.unwrap_or_else(|| "en usynlig vegg".into())
}

/// Husene holder avstand: hele huset (med takskjegg og luft) går ikke inn i
/// et annet hus eller muren, og innenfor muren står veggene utenfor gatene
/// og torget.
fn house_breaches(b: &Builder, layout: &Layout, out: &mut Vec<Breach>) {
    let houses: Vec<(usize, &str, Rect)> = layout
        .items
        .iter()
        .enumerate()
        .filter(|(_, it)| it.building)
        .filter_map(|(i, it)| {
            let m = it.model.as_deref()?;
            let (lo, hi) = b.bounds(m)?;
            Some((
                i,
                m,
                walls(
                    lo,
                    hi,
                    &it.at.map(|p| p.affine()).unwrap_or(Affine3A::IDENTITY),
                ),
            ))
        })
        .collect();
    for (k, (i, m, r)) in houses.iter().enumerate() {
        let f = r.full();
        for (j, m2, r2) in &houses[k + 1..] {
            if f.overlaps(&r2.full()) {
                out.push(Breach {
                    r: Ref::Town(*i),
                    why: format!("{m} står for tett på {m2}"),
                });
                out.push(Breach {
                    r: Ref::Town(*j),
                    why: format!("{m2} står for tett på {m}"),
                });
            }
        }
        let inside = terrain::town_edge(r.c.x, r.c.y) < 0.0;
        let edge = |p: &Vec2| terrain::town_edge(p.x, p.y);
        if inside && f.samples().iter().any(|p| edge(p) > -1.3)
            || !inside && f.samples().iter().any(|p| edge(p) < 1.3)
        {
            out.push(Breach {
                r: Ref::Town(*i),
                why: format!("{m} står for tett på muren"),
            });
        }
        if inside {
            if r.samples().iter().any(|p| town::square_dist(*p) < 0.4) {
                out.push(Breach {
                    r: Ref::Town(*i),
                    why: format!("{m} står ute på torget"),
                });
            }
            if r.samples().iter().any(|p| {
                town::streets()
                    .iter()
                    .any(|st| town::street_dist(st, *p).0 < st.half + 0.25)
            }) {
                out.push(Breach {
                    r: Ref::Town(*i),
                    why: format!("{m} står ute i gata"),
                });
            }
        }
    }
}
