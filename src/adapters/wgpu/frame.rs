//! Ett bilde: det som avgjøres én gang (`Plan`), hvilke instanser som
//! synes, verdiene til shaderne, og passene i rekkefølge (`passes.rs`).

use super::*;

/// Det som avgjøres én gang per bilde og brukes av alle passene.
pub(super) struct Plan {
    pub(super) view_proj: Mat4,
    /// Sola når ned (ikke nede i kjelleren).
    pub(super) sunlit: bool,
    pub(super) shadows: bool,
    /// Sidene per kaskade i skyggekartet, og hvor mange kaskader.
    pub(super) res: u32,
    pub(super) cascades: u32,
    pub(super) sun_vp: Mat4,
    pub(super) sun_vp2: Mat4,
    pub(super) sun_cull: Cull,
    pub(super) cam_cull: Cull,
    /// Uten «småting langt unna»: grovere detalj tidligere.
    pub(super) reduced: bool,
    /// Sikten (m), uendelig om verden tegnes så langt den går.
    pub(super) far: f32,
}

impl Renderer {
    /// Ortografisk projeksjon fra sola som dekker kamerafrustumet mellom
    /// `near` og `far`, låst til hele texels så kantene ikke skimrer.
    fn cascade(&self, f: &Frame, near: f32, far: f32, res: u32) -> Mat4 {
        let inv = (f.proj * f.view).inverse();
        let ndc_z = |d: f32| {
            let c = f.proj * Vec4::new(0.0, 0.0, -d, 1.0);
            c.z / c.w
        };
        let (zn, zf) = (ndc_z(near), ndc_z(far));
        let mut corners = [Vec3::ZERO; 8];
        let mut k = 0;
        for z in [zn, zf] {
            for y in [-1.0f32, 1.0] {
                for x in [-1.0f32, 1.0] {
                    let p = inv * Vec4::new(x, y, z, 1.0);
                    corners[k] = p.truncate() / p.w;
                    k += 1;
                }
            }
        }
        let center = corners.iter().fold(Vec3::ZERO, |a, c| a + *c) / 8.0;
        let radius = corners.iter().map(|c| c.distance(center)).fold(0.0, f32::max) * 1.05;
        let up = if f.sun_dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
        let view = Mat4::look_at_rh(f.sun_dir * 400.0, Vec3::ZERO, up);
        let texel = 2.0 * radius / res.max(1) as f32;
        let mut c = view.transform_point3(center);
        c.x = (c.x / texel).round() * texel;
        c.y = (c.y / texel).round() * texel;
        // z i lysrommet er negativt fremover. Ta med alt mellom lyset og sonen.
        let proj = Mat4::orthographic_rh(c.x - radius, c.x + radius, c.y - radius, c.y + radius, -c.z - radius - 200.0, -c.z + radius);
        proj * view
    }

    /// Skjul de faste instansene i `Frame.hidden` (flyttet langt ned, så
    /// verken bildet eller skyggen ser dem) og vis de som ikke lenger er
    /// skjult, med dataene de ble lastet opp med.
    fn apply_hidden(&self, f: &Frame) {
        let away = InstanceData::new(&Affine3A::from_translation(Vec3::new(0.0, -1.0e6, 0.0)), [0.0; 4]);
        for m in &self.models {
            let mut want: Vec<u32> = f.hidden.iter().filter(|(name, _)| *name == m.name).filter_map(|(_, n)| m.slot_of.get(*n).copied()).collect();
            let mut now = m.hidden.borrow_mut();
            if want.is_empty() && now.is_empty() {
                continue;
            }
            want.sort_unstable();
            want.dedup();
            for slot in now.iter().filter(|s| !want.contains(s)) {
                self.queue.write_buffer(&m.instances, *slot as u64 * INSTANCE_STRIDE, bytemuck::bytes_of(&m.originals[*slot as usize]));
            }
            for slot in want.iter().filter(|s| !now.contains(s)) {
                self.queue.write_buffer(&m.instances, *slot as u64 * INSTANCE_STRIDE, bytemuck::bytes_of(&away));
            }
            *now = want;
        }
    }

    /// Linjene til redigeringen i bufferet; gir hvor mange hjørner.
    fn upload_lines(&self, f: &Frame) -> u32 {
        if f.lines.is_empty() {
            return 0;
        }
        let data: Vec<[f32; 6]> = f.lines.iter().flat_map(|(a, b, c)| [[a.x, a.y, a.z, c.x, c.y, c.z], [b.x, b.y, b.z, c.x, c.y, c.z]]).collect();
        let bytes = bytemuck::cast_slice::<[f32; 6], u8>(&data);
        let mut buf = self.line_buf.borrow_mut();
        if buf.as_ref().is_none_or(|(_, cap)| *cap < bytes.len() as u64) {
            let cap = (bytes.len() as u64).next_power_of_two().max(4096);
            *buf = Some((self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("lines"), size: cap, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }), cap));
        }
        let (b, _) = buf.as_ref().expect("bufferet");
        self.queue.write_buffer(b, 0, bytes);
        data.len() as u32
    }

    fn plan(&self, f: &Frame) -> Plan {
        let view_proj = f.proj * f.view;
        // Nede i kjelleren når ikke sola ned: ingen skygge, og sollyset er av.
        let q = self.quality;
        let reduced = !q.detail;
        let sunlit = f.rooms.is_none() && f.band.1 >= 0.0;
        let shadows = sunlit && q.shadow && self.bench.shadow;
        let res = self.shadow_size.1;
        let cascades = if shadows && q.far_shadow { 2 } else { 1 };
        let sun_vp = self.cascade(f, 0.1, SHADOW_FAR, res);
        let sun_vp2 = if cascades == 2 { self.cascade(f, 0.1, SHADOW_FAR2, res) } else { sun_vp };
        // Småting kaster skygge bare nær. Med to kaskader tegnes det samme
        // utvalget i begge (det fjerne frustumet rommer det nære).
        let sun_cull = Cull { frustum: Frustum::from_view_proj(sun_vp2), eye: f.camera, fine_within: if reduced { 18.0 } else { 40.0 }, band: f.band };
        let cam_cull = Cull { frustum: Frustum::from_view_proj(view_proj), eye: f.camera, fine_within: if reduced { 35.0 } else { 70.0 }, band: f.band };
        let far = if q.far > 0.0 { q.far } else { f32::INFINITY };
        Plan { view_proj, sunlit, shadows, res, cascades, sun_vp, sun_vp2, sun_cull, cam_cull, reduced, far }
    }

    /// Modellinstansene: hvilke som er synlige i hver region, nær og fjern.
    /// De bevegelige lastes opp bak de faste.
    fn visible(&self, f: &Frame, p: &Plan) -> Vec<[ModelDraw; REGIONS as usize]> {
        let (reduced, far, shadows, cam_cull) = (p.reduced, p.far, p.shadows, p.cam_cull);
        // Kortere sikt: grovere detalj tidligere og småting bare nær.
        let model_near = if reduced { MODEL_NEAR * 0.5 } else { MODEL_NEAR };
        let culls = [p.sun_cull, p.cam_cull];
        let mut model_draws: Vec<[ModelDraw; REGIONS as usize]> = Vec::with_capacity(self.models.len());
        for m in &self.models {
            // De bevegelige instansene (figurene, fiendene, dyrene) legges
            // til de faste, innenfor plassen byen satte av.
            let moving: Vec<(InstanceData, Vec3, Vec3)> = f
                .instances
                .iter()
                .filter(|i| i.model == m.name)
                .take(m.reserve)
                .map(|i| {
                    let (lo, hi) = world_aabb(&i.transform, m.bounds.0, m.bounds.1);
                    // Blinket ligger over 1 i markeringen (extra.x).
                    let mut extra = m.extra;
                    extra[0] = if i.flash > 0.0 { 1.0 + i.flash } else { 0.0 };
                    if let Some(t) = i.tint {
                        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
                        extra[1] = (c(t.x) << 16 | c(t.y) << 8 | c(t.z)) as f32;
                    }
                    let inst = match i.skin.map(|(base, _)| base).filter(|base| m.skinned && (*base as usize) < SKIN_MAX) {
                        Some(base) => InstanceData::skinned(&i.transform, extra, base),
                        None => InstanceData::new(&i.transform, extra),
                    }
                    .with_look(i.look.as_ref());
                    (inst, lo, hi)
                })
                .collect();
            let mut draws: [ModelDraw; REGIONS as usize] = Default::default();
            let mut dynamic: Vec<InstanceData> = Vec::with_capacity(m.reserve * REGIONS as usize);
            for (r, cull) in culls.iter().enumerate() {
                if r == 0 && !shadows {
                    continue;
                }
                // Hvor langt unna noe i boksen er, og om det tegnes nær,
                // fjernt eller ikke.
                let lod = |lo: Vec3, hi: Vec3| -> Option<bool> {
                    let d = cull.eye.clamp(lo, hi).distance(cull.eye);
                    if m.interior && d > INTERIOR_FAR {
                        return None;
                    }
                    // Møblene kaster solskygge bare nær deg (i Smia, som
                    // er åpen mot torget, synes den).
                    if m.indoor && r == 0 && cam_cull.eye.clamp(lo, hi).distance(cam_cull.eye) > INDOOR_SHADOW {
                        return None;
                    }
                    if reduced && m.small && d > if r == 1 { 32.0 } else { 12.0 } {
                        return None;
                    }
                    // Kortere sikt: det som er langt unna tegnes ikke (tåka
                    // er tettere, så kanten glir bort).
                    if d > far {
                        return None;
                    }
                    Some(d < model_near)
                };
                let d = &mut draws[r];
                for c in &m.cells {
                    if !cull.sees(c.lo, c.hi) {
                        continue;
                    }
                    match lod(c.lo, c.hi) {
                        Some(true) => ModelDraw::push(&mut d.near, c.range.clone()),
                        Some(false) => ModelDraw::push(&mut d.far, c.range.clone()),
                        None => {}
                    }
                }
                // De bevegelige: hver region sin plass bak de faste.
                let base = m.statics + (r * m.reserve) as u32;
                let (mut near, mut far) = (vec![], vec![]);
                for (inst, lo, hi) in &moving {
                    if !cull.sees(*lo, *hi) {
                        continue;
                    }
                    match lod(*lo, *hi) {
                        Some(true) => near.push(*inst),
                        Some(false) => far.push(*inst),
                        None => {}
                    }
                }
                dynamic.resize(r * m.reserve, InstanceData::new(&Affine3A::IDENTITY, [0.0; 4]));
                ModelDraw::push(&mut d.near, base..base + near.len() as u32);
                ModelDraw::push(&mut d.far, base + near.len() as u32..base + (near.len() + far.len()) as u32);
                dynamic.extend(near);
                dynamic.extend(far);
            }
            if !dynamic.is_empty() {
                self.queue.write_buffer(&m.instances, m.statics as u64 * INSTANCE_STRIDE, bytemuck::cast_slice(&dynamic));
            }
            model_draws.push(draws);
        }
        model_draws
    }

    /// Verdiene alle shaderne leser: kameraet, sola, tåka, været, lysene,
    /// røyken, rommene og innstillingene.
    fn upload_globals(&self, f: &Frame, p: &Plan) {
        let t = &self.targets;
        let (w, h) = (t.width as f32, t.height as f32);
        let q = self.quality;
        let (far, res) = (p.far, p.res);
        let mut lights = [GpuLight::zeroed(); MAX_LIGHTS];
        for (i, l) in f.lights.iter().take(MAX_LIGHTS).enumerate() {
            lights[i] = GpuLight { pos_radius: [l.pos.x, l.pos.y, l.pos.z, l.radius], color_intensity: [l.color.x, l.color.y, l.color.z, l.intensity] };
        }
        let mut smoke = [[0.0f32; 4]; MAX_SMOKE];
        for (i, s) in f.smoke.iter().take(MAX_SMOKE).enumerate() {
            smoke[i] = s.to_array();
        }
        let v4 = |v: Vec3, w: f32| [v.x, v.y, v.z, w];
        let mut palette = [[0.0; 4]; PALETTE];
        for (i, c) in f.palette.iter().enumerate() {
            palette[i] = v4(*c, 1.0);
        }
        let mut tex_scale = [[1.0f32; 4]; 16];
        for (i, s) in self.materials.scales.iter().enumerate().take(LAYERS as usize) {
            tex_scale[i / 4][i % 4] = *s;
        }
        let wx = &f.weather;
        let mut room_info = [0.0f32; 4];
        let mut room_box = [[0.0f32; 4]; MAX_ROOMS];
        let mut room_y = [[0.0f32; 4]; MAX_ROOMS / 2];
        let mut light_rooms = [[u32::MAX; 4]; MAX_LIGHTS / 2];
        // Snittet (redigeringen): høyden i y og om det er på i z.
        if let Some(y) = f.clip {
            room_info[1] = y;
            room_info[2] = 1.0;
        }
        if let Some(r) = &f.rooms {
            let n = r.boxes.len().min(MAX_ROOMS);
            room_info[0] = n as f32;
            for (i, (lo, hi, y0, y1)) in r.boxes.iter().take(n).enumerate() {
                room_box[i] = [lo.x, lo.y, hi.x, hi.y];
                room_y[i / 2][(i % 2) * 2] = *y0;
                room_y[i / 2][(i % 2) * 2 + 1] = *y1;
            }
            for (i, m) in r.masks.iter().take(MAX_LIGHTS).enumerate() {
                light_rooms[i / 2][(i % 2) * 2] = *m as u32;
                light_rooms[i / 2][(i % 2) * 2 + 1] = (*m >> 32) as u32;
            }
        }
        let globals = Globals {
            view_proj: p.view_proj.to_cols_array_2d(),
            inv_view_proj: p.view_proj.inverse().to_cols_array_2d(),
            sun_vp: p.sun_vp.to_cols_array_2d(),
            camera: v4(f.camera, f.time),
            sun_dir: v4(f.sun_dir, if p.sunlit { f.sun_intensity } else { 0.0 }),
            sun_color: v4(f.sun_color, f.night),
            sky_top: v4(f.sky_top, 0.0),
            sky_horizon: v4(
                f.sky_horizon,
                if !q.fog {
                    0.0
                } else if far.is_finite() && f.fog_density > 0.0 {
                    (f.fog_density * 3.0).max(REDUCED_FOG / far)
                } else {
                    f.fog_density
                },
            ),
            ground_bounce: v4(f.ground_bounce, f.fog_falloff),
            fog_color: v4(f.fog_color, f.windows_lit),
            params: [f.exposure, f.lights.len().min(q.lights as usize).min(MAX_LIGHTS) as f32, 1.0 / res as f32, f.street_lights],
            weather: [wx.wet, wx.snow_cover, wx.rain, wx.snow],
            screen: [w, h, 1.0 / w, 1.0 / h],
            misc: [wx.wind, self.scene.samples as f32, SHADOW_FADE, SHADOW_FAR],
            sky: [wx.cloud, f.aurora, wx.lightning, f.debug as f32],
            tex_scale,
            palette,
            lights,
            smoke,
            room_info,
            room_box,
            room_y,
            light_rooms,
            quality: [
                if q.textures { 1.0 } else { 0.0 },
                if self.bench.flat { 1.0 } else { 0.0 },
                if !p.shadows {
                    0.0
                } else if q.soft_shadow {
                    2.0
                } else {
                    1.0
                },
                if q.ground { 1.0 } else { 0.0 },
            ],
            sun_vp2: p.sun_vp2.to_cols_array_2d(),
            shadow: [1.0 / self.shadow_size.0 as f32, 1.0 / self.shadow_size.1 as f32, p.cascades as f32, SHADOW_FAR2],
        };
        self.queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));
    }

    /// Verdiene per pass, instansen til den faste verdenen og leddpaletten.
    fn upload_frame(&self, f: &Frame) {
        let wx = &f.weather;
        // Per pass for partiklene: regn, snø og røyk.
        // Den fjerde er kaskade to i skyggepasset (x = 1).
        let mut pass_data = vec![0u8; PASS_STRIDE as usize * 4];
        for (i, v) in [[0.0, wx.rain, 0.0, 0.0], [1.0, wx.snow, 0.0, 0.0], [2.0, 1.0, 0.0, 0.0f32], [1.0, 0.0, 0.0, 0.0]].iter().enumerate() {
            let at = i * PASS_STRIDE as usize;
            pass_data[at..at + 16].copy_from_slice(bytemuck::cast_slice(v));
        }
        self.queue.write_buffer(&self.pass_buf, 0, &pass_data);

        // Den faste verdenen har én instans uten transform.
        let inst = [InstanceData::new(&Affine3A::IDENTITY, [0.0; 4])];
        self.queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&inst));

        // Leddpaletten: matrisene kolonne for kolonne, fire texels hver.
        if !f.skins.is_empty() {
            let n = f.skins.len().min(SKIN_MAX);
            if f.skins.len() > SKIN_MAX {
                log::warn!("For mange leddmatriser: {} av {}", f.skins.len(), SKIN_MAX);
            }
            let rows = (n as u32).div_ceil(SKIN_W / 4);
            let mut data = vec![0f32; (rows * SKIN_W * 4) as usize];
            for (i, m) in f.skins.iter().take(n).enumerate() {
                data[i * 16..i * 16 + 16].copy_from_slice(&m.to_cols_array());
            }
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &self.skins, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                bytemuck::cast_slice(&data),
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(SKIN_W * 16), rows_per_image: Some(rows) },
                wgpu::Extent3d { width: SKIN_W, height: rows, depth_or_array_layers: 1 },
            );
        }
    }

    pub fn render(&mut self, out: &wgpu::TextureView, f: &Frame) {
        if self.lost {
            return;
        }
        self.apply_hidden(f);
        let line_count = self.upload_lines(f);
        let plan = self.plan(f);
        let draws = self.visible(f, &plan);
        self.upload_globals(f, &plan);
        self.upload_frame(f);

        if let Some(tm) = &mut self.timer {
            let _ = self.device.poll(wgpu::PollType::Poll);
            tm.collect();
        }
        let mut enc = self.device.create_command_encoder(&Default::default());
        // 1. Skyggen, 2. verden, himmel og nedbør, 3. glød og tonemapping.
        let stats = [self.shadow_pass(&mut enc, &plan, &draws), self.main_pass(&mut enc, f, &plan, &draws, line_count)];
        self.post_passes(&mut enc, out, f, &plan);

        if let Some(tm) = &mut self.timer {
            tm.resolve(&mut enc);
        }
        self.pass_stats.set(stats);
        self.queue.submit([enc.finish()]);
        if let Some(tm) = &mut self.timer {
            tm.map();
        }
        // Lat lasting: teksturene med lavt tak oppgraderes etter bildet.
        self.upgrade_textures(f.camera, &f.instances);
    }
}
