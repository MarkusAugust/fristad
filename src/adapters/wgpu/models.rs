//! Modellene på grafikkortet: nær og fjern detalj i samme buffer, de faste
//! instansene sortert i ruter, og teksturene. Modeller som kommer etter at
//! rendereren er laget (bitene til et kryptnivå i nettleseren, plan del 10a)
//! legges til med `add_models` og slippes med `drop_models`; instansene
//! deres står i `waiting` til da.

use super::*;
use crate::domain::model::{Image, Model, Placement};

/// Bygger en modell med instansene sine.
pub(super) fn gpu_model(device: &wgpu::Device, queue: &wgpu::Queue, materials: &Materials, mipper: &texture::Mipper, pl: &Placement, m: &Model) -> GpuModel {
    let (lo, hi) = m.mesh.bounds();
    let skinned = m.rig.is_some();
    // En skinnet figur strekker seg utenfor hvileposen når den
    // angriper eller faller; boksen får halvannen gang plassen.
    let (lo, hi) = if skinned {
        let (c, h) = ((lo + hi) * 0.5, (hi - lo) * 0.75);
        (c - h, c + h)
    } else {
        (lo, hi)
    };
    // Lakkfargen pakkes som 8 bit per kanal i et heltall (eksakt i f32).
    let packed = |tint: &Vec3| {
        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
        (c(tint.x) << 16 | c(tint.y) << 8 | c(tint.z)) as f32
    };
    let placements: Vec<(InstanceData, Vec3, Vec3, u32)> = pl
        .transforms
        .iter()
        .zip(pl.tints.iter().chain(std::iter::repeat(&Vec3::ONE)))
        .zip(pl.looks.iter().chain(std::iter::repeat(&None)))
        .map(|((t, tint), look)| {
            let (wlo, whi) = world_aabb(t, lo, hi);
            let inst = match look {
                // Utseendet: lagene for de fire rollene (5 bit hver, bit 20
                // slår det på), tonen, og flammefargen i extra2.y.
                Some(l) => {
                    let roles = l.layers.iter().enumerate().fold(1u32 << 20, |acc, (k, layer)| acc | (*layer as u32 & 31) << (5 * k));
                    let mut inst = InstanceData::new(t, [0.0, packed(tint), roles as f32, packed(&l.tint)]);
                    inst.extra2[1] = packed(&l.flame);
                    inst
                }
                None => InstanceData::new(t, [0.0, packed(tint), m.roughness, m.metallic]),
            };
            (inst, wlo, whi)
        })
        .enumerate()
        .map(|(n, (inst, wlo, whi))| (inst, wlo, whi, n as u32))
        .collect();
    // De faste instansene sorteres i ruter (rad for rad) og lastes
    // opp én gang; bak dem står plassen til de bevegelige.
    let key = |(_, lo, hi, _): &(InstanceData, Vec3, Vec3, u32)| {
        let c = (*lo + *hi) * 0.5 / CELL_M;
        (c.z.floor() as i32, c.x.floor() as i32)
    };
    let mut placements = placements;
    placements.sort_by_key(key);
    let mut cells: Vec<InstanceCell> = vec![];
    let mut last = None;
    for (k, p) in placements.iter().enumerate() {
        let id = key(p);
        match cells.last_mut() {
            Some(c) if last == Some(id) => {
                c.lo = c.lo.min(p.1);
                c.hi = c.hi.max(p.2);
                c.range.end = k as u32 + 1;
            }
            _ => cells.push(InstanceCell { lo: p.1, hi: p.2, range: k as u32..k as u32 + 1 }),
        }
        last = Some(id);
    }
    let statics = placements.len() as u32;
    let mut slot_of = vec![0u32; placements.len()];
    for (slot, p) in placements.iter().enumerate() {
        slot_of[p.3 as usize] = slot as u32;
    }
    let originals: Vec<InstanceData> = placements.iter().map(|p| p.0).collect();
    let mut data: Vec<InstanceData> = placements.iter().map(|p| p.0).collect();
    data.resize(placements.len() + pl.reserve * REGIONS as usize, InstanceData::new(&Affine3A::IDENTITY, [0.0; 4]));
    if data.is_empty() {
        data.push(InstanceData::new(&Affine3A::IDENTITY, [0.0; 4]));
    }
    // Nær og fjern detalj i samme buffer.
    let size = (hi - lo).max_element();
    // Figurene får også et grovt nivå (leddene og vektene følger med
    // fra hjørnet som blir stående), så de ikke tegnes med full
    // detalj langt unna og i skyggene.
    // Lette modeller tegnes helt også fjernt (`lod::far_detail`).
    let far = m.far_mesh.clone().or_else(|| lod::far_detail(&m.mesh, if m.far_cell > 0.0 { m.far_cell } else { size / 70.0 }));
    let mut combined = m.mesh.clone();
    let base = combined.vertices.len() as u32;
    let near_count = m.mesh.indices.len() as u32;
    let far_range = match &far {
        Some(far) => {
            combined.vertices.extend_from_slice(&far.vertices);
            combined.indices.extend(far.indices.iter().map(|i| i + base));
            near_count..combined.indices.len() as u32
        }
        None => 0..near_count,
    };
    log::info!("Modell {}: {} trekanter nær, {} fjern", m.name, near_count / 3, far_range.len() / 3);
    let (views, sizes) = materials.model_views(device, queue, mipper, m);
    // Bildene med lavt tak først (nettleseren) oppgraderes i bakgrunnen.
    let pending = [&m.albedo, &m.normal, &m.metal_rough, &m.emissive]
        .into_iter()
        .enumerate()
        .filter_map(|(k, img)| {
            let img = img.as_ref()?;
            let (width, height) = img.later?;
            Some((k, Image { width, height, later: None, ..img.clone() }))
        })
        .collect();
    GpuModel {
        name: m.name.clone(),
        cutout: m.mesh.vertices.iter().any(|v| v.mat[2] & flag::ALPHA != 0 || v.mat[0] == kind::LEAF),
        mesh: GpuMesh::new(&device, &combined, &m.name),
        near: 0..near_count,
        far: far_range,
        textures: materials.model_bind_group(device, &views, &m.name),
        views,
        sizes,
        pending,
        instances: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&m.name),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }),
        cells,
        statics,
        reserve: pl.reserve,
        bounds: (lo, hi),
        extra: [0.0, packed(&Vec3::ONE), m.roughness, m.metallic],
        skinned,
        small: !skinned && size < 1.3,
        interior: m.name.ends_with("_interior") || pl.indoor,
        indoor: pl.indoor,
        slot_of,
        originals,
        hidden: Default::default(),
    }
}

impl Renderer {
    /// Legger til modeller som kom senere. Bare modeller med instanser som
    /// venter (`waiting`) blir med; teksturene pakkes ut én om gangen.
    /// Er enheten borte (`lose`), venter de videre.
    pub fn add_models(&mut self, models: &[Model]) {
        if self.lost {
            return;
        }
        for m in models {
            let Some(k) = self.waiting.iter().position(|pl| pl.model == m.name) else { continue };
            let pl = self.waiting.swap_remove(k);
            let g = gpu_model(&self.device, &self.queue, &self.materials, &self.mipper, &pl, m);
            self.models.push(g);
            self.later.push(pl);
        }
    }

    /// Slipper modellene igjen; instansene venter til de kommer på nytt.
    pub fn drop_models(&mut self, names: &[String]) {
        self.models.retain(|g| !names.contains(&g.name));
        let (gone, kept): (Vec<Placement>, Vec<Placement>) = std::mem::take(&mut self.later).into_iter().partition(|pl| names.iter().any(|n| n == pl.model));
        self.later = kept;
        self.waiting.extend(gone);
    }

    /// Antall modeller på grafikkortet (til testene).
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Setter tiden oppgraderingen av teksturene får per bilde (ms), og
    /// glemmer det som var brukt for mye.
    pub fn set_upgrade_budget(&mut self, ms: f64) {
        self.upgrade_ms = ms;
        self.upgrade_credit = 0.0;
    }

    /// Bildene som venter på å oppgraderes.
    pub fn pending_textures(&self) -> usize {
        self.models.iter().map(|g| g.pending.len()).sum()
    }

    /// Bytene teksturene til modellene tar på grafikkortet nå (RGBA8 med
    /// mipmaps), til testene: den lave kopien slippes når den byttes ut.
    pub fn texture_bytes(&self) -> u64 {
        self.models.iter().flat_map(|g| g.sizes).map(|(w, h)| w as u64 * h as u64 * 4 * 4 / 3).sum()
    }

    /// Lat lasting (nettleseren): oppgraderer teksturene som ble lastet opp
    /// med lavt tak til de endelige målene, ett bilde av gangen, de nærmeste
    /// modellene først (`delivery::upgrade_pick`), innenfor tiden
    /// `upgrade_ms` per bilde i snitt. Kalles etter at bildet er sendt, så
    /// det første bildet tegnes med det lave. Gir modellene som fikk et
    /// bilde, i rekkefølge.
    pub fn upgrade_textures(&mut self, eye: Vec3, instances: &[crate::domain::scene::ModelInstance]) -> Vec<String> {
        let mut done = vec![];
        if self.lost || self.models.iter().all(|g| g.pending.is_empty()) {
            return done;
        }
        self.upgrade_credit = (self.upgrade_credit + self.upgrade_ms).min(self.upgrade_ms);
        // Avstanden til nærmeste instans: de faste rutene og de bevegelige.
        let distance = |g: &GpuModel| {
            let fixed = g.cells.iter().map(|c| eye.clamp(c.lo, c.hi).distance(eye));
            let moving = instances.iter().filter(|i| i.model == g.name).map(|i| Vec3::from(i.transform.translation).distance(eye));
            fixed.chain(moving).fold(f32::INFINITY, f32::min)
        };
        let near: Vec<f32> = self.models.iter().map(|g| if g.pending.is_empty() { f32::INFINITY } else { distance(g) }).collect();
        while self.upgrade_credit > 0.0 {
            let cands: Vec<(usize, usize)> = self.models.iter().enumerate().flat_map(|(k, g)| (0..g.pending.len()).map(move |j| (k, j))).collect();
            let pick = crate::adapters::delivery::upgrade_pick(cands.iter().enumerate().map(|(n, &(k, j))| {
                let img = &self.models[k].pending[j].1;
                (n, near[k], img.width.max(img.height))
            }));
            let Some(n) = pick else { break };
            let (k, j) = cands[n];
            let start = now_ms();
            let g = &mut self.models[k];
            let (slot, img) = g.pending.remove(j);
            // Fargen og gløden er sRGB, som i `model_views`.
            g.views[slot] = texture::upload(&self.device, &self.queue, &self.mipper, &img, slot == 0 || slot == 3, &g.name);
            g.sizes[slot] = (img.width, img.height);
            g.textures = self.materials.model_bind_group(&self.device, &g.views, &g.name);
            done.push(g.name.clone());
            self.upgrade_credit -= (now_ms() - start).max(0.25);
        }
        done
    }
}

/// Millisekunder fra en fast start, til tidsbudsjettet.
fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window().and_then(|w| w.performance()).map_or_else(js_sys::Date::now, |p| p.now())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        START.get_or_init(std::time::Instant::now).elapsed().as_secs_f64() * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::{Mat, Mesh};

    /// En kvadratmeter gulv.
    fn quad() -> Mesh {
        let mut m = Mesh::default();
        m.quad(&Affine3A::IDENTITY, [Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 1.0), Vec3::Z], &Mat::plain(0x808080, 0.8), [1.0; 4]);
        m
    }

    /// Som i nettleseren: biten til et kryptnivå er utsatt når rendereren
    /// lages og kommer med `add_models`. Er enheten borte før den kommer,
    /// lages ingenting på den (Chromium kastet fra `createBuffer`, og wgpu
    /// gjorde det til en panikk som stoppet motoren), og biten venter.
    #[test]
    fn kryptbitene_venter_om_grafikkortet_er_borte() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() });
        let Ok((adapter, device, queue)) = pollster::block_on(crate::adapters::gpu(&instance, None)) else {
            eprintln!("Ingen GPU; testen hoppes over");
            return;
        };
        let world = quad();
        let piece = Model {
            name: "crypt_01_aa".into(),
            mesh: quad(),
            albedo: None,
            normal: None,
            metal_rough: None,
            emissive: None,
            roughness: 0.8,
            metallic: 0.0,
            far_cell: 0.0,
            far_mesh: None,
            rig: None,
            solids: vec![],
        };
        let pl = Placement { model: "crypt_01_aa", transforms: vec![Affine3A::from_translation(Vec3::new(0.0, -40.0, 0.0))], tints: vec![], looks: vec![], reserve: 0, indoor: false };
        let scene = ports::StaticScene { mesh: &world, chunks: &[], placements: std::slice::from_ref(&pl), models: &[] };
        let mut r = Renderer::new(device, queue, &adapter, wgpu::TextureFormat::Rgba8UnormSrgb, &scene, &[], 64, 64);
        assert_eq!(r.model_count(), 0, "biten er utsatt");
        r.add_models(std::slice::from_ref(&piece));
        assert_eq!(r.model_count(), 1, "biten kom");
        r.drop_models(&[piece.name.clone()]);
        assert_eq!(r.model_count(), 0, "biten er sluppet");
        r.lose("testen");
        assert!(r.is_lost());
        r.add_models(std::slice::from_ref(&piece));
        assert_eq!(r.model_count(), 0, "ingenting lages på en tapt enhet");
        assert_eq!(r.waiting.len(), 1, "biten venter");
        r.resize(128, 128);
        r.set_quality(Quality::default());
    }

    /// Lat lasting: teksturene lastes opp med lavt tak og oppgraderes ett
    /// bilde av gangen, de nærmeste modellene først og de uten instanser
    /// sist; alle ender på de endelige målene, og den lave kopien slippes,
    /// så minnet aldri er mer enn det endelige.
    #[test]
    fn teksturene_oppgraderes_naermest_forst() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() });
        let Ok((adapter, device, queue)) = pollster::block_on(crate::adapters::gpu(&instance, None)) else {
            eprintln!("Ingen GPU; testen hoppes over");
            return;
        };
        let mut jpeg = vec![];
        image::RgbImage::from_fn(64, 64, |x, y| image::Rgb([(x * 4) as u8, (y * 4) as u8, 90])).write_to(&mut std::io::Cursor::new(&mut jpeg), image::ImageFormat::Jpeg).unwrap();
        let model = |name: &str| {
            let img = || crate::adapters::gltf::encoded(&jpeg, 0);
            let mut m = Model { name: name.into(), mesh: quad(), albedo: img(), normal: img(), metal_rough: None, emissive: None, roughness: 0.8, metallic: 0.0, far_cell: 0.0, far_mesh: None, rig: None, solids: vec![] };
            crate::adapters::delivery::stage(&mut m, crate::adapters::gltf::Caps { color: 16, data: 8 });
            assert_eq!(m.albedo.as_ref().map(|i| (i.width, i.later)), Some((16, Some((64, 64)))));
            m
        };
        let models = [model("langt"), model("nær"), model("figur")];
        let at = |name: &'static str, z: f32| Placement { model: name, transforms: vec![Affine3A::from_translation(Vec3::new(0.0, 0.0, z))], tints: vec![], looks: vec![], reserve: 0, indoor: false };
        let figure = Placement { model: "figur", transforms: vec![], tints: vec![], looks: vec![], reserve: 1, indoor: false };
        let placements = [at("langt", 60.0), at("nær", 4.0), figure];
        let world = quad();
        let scene = ports::StaticScene { mesh: &world, chunks: &[], placements: &placements, models: &models };
        let mut r = Renderer::new(device, queue, &adapter, wgpu::TextureFormat::Rgba8UnormSrgb, &scene, &[], 64, 64);
        let full = 6 * (64 * 64 * 4 * 4 / 3);
        let low = r.texture_bytes();
        assert!(low * 8 < full, "lavt først: {low} mot {full}");
        assert_eq!(r.pending_textures(), 6);
        // Ett bilde per kall (budsjettet er mindre enn ett bilde; det som
        // gikk over, glemmes her og betales ellers av de neste bildene).
        let mut order = vec![];
        let mut last = low;
        for _ in 0..10 {
            r.set_upgrade_budget(0.01);
            order.extend(r.upgrade_textures(Vec3::ZERO, &[]));
            let now = r.texture_bytes();
            assert!(now >= last && now <= full, "minnet vokser bare mot det endelige: {now} (før {last}, endelig {full})");
            last = now;
        }
        assert_eq!(order, ["nær", "nær", "langt", "langt", "figur", "figur"]);
        assert_eq!(r.pending_textures(), 0);
        assert_eq!(r.texture_bytes(), full, "alle på fullt tak");
        // En bevegelig figur nær kameraet går foran de faste langt unna.
        let mut r2 = {
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() });
            let (adapter, device, queue) = pollster::block_on(crate::adapters::gpu(&instance, None)).unwrap();
            Renderer::new(device, queue, &adapter, wgpu::TextureFormat::Rgba8UnormSrgb, &scene, &[], 64, 64)
        };
        r2.set_upgrade_budget(0.01);
        let near_figure = crate::domain::scene::ModelInstance { model: "figur", transform: Affine3A::from_translation(Vec3::new(1.0, 0.0, 0.0)), skin: None, flash: 0.0, tint: None };
        assert_eq!(r2.upgrade_textures(Vec3::ZERO, std::slice::from_ref(&near_figure)), ["figur"]);
        assert!(r2.upgrade_textures(Vec3::ZERO, &[]).is_empty(), "neste bilde betaler for det forrige");
    }
}
