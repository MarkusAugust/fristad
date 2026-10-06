//! Det rendereren får per bilde: kamera, lys, vær, palett og hvor propsene
//! står. Rene tall, ingen GPU-typer.

use glam::{Affine3A, Mat4, Vec3};

pub const PALETTE: usize = 48;
pub const MAX_LIGHTS: usize = 48;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Weather {
    /// Hvor våte flatene er, 0..1.
    pub wet: f32,
    /// Snø på bakken og takene, 0..1.
    pub snow_cover: f32,
    /// Regn i lufta, 0..1.
    pub rain: f32,
    /// Snø i lufta, 0..1.
    pub snow: f32,
    /// Skydekke, 0..1.
    pub cloud: f32,
    pub wind: f32,
    pub lightning: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub pos: Vec3,
    pub radius: f32,
    pub color: Vec3,
    pub intensity: f32,
}

pub struct Frame {
    pub view: Mat4,
    pub proj: Mat4,
    pub camera: Vec3,
    pub time: f32,
    /// Retning mot sola eller månen.
    pub sun_dir: Vec3,
    pub sun_color: Vec3,
    pub sun_intensity: f32,
    /// 0 dag, 1 natt.
    pub night: f32,
    pub sky_top: Vec3,
    pub sky_horizon: Vec3,
    pub ground_bounce: Vec3,
    pub fog_color: Vec3,
    pub fog_density: f32,
    pub fog_falloff: f32,
    pub exposure: f32,
    pub bloom: f32,
    pub windows_lit: f32,
    /// Gatelysene, 0..1.
    pub street_lights: f32,
    pub aurora: f32,
    pub weather: Weather,
    pub palette: [Vec3; PALETTE],
    pub lights: Vec<Light>,
    /// Transform og ekstra (markering, animasjon) for hver prop, i samme
    /// rekkefølge som i byen.
    pub props: Vec<(Affine3A, [f32; 4])>,
    /// Bland med forrige bilde. Av første bilde og når kameraet hopper.
    pub taa: bool,
    /// Feilsøkingsvisning: 0 vanlig, 1 albedo, 2 normaler, 3 SSAO, 4 ruhet, 5 skygge.
    pub debug: u32,
}
