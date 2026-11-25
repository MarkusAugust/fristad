//! Passene i et bilde: skyggen, verden med himmel og nedbør, og
//! etterbehandlingen (skygge i krokene, glød, tonemapping, SMAA).

use super::*;

impl Renderer {
    /// Tidsmålingen for del `k` (`timer::SECTIONS`), om kortet kan måle.
    fn stamps(&self, k: usize, begin: bool, end: bool) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        self.timer.as_ref().filter(|_| self.bench.timers).and_then(|tm| tm.writes(k, begin, end))
    }

    /// 1. Skyggen (kartet tømmes også nede, der den ikke tegnes).
    pub(super) fn shadow_pass(&self, enc: &mut wgpu::CommandEncoder, p: &Plan, draws: &[[ModelDraw; REGIONS as usize]]) -> PassStats {
        let mut stats = PassStats::default();
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("shadow"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.shadow_view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: self.stamps(0, true, true),
            occlusion_query_set: None,
        });
        if p.shadows {
            let res = p.res;
            pass.set_bind_group(0, &self.globals_bg, &[]);
            pass.set_bind_group(2, &self.materials.bind_group, &[]);
            pass.set_bind_group(3, &self.materials.dummy_model, &[]);
            for k in 0..p.cascades {
                // Hver kaskade i sin del av kartet; pass-verdien x sier
                // hvilken matrise vertexshaderen bruker.
                pass.set_viewport((k * res) as f32, 0.0, res as f32, res as f32, 0.0, 1.0);
                pass.set_bind_group(1, &self.shadow_frame_bg, &[if k == 0 { 0 } else { (3 * PASS_STRIDE) as u32 }]);
                let s = self.draw_all(&mut pass, &p.sun_cull, draws, 0, &self.shadow_pipe, &self.shadow_cutout, false);
                stats.add(&s);
            }
        }
        stats
    }

    /// 2. Verden, himmel og nedbør, med MSAA løst opp i `hdr`.
    pub(super) fn main_pass(&self, enc: &mut wgpu::CommandEncoder, f: &Frame, p: &Plan, draws: &[[ModelDraw; REGIONS as usize]], line_count: u32) -> PassStats {
        let t = &self.targets;
        let q = self.quality;
        let wx = &f.weather;
        let (view, resolve_target, store) = match &t.color {
            Some(msaa) => (msaa, Some(&t.hdr), wgpu::StoreOp::Discard),
            None => (&t.hdr, None, wgpu::StoreOp::Store),
        };
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("main"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view, resolve_target, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store } })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &t.depth,
                // Okklusjonen leser dybden etterpå.
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: if t.ao.is_some() { wgpu::StoreOp::Store } else { wgpu::StoreOp::Discard } }),
                stencil_ops: None,
            }),
            timestamp_writes: self.stamps(1, true, true),
            occlusion_query_set: None,
        });
        pass.set_bind_group(0, &self.globals_bg, &[]);
        pass.set_bind_group(1, &self.frame_bg, &[0]);
        pass.set_bind_group(2, &self.materials.bind_group, &[]);
        pass.set_bind_group(3, &self.materials.dummy_model, &[]);
        let (opaque, cards) = if f.clip.is_some() { (&self.scene.clip, &self.scene.clip) } else { (&self.scene.world, &self.scene.cutout) };
        // Vannet blandes over bunnen etter himmelen (i snittet i
        // redigeringen er det ugjennomsiktig som før).
        let water = f.clip.is_none();
        let mut stats = self.draw_all(&mut pass, &p.cam_cull, draws, 1, opaque, cards, water);
        if self.bench.sky {
            pass.set_pipeline(&self.scene.sky);
            pass.draw(0..3, 0..1);
        }
        if water {
            stats.add(&self.draw_models(&mut pass, draws, 1, |m| m.water, &self.scene.water));
        }
        self.triangles.set((stats.world, 0, stats.models));
        pass.set_pipeline(&self.scene.particles);
        let particles = q.particles && self.bench.particles;
        if !f.smoke.is_empty() && particles {
            pass.set_bind_group(1, &self.frame_bg, &[(2 * PASS_STRIDE) as u32]);
            pass.draw(0..4, 0..f.smoke.len() as u32 * SMOKE_PUFFS);
        }
        if wx.rain > 0.0 && particles {
            pass.set_bind_group(1, &self.frame_bg, &[0]);
            pass.draw(0..4, 0..RAIN_COUNT);
        }
        if wx.snow > 0.0 && particles {
            pass.set_bind_group(1, &self.frame_bg, &[PASS_STRIDE as u32]);
            pass.draw(0..4, 0..SNOW_COUNT);
        }
        if line_count > 0 {
            if let Some((b, _)) = self.line_buf.borrow().as_ref() {
                pass.set_bind_group(1, &self.frame_bg, &[0]);
                pass.set_pipeline(&self.scene.lines);
                pass.set_vertex_buffer(0, b.slice(..));
                pass.draw(0..line_count, 0..1);
            }
        }
        stats
    }

    /// 3. Glød og tonemapping til skjermen.
    pub(super) fn post_passes(&self, enc: &mut wgpu::CommandEncoder, out: &wgpu::TextureView, f: &Frame, p: &Plan) {
        let t = &self.targets;
        let q = self.quality;
        // Solstrålene: sola på skjermen, sterkest midt i bildet og borte
        // når den er bak deg, under horisonten eller det er natt.
        let sun_post = {
            let c = p.view_proj * (f.camera + f.sun_dir * 1000.0).extend(1.0);
            let mut out = [0.5, 0.5, 0.0, if q.grade { 1.0 } else { 0.0 }];
            if q.rays && p.sunlit && c.w > 0.0 && f.sun_dir.y > 0.0 {
                let ndc = c.truncate() / c.w;
                let off = ndc.x.abs().max(ndc.y.abs());
                let k = (1.0 - ((off - 0.9) / 0.6).clamp(0.0, 1.0)) * (f.sun_dir.y / 0.06).clamp(0.0, 1.0) * (1.0 - f.night).clamp(0.0, 1.0);
                out = [ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, k, out[3]];
            }
            out
        };
        let post = |buf: &wgpu::Buffer, params: [f32; 4], sw: u32, sh: u32| {
            let u = PostUniform { params, texel: [sw as f32, sh as f32, 1.0 / sw as f32, 1.0 / sh as f32], sun: sun_post, extra: [if f.gamma > 0.0 { f.gamma } else { 1.0 }, 0.0, 0.0, 0.0] };
            self.queue.write_buffer(buf, 0, bytemuck::bytes_of(&u));
        };
        let fullscreen = |enc: &mut wgpu::CommandEncoder, label: &str, target: &wgpu::TextureView, pipe: &wgpu::RenderPipeline, bg: &wgpu::BindGroup, load: wgpu::LoadOp<wgpu::Color>, timestamps: Option<wgpu::RenderPassTimestampWrites>| {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: target, resolve_target: None, ops: wgpu::Operations { load, store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None,
                timestamp_writes: timestamps,
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &self.globals_bg, &[]);
            pass.set_bind_group(1, bg, &[]);
            pass.draw(0..3, 0..1);
        };
        let clear = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        // Enkel grafikk: ingen glød (tonemappingen får styrke 0).
        let bloom = q.bloom;
        // Skygge i krokene før gløden (tidsmålingen for etterbehandlingen
        // begynner da her).
        let ao_ran = match (&t.ao, &self.ao) {
            (Some(at), Some(a)) => {
                a.run(enc, &self.globals_bg, at, self.stamps(2, true, false));
                true
            }
            _ => false,
        };
        for i in (0..BLOOM_LEVELS).filter(|_| bloom) {
            let (bg, sw, sh) = if i == 0 { (&t.down0_bg, t.width, t.height) } else { (&t.down_bg[i - 1], t.bloom[i - 1].1, t.bloom[i - 1].2) };
            post(&bg.1, [0.0, 0.0, 0.0, if i == 0 { 1.0 } else { 0.0 }], sw, sh);
            fullscreen(enc, "bloom down", &t.bloom[i].0, &self.down_pipe, &bg.0, clear, if i == 0 { self.stamps(2, !ao_ran, false) } else { None });
        }
        for (k, i) in (1..BLOOM_LEVELS).rev().enumerate().filter(|_| bloom) {
            post(&t.up_bg[k].1, [0.0; 4], t.bloom[i].1, t.bloom[i].2);
            fullscreen(enc, "bloom up", &t.bloom[i - 1].0, &self.up_pipe, &t.up_bg[k].0, wgpu::LoadOp::Load, None);
        }
        match &t.smaa {
            // Med SMAA skriver tonemappingen gamma-kodet til mellombildet,
            // og kantutjevningen skriver til skjermen.
            Some(st) => {
                post(&t.tonemap_bg.1, [f.exposure, if bloom { f.bloom } else { 0.0 }, 1.0, f.hurt], t.width, t.height);
                fullscreen(enc, "tonemap", &st.ldr, &self.tonemap_ldr, &t.tonemap_bg.0, clear, self.stamps(2, !bloom && !ao_ran, false));
                self.smaa.run(enc, st, out, self.stamps(2, false, true));
            }
            None => {
                post(&t.tonemap_bg.1, [f.exposure, if bloom { f.bloom } else { 0.0 }, if self.encode_srgb { 1.0 } else { 0.0 }, f.hurt], t.width, t.height);
                fullscreen(enc, "tonemap", out, &self.tonemap_pipe, &t.tonemap_bg.0, clear, self.stamps(2, !bloom && !ao_ran, true));
            }
        }
    }

    /// Tegn verden og modellene. Verden tegnes bit for bit, bare
    /// de bitene som er innenfor synsfeltet, og småtingene bare nær
    /// kameraet. Naboer som begge skal tegnes slås sammen til ett kall.
    /// Returnerer hva som ble tegnet.
    /// Tegner bymeshen og modellene: det ugjennomsiktige med `opaque` først,
    /// så alfakortene med `cutout` (også bymeshen, om den har alfakort).
    /// Med `skip_water` venter vannflatene til sitt eget pass (`water`).
    #[allow(clippy::too_many_arguments)]
    fn draw_all(&self, pass: &mut wgpu::RenderPass, cull: &Cull, model_draws: &[[ModelDraw; REGIONS as usize]], region: usize, opaque: &wgpu::RenderPipeline, cutout: &wgpu::RenderPipeline, skip_water: bool) -> PassStats {
        pass.set_pipeline(if self.world_cutout { cutout } else { opaque });
        let mut calls = 0u32;
        pass.set_vertex_buffer(1, self.instances.slice(0..INSTANCE_STRIDE));
        self.world.bind(pass);
        let mut indices = 0u32;
        let mut run: Option<std::ops::Range<u32>> = None;
        let mut flush = |run: &mut Option<std::ops::Range<u32>>, pass: &mut wgpu::RenderPass| {
            if let Some(r) = run.take() {
                if !r.is_empty() {
                    indices += r.end - r.start;
                    calls += 1;
                    pass.draw_indexed(r, 0, 0..1);
                }
            }
        };
        for c in &self.chunks {
            if !cull.sees(c.min, c.max) {
                flush(&mut run, pass);
                continue;
            }
            let fine = cull.fine_within > 0.0 && c.distance(cull.eye) < cull.fine_within;
            let range = if fine { c.coarse.start..c.fine.end } else { c.coarse.clone() };
            match &mut run {
                Some(r) if r.end == range.start => r.end = range.end,
                _ => {
                    flush(&mut run, pass);
                    run = Some(range);
                }
            }
        }
        flush(&mut run, pass);
        let world = indices / 3;
        // Først de ugjennomsiktige, så alfakortene med sin pipeline.
        let mut s = self.draw_models(pass, model_draws, region, |m| !m.cutout && !(skip_water && m.water), opaque);
        s.add(&self.draw_models(pass, model_draws, region, |m| m.cutout && !(skip_water && m.water), cutout));
        PassStats { world, models: s.models, calls: s.calls + calls, instances: s.instances }
    }

    /// Tegner modellene `pick` velger med `pipe` (satt først når noe skal
    /// tegnes, så et pass uten slike modeller ikke bytter pipeline).
    fn draw_models(&self, pass: &mut wgpu::RenderPass, model_draws: &[[ModelDraw; REGIONS as usize]], region: usize, pick: impl Fn(&GpuModel) -> bool, pipe: &wgpu::RenderPipeline) -> PassStats {
        let mut models = 0;
        let mut calls = 0u32;
        let mut instances = 0u32;
        let mut switched = false;
        for (m, draws) in self.models.iter().zip(model_draws).filter(|(m, _)| pick(m)) {
            let d = &draws[region];
            // Målingen kan ta modellene ut av hovedpasset.
            if (d.near.is_empty() && d.far.is_empty()) || (region == 1 && !self.bench.models) {
                continue;
            }
            if !switched {
                pass.set_pipeline(pipe);
                switched = true;
            }
            pass.set_bind_group(3, &m.textures, &[]);
            pass.set_vertex_buffer(1, m.instances.slice(..));
            m.mesh.bind(pass);
            for r in &d.near {
                pass.draw_indexed(m.near.clone(), 0, r.clone());
                models += (m.near.end - m.near.start) / 3 * r.len() as u32;
                calls += 1;
                instances += r.len() as u32;
            }
            for r in &d.far {
                pass.draw_indexed(m.far.clone(), 0, r.clone());
                models += (m.far.end - m.far.start) / 3 * r.len() as u32;
                calls += 1;
                instances += r.len() as u32;
            }
        }
        PassStats { world: 0, models, calls, instances }
    }
}
