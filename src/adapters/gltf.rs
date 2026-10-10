//! glTF-adapteren: leser glTF til `domain::model::Model`, bak porten
//! `ModelSource`. `Embedded` er modellene fra `tools/models/` som er bygd
//! inn i binæren (ikke i nettleseren); `Files` er de samme filene hentet
//! av nettleseren (`MODEL_FILES` sier hvilke).

use crate::domain::anim::{Channel, Clip, Joint, Keys, Rig};
use crate::domain::geometry::{flag, kind, Layer, Mat, Mesh};
use crate::domain::model::{Image, Model};
use crate::ports::ModelSource;
use glam::{Affine3A, Mat4, Quat, Vec2, Vec3};

/// Modellene: navn, og fila under
/// `assets/models/`. Alle er laget i Blender av skriptene i
/// `tools/models/`, med origo og målestokk som skriptene beskriver.
macro_rules! models {
    ($($name:literal => $path:literal),* $(,)?) => {
        /// Navnet og fila under `assets/models/` for hver modell.
        pub const MODEL_FILES: &[(&str, &str)] = &[$(($name, $path)),*];
        // Nettleseren henter filene selv (`Files`), så wasm-fila holdes liten.
        #[cfg(not(target_arch = "wasm32"))]
        const MODELS: &[(&str, &[u8])] = &[$(($name, include_bytes!(concat!("../../assets/models/", $path)) as &[u8])),*];
    };
}

models! {
    // Dyrene fra tools/models/animals.py: kroppen har origo i bakken under
    // midten, beina i hofteleddet (domain/herd.rs poserer dem).
    "sheep_body" => "animals/sheep_body.glb",
    "sheep_leg" => "animals/sheep_leg.glb",
    "cow_body" => "animals/cow_body.glb",
    "cow_leg_front" => "animals/cow_leg_front.glb",
    "cow_leg_hind" => "animals/cow_leg_hind.glb",
    // Trærne og buskene fra tools/models/trees.py: stammen med bakt bark,
    // og bladverket som kort (to modeller med samme origo).
    // Gresstustene fra tools/models/grass.py (kort med eierens strå).
    "grass_a" => "trees/grass_a.glb",
    "grass_b" => "trees/grass_b.glb",
    "grass_c" => "trees/grass_c.glb",
    "grass_d" => "trees/grass_d.glb",
    "flowers_a" => "trees/flowers_a.glb",
    "flowers_b" => "trees/flowers_b.glb",
    "flowers_c" => "trees/flowers_c.glb",
    "flowers_d" => "trees/flowers_d.glb",
    "oak1_trunk" => "trees/oak1_trunk.glb",
    "oak1_leaves" => "trees/oak1_leaves.glb",
    "oak2_trunk" => "trees/oak2_trunk.glb",
    "oak2_leaves" => "trees/oak2_leaves.glb",
    "birch1_trunk" => "trees/birch1_trunk.glb",
    "birch1_leaves" => "trees/birch1_leaves.glb",
    "spruce1_trunk" => "trees/spruce1_trunk.glb",
    "spruce1_needles" => "trees/spruce1_needles.glb",
    "spruce2_trunk" => "trees/spruce2_trunk.glb",
    "spruce2_needles" => "trees/spruce2_needles.glb",
    "bush1_trunk" => "trees/bush1_trunk.glb",
    "bush1_leaves" => "trees/bush1_leaves.glb",
    "bush2_trunk" => "trees/bush2_trunk.glb",
    "bush2_leaves" => "trees/bush2_leaves.glb",
    // Pynten i kjelleren (tools/models/gorvek.py og house.py).
    "gorvek_chest" => "gorvek/chest.glb",
    "gorvek_keg" => "gorvek/keg.glb",
    "gorvek_torch" => "gorvek/torch.glb",
    "gorvek_skull" => "gorvek/skull.glb",
    "door_lamp" => "house/door_lamp.glb",
    // Fiendene og lootet i kjelleren fra tools/models/mobs.py.
    "mutant_button" => "mobs/mutant_button.glb",
    "mutant_tooltip" => "mobs/mutant_tooltip.glb",
    "mutant_input" => "mobs/mutant_input.glb",
    "mutant_modal" => "mobs/mutant_modal.glb",
    "mutant_table" => "mobs/mutant_table.glb",
    "framework" => "mobs/framework.glb",
    "token" => "mobs/token.glb",
    "bolt" => "mobs/bolt.glb",
    "key" => "mobs/key.glb",
    // Spillerens armer fra tools/models/arms.py, i kamerarommet (origo i
    // øyet, −Z fram), og våpnene og skjoldene i armenes rom.
    "arms" => "arms/arms.glb",
    "sword" => "arms/sword.glb",
    "axe" => "arms/axe.glb",
    "dagger" => "arms/dagger.glb",
    "shield_round" => "arms/shield_round.glb",
    "shield_tower" => "arms/shield_tower.glb",
    // Husene i kjøpstaden fra tools/models/town.py: teksturlag i meter-uv
    // og bakt okklusjon i hjørnefargen; døra mot +Z, origo midt på grunnflaten.
    "house_a" => "town/house_a.glb",
    "house_b" => "town/house_b.glb",
    "house_c" => "town/house_c.glb",
    "house_d" => "town/house_d.glb",
    "house_e" => "town/house_e.glb",
    "house_f" => "town/house_f.glb",
    "house_g" => "town/house_g.glb",
    "house_h" => "town/house_h.glb",
    "house_i" => "town/house_i.glb",
    "house_j" => "town/house_j.glb",
    "house_k" => "town/house_k.glb",
    "house_l" => "town/house_l.glb",
    "house_m" => "town/house_m.glb",
    "house_n" => "town/house_n.glb",
    "house_o" => "town/house_o.glb",
    "house_p" => "town/house_p.glb",
    "house_q" => "town/house_q.glb",
    "house_r" => "town/house_r.glb",
    "house_s" => "town/house_s.glb",
    "house_t" => "town/house_t.glb",
    "shed_a" => "town/shed_a.glb",
    "shed_b" => "town/shed_b.glb",
    "privy" => "town/privy.glb",
    "haystack" => "town/haystack.glb",
    "garden" => "town/garden.glb",
    "coop" => "town/coop.glb",
    // Bymuren fra tools/models/walls.py: segment, hjørnetårn og porthus.
    "town_wall" => "town/town_wall.glb",
    "wall_tower" => "town/wall_tower.glb",
    "gatehouse" => "town/gatehouse.glb",
    // Torget fra tools/models/props.py.
    "market_cross" => "town/market_cross.glb",
    "market_stall" => "town/market_stall.glb",
    "market_stall_b" => "town/market_stall_b.glb",
    "cart" => "town/cart.glb",
    "barrel" => "town/barrel.glb",
    "crate" => "town/crate.glb",
    "lantern_post" => "town/lantern_post.glb",
    "training_dummy" => "town/training_dummy.glb",
    "weapon_rack" => "town/weapon_rack.glb",
    // Gatelivet og landskapet fra tools/models/street.py.
    "well" => "town/well.glb",
    "npc_smith" => "npc/npc_smith.glb",
    "npc_innkeeper" => "npc/npc_innkeeper.glb",
    "npc_apothecary" => "npc/npc_apothecary.glb",
    "npc_clerk" => "npc/npc_clerk.glb",
    "npc_gorvek" => "npc/npc_gorvek.glb",
    "npc_woman" => "npc/npc_woman.glb",
    // Portalen ved sørporten (tools/models/portal.py).
    "portal_ring" => "town/portal_ring.glb",
    "portal_runes" => "town/portal_runes.glb",
    "portal_swirl" => "town/portal_swirl.glb",
    "portal_shard" => "town/portal_shard.glb",
    "npc_farmer" => "npc/npc_farmer.glb",
    "npc_boy" => "npc/npc_boy.glb",
    "npc_guard" => "npc/npc_guard.glb",
    "npc_merchant" => "npc/npc_merchant.glb",
    "npc_pilgrim" => "npc/npc_pilgrim.glb",
    "well_winch" => "town/well_winch.glb",
    "well_rope" => "town/well_rope.glb",
    "well_bucket" => "town/well_bucket.glb",
    "wattle_fence" => "town/wattle_fence.glb",
    "woodpile" => "town/woodpile.glb",
    "stone_wall" => "town/stone_wall.glb",
    "boulder_a" => "town/boulder_a.glb",
    "boulder_b" => "town/boulder_b.glb",
    "boulder_c" => "town/boulder_c.glb",
    // Veiskiltet til krypten.
    "signpost" => "town/signpost.glb",
    // Kista i kjelleren og lokket som åpnes (origo i hengslet).
    "chest" => "town/chest.glb",
    "chest_lid" => "town/chest_lid.glb",
    // Bygningene på torget fra tools/models/buildings.py.
    "forge" => "town/forge.glb",
    "forge_interior" => "town/forge_interior.glb",
    "tavern" => "town/tavern.glb",
    "tavern_interior" => "town/tavern_interior.glb",
    "home" => "town/home.glb",
    "home_interior" => "town/home_interior.glb",
    "gallery_piece" => "town/gallery_piece.glb",
    "apothecary" => "town/apothecary.glb",
    "apothecary_interior" => "town/apothecary_interior.glb",
    "scriptorium" => "town/scriptorium.glb",
    "scriptorium_interior" => "town/scriptorium_interior.glb",
    "stave_church" => "town/stave_church.glb",
    "stave_church_interior" => "town/stave_church_interior.glb",
    "gravestone" => "town/gravestone.glb",
    // Borgruinen, krypten, døra opp fra krypten og portgitteret fra
    // tools/models/castle.py.
    "castle_ruin" => "town/castle_ruin.glb",
    "crypt" => "town/crypt.glb",
    "crypt_stair" => "town/crypt_stair.glb",
    "portcullis" => "town/portcullis.glb",
    // Innbyggerne fra tools/models/npc.py: én modell, tunikaen farges per instans.
    "npc" => "npc/npc.glb",
}

// Kryptmodellene (`CRYPT_FILES`, `CRYPT_MODELS`): bitene og byggesettet i
// `assets/models/crypt/`, funnet av build.rs.
include!(concat!(env!("OUT_DIR"), "/crypt_models.rs"));

/// Alle modellfilene: tabellen over og kryptmodellene.
pub fn model_files() -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    MODEL_FILES.iter().chain(CRYPT_FILES.iter())
}

/// Modellene som er med, lest ved behov.
#[cfg(not(target_arch = "wasm32"))]
pub struct Embedded;

#[cfg(not(target_arch = "wasm32"))]
impl ModelSource for Embedded {
    fn names(&self) -> Vec<String> {
        MODELS.iter().chain(CRYPT_MODELS.iter()).map(|(n, _)| n.to_string()).collect()
    }

    fn load(&self, name: &str) -> Option<Model> {
        let (_, bytes) = MODELS.iter().chain(CRYPT_MODELS.iter()).find(|(n, _)| *n == name)?;
        load(name, &[("model.glb", bytes)], -1.0, 0.0).map_err(|e| log::warn!("Modell {name}: {e}")).ok()
    }
}

/// Modellfilene etter stien i `MODEL_FILES`, hentet utenfra.
pub struct Files(pub std::collections::HashMap<String, Vec<u8>>);

impl ModelSource for Files {
    fn names(&self) -> Vec<String> {
        model_files().filter(|(_, p)| self.0.contains_key(*p)).map(|(n, _)| n.to_string()).collect()
    }

    fn load(&self, name: &str) -> Option<Model> {
        let (_, path) = model_files().find(|(n, _)| *n == name)?;
        let bytes = self.0.get(*path)?;
        load(name, &[("model.glb", bytes)], -1.0, 0.0).map_err(|e| log::warn!("Modell {name}: {e}")).ok()
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
            // Okklusjonen Blender har bakt i hjørnefargen (rød kanal, lineær).
            let ao: Vec<f32> = reader.read_colors(0).map(|c| c.into_rgba_f32().map(|c| c[0]).collect()).unwrap_or_default();
            // «layer_<lag>»: et flisbart teksturlag lagt på etter uv i meter.
            let layer = mat.name().and_then(|n| n.strip_prefix("layer_")).map(|n| n.split('.').next().unwrap_or(n)).and_then(|n| Layer::ALL.iter().copied().find(|l| l.name() == n));
            if mat.name().is_some_and(|n| n.starts_with("layer_")) && layer.is_none() {
                log::warn!("Modell {name}: ukjent teksturlag i materialet {:?}", mat.name());
            }
            let idx: Vec<u32> = reader.read_indices().map(|i| i.into_u32().collect()).unwrap_or_else(|| (0..pos.len() as u32).collect());
            // Normalene kan være pakket som 8 bit med fortegn
            // (KHR_mesh_quantization, `tools/models/slim.py`); leseren i
            // gltf-kassa tar bare flyttall, så de pakkes ut her.
            let norms: Vec<[f32; 3]> = match prim.get(&gltf::Semantic::Normals) {
                Some(a) if a.data_type() == gltf::accessor::DataType::I8 => packed_normals(&a, &buffers).unwrap_or_else(|| flat_normals(&pos, &idx)),
                _ => match reader.read_normals() {
                    Some(n) => n.collect(),
                    None => flat_normals(&pos, &idx),
                },
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
            // «layer_<lag>.paint»: teksturlaget med lakkfargen fra instansen.
            if layer.is_some() && mat.name().is_some_and(|n| n.ends_with(".paint")) {
                mmat = mmat.flags(flag::PAINT);
            }
            // «layer_<lag>.magic»: teksturlaget gløder hele døgnet der bildet
            // er mettet (de lysende sømmene i `glitch_seam`), og blafrer.
            if layer.is_some() && mat.name().is_some_and(|n| n.contains(".magic")) {
                mmat = mmat.flags(flag::GLOW_ALWAYS);
            }
            // «glow» lyser om natta, som lampene ellers i byen; «glass»
            // speiler som vinduene.
            if named("glow") {
                mmat.kind = kind::LAMP;
            }
            if named("glass") {
                mmat.kind = kind::GLASS;
            }
            // «glass_day» er innsiden av en vindusrute: dagslyset skinner gjennom.
            if named("glass_day") {
                mmat = mmat.flags(flag::GLOW_ALWAYS);
            }
            // «glass_lit» er utsida av en rute i et hus man kan gå inn i:
            // lyset inne skinner ut om natta.
            if named("glass_lit") {
                mmat = mmat.flags(flag::GLOW_LAMP);
            }
            // «water» er vann (cisternen i kjelleren).
            if named("water") {
                mmat.kind = kind::WATER;
            }
            // Kjelleren ligger under bakken: verken regn eller snø (de gamle
            // `dng_`-modellene, kryptnivåene `crypt_NN_…` og byggesettet `kit_`).
            let crypt_level = name.strip_prefix("crypt_").is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()));
            if name.starts_with("dng_") || name.starts_with("kit_") || crypt_level {
                mmat = mmat.flags(flag::NO_WEATHER);
            }
            // «fire» brenner hele døgnet (fakler, ildsteder).
            if named("fire") {
                mmat.kind = kind::LAMP;
                mmat = mmat.flags(flag::GLOW_ALWAYS);
            }
            // «magic» gløder i sin egen farge hele døgnet (portalen).
            if named("magic") {
                mmat.kind = kind::MAGIC;
                mmat = mmat.flags(flag::GLOW_ALWAYS);
            }
            // Bladkort og nålekvister: alfatestes og svaier i vinden.
            if named("leaf") || named("needle") || named("grass") {
                mmat = mmat.flags(flag::ALPHA | flag::WIND | flag::NO_WEATHER);
            }
            // Gresstrå: mørkere mot rota (høyden i primitivet).
            let (y_lo, y_hi) = pos.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p[1]), hi.max(p[1])));
            let grass = named("grass");
            if metallic >= 0.5 && layer.is_none() {
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
                let occl = if grass { 0.45 + 0.55 * ((p[1] - y_lo) / (y_hi - y_lo).max(1e-3)) } else { ao.get(i).copied().unwrap_or(1.0) };
                let v = match layer {
                    Some(l) => Mat { kind: l as u8, flags: flags | flag::UVMAP, ..mmat },
                    None => Mat { kind: if mmat.kind == kind::PLAIN { kind::MODEL } else { mmat.kind }, flags, ..mmat },
                };
                let vi = mesh.vert(wp, wn, Vec2::from(t), &v, occl);
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

/// Normaler lagret som tre bytes med fortegn (normalisert, −127..127).
fn packed_normals(a: &gltf::Accessor, buffers: &[Vec<u8>]) -> Option<Vec<[f32; 3]>> {
    let view = a.view()?;
    let data = buffers.get(view.buffer().index())?;
    let stride = view.stride().unwrap_or(a.size());
    let start = view.offset() + a.offset();
    (0..a.count())
        .map(|i| {
            let o = start + i * stride;
            let b = data.get(o..o + 3)?;
            let f = |v: u8| (v as i8 as f32 / 127.0).max(-1.0);
            Some(Vec3::new(f(b[0]), f(b[1]), f(b[2])).normalize_or_zero().to_array())
        })
        .collect()
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
