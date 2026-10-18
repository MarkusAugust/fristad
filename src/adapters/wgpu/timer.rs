//! Tiden på grafikkortet per del av bildet, når kortet kan måle det
//! (`TIMESTAMP_QUERY`: WebGPU i de fleste nettlesere, ikke WebGL2). Hver del
//! får et tidsstempel når det første passet begynner og når det siste
//! slutter; tallene leses tilbake noen bilder etter og glattes.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Delene av bildet, i den rekkefølgen de tegnes.
pub const SECTIONS: [&str; 3] = ["skygge", "verden", "etterbehandling"];
const QUERIES: u32 = SECTIONS.len() as u32 * 2;

/// Tilbakelesingen: ledig, venter på kartlegging, klar til å leses.
const IDLE: u8 = 0;
const PENDING: u8 = 1;
const READY: u8 = 2;

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
        if self.state.load(Ordering::Acquire) != READY {
            return;
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
            state.store(if r.is_ok() { READY } else { IDLE }, Ordering::Release);
        });
    }
}
