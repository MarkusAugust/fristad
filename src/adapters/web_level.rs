//! Nivåredigeringen i nettleseren (`?nivaa`, `web/nivaa.js`,
//! `docs/nivaaredigering.md` del 12): motoren med dokumentet som eneste
//! kryptnivå, kommandoene fra kartet som json, kartet og dokumentet tilbake
//! som json, og prøveturen (verden bygges på nytt med dokumentet).

use super::{connect, Engine, Rebuild};
use crate::adapters::gltf::Files;
use crate::adapters::rapier::RapierPhysics;
use crate::adapters::wgpu::Renderer;
use crate::application::{prepare_level_editor, App};
use crate::domain::edit::level::{map, LevelEditor, LevelOp};
use crate::domain::world::crypt::doc::LevelDoc;
use crate::ports::driving::{Command, LevelCommand, Play};
use wasm_bindgen::prelude::*;

/// Navnene på nivådokumentene i repoet (`assets/levels/nivaa_<navn>.json`).
#[wasm_bindgen]
pub fn level_doc_names() -> Vec<String> {
    crate::adapters::data::level_doc_names()
}

/// Teksten til et nivådokument i repoet (tom om det ikke finnes; `brett` og
/// `moduler` lages av byggesettet).
#[wasm_bindgen]
pub fn level_doc_source(name: &str) -> String {
    match crate::adapters::data::level_doc_text(name) {
        Some(t) => t.to_string(),
        None => crate::adapters::data::level_doc(name).map(|d| crate::adapters::data::level_doc_json(&d)).unwrap_or_default(),
    }
}

/// Et nytt, tomt nivå (kapellet med døra opp og startstedet), som json.
#[wasm_bindgen]
pub fn level_blank(name: &str, number: u32, theme: &str) -> String {
    let kit = crate::adapters::data::kit();
    crate::adapters::data::level_doc_json(&LevelEditor::blank(name, number.clamp(1, 10), theme, &kit))
}

/// Paletten: modulene med cellene og åpningene, steinene, innholdet,
/// fiendeslagene og komponentene en kiste kan gi.
#[wasm_bindgen]
pub fn level_palette() -> String {
    let kit = crate::adapters::data::kit();
    let mobs = crate::domain::mob_kind::Kind::ALL.iter().map(|k| (k.key().to_string(), k.label().to_string())).collect();
    let loot = crate::domain::craft::GALLERY.iter().map(|c| c.to_string()).collect();
    serde_json::to_string(&map::palette(&kit, mobs, loot)).unwrap_or_default()
}

/// Les et dokument (json). Feilen som tekst.
fn parse_doc(json: &str) -> Result<LevelDoc, String> {
    serde_json::from_str::<LevelDoc>(json).map_err(|e| format!("Nivået kan ikke leses: {e}"))
}

/// Start motoren i nivåredigeringen med dokumentet `doc` (json; uten: et
/// nytt nivå 1). Som `start`, med alle modellene.
#[wasm_bindgen]
pub async fn start_level_editor(canvas: web_sys::HtmlCanvasElement, webgpu: bool, manifest_json: Option<String>, names: js_sys::Array, blobs: js_sys::Array, models: js_sys::Array, doc: Option<String>) -> Result<Engine, JsValue> {
    let (surface, adapter, device, queue) = connect(&canvas, webgpu).await.map_err(|e| JsValue::from_str(&e))?;
    let backend = format!("{:?}", adapter.get_info().backend);
    let caps = surface.get_capabilities(&adapter);
    let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: canvas.width().max(1),
        height: canvas.height().max(1),
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);
    let seed = 0x5eed;
    let files = Files(super::model_paths().into_iter().zip(models.iter()).filter(|(_, b)| !b.is_undefined()).map(|(p, b)| (p, js_sys::Uint8Array::new(&b).to_vec())).collect());
    let kit = crate::adapters::data::kit();
    let doc = match doc.as_deref().map(parse_doc) {
        Some(Ok(d)) => d,
        Some(Err(e)) => {
            log::warn!("{e}");
            LevelEditor::blank("Nytt nivå", 1, "grav", &kit)
        }
        None => LevelEditor::blank("Nytt nivå", 1, "grav", &kit),
    };
    let prepared = prepare_level_editor(&files, seed, LevelEditor::new(doc, kit));
    let layers = match manifest_json {
        Some(m) => {
            let files: std::collections::HashMap<String, Vec<u8>> = names.iter().zip(blobs.iter()).filter_map(|(n, b)| Some((n.as_string()?, js_sys::Uint8Array::new(&b).to_vec()))).collect();
            crate::adapters::layers::from_files(&m, &files)
        }
        None => vec![],
    };
    let renderer = Renderer::new(device.clone(), queue.clone(), &adapter, format, &App::<Renderer, RapierPhysics>::scene(&prepared), &layers, config.width, config.height);
    let mut app = App::new(prepared, renderer, RapierPhysics::new());
    // Kameraet i startstedet (ellers over origo), så 3D viser nivået innenfra.
    if let Some(l) = app.session.level_editing() {
        let (eye, yaw) = match l.start() {
            Some((feet, yaw)) => (feet + glam::Vec3::Y * 1.7, yaw),
            None => (l.origin + glam::Vec3::new(0.0, 1.7, 0.0), 0.0),
        };
        app.command(Command::Place { eye, yaw, pitch: -0.1 });
    }
    let rebuild = Some(Rebuild { files, queue, adapter, format, layers, seed });
    Ok(Engine { app, surface, config, device, backend, rebuild })
}

/// En kommando fra kartet: enten en endring (`{"op": …}`, `LevelOp`) eller
/// `{"cmd": …}`.
#[derive(serde::Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum LevelJson {
    Undo,
    Redo,
    Saved,
    Select { ids: Vec<String> },
    Light { work: bool },
    Floor { y: f32 },
    Place { x: f32, y: f32, z: f32, yaw: f32, pitch: f32 },
}

#[wasm_bindgen]
impl Engine {
    /// I nivåredigeringen (startet med `start_level_editor`).
    pub fn level_ready(&self) -> bool {
        self.app.session.level_editing().is_some()
    }

    /// En kommando fra kartet (json). Gir kartet etter kommandoen, eller
    /// `{"error": …}`.
    pub fn level(&mut self, json: &str) -> String {
        let value: serde_json::Value = match serde_json::from_str(json) {
            Ok(v) => v,
            Err(e) => return serde_json::json!({"error": format!("{e}")}).to_string(),
        };
        let command = if value.get("op").is_some() {
            match serde_json::from_value::<LevelOp>(value) {
                Ok(op) => LevelCommand::Op(op),
                Err(e) => return serde_json::json!({"error": format!("Ukjent endring: {e}")}).to_string(),
            }
        } else {
            match serde_json::from_value::<LevelJson>(value) {
                Ok(LevelJson::Undo) => LevelCommand::Undo,
                Ok(LevelJson::Redo) => LevelCommand::Redo,
                Ok(LevelJson::Saved) => LevelCommand::Saved,
                Ok(LevelJson::Select { ids }) => LevelCommand::Select(ids),
                Ok(LevelJson::Light { work }) => LevelCommand::WorkLight(work),
                Ok(LevelJson::Floor { y }) => LevelCommand::Floor(y),
                Ok(LevelJson::Place { x, y, z, yaw, pitch }) => LevelCommand::Place { at: glam::Vec3::new(x, y, z), yaw: yaw.to_radians(), pitch: pitch.to_radians() },
                Err(e) => return serde_json::json!({"error": format!("Ukjent kommando: {e}")}).to_string(),
            }
        };
        self.app.command(Command::Level(command));
        if let Some(e) = self.app.session.level_error.take() {
            return serde_json::json!({"error": e}).to_string();
        }
        self.level_map()
    }

    /// Kartet slik nivået står nå (json, `LevelMap`), med id-ene siste
    /// endring laget i `made`.
    pub fn level_map(&self) -> String {
        let Some(l) = self.app.session.level_editing() else {
            return serde_json::json!({"error": "Ikke i nivåredigeringen."}).to_string();
        };
        let mut v = serde_json::to_value(l.editor.map()).unwrap_or_default();
        v["made"] = serde_json::json!(self.app.session.level_made);
        v["playing"] = serde_json::json!(!l.active);
        v.to_string()
    }

    /// Dokumentet i fast form og stien det lagres i: `{"path", "text"}`.
    pub fn level_doc(&self) -> String {
        let Some(l) = self.app.session.level_editing() else {
            return serde_json::json!({"error": "Ikke i nivåredigeringen."}).to_string();
        };
        let doc = l.editor.doc();
        serde_json::json!({"path": crate::adapters::data::level_doc_path(doc), "text": crate::adapters::data::level_doc_json(doc)}).to_string()
    }

    /// Utvalget (json-liste med id-er) som mal i fast form
    /// (`LevelEditor::template`; lagres i `assets/levels/maler/<navn>.json`).
    pub fn level_template(&self, ids_json: &str, name: &str) -> String {
        let Some(l) = self.app.session.level_editing() else {
            return String::new();
        };
        let ids: Vec<String> = serde_json::from_str(ids_json).unwrap_or_default();
        crate::adapters::data::level_doc_json(&l.editor.template(&ids, name))
    }

    /// Åpne et annet dokument (json). Gir kartet eller `{"error": …}`.
    pub fn level_open(&mut self, json: &str) -> String {
        match parse_doc(json) {
            Ok(doc) => {
                self.app.command(Command::Level(LevelCommand::Open(Box::new(doc))));
                self.level_map()
            }
            Err(e) => serde_json::json!({"error": e}).to_string(),
        }
    }

    /// Punktet på gulvet under pekeren i 3D-vinduet (0..1), i dokumentets
    /// rom: [x, y, z], eller tomt.
    pub fn level_pick(&self, x: f32, y: f32) -> Vec<f32> {
        self.app.session.level_pick((x, y)).map(|p| p.to_array().to_vec()).unwrap_or_default()
    }

    /// Kameraet i dokumentets rom: [x, y, z, yaw, pitch] (grader).
    pub fn level_camera(&self) -> Vec<f32> {
        self.app.session.level_camera().map(|(e, yaw, pitch)| vec![e.x, e.y, e.z, yaw.to_degrees(), pitch.to_degrees()]).unwrap_or_default()
    }

    /// Prøveturen: bygg verden på nytt med dokumentet som nivå 1 og spill
    /// fra startstedet (`here`: der kameraet står). Du kan ikke dø. Gir om
    /// det gikk.
    pub fn level_play(&mut self, here: bool) -> bool {
        let Some(rb) = self.rebuild.as_ref().filter(|_| !self.app.renderer.is_lost()) else {
            return false;
        };
        let back = self.app.session.view_pose();
        let Some(editor) = self.app.session.take_level_editor() else {
            return false;
        };
        let prepared = prepare_level_editor(&rb.files, rb.seed, editor);
        let renderer = Renderer::new(self.device.clone(), rb.queue.clone(), &rb.adapter, rb.format, &App::<Renderer, RapierPhysics>::scene(&prepared), &rb.layers, self.config.width, self.config.height);
        self.app = App::new(prepared, renderer, RapierPhysics::new());
        self.app.session.level_play(back, here)
    }

    /// Tilbake fra prøveturen til redigeringen (Esc).
    pub fn level_back(&mut self) -> bool {
        self.app.input.release_all();
        self.app.session.level_back()
    }

    /// På prøvetur.
    pub fn level_playing(&self) -> bool {
        self.app.session.level_editing().is_some_and(|l| !l.active)
    }
}
