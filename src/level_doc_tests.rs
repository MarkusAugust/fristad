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
    for opening in ["wall", "gang", "door", "gate", "secret", "arch"] {
        assert!(k.edge("grav", opening).is_some(), "ingen halvkant for «{opening}»");
    }
    assert!(k.of_kind("grav", PieceKind::Corner).is_some() && k.of_kind("grav", PieceKind::Pillar).is_some());
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
    assert_eq!(ra.recomputed, doc.cells.len());
    // Bare det endrede: samme kompilator husker cellene.
    let mut c = Compiler::default();
    c.compile(&doc, &k);
    let (same, r) = c.compile(&doc, &k);
    assert_eq!(r.recomputed, 0, "ingenting var endret, men {} celler ble regnet på nytt", r.recomputed);
    assert_eq!(same, a);
    // Tre endringer: døra blir port, en celle forsvinner, og en ny kommer.
    let mut doc2 = doc.clone();
    doc2.edges = vec![DocEdge { cell: "k22".into(), side: "S".into(), kind: "port".into(), ..Default::default() }];
    doc2.cells.retain(|c| c.id != "b16");
    doc2.cells.push(DocCell { id: "ny".into(), bit: "grav/romfelt".into(), at: [-1, 0, 1], room: "gjemme".into(), ..Default::default() });
    let (inc, r) = c.compile(&doc2, &k);
    let (full, _) = compose::compile(&doc2, &k);
    assert_eq!(serde_json::to_string(&inc).unwrap(), serde_json::to_string(&full).unwrap(), "bare det endrede ga noe annet enn alt på nytt");
    // Nabolaget er 5 × 5 celler rundt hver endring.
    assert!(r.recomputed > 0 && r.recomputed < doc2.cells.len(), "regnet {} av {} celler på nytt", r.recomputed, doc2.cells.len());
    let mut doc3 = doc2.clone();
    doc3.cells.iter_mut().find(|c| c.id == "b04").unwrap().room = "beinhus2".into();
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
    for piece in k.pieces.iter().filter(|p| p.kind == PieceKind::Cell) {
        for rot in 0..4u8 {
            for mirror in [false, true] {
                let doc = LevelDoc { cells: vec![DocCell { id: "c".into(), bit: piece.id.clone(), at: [0, 0, 0], rot, mirror, room: "r".into(), ..Default::default() }], ..Default::default() };
                let (f, r) = compose::compile(&doc, &k);
                assert!(check::tight(&doc, &k, &f).is_empty(), "{} rot {rot} speilet {mirror} er ikke tett: {:?}", piece.id, check::tight(&doc, &k, &f));
                assert!(r.errors().all(|i| i.rule != "tett" && i.rule != "bit"), "{}: {:?}", piece.id, errors(&r));
                for v in &f.nav.verts {
                    assert!(v[0].abs() <= HALF + 1e-3 && v[2].abs() <= HALF + 1e-3, "{} rot {rot}: nettet går ut av cella ved {v:?}", piece.id);
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
        assert_eq!(at("grav_vegg", edge), if expect_wall { 2 } else { 0 }, "{what}: murene");
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
    f.instances.retain(|i| i.model != "grav_vegg");
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
    let placed: usize = p.world.placements.iter().filter(|pl| pl.model.starts_with("grav_")).map(|pl| pl.transforms.len()).sum();
    let (f, _) = compose::compile(&test_doc(), &kit());
    assert_eq!(placed, f.instances.len(), "ikke alle bitene står i verden");
    assert!(p.world.missing.iter().all(|m| !m.starts_with("grav_")), "modeller mangler: {:?}", p.world.missing);
    eprintln!("prøvenivået: {} instanser, {} kollidere, {} polygoner, {} rom", f.instances.len(), f.colliders.len(), f.nav.polys.len(), f.rooms.len());
}
