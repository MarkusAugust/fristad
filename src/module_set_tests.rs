//! Ett sett moduler (eieren 20. oktober: «det holder med en modell, og så
//! må jeg kunne kle flatene i den teksturen jeg vil ha i editoren»,
//! `docs/nivaaredigering.md` del 27): hver modul i hver stein har ett gulv
//! per rom, og faklene står under hvelvet.

use crate::adapters::data::kit;
use crate::domain::world::crypt::compose::{self, check, modules};
use crate::domain::world::crypt::kit::PieceKind;

/// Aldri blandet gulv i ett rom (eieren 20. oktober: «ikke bland ulike
/// teksturer på gulvet»): hver modul i hvert tema (steinen) har ett gulvlag
/// per rom, og gravhellene (`own_floor`) står bare på gravkjellerens gulv.
#[test]
fn hvert_rom_har_ett_gulv_i_hver_stein() {
    let k = kit();
    let slabs: Vec<&str> = k.listed().flat_map(|p| p.variants.iter()).filter(|v| v.own_floor).map(|v| v.model.as_str()).collect();
    assert!(!slabs.is_empty(), "variantene med gravheller er merket (`own_floor` i manifestet)");
    let mut slabs_in_grav = 0;
    for m in &k.modules {
        let name = m.id.split_once('/').unwrap().1;
        for t in k.stones.themes.keys() {
            let id = format!("{t}/{name}");
            let doc = modules::showcase(&k, &id).unwrap();
            assert_eq!(doc.theme, if t == "grav" { "" } else { t.as_str() }, "{id}");
            let (f, _) = compose::compile(&doc, &k);
            let mix = check::floor_mix(&f, &k);
            assert!(mix.is_empty(), "{id}: {:?}", mix.iter().map(|i| &i.msg).collect::<Vec<_>>());
            if t == "grav" {
                slabs_in_grav += f.instances.iter().filter(|i| slabs.contains(&i.model.as_str())).count();
            } else if m.cells.iter().all(|c| c.stone.is_none()) && m.rooms.iter().all(|r| r.stone.is_none()) {
                assert!(!f.instances.iter().any(|i| slabs.contains(&i.model.as_str())), "{id}: gravheller på et annet gulv");
            }
        }
    }
    assert!(slabs_in_grav > 0, "gravkjelleren har fortsatt gravhellene sine");
    // Og der eieren ser dem: modulene med naborommene i galleriet har samme
    // gulv hele veien (riddergangen i kvader gikk over i gravhellene i
    // naborommene, eieren 20. oktober: «den skal være samme hele veien»).
    for page in modules::gallery_pages(&k) {
        let (f, _) = compose::compile(&page, &k);
        let mix = check::floor_mix(&f, &k);
        assert!(mix.is_empty(), "{:?}: {:?}", page.level, mix.iter().map(|i| &i.msg).collect::<Vec<_>>());
    }
    // Gulvet som skifter i en åpen gang mellom to rom, er en advarsel.
    let mut doc = modules::gallery(&k);
    doc.modules.retain(|m| m.id.starts_with("riddergang"));
    assert_eq!(doc.modules.len(), 3, "riddergangen med to naborom");
    doc.modules.iter_mut().filter(|m| m.id.contains("_nabo")).for_each(|m| m.stone = None);
    let (f, _) = compose::compile(&doc, &k);
    assert!(check::floor_mix(&f, &k).iter().any(|i| i.msg.contains("skifter i åpningen")), "{:?}", check::floor_mix(&f, &k));
    // To steiner på gulvet i samme rom er en advarsel.
    let mut doc = modules::showcase(&k, "grav/rom_1x2").unwrap();
    doc.modules[0].stone = None;
    let mut flat = modules::flatten(&doc, &k).0;
    flat.cells[0].stone = Some(crate::domain::world::crypt::stone::StoneRef::Named("tegl".into()));
    let (f, _) = compose::compile(&flat, &k);
    assert!(check::floor_mix(&f, &k).iter().any(|i| i.msg.contains("to gulv")), "{:?}", check::floor_mix(&f, &k));
}

/// Faklene på sidemurene under et tønnehvelv (gangene og trappene) står så
/// lavt at flammen aldri når hvelvet (eieren 20. oktober om blindgangen i
/// borgen: «noen fakler går opp i taket, de på sideveggene»). Regnet ut for
/// seg: hvelvet over flammen etter gangprofilen (`gang_h`), der den står.
#[test]
fn faklene_star_under_hvelvet() {
    let k = kit();
    let barrels = ["grav/gang", "grav/gang_blind", "grav/halvtrapp", "grav/trapp2"];
    let mut side = 0;
    for m in &k.modules {
        let doc = modules::showcase(&k, &m.id).unwrap();
        let (f, _) = compose::compile(&doc, &k);
        let cells = compose::expand(&modules::flatten(&doc, &k).0, &k);
        for t in f.instances.iter().filter(|i| i.model == "grav_fakkel") {
            let p = t.pos;
            let (i, j) = ((p[0] / compose::CELL).round() as i32, (p[2] / compose::CELL).round() as i32);
            let Some(c) = cells.iter().find(|c| c.at[0] == i && c.at[2] == j && p[1] >= c.at[1] as f32 - 0.01 && p[1] < c.at[1] as f32 + 5.5) else { continue };
            if !barrels.contains(&c.bit.as_str()) || k.piece(&c.bit).is_none_or(|p| p.kind != PieceKind::Cell) {
                continue;
            }
            let lateral = if c.rot % 2 == 0 { (p[0] - i as f32 * compose::CELL).abs() } else { (p[2] - j as f32 * compose::CELL).abs() };
            if lateral < 1.0 {
                continue;
            }
            side += 1;
            let flame = lateral - 0.275;
            let top = p[1] + compose::TORCH_FLAME_Y + compose::TORCH_TIP;
            let vault = c.at[1] as f32 + compose::gang_h(flame);
            assert!(top <= vault - 0.1, "{}: fakkelen i {} når {top:.2} m, hvelvet er {vault:.2} m", m.id, c.id);
        }
    }
    assert!(side > 0, "ingen fakler på sidemurene i gangene (testen ser ingenting)");
    // Ingen fakkel midt i stigen (eieren 20. oktober, stigen i gruva).
    for t in k.stones.themes.keys() {
        let id = format!("{t}/stige");
        let (f, _) = compose::compile(&modules::showcase(&k, &id).unwrap(), &k);
        let ladder = f.marks.iter().find(|m| m.kind == "ladder").unwrap_or_else(|| panic!("{id}: ingen stige"));
        let (a, b) = (ladder.pos.unwrap(), ladder.top.unwrap());
        for i in f.instances.iter().filter(|i| i.model == "grav_fakkel") {
            let d = |p: [f32; 3]| ((i.pos[0] - p[0]).powi(2) + (i.pos[2] - p[2]).powi(2)).sqrt();
            let between = i.pos[1] > a[1].min(b[1]) - 0.5 && i.pos[1] < a[1].max(b[1]) + 0.5;
            assert!(!(between && d(a).min(d(b)) < 1.0), "{id}: fakkelen ved {:?} står i stigen", i.pos);
        }
    }
}
