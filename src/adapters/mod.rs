//! Adapterne: det som kobler applikasjonen til GPU, fysikkmotor,
//! filformater og verten (nettleser eller vindu).
//!
//! Drevne: `wgpu` (`ports::Renderer`), `rapier` (`ports::Physics`), `gltf`
//! (`ports::ModelSource`) og `layers` (teksturlagene til rendereren).
//! Drivende: `web` (nettleseren), `native` (eget vindu) og
//! `src/bin/snapshot.rs` (skjermbilder), som alle snakker med
//! `ports::driving::Play`.

pub mod data;
pub mod delivery;
pub mod gltf;
pub mod layers;
pub mod rapier;
pub mod wgpu;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod web;

/// Ber om GPU og enhet. WebGL2 får lavere grenser enn WebGPU og native.
pub async fn gpu(
    instance: &::wgpu::Instance,
    surface: Option<&::wgpu::Surface<'_>>,
) -> Result<(::wgpu::Adapter, ::wgpu::Device, ::wgpu::Queue), String> {
    let adapter = instance
        .request_adapter(&::wgpu::RequestAdapterOptions {
            power_preference: ::wgpu::PowerPreference::HighPerformance,
            compatible_surface: surface,
            force_fallback_adapter: false,
        })
        .await
        .map_err(|e| format!("Fant ingen GPU: {e}"))?;
    let info = adapter.get_info();
    log::info!("GPU: {} ({:?})", info.name, info.backend);
    let limits = if info.backend == ::wgpu::Backend::Gl {
        ::wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
    } else {
        ::wgpu::Limits::default().using_resolution(adapter.limits())
    };
    let (device, queue) = adapter
        .request_device(&::wgpu::DeviceDescriptor {
            label: Some("fristad"),
            // Tidsmåling per pass når kortet kan (ikke i WebGL2), og kortets
            // egne formategenskaper (MSAA 8× i avansert grafikk).
            required_features: adapter.features() & (::wgpu::Features::TIMESTAMP_QUERY | ::wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES | ::wgpu::Features::RG11B10UFLOAT_RENDERABLE),
            required_limits: limits,
            memory_hints: ::wgpu::MemoryHints::Performance,
            trace: ::wgpu::Trace::Off,
        })
        .await
        .map_err(|e| format!("Fikk ikke GPU-enhet: {e}"))?;
    // Nettleseren kan miste enheten (grafikkprosessen startes på nytt, eller
    // SwiftShader i Chromium uten skjerm); da tegnes ingenting mer, men
    // spillet skal gå videre uten å stoppe (`timer.rs`).
    device.set_device_lost_callback(|reason, message| {
        if reason != ::wgpu::DeviceLostReason::Destroyed {
            log::error!("Grafikkortet er borte ({reason:?}): {message}. Last siden på nytt.");
        }
    });
    Ok((adapter, device, queue))
}
