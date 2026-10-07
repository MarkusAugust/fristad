//! glTF-adapteren: leser glTF til `domain::model::Model`. `Embedded` er
//! modellene som er bygget inn i binæren (lykta, reven og bilene fra
//! `tools/models/cars.py`); eksterne modeller kommer gjennom
//! `adapters::assets`.

use crate::domain::anim::{Channel, Clip, Joint, Keys, Rig};
use crate::domain::geometry::{flag, kind, Mat, Mesh};
use crate::domain::model::{Image, Model};
use crate::domain::ports::ModelSource;
use glam::{Affine3A, Mat4, Quat, Vec2, Vec3};

macro_rules! embed {
    ($dir:literal, [$($file:literal),*]) => {
        &[$(($file, include_bytes!(concat!("../../assets/models/", $dir, "/", $file)) as &[u8])),*]
    };
}

/// Modellene som er med, lest ved behov.
pub struct Embedded;

impl ModelSource for Embedded {
    fn names(&self) -> Vec<String> {
        vec!["lantern".into(), "fox".into(), "car_estate".into(), "car_hatch".into(), "car_van".into(), "lamp".into(), "bench".into(), "bicycle".into(), "bin".into(), "cafe_table".into(), "cafe_chair".into(), "parasol".into(), "market_stall".into(), "mailbox".into(), "window_22".into(), "window_23".into(), "door_lamp".into(), "flower_box".into(), "sheep_body".into(), "sheep_leg".into(), "cow_body".into(), "cow_leg_front".into(), "cow_leg_hind".into(), "gorvek_table".into(), "gorvek_bench".into(), "gorvek_throne".into(), "gorvek_bed".into(), "gorvek_bearskin".into(), "gorvek_shield".into(), "gorvek_axe".into(), "gorvek_sword".into(), "gorvek_chest".into(), "gorvek_keg".into(), "gorvek_cauldron".into(), "gorvek_torch".into(), "gorvek_antlers".into(), "gorvek_stool".into(), "gorvek_skull".into(), "oak1_trunk".into(), "oak1_leaves".into(), "oak2_trunk".into(), "oak2_leaves".into(), "birch1_trunk".into(), "birch1_leaves".into(), "spruce1_trunk".into(), "spruce1_needles".into(), "spruce2_trunk".into(), "spruce2_needles".into(), "bush1_trunk".into(), "bush1_leaves".into(), "bush2_trunk".into(), "bush2_leaves".into(), "site_container".into(), "site_excavator".into(), "site_mixer".into(), "site_pallet".into(), "site_fence".into(), "site_wheelbarrow".into(), "site_toilet".into(), "site_crane".into(), "mutant_button".into(), "token".into(), "key".into(), "arms".into()]
    }

    fn load(&self, name: &str) -> Option<Model> {
        let r = match name {
            "lantern" => load(
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
                0.0,
            ),
            "fox" => load("fox", embed!("fox", ["Fox.gltf", "Fox.bin", "Texture.png"]), 0.75, 0.0),
            // Bilene er modellert i meter med fronten mot +X.
            // Med bakt tekstur (tools/models/bake_cars.py): AO, skitt og
            // ruhet/metall i ett atlas per bil.
            "car_estate" => load("car_estate", embed!("cars", ["estate_tex.glb"]), 0.0, 0.0),
            "car_hatch" => load("car_hatch", embed!("cars", ["hatch_tex.glb"]), 0.0, 0.0),
            "car_van" => load("car_van", embed!("cars", ["van_tex.glb"]), 0.0, 0.0),
            // Gatemøblene fra tools/models/street.py, i meter.
            "lamp" => load("lamp", embed!("street", ["lamp.glb"]), 0.0, 0.0),
            "bench" => load("bench", embed!("street", ["bench.glb"]), 0.0, 0.0),
            "bicycle" => load("bicycle", embed!("street", ["bicycle.glb"]), 0.0, 0.0),
            "bin" => load("bin", embed!("street", ["bin.glb"]), 0.0, 0.0),
            // Torget fra tools/models/plaza.py.
            "cafe_table" => load("cafe_table", embed!("plaza", ["cafe_table.glb"]), 0.0, 0.0),
            "cafe_chair" => load("cafe_chair", embed!("plaza", ["cafe_chair.glb"]), 0.0, 0.0),
            "parasol" => load("parasol", embed!("plaza", ["parasol.glb"]), 0.0, 0.0),
            "market_stall" => load("market_stall", embed!("plaza", ["market_stall.glb"]), 0.0, 0.0),
            "mailbox" => load("mailbox", embed!("plaza", ["mailbox.glb"]), 0.0, 0.0),
            // Husdetaljene fra tools/models/house.py.
            "window_22" => load("window_22", embed!("house", ["window_22.glb"]), -1.0, 0.0),
            "window_23" => load("window_23", embed!("house", ["window_23.glb"]), -1.0, 0.0),
            "door_lamp" => load("door_lamp", embed!("house", ["door_lamp.glb"]), -1.0, 0.0),
            "flower_box" => load("flower_box", embed!("house", ["flower_box.glb"]), -1.0, 0.0),
            // Dyrene fra tools/models/animals.py: kroppen har origo i bakken
            // under midten, beina i hofteleddet (domain/herd.rs poserer dem).
            "sheep_body" => load("sheep_body", embed!("animals", ["sheep_body.glb"]), -1.0, 0.0),
            "sheep_leg" => load("sheep_leg", embed!("animals", ["sheep_leg.glb"]), -1.0, 0.0),
            "cow_body" => load("cow_body", embed!("animals", ["cow_body.glb"]), -1.0, 0.0),
            "cow_leg_front" => load("cow_leg_front", embed!("animals", ["cow_leg_front.glb"]), -1.0, 0.0),
            "cow_leg_hind" => load("cow_leg_hind", embed!("animals", ["cow_leg_hind.glb"]), -1.0, 0.0),
            // Innboet i Gorveks hus fra tools/models/gorvek.py, origo beholdt.
            "gorvek_table" => load("gorvek_table", embed!("gorvek", ["table.glb"]), -1.0, 0.0),
            "gorvek_bench" => load("gorvek_bench", embed!("gorvek", ["bench_long.glb"]), -1.0, 0.0),
            "gorvek_throne" => load("gorvek_throne", embed!("gorvek", ["throne.glb"]), -1.0, 0.0),
            "gorvek_bed" => load("gorvek_bed", embed!("gorvek", ["bed.glb"]), -1.0, 0.0),
            "gorvek_bearskin" => load("gorvek_bearskin", embed!("gorvek", ["bearskin.glb"]), -1.0, 0.0),
            "gorvek_shield" => load("gorvek_shield", embed!("gorvek", ["shield.glb"]), -1.0, 0.0),
            "gorvek_axe" => load("gorvek_axe", embed!("gorvek", ["axe.glb"]), -1.0, 0.0),
            "gorvek_sword" => load("gorvek_sword", embed!("gorvek", ["sword.glb"]), -1.0, 0.0),
            "gorvek_chest" => load("gorvek_chest", embed!("gorvek", ["chest.glb"]), -1.0, 0.0),
            "gorvek_keg" => load("gorvek_keg", embed!("gorvek", ["keg.glb"]), -1.0, 0.0),
            "gorvek_cauldron" => load("gorvek_cauldron", embed!("gorvek", ["cauldron.glb"]), -1.0, 0.0),
            "gorvek_torch" => load("gorvek_torch", embed!("gorvek", ["torch.glb"]), -1.0, 0.0),
            "gorvek_antlers" => load("gorvek_antlers", embed!("gorvek", ["antlers.glb"]), -1.0, 0.0),
            "gorvek_stool" => load("gorvek_stool", embed!("gorvek", ["stool.glb"]), -1.0, 0.0),
            "gorvek_skull" => load("gorvek_skull", embed!("gorvek", ["skull.glb"]), -1.0, 0.0),
            // Trærne og buskene fra tools/models/trees.py: stammen med bakt
            // bark, og bladverket som kort (to modeller med samme origo).
            "oak1_trunk" => load("oak1_trunk", embed!("trees", ["oak1_trunk.glb"]), -1.0, 0.0),
            "oak1_leaves" => load("oak1_leaves", embed!("trees", ["oak1_leaves.glb"]), -1.0, 0.0),
            "oak2_trunk" => load("oak2_trunk", embed!("trees", ["oak2_trunk.glb"]), -1.0, 0.0),
            "oak2_leaves" => load("oak2_leaves", embed!("trees", ["oak2_leaves.glb"]), -1.0, 0.0),
            "birch1_trunk" => load("birch1_trunk", embed!("trees", ["birch1_trunk.glb"]), -1.0, 0.0),
            "birch1_leaves" => load("birch1_leaves", embed!("trees", ["birch1_leaves.glb"]), -1.0, 0.0),
            "spruce1_trunk" => load("spruce1_trunk", embed!("trees", ["spruce1_trunk.glb"]), -1.0, 0.0),
            "spruce1_needles" => load("spruce1_needles", embed!("trees", ["spruce1_needles.glb"]), -1.0, 0.0),
            "spruce2_trunk" => load("spruce2_trunk", embed!("trees", ["spruce2_trunk.glb"]), -1.0, 0.0),
            "spruce2_needles" => load("spruce2_needles", embed!("trees", ["spruce2_needles.glb"]), -1.0, 0.0),
            "bush1_trunk" => load("bush1_trunk", embed!("trees", ["bush1_trunk.glb"]), -1.0, 0.0),
            "bush1_leaves" => load("bush1_leaves", embed!("trees", ["bush1_leaves.glb"]), -1.0, 0.0),
            "bush2_trunk" => load("bush2_trunk", embed!("trees", ["bush2_trunk.glb"]), -1.0, 0.0),
            "bush2_leaves" => load("bush2_leaves", embed!("trees", ["bush2_leaves.glb"]), -1.0, 0.0),
            // Byggeplassen fra tools/models/site.py, med bakte teksturer.
            "site_container" => load("site_container", embed!("site", ["container.glb"]), -1.0, 0.0),
            "site_excavator" => load("site_excavator", embed!("site", ["excavator.glb"]), -1.0, 0.0),
            "site_mixer" => load("site_mixer", embed!("site", ["mixer.glb"]), -1.0, 0.0),
            "site_pallet" => load("site_pallet", embed!("site", ["pallet.glb"]), -1.0, 0.0),
            "site_fence" => load("site_fence", embed!("site", ["fence.glb"]), -1.0, 0.0),
            "site_wheelbarrow" => load("site_wheelbarrow", embed!("site", ["wheelbarrow.glb"]), -1.0, 0.0),
            "site_toilet" => load("site_toilet", embed!("site", ["toilet.glb"]), -1.0, 0.0),
            "site_crane" => load("site_crane", embed!("site", ["crane.glb"]), -1.0, 0.0),
            // Fiendene og lootet i kjelleren fra tools/models/mobs.py.
            "mutant_button" => load("mutant_button", embed!("mobs", ["mutant_button.glb"]), -1.0, 0.0),
            "token" => load("token", embed!("mobs", ["token.glb"]), -1.0, 0.0),
            "key" => load("key", embed!("mobs", ["key.glb"]), -1.0, 0.0),
            // Spillerens armer med sverd og skjold fra tools/models/arms.py,
            // i kamerarommet (origo i øyet, −Z fram).
            "arms" => load("arms", embed!("arms", ["arms.glb"]), -1.0, 0.0),
            _ => return None,
        };
        r.map_err(|e| log::warn!("Modell {name}: {e}")).ok()
    }
}


fn decode(bytes: &[u8]) -> Option<Image> {
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    Some(Image { width: img.width(), height: img.height(), rgba: img.into_raw() })
}

/// Leser en glTF-fil med tilhørende filer fra minnet (første fil er
/// .gltf-fila, stiene er som i fila), flater ut nodene, dreier `yaw`
/// grader om Y og setter føttene i y = 0. `height` > 0 skalerer modellen
/// til den høyden; 0 beholder modellens egne mål (meter).
pub fn load(name: &str, files: &[(&str, &[u8])], height: f32, yaw: f32) -> Result<Model, String> {
    let get = |n: &str| -> Result<&[u8], String> {
        // Stier i glTF kan være URL-kodet.
        let plain = n.replace("%20", " ");
        files.iter().find(|(f, _)| *f == n || *f == plain || f.rsplit('/').next() == Some(n)).map(|(_, b)| *b).ok_or_else(|| format!("mangler {n}"))
    };
    let doc = gltf::Gltf::from_slice(get(files[0].0)?).map_err(|e| format!("ugyldig glTF: {e}"))?;
    let mut buffers: Vec<Vec<u8>> = vec![];
    for b in doc.buffers() {
        buffers.push(match b.source() {
            gltf::buffer::Source::Uri(u) => get(u)?.to_vec(),
            gltf::buffer::Source::Bin => doc.blob.clone().ok_or("glb uten blob")?,
        });
    }
    let image = |t: Option<gltf::Texture>| -> Option<Image> {
        match t?.source().source() {
            gltf::image::Source::Uri { uri, .. } => decode(get(uri).ok()?),
            gltf::image::Source::View { view, .. } => {
                let b = &buffers[view.buffer().index()];
                decode(&b[view.offset()..view.offset() + view.length()])
            }
        }
    };

    // Skjelettet: det første skinnet, med leddene i glTF-rekkefølge og
    // foreldrene funnet i nodetreet. Noder mellom rota og første ledd
    // (armaturobjektet) foldes inn i rotleddets lokale transform.
    let node_count = doc.nodes().count();
    let mut node_parent: Vec<Option<usize>> = vec![None; node_count];
    for n in doc.nodes() {
        for c in n.children() {
            node_parent[c.index()] = Some(n.index());
        }
    }
    let node_local = |i: usize| doc.nodes().nth(i).map(|n| Mat4::from_cols_array_2d(&n.transform().matrix())).unwrap_or(Mat4::IDENTITY);
    let mut rig: Option<Rig> = None;
    if let Some(skin) = doc.skins().next() {
        let joint_nodes: Vec<gltf::Node> = skin.joints().collect();
        let mut joint_of: std::collections::HashMap<usize, usize> = Default::default();
        for (k, j) in joint_nodes.iter().enumerate() {
            joint_of.insert(j.index(), k);
        }
        let reader = skin.reader(|b| Some(&buffers[b.index()]));
        let ibms: Vec<Mat4> = reader.read_inverse_bind_matrices().map(|it| it.map(|m| Mat4::from_cols_array_2d(&m)).collect()).unwrap_or_default();
        let mut joints = vec![];
        for (k, j) in joint_nodes.iter().enumerate() {
            let mut local = Mat4::from_cols_array_2d(&j.transform().matrix());
            let mut p = node_parent[j.index()];
            let mut parent = None;
            while let Some(pi) = p {
                if let Some(&pj) = joint_of.get(&pi) {
                    parent = Some(pj);
                    break;
                }
                local = node_local(pi) * local;
                p = node_parent[pi];
            }
            if parent.is_some_and(|pj| pj >= k) {
                return Err(format!("leddet {k} ({}) kommer før forelderen sin", j.name().unwrap_or("")));
            }
            let (s, r, t) = local.to_scale_rotation_translation();
            joints.push(Joint { name: j.name().unwrap_or("").to_string(), parent, translation: t, rotation: r, scale: s, inverse_bind: ibms.get(k).copied().unwrap_or(Mat4::IDENTITY) });
        }
        // Klippene: én kanal per ledd og egenskap. Kubiske splines har
        // tre verdier per nøkkel (tangent inn, verdi, tangent ut).
        fn middle<T: Clone>(v: Vec<T>, cubic: bool) -> Vec<T> {
            if cubic {
                v.into_iter().skip(1).step_by(3).collect()
            } else {
                v
            }
        }
        let mut clips = vec![];
        for a in doc.animations() {
            let mut channels = vec![];
            let mut duration: f32 = 0.0;
            for ch in a.channels() {
                let Some(&joint) = joint_of.get(&ch.target().node().index()) else { continue };
                let r = ch.reader(|b| Some(&buffers[b.index()]));
                let times: Vec<f32> = r.read_inputs().map(|i| i.collect()).unwrap_or_default();
                let Some(outputs) = r.read_outputs() else { continue };
                let cubic = ch.sampler().interpolation() == gltf::animation::Interpolation::CubicSpline;
                let keys = match outputs {
                    gltf::animation::util::ReadOutputs::Translations(v) => Keys::Translation(middle(v.map(Vec3::from).collect(), cubic)),
                    gltf::animation::util::ReadOutputs::Rotations(v) => Keys::Rotation(middle(v.into_f32().map(|q| Quat::from_array(q).normalize()).collect(), cubic)),
                    gltf::animation::util::ReadOutputs::Scales(v) => Keys::Scale(middle(v.map(Vec3::from).collect(), cubic)),
                    gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => continue,
                };
                let n = match &keys {
                    Keys::Translation(v) | Keys::Scale(v) => v.len(),
                    Keys::Rotation(v) => v.len(),
                };
                if n != times.len() || n == 0 {
                    continue;
                }
                duration = duration.max(times.last().copied().unwrap_or(0.0));
                channels.push(Channel { joint, times, keys, step: ch.sampler().interpolation() == gltf::animation::Interpolation::Step });
            }
            clips.push(Clip { name: a.name().unwrap_or("").to_string(), duration, channels });
        }
        log::info!("Modell {name}: {} ledd, klipp: {}", joints.len(), clips.iter().map(|c| format!("{} ({:.2} s)", c.name, c.duration)).collect::<Vec<_>>().join(", "));
        rig = Some(Rig { joints, clips, root: Mat4::IDENTITY });
    }

    let mut mesh = Mesh::default();
    let mut albedo = None;
    let mut normal = None;
    let mut metal_rough = None;
    let mut emissive = None;
    let mut roughness = 0.6;
    let mut metallic = 0.0;
    let scene = doc.default_scene().or_else(|| doc.scenes().next()).ok_or("ingen scene")?;
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
            let Some(positions) = reader.read_positions() else { continue };
            let pos: Vec<[f32; 3]> = positions.collect();
            // Skinnede hjørner ligger i skjelettets rom: nodens transform
            // gjelder ikke, leddmatrisene gjør alt.
            let joints: Vec<[u16; 4]> = reader.read_joints(0).map(|j| j.into_u16().collect()).unwrap_or_default();
            let weights: Vec<[f32; 4]> = reader.read_weights(0).map(|w| w.into_f32().collect()).unwrap_or_default();
            let skinned = rig.is_some() && joints.len() == pos.len() && weights.len() == pos.len();
            let world = if skinned { Mat4::IDENTITY } else { world };
            let uv: Vec<[f32; 2]> = reader.read_tex_coords(0).map(|t| t.into_f32().collect()).unwrap_or_default();
            let idx: Vec<u32> = reader.read_indices().map(|i| i.into_u32().collect()).unwrap_or_else(|| (0..pos.len() as u32).collect());
            let norms: Vec<[f32; 3]> = match reader.read_normals() {
                Some(n) => n.collect(),
                None => flat_normals(&pos, &idx),
            };
            let base = mesh.vertices.len() as u32;
            let tint = pbr.base_color_factor();
            // Fargefaktoren er lineær; hjørnefargen er sRGB.
            let mut mmat = Mat::plain(0xffffff, roughness).tint(Vec3::new(tint[0], tint[1], tint[2]));
            // «paint» får fargen fra instansen; metall merkes med flagg.
            // Blender legger på .001 når navnet finnes fra før, så prefikset holder.
            let named = |n: &str| mat.name().map(|m| m.starts_with(n)).unwrap_or(false);
            if named("paint") {
                mmat = mmat.tint(Vec3::ONE).flags(flag::PAINT);
            }
            // «glow» lyser om natta, som lampene ellers i byen; «glass»
            // speiler som vinduene.
            if named("glow") {
                mmat.kind = kind::LAMP;
            }
            if named("glass") {
                mmat.kind = kind::GLASS;
            }
            // «fire» brenner hele døgnet (fakler, ildsteder).
            if named("fire") {
                mmat = mmat.flags(flag::GLOW_ALWAYS);
            }
            // Bladkort og nålekvister: alfatestes og svaier i vinden.
            if named("leaf") || named("needle") {
                mmat = mmat.flags(flag::ALPHA | flag::WIND | flag::NO_WEATHER);
            }
            if metallic >= 0.5 {
                mmat = mmat.flags(flag::METAL);
            }
            let nmat = Mat4::from_mat3(glam::Mat3::from_mat4(world).inverse().transpose());
            // Blader og kvister på kort: flagget får rendereren til å alfateste.
            let flags = mmat.flags
                | match mat.alpha_mode() {
                    gltf::material::AlphaMode::Opaque => 0,
                    _ => flag::ALPHA,
                };
            for (i, p) in pos.iter().enumerate() {
                let wp = world.transform_point3(Vec3::from(*p));
                let wn = nmat.transform_vector3(Vec3::from(norms[i])).normalize_or_zero();
                let t = uv.get(i).copied().unwrap_or([0.0, 0.0]);
                let v = Mat { kind: if mmat.kind == kind::PLAIN { kind::MODEL } else { mmat.kind }, flags, ..mmat };
                let vi = mesh.vert(wp, wn, Vec2::from(t), &v, 1.0);
                if skinned {
                    let (j, w) = skin_weights(joints[i], weights[i]);
                    let vert = &mut mesh.vertices[vi as usize];
                    vert.joints = j;
                    vert.weights = w;
                }
            }
            mesh.indices.extend(idx.iter().map(|i| i + base));
        }
    }
    if mesh.indices.is_empty() {
        return Err("ingen geometri".into());
    }
    // Drei, skaler og sett føttene på bakken. `height` < 0 beholder
    // modellens eget origo (husdetaljene som står i et hull i veggen).
    if yaw != 0.0 {
        mesh.transform(&Affine3A::from_rotation_y(yaw.to_radians()));
    }
    let (lo, hi) = mesh.bounds();
    let s = if height > 0.0 { height / (hi.y - lo.y).max(1e-3) } else { 1.0 };
    let center = if height < 0.0 { Vec3::ZERO } else { Vec3::new((lo.x + hi.x) * 0.5, lo.y, (lo.z + hi.z) * 0.5) };
    mesh.transform(&(Affine3A::from_scale(Vec3::splat(s)) * Affine3A::from_translation(-center)));
    if let Some(r) = &mut rig {
        r.root = Mat4::from(Affine3A::from_scale(Vec3::splat(s)) * Affine3A::from_translation(-center)) * Mat4::from_rotation_y(yaw.to_radians());
    }
    let (lo, hi) = mesh.bounds();
    log::info!("Modell {name}: {} trekanter, {:.2} × {:.2} × {:.2} m", mesh.indices.len() / 3, hi.x - lo.x, hi.y - lo.y, hi.z - lo.z);
    // Bladverk som kort tåler ikke å slås sammen på avstand: det grove
    // detaljnivået er hvert tredje kort, forstørret.
    let far_mesh = (name.ends_with("_leaves") || name.ends_with("_needles")).then(|| mesh.thinned_cards(3, 1.7));
    let far_cell = if far_mesh.is_some() {
        0.0005
    } else if name.ends_with("_trunk") {
        // Stammene: grovt på avstand, kvistene forsvinner i bladverket.
        0.6
    } else {
        0.0
    };
    Ok(Model { name: name.to_string(), mesh, albedo, normal, metal_rough, emissive, roughness, metallic, far_cell, far_mesh, rig })
}

/// Fire ledd og vekter per hjørne, kvantisert til byte med summen 255.
/// Hjørner uten vekter henger på det første leddet.
fn skin_weights(j: [u16; 4], w: [f32; 4]) -> ([u8; 4], [u8; 4]) {
    let sum: f32 = w.iter().map(|x| x.max(0.0)).sum();
    if sum <= 1e-6 {
        return ([0; 4], [255, 0, 0, 0]);
    }
    let mut q = [0u8; 4];
    let mut total = 0i32;
    for k in 0..4 {
        q[k] = ((w[k].max(0.0) / sum) * 255.0).round().clamp(0.0, 255.0) as u8;
        total += q[k] as i32;
    }
    // Avrundingen rettes på den største vekta.
    let big = (0..4).max_by_key(|k| q[*k]).unwrap_or(0);
    q[big] = (q[big] as i32 + 255 - total).clamp(0, 255) as u8;
    (j.map(|x| x.min(255) as u8), q)
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
