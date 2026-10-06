//! glTF-adapteren: modellene bygges inn i binæren (`assets/models/`) og
//! leses til `domain::model::Model` bak `ports::ModelSource`.

use crate::domain::geometry::{kind, Mat, Mesh};
use crate::domain::model::{Image, Model};
use crate::domain::ports::ModelSource;
use glam::{Affine3A, Mat4, Vec2, Vec3};

macro_rules! embed {
    ($dir:literal, [$($file:literal),*]) => {
        &[$(($file, include_bytes!(concat!("../../assets/models/", $dir, "/", $file)) as &[u8])),*]
    };
}

/// Modellene som er med, lest ved behov.
pub struct Embedded;

impl ModelSource for Embedded {
    fn load(&self, name: &str) -> Option<Model> {
        match name {
            "lantern" => Some(load(
                "lantern",
                embed!("lantern", [
                    "Lantern.gltf",
                    "Lantern.bin",
                    "Lantern_baseColor.jpg",
                    "Lantern_roughnessMetallic.jpg",
                    "Lantern_normal.jpg",
                    "Lantern_emissive.jpg"
                ]),
                // Lykta er 4,2 m høy.
                4.2,
            )),
            "fox" => Some(load("fox", embed!("fox", ["Fox.gltf", "Fox.bin", "Texture.png"]), 0.75)),
            _ => None,
        }
    }
}


fn decode(bytes: &[u8]) -> Option<Image> {
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    Some(Image { width: img.width(), height: img.height(), rgba: img.into_raw() })
}

/// Leser en glTF-fil med tilhørende filer fra minnet, flater ut nodene og
/// skalerer så modellen blir `height` meter høy med føttene i y = 0.
fn load(name: &'static str, files: &[(&str, &[u8])], height: f32) -> Model {
    let get = |n: &str| files.iter().find(|(f, _)| *f == n).map(|(_, b)| *b).unwrap_or_else(|| panic!("{name}: mangler {n}"));
    let doc = gltf::Gltf::from_slice(get(files[0].0)).expect("ugyldig glTF");
    let buffers: Vec<Vec<u8>> = doc
        .buffers()
        .map(|b| match b.source() {
            gltf::buffer::Source::Uri(u) => get(u).to_vec(),
            gltf::buffer::Source::Bin => doc.blob.clone().expect("glb-blob"),
        })
        .collect();
    let image = |t: Option<gltf::Texture>| -> Option<Image> {
        match t?.source().source() {
            gltf::image::Source::Uri { uri, .. } => decode(get(uri)),
            gltf::image::Source::View { view, .. } => {
                let b = &buffers[view.buffer().index()];
                decode(&b[view.offset()..view.offset() + view.length()])
            }
        }
    };

    let mut mesh = Mesh::default();
    let mut albedo = None;
    let mut normal = None;
    let mut metal_rough = None;
    let mut emissive = None;
    let mut roughness = 0.6;
    let mut metallic = 0.0;
    let scene = doc.default_scene().or_else(|| doc.scenes().next()).expect("ingen scene");
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    while let Some((node, parent)) = stack.pop() {
        let local = Mat4::from_cols_array_2d(&node.transform().matrix());
        let world = parent * local;
        for child in node.children() {
            stack.push((child, world));
        }
        let Some(m) = node.mesh() else { continue };
        for prim in m.primitives() {
            let mat = prim.material();
            let pbr = mat.pbr_metallic_roughness();
            roughness = pbr.roughness_factor();
            metallic = pbr.metallic_factor();
            if albedo.is_none() {
                albedo = image(pbr.base_color_texture().map(|t| t.texture()));
                metal_rough = image(pbr.metallic_roughness_texture().map(|t| t.texture()));
                normal = image(mat.normal_texture().map(|t| t.texture()));
                emissive = image(mat.emissive_texture().map(|t| t.texture()));
            }
            let reader = prim.reader(|b| Some(&buffers[b.index()]));
            let pos: Vec<[f32; 3]> = reader.read_positions().expect("posisjoner").collect();
            let uv: Vec<[f32; 2]> = reader.read_tex_coords(0).map(|t| t.into_f32().collect()).unwrap_or_default();
            let idx: Vec<u32> = reader.read_indices().map(|i| i.into_u32().collect()).unwrap_or_else(|| (0..pos.len() as u32).collect());
            let norms: Vec<[f32; 3]> = match reader.read_normals() {
                Some(n) => n.collect(),
                None => flat_normals(&pos, &idx),
            };
            let base = mesh.vertices.len() as u32;
            let tint = pbr.base_color_factor();
            let mmat = Mat::plain(
                ((tint[0] * 255.0) as u32) << 16 | ((tint[1] * 255.0) as u32) << 8 | (tint[2] * 255.0) as u32,
                roughness,
            );
            let nmat = Mat4::from_mat3(glam::Mat3::from_mat4(world).inverse().transpose());
            for (i, p) in pos.iter().enumerate() {
                let wp = world.transform_point3(Vec3::from(*p));
                let wn = nmat.transform_vector3(Vec3::from(norms[i])).normalize_or_zero();
                let t = uv.get(i).copied().unwrap_or([0.0, 0.0]);
                let v = Mat { kind: kind::MODEL, ..mmat };
                mesh.vert(wp, wn, Vec2::from(t), &v, 1.0);
            }
            mesh.indices.extend(idx.iter().map(|i| i + base));
        }
    }
    // Skaler og sett føttene på bakken.
    let (lo, hi) = mesh.bounds();
    let s = height / (hi.y - lo.y).max(1e-3);
    let center = Vec3::new((lo.x + hi.x) * 0.5, lo.y, (lo.z + hi.z) * 0.5);
    mesh.transform(&(Affine3A::from_scale(Vec3::splat(s)) * Affine3A::from_translation(-center)));
    log::info!("Modell {name}: {} trekanter", mesh.indices.len() / 3);
    Model { name: name.to_string(), mesh, albedo, normal, metal_rough, emissive, roughness, metallic }
}

fn flat_normals(pos: &[[f32; 3]], idx: &[u32]) -> Vec<[f32; 3]> {
    let mut n = vec![Vec3::ZERO; pos.len()];
    for t in idx.chunks(3) {
        if t.len() < 3 {
            break;
        }
        let (a, b, c) = (Vec3::from(pos[t[0] as usize]), Vec3::from(pos[t[1] as usize]), Vec3::from(pos[t[2] as usize]));
        let fnorm = (b - a).cross(c - a);
        for &i in t {
            n[i as usize] += fnorm;
        }
    }
    n.iter().map(|v| v.normalize_or_zero().to_array()).collect()
}
