//! Kjøpstaden Fristad rundt 1350, liten og samlet (docs/ny-by.md): ett
//! torg midt i byen avgrenset av husfasader, med Vertshuset, Smia, Apoteket
//! og Skriptoriet langs kantene, og Storgata fra sørporten over torget til
//! nordporten. Husene står i tette rekker langs gata og rundt torget, og
//! det tomme bak dem er sperret av med usynlige vegger, markert med gjerder
//! og vedstabler. Utenfor nordporten ligger kirkegården med stavkirka og
//! krypten, og utenfor muren går veiene ut mellom åsene.
//!
//! Alt som synes er modeller; byen bestemmer bare hvor de står. Husmodellene
//! har døra mot sin lokale +Z og origo midt på grunnflaten i bakkehøyde, så
//! `yaw` vender døra mot gata.

use super::{at, layout, terrain, Builder, NpcSpec, Shape, Use};
use glam::{Affine3A, Quat, Vec2, Vec3};

use std::sync::OnceLock;

/// Torget: et skjevt firkantet åpent rom (x, z), hjørnene i rekkefølge
/// nordvest, sørvest, sørøst, nordøst.
pub const SQUARE: [Vec2; 4] = [Vec2::new(-12.0, -9.0), Vec2::new(-12.8, 10.0), Vec2::new(11.5, 11.8), Vec2::new(13.0, -10.6)];
/// Halv bredde på Storgata.
pub const MAIN: f32 = 3.2;
/// Krypten på kirkegården: midten av gravkapellet, med døra mot øst (mot
/// veien).
pub const CRYPT: Vec2 = Vec2::new(-9.0, -52.0);
/// Torgils' øvingsplass innenfor nordporten, øst for gata: midten og halve mål.
pub const YARD: (Vec2, Vec2) = (Vec2::new(9.0, -24.5), Vec2::new(4.0, 3.0));

/// Murverk med fundament under bakken (fot 80 cm ned), som får stå nede i
/// skråninger.
pub const FOUNDED: [&str; 5] = ["town_wall", "wall_tower", "gatehouse", "crypt", "signpost"];

/// Det byen forteller resten av verden.
pub struct Town {
    pub spawn: Vec3,
    pub spawn_yaw: f32,
    /// Der kjelleren ligger under (x, z).
    pub crypt_anchor: Vec2,
    /// Der du står når du kommer opp fra kjelleren.
    pub crypt_exit: (Vec3, f32),
}

// --- Gatene ---------------------------------------------------------------

/// En gate: midtlinja som punkter (x, z), halv bredde, og om den er
/// brolagt (Storgata) eller tråkket jord.
pub struct Street {
    pub pts: Vec<Vec2>,
    pub half: f32,
    pub cobbled: bool,
}

/// Storgata: midtlinja som x for en z, en svak S fra sør til nord.
pub fn main_x(z: f32) -> f32 {
    1.4 * (z * 0.05 + 0.5).sin() + 0.6 * (z * 0.13 + 2.0).sin()
}

/// Gatene i byen, regnet ut én gang.
pub fn streets() -> &'static [Street] {
    static STREETS: OnceLock<Vec<Street>> = OnceLock::new();
    STREETS.get_or_init(|| {
        let curve = |a: f32, b: f32, n: usize, f: &dyn Fn(f32) -> Vec2| (0..=n).map(|i| f(a + (b - a) * i as f32 / n as f32)).collect::<Vec<_>>();
        vec![
            // Storgata, fra utenfor sørporten til utenfor nordporten.
            Street { pts: curve(48.0, -46.0, 47, &|z| Vec2::new(main_x(z), z)), half: MAIN, cobbled: true },
        ]
    })
}

/// Avstand fra (x, z) til midtlinja av gata, og retningen gata går der.
pub fn street_dist(st: &Street, p: Vec2) -> (f32, Vec2) {
    let mut best = (f32::MAX, Vec2::Y);
    for w in st.pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = b - a;
        let t = ((p - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0.0, 1.0);
        let d = (a + ab * t).distance(p);
        if d < best.0 {
            best = (d, ab.normalize_or_zero());
        }
    }
    best
}

/// Som `street_dist`, med koordinatene langs gata: (avstand, fortegnet
/// avstand på tvers, meter langs midtlinja fra starten).
pub fn street_coords(st: &Street, p: Vec2) -> (f32, f32, f32) {
    let mut best = (f32::MAX, 0.0, 0.0);
    let mut run = 0.0;
    for w in st.pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = b - a;
        let len = ab.length().max(1e-6);
        let t = ((p - a).dot(ab) / (len * len)).clamp(0.0, 1.0);
        let d = (a + ab * t).distance(p);
        if d < best.0 {
            let across = (p - a).perp_dot(ab) / len;
            best = (d, across, run + (p - a).dot(ab) / len);
        }
        run += len;
    }
    best
}

/// Fortegnet avstand fra (x, z) til kanten av torget (negativ innenfor).
pub fn square_dist(p: Vec2) -> f32 {
    let mut d = f32::MIN;
    for i in 0..4 {
        let (a, b) = (SQUARE[i], SQUARE[(i + 1) % 4]);
        let e = (b - a).normalize();
        // Hjørnene går med klokka i (x, z), så utsida er til venstre.
        let out = Vec2::new(-e.y, e.x);
        d = d.max((p - a).dot(out));
    }
    d
}

/// Der Storgata møter muren (sør og nord), og retningen ut av byen der.
pub fn gate(south: bool) -> (Vec2, Vec2) {
    let (mut a, mut b) = if south { (0.0, 75.0) } else { (0.0, -75.0) };
    for _ in 0..40 {
        let m = (a + b) * 0.5;
        if terrain::town_edge(main_x(m), m) < 0.0 {
            a = m;
        } else {
            b = m;
        }
    }
    let p = Vec2::new(main_x(a), a);
    (p, p.normalize())
}

/// Åpningen i gjerdet rundt Torgils' plass, mot gata.
pub const YARD_GAP: f32 = 3.0;
/// Gjerdevariantene (`tools/models/street.py`) og hvor tykke de er:
/// flettverk, skigard, stakitt og lav steinmur.
pub const FENCES: [(&str, f32); 4] = [("wattle_fence", 0.16), ("rail_fence", 0.3), ("picket_fence", 0.14), ("stone_fence", 0.44)];
/// Høyden på kollideren til et sperregjerde. Den står akkurat der gjerdet
/// står (like lang, så tykk som varianten), og slik kjenner testen
/// `sperringene_er_synlige` den igjen.
pub const FENCE_SOLID_H: f32 = 1.1;

/// Bondegårdene utenfor muren: midten av tunet. Flate steder på åsene,
/// unna veiene, kirkegården og beitene, med god avstand imellom.
pub fn farms() -> &'static [Vec2] {
    static FARMS: OnceLock<Vec<Vec2>> = OnceLock::new();
    FARMS.get_or_init(|| {
        let mut out: Vec<Vec2> = vec![];
        let herds = crate::domain::herd::HERDS;
        for k in 0..400 {
            if out.len() >= FARM_COUNT {
                break;
            }
            let a = k as f32 * 2.399 + 0.7;
            let r = 66.0 + (k as f32 * 0.37).sin().abs() * 55.0;
            let p = Vec2::new(a.cos(), a.sin()) * r;
            let flat = (0..8).map(|i| {
                let q = p + Vec2::new((i as f32 * 0.785).cos(), (i as f32 * 0.785).sin()) * 8.0;
                terrain::height(q.x, q.y)
            });
            let (lo, hi) = flat.fold((f32::MAX, f32::MIN), |(lo, hi), h| (lo.min(h), hi.max(h)));
            let clear = road_dist(p.x, p.y) > 16.0
                && terrain::town_edge(p.x, p.y) > 28.0
                && herds.iter().all(|(_, c, rad, _)| c.distance(p) > rad + FARM_R + 12.0)
                && out.iter().all(|q| q.distance(p) > 40.0)
                && {
                    let (cx, cz, hx, hz) = terrain::YARD_FLAT;
                    ((p.x - cx).abs() - hx).max((p.y - cz).abs() - hz) > 18.0
                };
            if clear && hi - lo < 2.2 {
                out.push(p);
            }
        }
        out
    })
}

/// Gårdene utenfor muren (`farms`).
const FARM_COUNT: usize = 5;

/// Tunet på en gård: radius på jorda rundt husene.
const FARM_R: f32 = 13.0;

/// Brønnvinda: vinda (med sveiv) dreier om x i høyden `WELL_AXLE`, tauet
/// (1 m langt, skaleres) henger fra forkanten av valsen (`WELL_ROPE_Z`) ned
/// til bøyla på bøtta. Bøtta henger i `WELL_REST` når ingen bruker brønnen
/// og senkes ned i vannet til `WELL_DEPTH`.
pub const WELL_PARTS: [&str; 3] = ["well_winch", "well_rope", "well_bucket"];
pub const WELL_AXLE: f32 = 1.55;
pub const WELL_ROPE_Z: f32 = 0.09;
pub const WELL_REST: f32 = 1.05;
pub const WELL_DEPTH: f32 = -0.25;
pub const WELL_DRUM: f32 = 0.08;

/// Byen er data (`assets/layout/town.json`, lest av adapterne): hver ting
/// med modell, sted og det den har med seg, og innbyggerne
/// (`layout::apply`). Uten data står bare landskapet.
pub fn build(b: &mut Builder, data: Option<&layout::Layout>) -> Town {
    match data {
        Some(l) => layout::apply(b, l),
        None => {
            log::warn!("Byen mangler (assets/layout/town.json)");
            layout::apply(b, &layout::Layout::default())
        }
    }
}

/// Det et hus har med seg: kollidere (midte, mål, yaw) og lys (posisjon,
/// farge, styrke, radius) i modellens rom, fra Blender (`<navn>_inside.json`,
/// lest av adapterne og gitt kjernen i `world::Data`).
#[derive(serde::Deserialize, Default, Clone, Debug)]
pub struct Inside {
    pub boxes: Vec<([f32; 3], [f32; 3], f32)>,
    pub lights: Vec<([f32; 3], [f32; 3], f32, f32)>,
    /// Ramper over trinnene (punktene i et konvekst skall).
    #[serde(default)]
    pub hulls: Vec<Vec<[f32; 3]>>,
    /// Lykter som tennes om natta (vegglykta ved døra).
    #[serde(default)]
    pub lamps: Vec<([f32; 3], [f32; 3], f32, f32)>,
    /// Pipene: toppen og hvor tykk røyken er.
    #[serde(default)]
    pub smoke: Vec<([f32; 3], f32)>,
    /// Småting ute som hører til huset (modell, sted i husets rom, yaw):
    /// settes på bakken av byen, så de ikke svever der tomta skrår.
    #[serde(default)]
    pub props: Vec<(String, [f32; 3], f32)>,
    /// Kjelleren under huset: midten og halve mål (x, z) i modellens rom.
    /// Bakken tas bort der (`Builder::hole`).
    #[serde(default)]
    pub cellar: Option<([f32; 3], [f32; 2])>,
    /// Kjellergulvet: midten (y er overkanten) og halve mål (x, z).
    #[serde(default)]
    pub cellar_floor: Option<([f32; 3], [f32; 2])>,
    /// Spillerens hus: midten av senga (det du sikter på) og halve mål.
    #[serde(default)]
    pub bed: Option<([f32; 3], [f32; 3])>,
    /// Spillerens hus: rammene på galleriveggen, midten og yaw (rammen
    /// vender ut mot lokal +Z dreid med yaw), i `craft::GALLERY`-rekkefølge.
    #[serde(default)]
    pub gallery: Vec<([f32; 3], f32)>,
}

/// Husene med en egen innsidemodell, som bare tegnes når du er nær
/// (rendereren kjenner dem på navnet `…_interior`).
const INTERIORS: [(&str, &str); 6] = [("tavern", "tavern_interior"), ("home", "home_interior"), ("forge", "forge_interior"), ("apothecary", "apothecary_interior"), ("scriptorium", "scriptorium_interior"), ("stave_church", "stave_church_interior")];

/// Et hus med det det har med seg (`place_colliders`), som én ting i
/// dataene (`layout::Item::building`).
/// Uten `lamps` tar det ikke med seg lysene sine (de står i dataene).
pub fn building_at(b: &mut Builder, model: &'static str, t: Affine3A, lamps: bool) {
    if b.place(model, t) {
        place_colliders(b, model, t, layout::Pose::of(t).yaw(), lamps);
    }
    // Det som står inne, i husets rom: følger huset.
    if let Some(items) = b.contents.get(model).cloned() {
        let outer = b.origin.take();
        for (i, item) in items.iter().enumerate() {
            b.origin = Some(layout::Ref::Inside { house: model.to_string(), i });
            layout::apply_item(b, item, t);
            if let Some(p) = item.model.as_deref().and_then(|m| b.placements.iter_mut().find(|p| p.model == m)) {
                p.indoor = true;
            }
        }
        b.origin = outer;
    }
}

/// Kolliderne, rampene, lysene, lyktene og røyken en modell har med seg
/// (`<navn>_inside.json`), satt ut med posen `t` (lysene bare med `lamps`).
/// Gir om den hadde kollidere.
pub fn carry(b: &mut Builder, model: &str, t: Affine3A, lamps: bool) -> bool {
    let Some(inside) = b.insides.get(model).cloned() else { return false };
    let yaw = layout::Pose::of(t).yaw();
    let w = |p: &[f32; 3]| t.transform_point3(Vec3::from(*p));
    if lamps {
        for l in model_lamps(&inside) {
            b.push_light(crate::domain::scene::Light { pos: w(&l.pos.to_array()), radius: l.radius, color: l.color, intensity: l.intensity, always: l.always, flicker: l.flicker, room: None });
        }
    }
    for (p, strength) in &inside.smoke {
        b.smoke(w(p), *strength);
    }
    for (c, size, y) in &inside.boxes {
        b.solid_box(w(c), Vec3::from(*size), yaw + y);
    }
    for pts in &inside.hulls {
        b.solid(Shape::Hull { points: pts.iter().map(w).collect() });
    }
    !inside.boxes.is_empty()
}

/// Lysene en modell har med seg, i modellens rom: lyktene som tennes om
/// natta (som `Builder::lamp_light`), så ilden som brenner hele døgnet (som
/// `Builder::fire_light`).
pub fn model_lamps(inside: &Inside) -> Vec<layout::LightDef> {
    let lamp = |(p, c, i, r): &([f32; 3], [f32; 3], f32, f32), always: bool, flicker: f32| layout::LightDef { pos: Vec3::from(*p), radius: *r, color: Vec3::from(*c), intensity: *i, always, flicker };
    inside.lamps.iter().map(|l| lamp(l, false, 0.12)).chain(inside.lights.iter().map(|l| lamp(l, true, 0.18))).collect()
}

/// Det et hus har med seg: innsidemodellen, kjelleren, lysene, røyken og
/// kolliderne (fra `<navn>_inside.json`, ellers en boks rundt modellen).
fn place_colliders(b: &mut Builder, model: &'static str, t: Affine3A, yaw: f32, lamps: bool) {
    let base = Vec3::from(t.translation);
    if let Some((_, interior)) = INTERIORS.iter().find(|(m, _)| *m == model) {
        // Innsida står på husets gulv, ikke på bakken.
        b.place_mounted(interior, t);
    }
    let inside = b.insides.get(model).cloned();
    if let Some((c, h)) = inside.as_ref().and_then(|i| i.cellar) {
        b.hole(t.transform_point3(Vec3::from(c)), Vec2::from(h), yaw);
    }
    if let Some(inside) = &inside {
        if let Some((c, h)) = inside.bed {
            b.target(Use::Sleep, t, Vec3::from(c), Vec3::from(h));
        }
        for (i, (c, y)) in inside.gallery.iter().enumerate().take(crate::domain::craft::GALLERY.len()) {
            let pose = t * Affine3A::from_rotation_translation(Quat::from_rotation_y(*y), Vec3::from(*c));
            b.target(Use::Gallery(i), pose, Vec3::new(0.0, 0.0, 0.03), Vec3::new(0.3, 0.3, 0.05));
        }
    }
    if let Some((c, h)) = inside.as_ref().and_then(|i| i.cellar_floor) {
        let c = t.transform_point3(Vec3::from(c));
        b.floor(c, Vec2::from(h), yaw, c.y);
    }
    if let Some(inside) = &inside {
        for (name, p, y) in &inside.props {
            house_prop(b, name, t.transform_point3(Vec3::from(*p)), yaw + y);
        }
    }
    if carry(b, model, t, lamps) {
        return;
    }
    if let Some((lo, hi)) = b.bounds(model) {
        let size = hi - lo;
        let mid = (lo + hi) * 0.5;
        let c = at(base, yaw).transform_point3(Vec3::new(mid.x, size.y * 0.5, mid.z));
        b.solid_box(c, size, yaw);
    }
}

/// De frittstående småtingene et hus kan ha med seg ute (`props` i
/// `<navn>_inside.json`, fra `town.prop` i Blender).
const HOUSE_PROPS: [&str; 3] = ["barrel", "crate", "woodpile"];

/// En småting ute ved et hus: på bakken der den står (ikke på husets
/// z = 0), med husets rotasjon, og med kollider; `Builder::audit` måler
/// den som alt annet.
fn house_prop(b: &mut Builder, name: &str, at_house: Vec3, yaw: f32) {
    let Some(model) = HOUSE_PROPS.iter().copied().find(|m| *m == name) else {
        log::warn!("småting «{name}» ved et hus er ikke blant HOUSE_PROPS");
        return;
    };
    let base = b.on_ground(Vec3::new(at_house.x, 0.0, at_house.z));
    if b.place(model, at(base, yaw)) {
        if let Some((lo, hi)) = b.bounds(model) {
            let size = hi - lo;
            b.solid_box(base + Vec3::Y * size.y * 0.5, size, yaw);
        }
    }
}

/// Hvem som står hvor, og hva de sier (`cast`).

/// En innbygger i byen: figuren (`figure`), høyden fra den, og
/// siktemålet over hodet.
pub fn settle(b: &mut Builder, mut npc: NpcSpec) {
    let model = figure(b, npc.model);
    npc.height = npc_height(b, model);
    let (pos, h) = (npc.pos, npc.height);
    b.npcs.push(npc);
    b.target(Use::Talk(b.npcs.len() - 1), Affine3A::from_translation(pos), Vec3::new(0.0, h * 0.5, 0.0), Vec3::new(0.3, h * 0.5, 0.3));
}

/// Figuren til en innbygger: sin egen modell, satt av én plass til. Finnes
/// den ikke (den er ikke laget, eller ikke godkjent ennå), er innbyggeren
/// usynlig, men siktemålet og samtalen virker.
fn figure(b: &mut Builder, model: &'static str) -> &'static str {
    // Mangler figuren, meldes det (`Builder.missing`), og innbyggeren er
    // usynlig, men samtalen virker.
    b.reserve(model, 1);
    model
}

/// Høyden på en figur: modellens mål, eller 1,8 m om modellen mangler
/// (figurene står med føttene i origo).
fn npc_height(b: &Builder, model: &str) -> f32 {
    let h = b.model_height(model);
    if h > 0.5 { h } else { 1.8 }
}

/// Portalen ved sørporten (`tools/models/portal.py`): står ikke der før du
/// har gått hjem fra kjelleren med H (eller dødd nede). Da stiger
/// steinringen opp av bakken på plassen innenfor sørporten, litt bak og til
/// venstre for stedet du kommer til (`spawn`), inntil steingjerdet og vendt
/// mot deg, så du ser den når du snur deg (eieren 18. oktober), og virvelen
/// åpner seg. Porten og veien ut står fri øst for den. Økta
/// setter ringen, runene, virvlene og splintene (`Session::portal_poses`)
/// og flytter kolliderne inn (`PORTAL_SOLIDS`). Stedet er siktemålet
/// `Use::Portal` i `assets/layout/town.json`.
pub const PORTAL_RING: &str = "portal_ring";
pub const PORTAL_PARTS: [&str; 3] = ["portal_runes", "portal_swirl", "portal_shard"];
/// Midten av ringen over plattformen, og ringens indre radius.
pub const PORTAL_CENTER: f32 = 1.62;
pub const PORTAL_RADIUS: f32 = 1.12;
/// Kolliderne til ringen i dens eget rom (midte, hele mål): plattformen,
/// sidene og toppen; åpningen er fri, så virvelen kan siktes på.
pub const PORTAL_SOLIDS: [(Vec3, Vec3); 4] = [
    (Vec3::new(0.0, 0.06, 0.0), Vec3::new(4.0, 0.2, 2.0)),
    (Vec3::new(-1.3, 1.4, 0.0), Vec3::new(0.5, 2.8, 0.55)),
    (Vec3::new(1.3, 1.4, 0.0), Vec3::new(0.5, 2.8, 0.55)),
    (Vec3::new(0.0, 3.0, 0.0), Vec3::new(2.4, 0.45, 0.55)),
];

/// Splintene som svever rundt den åpne portalen.
pub const PORTAL_SHARDS: usize = 8;

/// Hva bakken er dekket av i et punkt, som vekter mellom 0 og 1: brostein
/// på torget og i Storgata, tråkket jord i Tverrgata, på veiene og rundt
/// husene, berg i bratte skråninger og på borghaugen, skogbunn under
/// granskogen, og gress i resten. `dry` gjør gresset gult (åkrene,
/// ryggene), `wet` gir søle i kantene av veiene.
pub struct Ground {
    pub cobble: f32,
    pub dirt: f32,
    pub rock: f32,
    pub forest: f32,
    pub dry: f32,
    pub wet: f32,
    /// Koordinatene langs nærmeste jordvei i meter: (på tvers fra
    /// midtlinja, langs veien). Søla med hjulspor legges etter dem, så
    /// sporene følger veien og bøyer seg i svingene.
    pub road: Vec2,
    /// Hvor mye av jorda som er tråkket jord (gårdsplassene) i stedet
    /// for veisøle med hjulspor.
    pub earth: f32,
}

fn soft(edge: f32, d: f32, width: f32) -> f32 {
    let t = ((edge - d) / width + 0.5).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn ground_weights(x: f32, z: f32) -> Ground {
    let n1 = terrain::noise(x * 0.35, z * 0.35);
    let n2 = terrain::noise(x * 0.09 + 5.0, z * 0.09);
    let inside = soft(0.0, terrain::town_edge(x, z) + 1.0, 2.0);
    // Brosteinen: torget og Storgata, med ujevn kant der jorda tar over.
    let p = Vec2::new(x, z);
    let off = (1.0 - inside) * 50.0;
    let st = streets();
    let main = street_dist(&st[0], p).0 - MAIN + off;
    let cobble = soft(0.0, square_dist(p).min(main) + (n1 - 0.5) * 1.2, 1.4);
    // Jorda: Tverrgata og smauene, veiene ut, og gårdsplassene rundt husene.
    let mut lanes = f32::MAX;
    let mut coords = Vec2::ZERO;
    for s in st.iter().filter(|s| !s.cobbled) {
        let (d, across, along) = street_coords(s, p);
        if d - s.half < lanes {
            lanes = d - s.half;
            coords = Vec2::new(across, along);
        }
    }
    let mut road = f32::MAX;
    for (d, across, along) in paths(x, z) {
        if d - 1.6 < road {
            road = d - 1.6;
            if road < lanes + off {
                coords = Vec2::new(across, along);
            }
        }
    }
    let roads = soft(0.0, (lanes + off).min(road) + (n1 - 0.5) * 1.6, 1.8);
    // Gårdsplassene er tråkket jord (eget lag uten hjulspor), i byen og på
    // tunene på bondegårdene.
    let farm = farms().iter().map(|c| c.distance(p) - FARM_R).fold(f32::MAX, f32::min);
    let yards = (inside * soft(0.0, (0.3 - n2) * 12.0, 3.0) * 0.85).max(soft(0.0, farm + (n1 - 0.5) * 3.0, 3.0) * 0.9);
    let dirt = roads.max(yards);
    let earth = (yards / (roads + yards).max(1e-4)).clamp(0.0, 1.0);
    // Berg der det er bratt.
    let slope = 1.0 - terrain::normal(x, z).y;
    let rock = soft(0.0, 0.3 - slope + (n1 - 0.5) * 0.05, 0.05);
    let forest = 0.0f32;
    // Fordel: brostein over jord over berg over skogbunn, gress resten.
    let cobble = cobble.clamp(0.0, 1.0);
    let dirt = (dirt * (1.0 - cobble)).clamp(0.0, 1.0);
    let rock = (rock * (1.0 - cobble - dirt)).clamp(0.0, 1.0);
    let forest = (forest * (1.0 - cobble - dirt - rock)).clamp(0.0, 1.0);
    // Tørt gress på ryggene og i åkrene.
    let dry = (terrain::noise(x * 0.05 + 2.0, z * 0.05) * 1.3 - 0.35).clamp(0.0, 1.0) * 0.6;
    // Søle i kantene av jordveiene og i gatene i byen.
    let wet = (dirt * (1.0 - dirt) * 4.0 * soft(0.0, 0.55 - n2, 0.2)).clamp(0.0, 1.0);
    Ground { cobble, dirt, rock, forest, dry, wet, road: coords, earth }
}

/// Avstand fra et punkt til veiene og stiene utenfor muren.
pub fn path_dist(x: f32, z: f32) -> f32 {
    paths(x, z).into_iter().map(|(d, _, _)| d).fold(f32::MAX, f32::min)
}

/// Avstand fra et punkt til de to veiene (uten stiene til gårdene).
fn road_dist(x: f32, z: f32) -> f32 {
    roads(x, z).into_iter().map(|(d, _, _)| d).fold(f32::MAX, f32::min)
}

/// Midtlinja til veien ut av porten (sør eller nord) for en z.
fn road_x(south: bool, z: f32) -> f32 {
    let (from, _) = gate(south);
    let sign = if south { 1.0 } else { -1.0 };
    let k = (((z - from.y) * sign) / 50.0).clamp(0.0, 1.0);
    let n = terrain::noise(z * 0.02, if south { 9.0 } else { 3.0 });
    from.x * (1.0 - k) + (n - 0.5) * 40.0 * k
}

/// Stiene fra veiene inn til gårdene: en svak bue fra der stien tar av
/// fra veien til grinda i gjerdet, i samme rekkefølge som `farms()`.
/// Første punkt er inne på tunet, siste ute på veien.
pub fn farm_lanes() -> &'static [Vec<Vec2>] {
    static LANES: OnceLock<Vec<Vec<Vec2>>> = OnceLock::new();
    LANES.get_or_init(|| {
        let (south, _) = gate(true);
        let (north, _) = gate(false);
        // Punkter langs begge veiene.
        let mut road: Vec<Vec2> = vec![];
        for k in 0..90 {
            let z = south.y + 4.0 + k as f32 * 2.0;
            road.push(Vec2::new(road_x(true, z), z));
            let z = north.y - 4.0 - k as f32 * 2.0;
            road.push(Vec2::new(road_x(false, z), z));
        }
        let (cx, cz, hx, hz) = terrain::YARD_FLAT;
        let farms = farms();
        // Nærmest veien først; de andre kan ta av fra en sti som alt går
        // ut, så to stier ikke går side om side.
        let mut order: Vec<usize> = (0..farms.len()).collect();
        order.sort_by(|a, b| road_dist(farms[*a].x, farms[*a].y).total_cmp(&road_dist(farms[*b].x, farms[*b].y)));
        let mut lanes: Vec<Vec<Vec2>> = vec![vec![]; farms.len()];
        for i in order {
            let c = &farms[i];
            let lane = {
                // Fri vei: unna byen, kirkegården og de andre gårdene.
                let clear = |q: Vec2| {
                    terrain::town_edge(q.x, q.y) > 14.0 && ((q.x - cx).abs() - hx).max((q.y - cz).abs() - hz) > 6.0 && farms.iter().enumerate().all(|(j, f)| j == i || f.distance(q) > FARM_R + 3.0)
                };
                // Buen: midten skjøvet litt til siden, bort fra byen.
                let curve = |end: Vec2| -> Vec<Vec2> {
                    let dir = (end - *c).normalize_or(Vec2::X);
                    let start = *c + dir * (FARM_R - 4.0);
                    let mut side = Vec2::new(-dir.y, dir.x) * (start.distance(end) * 0.08);
                    if side.dot((start + end) * 0.5) < 0.0 {
                        side = -side;
                    }
                    (0..=12)
                        .map(|k| {
                            let t = k as f32 / 12.0;
                            start.lerp(end, t) + side * (4.0 * t * (1.0 - t))
                        })
                        .collect()
                };
                // Tar stien av fra en annen sti, er det på skrå, ikke langs den.
                let mut options = road.clone();
                for l in &lanes {
                    for k in 3..l.len().saturating_sub(1) {
                        let along = (l[k + 1] - l[k - 1]).normalize_or_zero();
                        if (l[k] - *c).normalize_or_zero().dot(along).abs() < 0.87 {
                            options.push(l[k]);
                        }
                    }
                }
                options.sort_by(|a, b| a.distance(*c).total_cmp(&b.distance(*c)));
                let fine = |lane: &Vec<Vec2>| lane.windows(2).all(|w| (0..4).all(|k| clear(w[0].lerp(w[1], k as f32 / 4.0))));
                options.into_iter().map(curve).find(fine).unwrap_or_else(|| curve(*c + Vec2::X))
            };
            lanes[i] = lane;
        }
        lanes
    })
}

/// Veiene og stiene utenfor muren som gjelder for punktet: (avstand til
/// midtlinja, fortegnet avstand på tvers, meter langs veien). Stiene til
/// gårdene er smalere enn veiene, så avstanden regnes litt lenger.
fn paths(x: f32, z: f32) -> Vec<(f32, f32, f32)> {
    let mut out = roads(x, z);
    let p = Vec2::new(x, z);
    for lane in farm_lanes() {
        let (lo, hi) = lane.iter().fold((Vec2::MAX, Vec2::MIN), |(lo, hi), q| (lo.min(*q), hi.max(*q)));
        if p.x < lo.x - 6.0 || p.y < lo.y - 6.0 || p.x > hi.x + 6.0 || p.y > hi.y + 6.0 {
            continue;
        }
        let st = Street { pts: lane.clone(), half: 0.0, cobbled: false };
        let (d, across, along) = street_coords(&st, p);
        out.push((d + 0.6, across, along));
    }
    out
}

/// De to veiene utenfor muren: sørover fra sørporten og nordover fra
/// nordporten forbi kirkegården. Begge går omtrent nord–sør, så midtlinja
/// er en x for hver z.
fn roads(x: f32, z: f32) -> Vec<(f32, f32, f32)> {
    let mut out = vec![];
    let (south, _) = gate(true);
    let (north, _) = gate(false);
    // Lengden langs veien regnes med helningen til midtlinja.
    let along = |cx: &dyn Fn(f32) -> f32, z0: f32, z: f32| -> (f32, f32) {
        let c = cx(z);
        let slope = (cx(z + 0.5) - cx(z - 0.5)).abs();
        let stretch = (1.0 + slope * slope).sqrt();
        ((x - c) / stretch, (z - z0) * stretch)
    };
    // Sørover fra sørporten og nordover fra nordporten forbi kirkegården,
    // svingende ut mellom åsene.
    for (from, sign) in [(south, 1.0f32), (north, -1.0)] {
        if (z - from.y) * sign < -2.0 {
            continue;
        }
        let cx = move |z: f32| road_x(sign > 0.0, z);
        let (u, v) = along(&cx, from.y, z);
        out.push(((x - cx(z)).abs(), u, v * sign));
    }
    out
}

/// Avstand fra et punkt til det bygde: byen innenfor muren, og kirkegården.
/// Der holder enga, trærne og dyrene seg unna.
pub fn town_dist(x: f32, z: f32) -> f32 {
    let (cx, cz, hx, hz) = terrain::YARD_FLAT;
    let yard = ((x - cx).abs() - hx).max((z - cz).abs() - hz);
    let farm = farms().iter().map(|c| c.distance(Vec2::new(x, z)) - FARM_R - 1.0).fold(f32::MAX, f32::min);
    terrain::town_edge(x, z).min(yard).min(farm)
}

/// Hvor torget er, til testene.
pub fn square_center() -> Vec3 {
    terrain::at(Vec3::ZERO)
}

