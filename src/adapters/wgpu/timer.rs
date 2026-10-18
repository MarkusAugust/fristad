//! Tiden på grafikkortet per del av bildet, når kortet kan måle det
//! (`TIMESTAMP_QUERY`: WebGPU i de fleste nettlesere, ikke WebGL2). Hver del
//! får et tidsstempel når det første passet begynner og når det siste
//! slutter; tallene leses tilbake noen bilder etter og glattes.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Delene av bildet, i den rekkefølgen de tegnes.
pub const SECTIONS: [&str; 3] = ["skygge", "verden", "etterbehandling"];
const QUERIES: u32 = SECTIONS.len() as u32 * 2;

/// Tilbakelesingen: ledig, venter på kartlegging, klar til å leses, eller
/// kartleggingen feilet (enheten er borte). wgpu regner bufferen som
/// kartlagt også når kartleggingen feiler, og en ny `map_async` på den
/// stopper motoren («Buffer is already mapped»); etter en feil måles det
/// ikke mer.
const IDLE: u8 = 0;
const PENDING: u8 = 1;
const READY: u8 = 2;
const FAILED: u8 = 3;

pub struct GpuTimer {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    state: Arc<AtomicU8>,
    /// Kopieres i dette bildet (og kartlegges etter innsendingen).
    copying: bool,
    /// Nanosekunder per tikk.
    period: f32,
    /// Millisekunder per del, glattet.
    pub ms: [f32; SECTIONS.len()],
}

impl GpuTimer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Self> {
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        let bytes = QUERIES as u64 * 8;
        Some(Self {
            set: device.create_query_set(&wgpu::QuerySetDescriptor { label: Some("tider"), ty: wgpu::QueryType::Timestamp, count: QUERIES }),
            resolve: device.create_buffer(&wgpu::BufferDescriptor { label: Some("tider"), size: bytes, usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false }),
            readback: device.create_buffer(&wgpu::BufferDescriptor { label: Some("tider lest"), size: bytes, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }),
            state: Arc::new(AtomicU8::new(IDLE)),
            copying: false,
            period: queue.get_timestamp_period(),
            ms: [0.0; SECTIONS.len()],
        })
    }

    /// Tidsstemplene til et pass: begynnelsen og/eller slutten av delen.
    pub fn writes(&self, section: usize, begin: bool, end: bool) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        if !begin && !end {
            return None;
        }
        Some(wgpu::RenderPassTimestampWrites {
            query_set: &self.set,
            beginning_of_pass_write_index: begin.then_some(section as u32 * 2),
            end_of_pass_write_index: end.then_some(section as u32 * 2 + 1),
        })
    }

    /// Før bildet: les tidene fra et tidligere bilde om de er klare.
    pub fn collect(&mut self) {
        match self.state.load(Ordering::Acquire) {
            READY => {}
            FAILED => {
                if self.ms != [0.0; SECTIONS.len()] {
                    log::warn!("Tidsmålingen på grafikkortet stoppet: tilbakelesingen feilet");
                }
                self.ms = [0.0; SECTIONS.len()];
                return;
            }
            _ => return,
        }
        {
            let data = self.readback.slice(..).get_mapped_range();
            let ticks: &[u64] = bytemuck::cast_slice(&data);
            for (k, ms) in self.ms.iter_mut().enumerate() {
                let (a, b) = (ticks[k * 2], ticks[k * 2 + 1]);
                if b > a {
                    let now = (b - a) as f32 * self.period / 1.0e6;
                    *ms = if *ms == 0.0 { now } else { *ms * 0.9 + now * 0.1 };
                }
            }
        }
        self.readback.unmap();
        self.state.store(IDLE, Ordering::Release);
    }

    /// Sist i bildet: kopier tidsstemplene ut, om tilbakelesingen er ledig.
    pub fn resolve(&mut self, enc: &mut wgpu::CommandEncoder) {
        self.copying = self.state.load(Ordering::Acquire) == IDLE;
        if self.copying {
            enc.resolve_query_set(&self.set, 0..QUERIES, &self.resolve, 0);
            enc.copy_buffer_to_buffer(&self.resolve, 0, &self.readback, 0, QUERIES as u64 * 8);
        }
    }

    /// Etter innsendingen: be om å få lese det som ble kopiert.
    pub fn map(&mut self) {
        if !self.copying {
            return;
        }
        self.copying = false;
        self.state.store(PENDING, Ordering::Release);
        let state = self.state.clone();
        self.readback.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            state.store(if r.is_ok() { READY } else { FAILED }, Ordering::Release);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ett bilde slik rendereren tegner det: les tidene, ett pass med
    /// tidsstempler, kopier dem ut, send inn og be om å få lese dem.
    fn frame(device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, tm: &mut GpuTimer) {
        let _ = device.poll(wgpu::PollType::Poll);
        tm.collect();
        let mut enc = device.create_command_encoder(&Default::default());
        drop(enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
            depth_stencil_attachment: None,
            timestamp_writes: tm.writes(0, true, true),
            occlusion_query_set: None,
        }));
        tm.resolve(&mut enc);
        queue.submit([enc.finish()]);
        tm.map();
    }

    /// Mister nettleseren grafikkortet (i Chromium med SwiftShader skjedde
    /// det i første bilde etter «Gå inn»), feiler kartleggingen av
    /// tilbakelesingen. wgpu regner da bufferen som kartlagt, og neste
    /// `map_async` på den stoppet hele motoren med «Buffer is already
    /// mapped»; alt etterpå feilet med «recursive use of an object».
    #[test]
    fn tidsmalingen_overlever_at_grafikkortet_forsvinner() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() });
        // Actions har ikke grafikkort (lokalt: lavapipe, `mesa-vulkan-drivers`).
        let Ok((_, device, queue)) = pollster::block_on(crate::adapters::gpu(&instance, None)) else {
            eprintln!("Ingen GPU; testen hoppes over");
            return;
        };
        let Some(mut tm) = GpuTimer::new(&device, &queue) else {
            eprintln!("Kortet kan ikke måle tid (TIMESTAMP_QUERY); testen hoppes over");
            return;
        };
        // Feilene etter at enheten er borte er ventet her.
        device.on_uncaptured_error(Box::new(|_| {}));
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: 8, height: 8, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        for _ in 0..4 {
            frame(&device, &queue, &view, &mut tm);
            let _ = device.poll(wgpu::PollType::Wait);
        }
        // Kartleggingen feiler mens den venter, som når nettleseren mister
        // enheten (der avvises `mapAsync`; her ødelegges bufferen).
        frame(&device, &queue, &view, &mut tm);
        assert_eq!(tm.state.load(Ordering::Acquire), PENDING);
        tm.readback.destroy();
        for _ in 0..4 {
            frame(&device, &queue, &view, &mut tm);
        }
        assert_eq!(tm.state.load(Ordering::Acquire), FAILED);
        assert_eq!(tm.ms, [0.0; SECTIONS.len()]);
    }
}
