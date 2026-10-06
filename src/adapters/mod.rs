//! Adapterne: det som kobler kjernen til GPU, fysikkmotor, filformater og
//! verten (nettleser eller vindu).

pub mod assets;
pub mod gltf;
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
            required_features: ::wgpu::Features::empty(),
            required_limits: limits,
            memory_hints: ::wgpu::MemoryHints::Performance,
            trace: ::wgpu::Trace::Off,
        })
        .await
        .map_err(|e| format!("Fikk ikke GPU-enhet: {e}"))?;
    Ok((adapter, device, queue))
}
