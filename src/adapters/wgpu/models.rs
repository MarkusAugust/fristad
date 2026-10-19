//! Modellene på grafikkortet: nær og fjern detalj i samme buffer, de faste
//! instansene sortert i ruter, og teksturene. Modeller som kommer etter at
//! rendereren er laget (bitene til et kryptnivå i nettleseren, plan del 10a)
//! legges til med `add_models` og slippes med `drop_models`; instansene
//! deres står i `waiting` til da.

use super::*;
use crate::domain::model::{Model, Placement};

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
    GpuModel {
        name: m.name.clone(),
        cutout: m.mesh.vertices.iter().any(|v| v.mat[2] & flag::ALPHA != 0 || v.mat[0] == kind::LEAF),
        mesh: GpuMesh::new(&device, &combined, &m.name),
        near: 0..near_count,
        far: far_range,
        textures: materials.model_bind_group(&device, &queue, &mipper, m),
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
}
