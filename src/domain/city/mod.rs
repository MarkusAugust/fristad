//! Fristad bygges her, prosedyrisk, fra Fristil-tokens.
//!
//! Målestokken er at 1 rem er en meter: `spacing-4` (16px) er 1 m,
//! `spacing-12` (48px) er 3 m, som er etasjehøyden, og `spacing-6` (24px) er
//! 1,5 m, som er vindushøyden. Komponentene i byen har egen målestokk:
//! én CSS-piksel er én centimeter.
//!
//! Koordinater: X øst, Y opp, Z sør. Torget ligger rundt origo, og porten
//! med den store knappen står i nord.

use crate::domain::geometry::{Chunk, Layer, Mat, Mesh};
use crate::domain::installations::Installation;
use crate::domain::model::{ModelInfo, Placement};
use std::collections::HashMap;
use crate::domain::scene::Light;
use crate::domain::text::Font;
use crate::domain::tokens::{Theme, Tokens};
use glam::{Affine3A, Quat, Vec2, Vec3};

pub mod furniture;
pub mod house;
pub mod nature;
pub mod parachute;
pub mod park;
pub mod poster;
pub mod street_life;
pub mod terrain;
pub mod dungeon;
pub mod text_audit;
pub mod towers;

#[derive(Clone, Debug)]
pub enum Shape {
    /// Boks med sentrum, hele mål og rotasjon om Y.
    Cuboid { center: Vec3, size: Vec3, yaw: f32 },
    /// Stående sylinder med sentrum, radius og halv høyde.
    Cylinder { center: Vec3, radius: f32, half_height: f32 },
    /// Terrenget: et rutenett av høyder, `cols` langs X og `rows` langs Z,
    /// sentrert i `center` og `size` meter stort.
    Heightfield { center: Vec3, size: Vec3, cols: usize, rows: usize, heights: Vec<f32> },
    /// Kapsel mellom to punkter (trestammer og greiner).
    Capsule { a: Vec3, b: Vec3, radius: f32 },
    /// Konveks omhylling av punktene (fjell, åser).
    Hull { points: Vec<Vec3> },
}

/// Noe i byen som flytter seg eller kan siktes på: knapper, dører, brytere.
pub struct Prop {
    pub name: &'static str,
    /// En bevegelig del (portdør, fanedør) som installasjonen poserer, men
    /// som ikke er komponenten: kan ikke siktes på, og testene hopper over den.
    pub part: bool,
    pub mesh: Mesh,
    /// Hvor den står i ro.
    pub rest: Affine3A,
    /// Kollisjon i propens eget koordinatsystem: sentrum og halve mål.
    pub collider: Option<(Vec3, Vec3)>,
    /// Installasjonen som eier den, og dermed hva E, R og F gjør.
    pub owner: Option<usize>,
}

pub struct City {
    /// Den faste geometrien, stokket i biter (`chunks`).
    pub mesh: Mesh,
    pub chunks: Vec<Chunk>,
    pub colliders: Vec<Shape>,
    pub lights: Vec<Light>,
    pub props: Vec<Prop>,
    pub placements: Vec<Placement>,
    pub spawn: Vec3,
    pub spawn_yaw: f32,
    /// Det `Builder::audit` fant: modeller som svever eller synker. Tomt
    /// når alt står på bakken; testen i `app.rs` krever det.
    pub audit: Vec<String>,
    pub dungeon: dungeon::Dungeon,
}

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.next()
    }
    pub fn pick<T: Copy>(&mut self, v: &[T]) -> T {
        v[((self.next() * v.len() as f32) as usize).min(v.len() - 1)]
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.next() < p
    }
}

/// Det installasjonene og bydelene bygger med.
/// En husfasade, så gatelivet kan stille ting inntil ekte vegger.
#[derive(Clone, Copy, Debug)]
pub struct Facade {
    /// Midt på fasaden, i bakkehøyde.
    pub origin: Vec3,
    /// Langs fasaden (lokal +X for huset).
    pub along: Vec3,
    /// Ut fra fasaden, mot gata.
    pub facing: Vec3,
    pub width: f32,
    /// Dørposisjonen langs fasaden, fra midten.
    pub door: f32,
    pub shop: bool,
}

pub struct Builder<'a> {
    pub mesh: Mesh,
    pub facades: Vec<Facade>,
    pub colliders: Vec<Shape>,
    pub lights: Vec<Light>,
    pub props: Vec<Prop>,
    pub placements: Vec<Placement>,
    pub font: &'a Font,
    pub tokens: &'a Tokens,
    /// Komponentarkene rendret fra CSS, til `signs::sheet`.
    pub sheets: &'a crate::domain::sheets::Sheets,
    pub rng: Rng,
    /// Installasjonen som bygger nå, så propsene får riktig eier.
    pub owner: Option<usize>,
    /// Modellene som finnes, med mål. Byen bruker dem framfor prosedyrisk
    /// geometri når de er der.
    pub models: HashMap<String, ModelInfo>,
    /// Gulv over terrenget, til `audit` (`floor`).
    pub floors: Vec<(Vec3, Vec2, f32, f32)>,
    /// Tekst som er bredere enn plata si (`fits`), til `audit`.
    pub text_audit: Vec<String>,
    /// Instanser som henger på vegger eller står oppå noe (`place_mounted`):
    /// (modell, instansnummer), som `audit` hopper over.
    pub mounted: Vec<(&'static str, usize)>,
    /// Kjelleren, når den er bygd (før installasjonene), så brønnen kan
    /// finne startrommet.
    pub dungeon: Option<dungeon::Dungeon>,
}

impl Builder<'_> {
    pub fn prop(&mut self, name: &'static str, mesh: Mesh, rest: Affine3A, collider: Option<(Vec3, Vec3)>) -> usize {
        self.props.push(Prop { name, part: false, mesh, rest, collider, owner: self.owner });
        self.props.len() - 1
    }

    /// En bevegelig del av en installasjon (dører): poseres av den, men er
    /// ikke komponenten, så den kan ikke siktes på.
    pub fn prop_part(&mut self, name: &'static str, mesh: Mesh, rest: Affine3A, collider: Option<(Vec3, Vec3)>) -> usize {
        let i = self.prop(name, mesh, rest, collider);
        self.props[i].part = true;
        i
    }

    pub fn light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: false, flicker: 0.0 });
    }

    /// Et lys som brenner hele døgnet (ild), med et lite flakk.
    pub fn fire_light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: true, flicker: 0.18 });
    }

    /// Et lys som brenner hele døgnet og flakker så mye du vil (lysrør).
    pub fn flicker_light(&mut self, pos: Vec3, color: Vec3, intensity: f32, radius: f32, flicker: f32) {
        self.lights.push(Light { pos, radius, color, intensity, always: true, flicker });
    }

    /// Et gulv over terrenget (inne i et hus): `audit` måler mot det i
    /// stedet for bakken innenfor rektangelet. `center` er midten, `half`
    /// halve målene i x og z, `top` gulvets overkant (y i verden).
    pub fn floor(&mut self, center: Vec3, half: Vec2, yaw: f32, top: f32) {
        self.floors.push((center, half, yaw, top));
    }

    /// Høyden på det noe kan stå på i et punkt: det registrerte gulvet (hus,
    /// kjelleretasjene) eller terrenget som ligger nærmest under `y`, så
    /// ting på torget ikke måles mot kjelleren under.
    pub fn stand_height(&self, x: f32, z: f32, y: f32) -> f32 {
        let mut best = terrain::height(x, z);
        let score = |top: f32| if top <= y + 0.3 { y - top } else { 1e6 + top - y };
        for (c, half, yaw, top) in &self.floors {
            let (s, co) = yaw.sin_cos();
            let (dx, dz) = (x - c.x, z - c.z);
            // Inn i gulvets lokale akser (som `at`).
            let lx = dx * co - dz * s;
            let lz = dx * s + dz * co;
            if lx.abs() <= half.x && lz.abs() <= half.y && score(*top) < score(best) {
                best = *top;
            }
        }
        best
    }

    pub fn solid(&mut self, shape: Shape) {
        self.colliders.push(shape);
    }

    pub fn solid_box(&mut self, center: Vec3, size: Vec3, yaw: f32) {
        self.colliders.push(Shape::Cuboid { center, size, yaw });
    }

    /// Om en boks (sentrum, hele mål, rotasjon om Y) overlapper et hus
    /// eller en annen bokskollider som alt er lagt inn. Brukes til å
    /// parkere biler uten å kjøre dem inn i veggene.
    pub fn overlaps_solid(&self, center: Vec3, size: Vec3, yaw: f32) -> bool {
        let hit = self.colliders.iter().find(|c| match c {
            Shape::Cuboid { center: c2, size: s2, yaw: y2 } => {
                // Bare bokser som når ned til bakken teller (hus, sokler).
                let (lo, hi) = (c2.y - s2.y * 0.5, c2.y + s2.y * 0.5);
                hi > center.y - size.y * 0.5 && lo < center.y + size.y * 0.5 && rects_overlap((center.x, center.z), (size.x * 0.5, size.z * 0.5), yaw, (c2.x, c2.z), (s2.x * 0.5, s2.z * 0.5), *y2)
            }
            Shape::Cylinder { center: c2, radius, half_height } => {
                // Sylinderen tilnærmes med den omskrevne boksen.
                let (lo, hi) = (c2.y - half_height, c2.y + half_height);
                hi > center.y - size.y * 0.5 && lo < center.y + size.y * 0.5 && rects_overlap((center.x, center.z), (size.x * 0.5, size.z * 0.5), yaw, (c2.x, c2.z), (*radius, *radius), 0.0)
            }
            Shape::Capsule { a, b, radius } => {
                // Kapselen tilnærmes med boksen rundt den.
                let (lo, hi) = (a.y.min(b.y) - radius, a.y.max(b.y) + radius);
                let c2 = (*a + *b) * 0.5;
                let half = ((*a - *b) * 0.5).abs() + Vec3::splat(*radius);
                hi > center.y - size.y * 0.5 && lo < center.y + size.y * 0.5 && rects_overlap((center.x, center.z), (size.x * 0.5, size.z * 0.5), yaw, (c2.x, c2.z), (half.x, half.z), 0.0)
            }
            Shape::Heightfield { .. } | Shape::Hull { .. } => false,
        });
        if let Some(c) = hit {
            log::debug!("boks ({:.1}, {:.1}) {:.1}×{:.1} treffer {:?}", center.x, center.z, size.x, size.z, c);
        }
        hit.is_some()
    }

    pub fn has_model(&self, name: &str) -> bool {
        self.models.contains_key(name)
    }

    /// Høyden på en modell, eller 0 om den ikke finnes.
    pub fn model_height(&self, name: &str) -> f32 {
        self.models.get(name).map(|m| m.max.y - m.min.y).unwrap_or(0.0)
    }

    pub fn place(&mut self, model: &'static str, t: Affine3A) {
        self.place_tinted(model, t, Vec3::ONE);
    }

    /// Sett ut en modell med egen lakkfarge (lineær RGB) på flatene som er
    /// merket «paint» i modellen.
    pub fn place_tinted(&mut self, model: &'static str, t: Affine3A, tint: Vec3) {
        if let Some(p) = self.placements.iter_mut().find(|p| p.model == model) {
            p.transforms.push(t);
            p.tints.push(tint);
        } else {
            self.placements.push(Placement { model, transforms: vec![t], tints: vec![tint], reserve: 0 });
        }
    }

    /// Sett ut en modell som henger på en vegg eller står oppå noe annet,
    /// så `audit` ikke måler den mot bakken.
    pub fn place_mounted(&mut self, model: &'static str, t: Affine3A) {
        self.place(model, t);
        if let Some(p) = self.placements.iter().find(|p| p.model == model) {
            self.mounted.push((model, p.transforms.len() - 1));
        }
    }

    /// Sett av plass til `n` instanser av en modell som spillet flytter
    /// per bilde (dyrene). Gjør ingenting om modellen mangler.
    pub fn reserve(&mut self, model: &'static str, n: usize) {
        if !self.has_model(model) {
            log::warn!("Modellen {model} mangler, ingen plass satt av");
            return;
        }
        if let Some(p) = self.placements.iter_mut().find(|p| p.model == model) {
            p.reserve += n;
        } else {
            self.placements.push(Placement { model, transforms: vec![], tints: vec![], reserve: n });
        }
    }

    /// Bakkehøyden der noe skal stå.
    pub fn ground(&self, x: f32, z: f32) -> f32 {
        terrain::height(x, z)
    }

    /// Punktet løftet opp på bakken.
    pub fn on_ground(&self, p: Vec3) -> Vec3 {
        Vec3::new(p.x, p.y + terrain::height(p.x, p.z), p.z)
    }

    /// Noe som står på bakken og vippes etter hellingen (biler, dyr), så
    /// ingen av hjørnene svever på nedsida eller synker på oppsida.
    pub fn on_slope(&self, p: Vec3, yaw: f32) -> Affine3A {
        let n = terrain::normal(p.x, p.z);
        Affine3A::from_rotation_translation(Quat::from_rotation_arc(Vec3::Y, n) * Quat::from_rotation_y(yaw), self.on_ground(p))
    }

    /// Rutinen mot tekst utenfor skilt: meld til `audit` om teksten (i
    /// `px` med `scale` meter per px) er bredere enn plata minus 8 cm marg.
    /// Bruk den der ei plate har fast bredde; ellers regn plata ut fra
    /// `font.width`.
    pub fn fits(&mut self, what: &str, text: &str, px: f32, scale: f32, plate_w: f32) {
        let w = self.font.width(text, px) * scale;
        if w > plate_w - 0.08 {
            self.text_audit.push(format!("{what}: «{text}» er {:.0} cm bred, plata {:.0} cm", w * 100.0, plate_w * 100.0));
        }
    }

    /// Rutinen mot svevende modeller: hver instans av hver modell sjekkes
    /// mot terrenget. Svever det laveste hjørnet mer enn 8 cm over bakken,
    /// eller står et hjørne mer enn 30 cm under den, meldes det. Modeller
    /// som henger på vegger (vinduer, dørlamper, blomsterkasser) hoppes over.
    pub fn audit(&self) -> Vec<String> {
        const ON_WALLS: &[&str] = &["window_22", "window_23", "door_lamp", "flower_box"];
        let mut out = vec![];
        for pl in &self.placements {
            if ON_WALLS.contains(&pl.model) {
                continue;
            }
            let Some(info) = self.models.get(pl.model) else { continue };
            for (n, t) in pl.transforms.iter().enumerate() {
                if self.mounted.contains(&(pl.model, n)) {
                    continue;
                }
                // Måles innen 60 cm fra origo: trær og busker har krona langt
                // utenfor stammen, og der er bakken en annen.
                let (hx, hz) = ((info.max.x - info.min.x).min(1.2) * 0.5, (info.max.z - info.min.z).min(1.2) * 0.5);
                let corners = [(-hx, -hz), (hx, -hz), (-hx, hz), (hx, hz)];
                let gaps = corners.map(|(x, z)| {
                    let w = t.transform_point3(Vec3::new(x, info.min.y, z));
                    w.y - self.stand_height(w.x, w.z, w.y)
                });
                let lowest = gaps.iter().cloned().fold(f32::MAX, f32::min);
                let at = t.translation;
                if lowest > 0.08 {
                    out.push(format!("{} ved ({:.1}, {:.1}) svever {:.0} cm over bakken", pl.model, at.x, at.z, lowest * 100.0));
                } else if lowest < -0.3 {
                    out.push(format!("{} ved ({:.1}, {:.1}) står {:.0} cm nede i bakken", pl.model, at.x, at.z, -lowest * 100.0));
                }
            }
        }
        out
    }

    /// Meter per spacing-trinn.
    pub fn unit(&self, step: &str) -> f32 {
        self.tokens.spacing_px(step) / 16.0
    }

    /// Fasadefargene hentes fra Fristil sine lyse tokens og blandes mot
    /// kalkhvitt, slik malte nordiske trehus og pussede murhus ser ut.
    pub fn paints(&self) -> Vec<Vec3> {
        let lime = Vec3::new(0.93, 0.9, 0.84);
        [
            ("neutral-surface", 0.2),
            ("brand3-fill", 0.82),
            ("warning-fill", 0.45),
            ("accent-border-subtle", 0.3),
            ("brand1-border-subtle", 0.25),
            ("neutral-raised", 0.1),
            ("brand3-fill", 0.62),
            ("warning-border-subtle", 0.55),
            ("success-border-subtle", 0.4),
            ("brand2-border-subtle", 0.35),
        ]
        .iter()
        .map(|(name, amount)| lime.lerp(self.tokens.color(Theme::Light, name), *amount))
        .collect()
    }
}

pub const WHITE_TRIM: Mat = Mat::plain(0xf2efe8, 0.55);
pub const DARK_TRIM: Mat = Mat::plain(0x2b2d30, 0.6);
pub const IRON: Mat = Mat::tex(Layer::Metal, 0xffffff);
pub const STONE: Mat = Mat::tex(Layer::Stone, 0xffffff);
pub const CONCRETE: Mat = Mat::tex(Layer::Concrete, 0xffffff);
pub const COBBLE: Mat = Mat::tex(Layer::Cobble, 0xffffff);
pub const OAK: Mat = Mat::tex(Layer::Oak, 0xffffff);

/// Der du starter, og dit «Torget»-skiltene og H tar deg: føttene på torget
/// (y er høyde over terrenget) og retningen, mot plakaten.
pub const SPAWN: Vec3 = Vec3::new(7.0, 0.0, 9.0);
pub const SPAWN_YAW: f32 = -0.9;

pub fn build(tokens: &Tokens, font: &Font, sheets: &crate::domain::sheets::Sheets, models: &[ModelInfo], installations: &mut [Box<dyn Installation>]) -> City {
    let mut b = Builder {
        mesh: Mesh::default(),
        facades: vec![],
        colliders: vec![],
        lights: vec![],
        props: vec![],
        placements: vec![],
        font,
        tokens,
        sheets,
        rng: Rng(0x5eed_f215_7a1d),
        owner: None,
        models: models.iter().map(|m| (m.name.clone(), m.clone())).collect(),
        floors: vec![],
        mounted: vec![],
        text_audit: vec![],
        dungeon: None,
    };

    // Bakken: et rutenett som følger terrenget, med brostein på torget og i
    // gatene, grus rundt husene og gress utenfor.
    ground(&mut b);

    house::district(&mut b);
    furniture::plaza(&mut b);
    poster::build(&mut b);
    parachute::build(&mut b);
    furniture::streets(&mut b);
    street_life::all(&mut b);
    park::build(&mut b);
    towers::build(&mut b);
    nature::plaza_trees(&mut b);
    nature::tufts(&mut b);
    nature::meadow(&mut b);
    nature::grass(&mut b);
    nature::landscape(&mut b);
    nature::world_edge(&mut b);
    // Legacy-kjelleren under torget. Et nytt frø gir en ny etasje.
    let dungeon = dungeon::build(&mut b, dungeon_seed());
    b.dungeon = Some(dungeon);
    for (model, n) in crate::domain::mobs::Mobs::reserve() {
        b.reserve(model, n);
    }
    // Armene med sverdet, og rusket som spruter.
    for (model, n) in crate::domain::combat::Combat::reserve() {
        b.reserve(model, n);
    }
    // Dyrene: spillet poserer dem, byen setter av plass til instansene.
    for (model, n) in crate::domain::herd::Herd::reserve() {
        b.reserve(model, n);
    }

    for (i, inst) in installations.iter_mut().enumerate() {
        b.owner = Some(i);
        inst.build(&mut b);
    }
    b.owner = None;

    let mut audit = b.audit();
    audit.extend(b.text_audit.iter().cloned());
    if std::env::var("TEXT_AUDIT").is_ok() {
        // Tar noen sekunder; kjøres av cargo test og `TEXT_AUDIT=1 snapshot`.
        let props: Vec<(&Mesh, Affine3A)> = b.props.iter().map(|p| (&p.mesh, p.rest)).collect();
        audit.extend(text_audit::audit(&b.mesh, &props));
    }
    for line in &audit {
        log::warn!("{line}");
    }

    // Biter på 24 m, med småting (under 18 cm) sist i hver bit.
    let (mesh, chunks) = b.mesh.chunked(24.0, 0.18);
    log::info!("Byen: {} biter", chunks.len());
    City {
        mesh,
        chunks,
        colliders: b.colliders,
        lights: b.lights,
        props: b.props,
        placements: b.placements,
        dungeon: b.dungeon.take().expect("kjelleren er bygd"),
        spawn: SPAWN,
        spawn_yaw: SPAWN_YAW,
        audit,
    }
}

/// Avstand fra et punkt til et rektangel (0 innenfor).
fn rect_dist(x: f32, z: f32, hx: f32, z0: f32, z1: f32) -> f32 {
    let dx = (x.abs() - hx).max(0.0);
    let dz = (z0 - z).max(z - z1).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

/// Avstand fra et punkt til et vilkårlig rektangel (0 innenfor).
fn box_dist(x: f32, z: f32, x0: f32, x1: f32, z0: f32, z1: f32) -> f32 {
    let dx = (x0 - x).max(x - x1).max(0.0);
    let dz = (z0 - z).max(z - z1).max(0.0);
    (dx * dx + dz * dz).sqrt()
}

/// Avstand fra et punkt til bebyggelsen eller Tokenparken: der holder enga
/// seg unna.
pub fn town_dist(x: f32, z: f32) -> f32 {
    buildings_dist(x, z)
        .min(box_dist(x, z, park::X0 - 1.0, park::X1 + 1.0, park::Z0 - 1.0, park::Z1 + 1.0))
        .min(((x - towers::CENTER.x).powi(2) + (z - towers::CENTER.z).powi(2)).sqrt() - towers::RADIUS - 4.0)
}

/// Avstand fra et punkt til husene: torget med husrekkene og gatene. Rundt
/// dem ligger grusbeltet.
pub fn buildings_dist(x: f32, z: f32) -> f32 {
    rect_dist(x, z, 34.0, -32.0, 32.0)
        .min(box_dist(x, z, 6.0, 26.0, 66.0, 86.0))
        .min(rect_dist(x, z, 13.0, -66.0, -19.0))
        .min(rect_dist(x, z, 12.0, 19.0, 62.0))
        .min(rect_dist(z, x, 11.0, -46.0, -19.0))
        .min(rect_dist(z, x, 17.0, 19.0, 52.0))
        // Gorveks tomt ved veien nordover, og byggeplassen ved Østre gate.
        .min(box_dist(x, z, 10.5, 24.5, -78.0, -65.0))
        .min(box_dist(x, z, 42.0, 56.0, -29.0, -10.0))
}

/// Midten av grusveien østover (z for en x øst for bebyggelsen).
pub fn road_east_z(x: f32) -> f32 {
    (terrain::noise(x * 0.03, 4.0) - 0.5) * 26.0
}

/// Stiene ut av byen: grusveier som slynger seg videre fra gatene.
/// Gir avstanden til nærmeste sti.
pub fn path_dist(x: f32, z: f32) -> f32 {
    let mut d = f32::MAX;
    // Nordover fra gata, østover og vestover fra torget, sørover fra sørgata.
    let n = terrain::noise(z * 0.03, 1.0);
    if z < -60.0 {
        // Svinger inn mot plassen mellom tårnene, som er grus.
        let blend = ((-z - 60.0) / 16.0).clamp(0.0, 1.0);
        d = d.min((x - (n - 0.5) * 24.0 * (1.0 - blend)).abs());
        d = d.min((((x - towers::CENTER.x).powi(2) + (z - towers::CENTER.z).powi(2)).sqrt() - 10.0).max(0.0));
    }
    if z > 56.0 {
        let n = terrain::noise(z * 0.03, 9.0);
        d = d.min((x - (n - 0.5) * 30.0 - (z - 56.0) * 0.25).abs());
    }
    if x > 44.0 {
        d = d.min((z - road_east_z(x)).abs());
    }
    if x < -44.0 {
        // Rett inn porten til Tokenparken, og stiene inne i parken.
        d = d.min(z.abs());
        if (park::X0..=park::X1).contains(&x) && (park::Z0..=park::Z1).contains(&z) {
            d = d.min((x - (park::X0 + 12.0)).abs()).min((x - (park::X0 + 24.0)).abs());
        }
        // Videre vestover fra parken.
        if x < park::X0 {
            let n = terrain::noise(x * 0.03, 6.0);
            d = d.min((z - (n - 0.5) * 20.0 + (x - park::X0) * 0.2).abs());
        }
    }
    d
}

/// Fargen på enga i et punkt: grønnere i søkk, gulere på ryggene, med
/// flekker, og et par åkrer i gult.
pub(crate) fn meadow_tint(x: f32, z: f32) -> u32 {
    let n1 = terrain::noise(x * 0.05 + 2.0, z * 0.05);
    let n2 = terrain::noise(x * 0.17, z * 0.17 + 3.0);
    let green = Vec3::new(0.72, 0.95, 0.55);
    let straw = Vec3::new(1.0, 0.93, 0.62);
    let dark = Vec3::new(0.6, 0.82, 0.48);
    let mut c = green.lerp(straw, (n1 * 1.3 - 0.2).clamp(0.0, 1.0));
    c = c.lerp(dark, (n2 - 0.55).max(0.0) * 1.5);
    // Åkrer: to ovaler.
    for (cx, cz, rx, rz) in [(95.0, -40.0, 40.0, 22.0), (-80.0, 70.0, 30.0, 26.0)] {
        let e = ((x - cx) / rx).powi(2) + ((z - cz) / rz).powi(2);
        if e < 1.0 {
            c = Vec3::new(1.05, 0.95, 0.6);
        }
    }
    let to8 = |v: f32| ((v.clamp(0.0, 1.0)) * 255.0) as u32;
    (to8(c.x) << 16) | (to8(c.y) << 8) | to8(c.z)
}

/// Hva bakken er dekket av i et punkt.
fn ground_mat(x: f32, z: f32) -> Mat {
    let in_square = x.abs() < 21.0 && z.abs() < 19.0;
    let in_north = x.abs() < 6.0 && (-62.0..-19.0).contains(&z);
    let in_south = x.abs() < 5.0 && (19.0..58.0).contains(&z);
    let in_west = z.abs() < 4.5 && (-42.0..-19.0).contains(&x);
    let in_east = z.abs() < 4.5 && (19.0..42.0).contains(&x);
    if in_square || in_north || in_south || in_west || in_east {
        return COBBLE;
    }
    // Et smalt belte med grus rundt husene, med ujevn kant, og grusstiene.
    let edge = 2.0 + terrain::noise(x * 0.09, z * 0.09) * 4.0;
    if buildings_dist(x, z) < edge || path_dist(x, z) < 1.6 + terrain::noise(x * 0.2, z * 0.2) * 1.2 {
        return Mat::tex(Layer::Gravel, 0xf0e6d6);
    }
    Mat::tex(Layer::Grass, meadow_tint(x, z))
}

fn ground(b: &mut Builder) {
    let half = 200.0f32;
    let cell = 2.0f32;
    let n = (2.0 * half / cell) as usize;
    let mut heights = Vec::with_capacity((n + 1) * (n + 1));
    for iz in 0..=n {
        for ix in 0..=n {
            let x = -half + ix as f32 * cell;
            let z = -half + iz as f32 * cell;
            heights.push(terrain::height(x, z));
        }
    }
    for iz in 0..n {
        for ix in 0..n {
            let x0 = -half + ix as f32 * cell;
            let z0 = -half + iz as f32 * cell;
            let h = |i: usize, j: usize| heights[j * (n + 1) + i];
            let p = [
                Vec3::new(x0, h(ix, iz + 1), z0 + cell),
                Vec3::new(x0 + cell, h(ix + 1, iz + 1), z0 + cell),
                Vec3::new(x0 + cell, h(ix + 1, iz), z0),
                Vec3::new(x0, h(ix, iz), z0),
            ];
            let nrm = [
                terrain::normal(p[0].x, p[0].z),
                terrain::normal(p[1].x, p[1].z),
                terrain::normal(p[2].x, p[2].z),
                terrain::normal(p[3].x, p[3].z),
            ];
            let mat = ground_mat(x0 + cell * 0.5, z0 + cell * 0.5);
            b.mesh.quad_n(p, nrm, &mat, 1.0);
        }
    }
    b.solid(Shape::Heightfield { center: Vec3::ZERO, size: Vec3::new(2.0 * half, 1.0, 2.0 * half), cols: n + 1, rows: n + 1, heights });
}

/// Frøet til kjelleren: fast i tester og skjermbilder (`FRISTAD_SEED`
/// overstyrer), ellers fra klokka i nettleseren.
fn dungeon_seed() -> u64 {
    if let Ok(s) = std::env::var("FRISTAD_SEED") {
        return s.parse().unwrap_or(7);
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() as u64) ^ 0x5eed
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        7
    }
}

/// Transform med rotasjon om Y og posisjon.
/// To roterte rektangler i xz-planet, testet med skilleakser.
fn rects_overlap(c1: (f32, f32), h1: (f32, f32), yaw1: f32, c2: (f32, f32), h2: (f32, f32), yaw2: f32) -> bool {
    let corners = |c: (f32, f32), h: (f32, f32), yaw: f32| -> [(f32, f32); 4] {
        // Lokal x-akse (cos, −sin) og z-akse (sin, cos), som `at`.
        let (s, co) = yaw.sin_cos();
        let ax = (co, -s);
        let az = (s, co);
        let mut out = [(0.0, 0.0); 4];
        for (k, (sx, sz)) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].into_iter().enumerate() {
            out[k] = (c.0 + ax.0 * h.0 * sx + az.0 * h.1 * sz, c.1 + ax.1 * h.0 * sx + az.1 * h.1 * sz);
        }
        out
    };
    let (a, b) = (corners(c1, h1, yaw1), corners(c2, h2, yaw2));
    for yaw in [yaw1, yaw2] {
        let (s, co) = yaw.sin_cos();
        for axis in [(co, -s), (s, co)] {
            let proj = |r: &[(f32, f32); 4]| {
                let ps = r.map(|(x, z)| x * axis.0 + z * axis.1);
                (ps.iter().cloned().fold(f32::MAX, f32::min), ps.iter().cloned().fold(f32::MIN, f32::max))
            };
            let (a0, a1) = proj(&a);
            let (b0, b1) = proj(&b);
            if a1 < b0 || b1 < a0 {
                return false;
            }
        }
    }
    true
}

pub fn at(p: Vec3, yaw: f32) -> Affine3A {
    Affine3A::from_rotation_translation(Quat::from_rotation_y(yaw), p)
}
