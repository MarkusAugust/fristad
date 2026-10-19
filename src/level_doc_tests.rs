//! Nivåredigeringen, fase 1 (`docs/nivaaredigering.md`): kompilatoren i
//! kjernen (`crypt::compose`) over byggesettet (`assets/models/crypt/kit/`),
//! prøvenivået (`assets/levels/nivaa_test.json`) og prøvebrettet (hver bit
//! mot hver bit i alle dreininger). Prøvenivået går også gjennom de gamle
//! testene over krypten (`provenivaaet_kan_gas_gjennom`,
//! `ingen_usynlige_vegger_i_provenivaaet`, `fiendene_finner_fram_i_provenivaaet`,
//! `lyset_folger_rommene_i_provenivaaet`).

use crate::adapters::data::{kit, level_doc};
use crate::domain::world::crypt::compose::{self, board::board, check, Compiler, Severity, *};
use crate::domain::world::crypt::doc::{DocCell, DocEdge, LevelDoc};
use crate::domain::world::crypt::kit::PieceKind;
use glam::{Vec2, Vec3};

fn test_doc() -> LevelDoc {
    level_doc("test").expect("assets/levels/nivaa_test.json")
}

/// Modellene en bit kan ha (modellen og variantene).
fn family(k: &crate::domain::world::crypt::kit::KitDef, id: &str) -> Vec<String> {
    let p = k.piece(id).unwrap_or_else(|| panic!("{id} finnes ikke"));
    let mut out: Vec<String> = std::iter::once(p.model.clone()).chain(p.variants.iter().map(|v| v.model.clone())).collect();
    out.sort();
    out.dedup();
    out
}

fn errors(r: &compose::Report) -> Vec<String> {
    r.errors().map(|i| format!("{} {:?}: {}", i.rule, i.at, i.msg)).collect()
}

/// Byggesettet finnes, og hver bit har modellen sin.
#[test]
fn byggesettet_har_bitene() {
    let k = kit();
    assert!(k.pieces.iter().filter(|p| p.kind == PieceKind::Cell).count() >= 6, "for få cellebiter: {:?}", k.pieces.iter().map(|p| &p.id).collect::<Vec<_>>());
    let models: Vec<&str> = crate::adapters::gltf::model_files().map(|(n, _)| *n).collect();
    for m in k.models() {
        assert!(models.contains(&m), "byggesettet bruker {m}, som ikke finnes i assets/models/crypt/kit/");
    }
    for opening in ["wall", "gang", "door", "gate", "secret", "arch", "overmur", "rail"] {
        assert!(k.edge("grav", opening).is_some(), "ingen halvkant for «{opening}»");
    }
    assert!(k.of_kind("grav", PieceKind::Corner).is_some() && k.of_kind("grav", PieceKind::Pillar).is_some());
    assert!(k.of_kind("grav", PieceKind::Leaf).is_some() && k.of_kind("grav", PieceKind::Prop).is_some(), "den falske veggen og fakkelen mangler");
    for class in ["inner", "wall", "pilaster", "outer"] {
        assert!(k.corner("grav", class).is_some_and(|p| p.fits.iter().any(|f| f == class)), "ingen kvarthjørne for «{class}»");
    }
    // Pilotsettet (del 4.4): rundt 20 biter, de vanligste i minst to utseender.
    assert!(k.models().len() >= 20, "pilotsettet har {} modeller", k.models().len());
    for id in ["grav/romfelt", "grav/gang", "grav/vegg"] {
        assert!(family(&k, id).len() >= 2, "{id} har bare ett utseende");
    }
    assert!(k.listed().any(|p| !p.footprint.is_empty()), "ingen sammensatt bit (rotunden)");
}

/// Samme dokument gir det samme, byte for byte; og å regne bare det som er
/// endret gir det samme som å regne alt på nytt.
#[test]
fn nivaadokumentet_kompileres_likt() {
    let k = kit();
    let doc = test_doc();
    let (a, ra) = compose::compile(&doc, &k);
    let (b, _) = compose::compile(&doc, &k);
    assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap(), "to kompileringer ga ulikt resultat");
    assert_eq!(ra.recomputed, ra.cells);
    // Bare det endrede: samme kompilator husker cellene.
    let mut c = Compiler::default();
    c.compile(&doc, &k);
    let (same, r) = c.compile(&doc, &k);
    assert_eq!(r.recomputed, 0, "ingenting var endret, men {} celler ble regnet på nytt", r.recomputed);
    assert_eq!(same, a);
    // Tre endringer: døra blir port, en celle forsvinner, og en ny kommer.
    let mut doc2 = doc.clone();
    doc2.edges = vec![DocEdge { cell: "k22".into(), side: "S".into(), kind: "port".into(), ..Default::default() }];
    doc2.cells.retain(|c| c.id != "g06");
    doc2.cells.push(DocCell { id: "ny".into(), bit: "grav/romfelt".into(), at: [-1, 0, 1], room: "gjemme".into(), ..Default::default() });
    let (inc, r) = c.compile(&doc2, &k);
    let (full, _) = compose::compile(&doc2, &k);
    assert_eq!(serde_json::to_string(&inc).unwrap(), serde_json::to_string(&full).unwrap(), "bare det endrede ga noe annet enn alt på nytt");
    // Nabolaget er 5 × 5 celler rundt hver endring.
    assert!(r.recomputed > 0 && r.recomputed < doc2.cells.len(), "regnet {} av {} celler på nytt", r.recomputed, doc2.cells.len());
    let mut doc3 = doc2.clone();
    doc3.cells.iter_mut().find(|c| c.id == "f46").unwrap().room = "forhall2".into();
    let (inc, r) = c.compile(&doc3, &k);
    assert!(r.recomputed <= 9, "én celle i hjørnet endret, men {} regnet på nytt", r.recomputed);
    assert_eq!(inc, compose::compile(&doc3, &k).0);
    // Og det endrede synes: porten står der døra sto.
    assert!(inc.instances.iter().any(|i| i.model == "grav_vegg_port") && !inc.instances.iter().any(|i| i.model == "grav_vegg_dor"));
}

/// Dreiningene og speilingen: sidene, gulvene, det gangbare og bitene
/// selv følger hverandre (fire dreininger og to speilinger er ingenting).
#[test]
fn bitene_dreies_og_speiles_riktig() {
    let k = kit();
    for rot in 0..8u8 {
        for mirror in [false, true] {
            let t = transform(Vec3::ZERO, cell_yaw(rot), mirror);
            for l in 0..4 {
                let w = local_to_world(l, rot % 4, mirror);
                assert_eq!(world_to_local(w, rot % 4, mirror), l);
                let n = outward_v(l);
                let nw = t.transform_vector3(Vec3::new(n.x, 0.0, n.y));
                assert!((Vec2::new(nw.x, nw.z) - outward_v(w)).length() < 1e-5, "rot {rot} speilet {mirror}: siden {l} skulle bli {w}, ble {nw:?}");
            }
            // Fire dreininger er ingenting; to speilinger er ingenting.
            let p = Vec3::new(0.7, 1.0, -1.3);
            let four = (0..4).fold(p, |q, _| transform(Vec3::ZERO, cell_yaw(rot), false).transform_point3(q));
            assert!((four - p).length() < 1e-4);
            let twice = transform(Vec3::ZERO, 0.0, true).transform_point3(transform(Vec3::ZERO, 0.0, true).transform_point3(p));
            assert!((twice - p).length() < 1e-6);
        }
    }
    // Halvkanten og hjørnet vender inn i cella.
    for w in 0..4 {
        let into = transform(Vec3::ZERO, side_yaw(w), false).transform_vector3(Vec3::Z);
        assert!((Vec2::new(into.x, into.z) + outward_v(w)).length() < 1e-5, "halvkanten på siden {w} vender ikke inn");
    }
    for c in 0..4 {
        let into = transform(Vec3::ZERO, corner_yaw(c), false).transform_vector3(Vec3::new(1.0, 0.0, 1.0));
        let v = vertex_pos(corner_vertex(0, 0, c));
        assert!(Vec2::new(into.x, into.z).dot(-v) > 0.0, "hjørnet {c} vender ikke inn i cella");
    }
    // Hver cellebit alene i alle dreininger og speilet: trappa stiger mot
    // siden som har det høye gulvet, og det gangbare står inne i cella.
    for piece in k.listed().filter(|p| p.kind == PieceKind::Cell) {
        // En sammensatt bit (rotunden) dekker 3 × 3 celler.
        let reach = if piece.footprint.is_empty() { HALF } else { 3.0 * HALF };
        for rot in 0..4u8 {
            for mirror in [false, true] {
                let doc = LevelDoc { cells: vec![DocCell { id: "c".into(), bit: piece.id.clone(), at: [0, 0, 0], rot, mirror, room: "r".into(), ..Default::default() }], ..Default::default() };
                let (f, r) = compose::compile(&doc, &k);
                assert!(check::tight(&doc, &k, &f).is_empty(), "{} rot {rot} speilet {mirror} er ikke tett: {:?}", piece.id, check::tight(&doc, &k, &f));
                assert!(r.errors().all(|i| i.rule != "tett" && i.rule != "bit"), "{}: {:?}", piece.id, errors(&r));
                for v in &f.nav.verts {
                    assert!(v[0].abs() <= reach + 1e-3 && v[2].abs() <= reach + 1e-3, "{} rot {rot}: nettet går ut av cella ved {v:?}", piece.id);
                }
                let hi = (0..4).max_by(|a, b| piece.floors[*a].total_cmp(&piece.floors[*b])).unwrap();
                if piece.floors[hi] > 0.0 {
                    let w = local_to_world(hi, rot, mirror && piece.mirror);
                    let top = f.nav.verts.iter().max_by(|a, b| a[1].total_cmp(&b[1])).unwrap();
                    let dir = outward_v(w);
                    assert!(Vec2::new(top[0], top[2]).dot(dir) > 0.5, "{} rot {rot} speilet {mirror}: trappa stiger ikke mot {}", piece.id, SIDES[w]);
                }
            }
        }
    }
}

/// Kantene: prøvenivået har ingen brudd, prøvebrettet bare de ventede
/// (trapper som ender i vegg), og kantene følger tabellen i del 5.1.
#[test]
fn kantene_passer() {
    let k = kit();
    let (_, r) = compose::compile(&test_doc(), &k);
    assert!(r.issues.is_empty(), "prøvenivået har brudd:\n{}", r.issues.iter().map(|i| format!("{:?} {}: {}", i.severity, i.rule, i.msg)).collect::<Vec<_>>().join("\n"));
    let b = board(&k);
    let (f, r) = compose::compile(&b, &k);
    for i in check::edges(&r.issues) {
        assert!(i.msg.contains("ender i vegg"), "prøvebrettet: {}", i.msg);
    }
    // Tabellen: for hvert par, kanten mellom a og b.
    let at = |model: &str, e: Vec3| f.instances.iter().filter(|i| i.model == model && Vec2::new(i.pos[0] - e.x, i.pos[2] - e.z).length() < 0.01).count();
    for (n, pair) in b.cells.chunks(2).enumerate() {
        let (a, bb) = (&pair[0], &pair[1]);
        let (pa, pb) = (k.piece(&a.bit).unwrap(), k.piece(&bb.bit).unwrap());
        let fa = &pa.faces[world_to_local(1, a.rot, false)];
        let fb = &pb.faces[world_to_local(3, bb.rot, false)];
        let meet = (pa.floors[world_to_local(1, a.rot, false)] - (bb.at[1] as f32 + pb.floors[world_to_local(3, bb.rot, false)])).abs() < 0.01;
        let edge = Vec3::new(a.at[0] as f32 * CELL + HALF, 0.0, a.at[2] as f32 * CELL);
        let what = format!("par {n} ({} dreid {} mot {} dreid {})", a.bit, a.rot, bb.bit, bb.rot);
        let expect_wall = !meet || fa.kind == "closed" || fb.kind == "closed";
        let walls: usize = family(&k, "grav/vegg").iter().map(|m| at(m, edge)).sum();
        assert_eq!(walls, if expect_wall { 2 } else { 0 }, "{what}: murene");
        if expect_wall {
            continue;
        }
        if fa.kind == "open" && fb.kind == "open" {
            assert_eq!(at("grav_gjordbue", edge), 2, "{what}: to romfelt i samme rom skal ha gjordbuen");
        } else {
            // En gangåpning: overmuren på sidene som ikke har gangprofilen.
            let want = [fa, fb].iter().filter(|f| f.profile != "gang").count();
            assert_eq!(at("grav_vegg_gang", edge), want, "{what}: overmuren");
        }
    }
    // Eierens valg som ikke går: en dør mot fjellet.
    let mut bad = test_doc();
    bad.edges.push(DocEdge { cell: "k10".into(), side: "N".into(), kind: "dor".into(), ..Default::default() });
    let (f2, r) = compose::compile(&bad, &k);
    assert!(r.errors().any(|i| i.rule == "kant" && i.msg.contains("går ingen vei")), "døra mot fjellet ble ikke meldt");
    assert!(check::tight(&bad, &k, &f2).is_empty(), "døra mot fjellet skal bli vegg");
}

/// Tett: hver side er lukket eller åpen mot en celle, og muren ender aldri
/// i lufta i et hjørne (prøvenivået og hele prøvebrettet).
#[test]
fn nivaaet_er_tett() {
    let k = kit();
    for doc in [test_doc(), board(&k)] {
        let (f, _) = compose::compile(&doc, &k);
        let leaks = check::tight(&doc, &k, &f);
        assert!(leaks.is_empty(), "{}: {} hull:\n{}", doc.name, leaks.len(), leaks.iter().take(30).map(|i| i.msg.clone()).collect::<Vec<_>>().join("\n"));
    }
    // Og regelen finner et hull: uten halvkantene er alt åpent.
    let doc = test_doc();
    let (mut f, _) = compose::compile(&doc, &k);
    let walls = family(&k, "grav/vegg");
    f.instances.retain(|i| !walls.contains(&i.model));
    assert!(!check::tight(&doc, &k, &f).is_empty(), "regelen fant ingen hull uten murene");
}

/// Bitene overlapper ikke i volum, og regelen finner det når de gjør.
#[test]
fn bitene_overlapper_ikke() {
    let k = kit();
    for doc in [test_doc(), board(&k)] {
        assert!(check::overlaps(&doc, &k).is_empty(), "{}: {:?}", doc.name, check::overlaps(&doc, &k));
    }
    let mut bad = test_doc();
    bad.cells.push(DocCell { id: "oppå".into(), bit: "grav/romfelt".into(), at: [2, 3, 1], room: "kapell".into(), ..Default::default() });
    assert_eq!(check::overlaps(&bad, &k).len(), 1);
    let (_, r) = compose::compile(&bad, &k);
    assert!(r.errors().any(|i| i.severity == Severity::Error && i.rule == "overlapp"));
    // Seks meter opp er en egen etasje: lov.
    bad.cells.last_mut().unwrap().at[1] = 6;
    assert!(check::overlaps(&bad, &k).is_empty());
}

/// Prøvenivået i spillet: bitene står som instanser, og nivået har det et
/// nivå skal ha (start, opp, hvilested, ned, porten med spaken).
#[test]
fn provenivaaet_bygges_av_bitene() {
    let p = crate::tests::doc_world();
    let c = &p.world.crypt;
    assert_eq!(c.levels.len(), 1);
    let l = &c.levels[0];
    assert!(l.down.is_some() && l.up.is_some() && c.rests.len() == 1 && c.gates.len() == 1 && c.levers.len() == 1);
    // Den falske veggen (`grav_falsk`) settes ut av mekanismen, ikke som bit.
    let placed: usize = p.world.placements.iter().filter(|pl| pl.model.starts_with("grav_") && pl.model != "grav_falsk").map(|pl| pl.transforms.len()).sum();
    let (f, _) = compose::compile(&test_doc(), &kit());
    assert_eq!(placed, f.instances.len(), "ikke alle bitene står i verden");
    assert!(p.world.missing.iter().all(|m| !m.starts_with("grav_")), "modeller mangler: {:?}", p.world.missing);
    eprintln!("prøvenivået: {} instanser, {} kollidere, {} polygoner, {} rom", f.instances.len(), f.colliders.len(), f.nav.polys.len(), f.rooms.len());
}

/// Manifestet og glb-filene sier det samme: hver modell i byggesettet har
/// manifestet for bitene som bruker den i `extras.kit` (skrevet av
/// `tools/crypt/kit_bake.py` ved eksporten, likt `assets/models/crypt/kit/grav.json`).
#[test]
fn manifestet_er_likt_glb_filene() {
    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string("assets/models/crypt/kit/grav.json").unwrap()).unwrap();
    let pieces = manifest["pieces"].as_array().unwrap();
    let files: std::collections::HashMap<&str, &str> = crate::adapters::gltf::model_files().map(|(n, p)| (*n, *p)).collect();
    let k = kit();
    for model in k.models() {
        let want: Vec<&serde_json::Value> = pieces
            .iter()
            .filter(|p| p["model"] == model || p["variants"].as_array().is_some_and(|vs| vs.iter().any(|v| v["model"] == model)))
            .collect();
        assert!(!want.is_empty(), "{model}: ingen bit i manifestet bruker modellen");
        let path = format!("assets/models/{}", files.get(model).unwrap_or_else(|| panic!("{model} finnes ikke")));
        let bytes = std::fs::read(&path).unwrap();
        let doc = gltf::Gltf::from_slice(&bytes).unwrap();
        let extras: serde_json::Value = serde_json::from_str(doc.as_json().extras.as_ref().map(|r| r.get()).unwrap_or("{}")).unwrap();
        let got = extras["kit"].as_array().unwrap_or_else(|| panic!("{path}: mangler extras.kit (eksporter med kit_cells.py)"));
        assert_eq!(got.iter().collect::<Vec<_>>(), want, "{path}: extras.kit er ikke likt manifestet");
    }
}

/// Hver bit har en fjernversjon (noden `far`) som er lettere enn den nære,
/// og budsjettet per bit holder (del 14: rundt 2 500 trekanter per celle med
/// mur, hjørner, gulv og hvelv).
#[test]
fn bitene_har_fjernversjon_og_holder_budsjettet() {
    let files: std::collections::HashMap<&str, &str> = crate::adapters::gltf::model_files().map(|(n, p)| (*n, *p)).collect();
    let k = kit();
    for model in k.models() {
        let bytes = std::fs::read(format!("assets/models/{}", files[model])).unwrap();
        let m = crate::adapters::gltf::load(model, &[("model.glb", &bytes)], -1.0, 0.0).unwrap();
        let near = m.mesh.indices.len() / 3;
        let far = m.far_mesh.as_ref().map(|f| f.indices.len() / 3).unwrap_or_else(|| panic!("{model}: ingen fjernversjon"));
        assert!(far > 0 && far <= near, "{model}: fjernversjonen har {far} trekanter, den nære {near}");
        let cells = k.listed().filter(|p| p.model == model || p.variants.iter().any(|v| v.model == model)).map(|p| p.footprint.len().max(1)).max().unwrap_or(1);
        assert!(near <= 1_200 * cells, "{model}: {near} trekanter (budsjettet er 1 200 per celle for én bit)");
    }
}

/// Variantene (del 9): samme fil gir samme utseende; frøet og plassen
/// velger, et låst utseende står, og et nytt frø stokker uten å endre noe
/// annet enn utseendet.
#[test]
fn variantene_er_faste() {
    let k = kit();
    let doc = test_doc();
    let (a, _) = compose::compile(&doc, &k);
    for id in ["grav/vegg", "grav/romfelt"] {
        for m in family(&k, id) {
            assert!(a.instances.iter().any(|i| i.model == m), "prøvenivået bruker ikke {m} ({id})");
        }
    }
    // Lås et utseende i en celle: bare den cella endres.
    let cell = |f: &crate::domain::world::crypt::level::LevelFile, at: [f32; 3]| f.instances.iter().find(|i| family(&k, "grav/romfelt").contains(&i.model) && Vec3::from(i.pos).distance(Vec3::from(at)) < 0.01).map(|i| i.model.clone());
    let k21 = [8.0, 0.0, 4.0];
    let now = cell(&a, k21).unwrap();
    let other = family(&k, "grav/romfelt").into_iter().find(|m| *m != now).unwrap();
    let vid = k.piece("grav/romfelt").unwrap().variants.iter().find(|v| v.model == other).unwrap().id.clone();
    let mut locked = doc.clone();
    locked.cells.iter_mut().find(|c| c.id == "k21").unwrap().variant = Some(vid);
    let (b, _) = compose::compile(&locked, &k);
    assert_eq!(cell(&b, k21).as_deref(), Some(other.as_str()), "det låste utseendet står ikke");
    let changed = a.instances.iter().zip(&b.instances).filter(|(x, y)| x != y).count();
    assert_eq!((changed, a.instances.len()), (1, b.instances.len()), "låsingen endret mer enn én bit");
    // Et annet frø: andre utseender, men de samme bitene på de samme stedene.
    let mut reseeded = doc.clone();
    reseeded.seed = 7;
    let (c, _) = compose::compile(&reseeded, &k);
    assert_ne!(serde_json::to_string(&a.instances).unwrap(), serde_json::to_string(&c.instances).unwrap(), "frøet endret ingenting");
    let places = |f: &crate::domain::world::crypt::level::LevelFile| f.instances.iter().filter(|i| i.model != "grav_fakkel").map(|i| (i.pos.map(|v| (v * 100.0) as i64), (i.yaw * 100.0) as i64)).collect::<Vec<_>>();
    assert_eq!(places(&a), places(&c), "frøet flyttet biter");
    // Kolliderne følger formen til utseendet (nisjene), nettet ikke.
    assert_eq!(a.nav, c.nav);
}

/// Faklene: festene i murene fylles etter rommets fakkeltetthet, hver fakkel
/// står i et feste på en mur og lyser med et lys ved flammen i sitt rom, og
/// rommene uten tetthet får ingen.
#[test]
fn faklene_star_i_festene() {
    let k = kit();
    let doc = test_doc();
    let (f, _) = compose::compile(&doc, &k);
    let torches: Vec<_> = f.instances.iter().filter(|i| i.model == "grav_fakkel").collect();
    assert!(torches.len() >= 6, "bare {} fakler i prøvenivået", torches.len());
    let walls = family(&k, "grav/vegg");
    let mount = k.piece("grav/vegg").unwrap().mounts.iter().find(|m| m.kind == "fakkel").unwrap().pos;
    let rot = k.piece("grav/rotunde").unwrap();
    for t in &torches {
        let on_wall = f.instances.iter().filter(|i| walls.contains(&i.model)).any(|w| transform(Vec3::from(w.pos), w.yaw, false).transform_point3(Vec3::from(mount)).distance(Vec3::from(t.pos)) < 0.01)
            || rot.mounts.iter().any(|m| (Vec3::from(m.pos) + Vec3::new(0.0, 0.0, 16.0)).distance(Vec3::from(t.pos)) < 0.01);
        assert!(on_wall, "fakkelen ved {:?} står ikke i et feste", t.pos);
    }
    let lights: Vec<_> = f.lights.iter().filter(|l| l.flicker && l.range == 7.5).collect();
    assert_eq!(lights.len(), torches.len(), "hver fakkel skal ha ett lys");
    for l in &lights {
        let room = l.room.as_deref().unwrap();
        assert!(doc.rooms.iter().any(|r| r.id == room && r.mood.torches > 0.0), "fakkellyset ved {:?} står i {room}, som ikke skal ha fakler", l.pos);
    }
    assert!(!f.lights.iter().any(|l| l.room.as_deref() == Some("gjemme") && l.range == 7.5), "gjemmestedet har ingen fakkeltetthet");
}

/// Kvarthjørnene etter sidene cella har i hjørnet: innvendig, muren går
/// videre, pilaster under en gjordbue og utvendig (avfaset). Prøvenivået og
/// prøvebrettet har alle.
#[test]
fn hjornene_passer_til_murene() {
    let k = kit();
    // Prøvebrettet har bare par av celler (ingen L og ingen fire romfelt).
    for (doc, want) in [(test_doc(), &["grav_hjorne", "grav_pilaster", "grav_hjorne_ut", "grav_soyle"][..]), (board(&k), &["grav_hjorne", "grav_pilaster"][..])] {
        let (f, _) = compose::compile(&doc, &k);
        for m in want {
            assert!(f.instances.iter().any(|i| i.model == *m), "{}: ingen {m}", doc.name);
        }
    }
    // Kapellet: en pilaster der gjordbuen mellom k10 og k20 lander på
    // nordmuren (hjørnet i (6, −2)).
    let (f, _) = compose::compile(&test_doc(), &k);
    assert!(f.instances.iter().any(|i| i.model == "grav_pilaster" && Vec2::new(i.pos[0] - 6.0, i.pos[2] + 2.0).length() < 0.01));
}

/// Rotunden (en sammensatt bit på 3 × 3): én modell midt i, ingen murer
/// inne i den, nettet over hele det runde gulvet, og gangene ut møter
/// naboene (gangkrysset i øst og gravkammeret i sør).
#[test]
fn rotunden_settes_sammen() {
    let k = kit();
    let doc = test_doc();
    let (f, r) = compose::compile(&doc, &k);
    assert!(r.errors().next().is_none(), "{:?}", errors(&r));
    let rot: Vec<_> = f.instances.iter().filter(|i| i.model == "grav_rotunde").collect();
    assert_eq!(rot.len(), 1);
    assert_eq!(Vec3::from(rot[0].pos), Vec3::new(0.0, 0.0, 16.0));
    // Ingen halvkanter inne i rotunden (|x| og |z − 16| under 6, unntatt
    // kanten mot naboene).
    let edges: Vec<String> = k.pieces.iter().filter(|p| p.kind == PieceKind::Edge).flat_map(|p| std::iter::once(p.model.clone()).chain(p.variants.iter().map(|v| v.model.clone()))).collect();
    for i in f.instances.iter().filter(|i| edges.contains(&i.model)) {
        assert!(i.pos[0].abs() > 3.9 || (i.pos[2] - 16.0).abs() > 3.9, "halvkanten {} ved {:?} står inne i rotunden", i.model, i.pos);
    }
    // Nettet: polygoner i rommet nær midten og ut i gangene.
    let room = f.rooms.iter().find(|r| r.id == "beinhus").unwrap();
    let rp: Vec<_> = (0..f.nav.polys.len()).filter(|&p| f.nav.poly_room[p].as_deref() == Some("beinhus")).collect();
    assert!(rp.len() >= 4, "for få polygoner i rotunden");
    let reach = |x: f32, z: f32| {
        rp.iter().any(|&p| {
            let vs: Vec<[f32; 3]> = f.nav.polys[p].iter().map(|&v| f.nav.verts[v as usize]).collect();
            let (lo, hi) = vs.iter().fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), v| (lo.min(Vec2::new(v[0], v[2])), hi.max(Vec2::new(v[0], v[2]))));
            x >= lo.x && x <= hi.x && z >= lo.y && z <= hi.y
        })
    };
    // Midten, gangene ut og et stykke inn mot muren på skrå.
    for (x, z) in [(0.0, 16.0), (5.6, 16.0), (0.0, 21.6), (-5.6, 16.0), (0.0, 10.4), (3.0, 19.0), (-3.0, 13.0)] {
        assert!(reach(x, z), "nettet når ikke ({x}, {z})");
    }
    assert!(!reach(4.5, 20.5), "nettet går ut i fjellet mellom sirkelen og hjørnet");
    assert!(room.poly.len() >= 4);
}

