//! Delene av et kryptnivå som spillet bruker: bryterne og det de styrer
//! (porter, snarveier, falske vegger, fallemmer, broer), kistene, spakene,
//! fellene, hvilestedene, lappene, fiendene og hintene. Lest fra merkene i
//! nivåfila av `build_level` (`mod.rs`).

use glam::{Vec2, Vec3};

/// Noe som åpnes: en port, en snarveidør, en falsk vegg, en fallem.
#[derive(Clone, Debug)]
pub struct Switch {
    pub level: usize,
    pub id: String,
}

#[derive(Clone, Debug)]
pub struct Chest {
    pub pos: Vec3,
    pub yaw: f32,
    pub level: usize,
    /// En bestemt komponent (sjelden), ellers fra nivåets liste.
    pub loot: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Gate {
    pub pos: Vec3,
    pub yaw: f32,
    pub width: f32,
    pub level: usize,
    pub switch: usize,
    /// Åpnes med nøkkelen eliten bærer (ellers med en spak).
    pub key: bool,
    /// Porten til en bossarena (rommet i `arena`): står oppe, slår igjen bak
    /// deg når du går inn mens bossen lever, og går opp når den faller.
    /// `exit` (merket «exit»): porten ut, stengt så lenge bossen lever.
    pub arena: Option<usize>,
    pub exit: bool,
}

#[derive(Clone, Debug)]
pub struct Lever {
    pub pos: Vec3,
    pub yaw: f32,
    pub level: usize,
    pub opens: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Shortcut {
    pub pos: Vec3,
    pub yaw: f32,
    pub width: f32,
    pub level: usize,
    pub switch: usize,
    /// Retningen (flatt) fra døra mot sida der slåen sitter.
    pub latch: Vec3,
}

#[derive(Clone, Debug)]
pub struct Secret {
    /// Midt på foten av den falske delen, i flukt med muren rundt; fronten
    /// (+z) mot rommet du ser den fra.
    pub pos: Vec3,
    pub yaw: f32,
    pub width: f32,
    pub height: f32,
    pub level: usize,
    pub switch: usize,
    /// Modellen: nivåets egen bit av muren (`crypt_NN_secret_<id>`, med
    /// murens tekstur og sprekkene), ellers `KIT_SECRET`.
    pub model: &'static str,
    /// Steinen i muren den står i (`stone`).
    pub look: Option<crate::domain::model::Look>,
}

#[derive(Clone, Debug)]
pub struct Trapdoor {
    pub pos: Vec3,
    pub yaw: f32,
    /// Bredde og dybde.
    pub size: Vec2,
    pub level: usize,
    pub switch: usize,
    /// Rommet du havner i.
    pub to: Option<usize>,
    /// En lem som faller opp når du står på den (ellers et åpent hull).
    pub lid: bool,
    /// Boltet fra undersida: står fast når du tråkker på den, og åpnes
    /// bare når du klatrer opp stigen under den.
    pub bolted: bool,
}

/// Vann som står høyt til bryteren åpnes (sluseratten), og så synker
/// (`kit_water`, en flate på 1 × 1 m skalert til `size`).
#[derive(Clone, Debug)]
pub struct Water {
    /// Midt på flaten (x, z), høyden når vannet står høyt og når det er borte.
    pub pos: Vec3,
    pub yaw: f32,
    pub size: Vec2,
    pub high: f32,
    pub low: f32,
    pub level: usize,
    pub switch: usize,
}

impl Water {
    /// Vann som synker når slusa dreies (ikke sjøen i grottene, som står).
    pub fn sinks(&self) -> bool {
        self.high - self.low > 0.1
    }
}

/// En bro som senkes over en avgrunn når bryteren åpnes (begge spakene).
#[derive(Clone, Debug)]
pub struct Bridge {
    /// Midt på dekket når broa er nede, og retningen (lengden langs z).
    pub pos: Vec3,
    pub yaw: f32,
    /// Bredde og lengde.
    pub size: Vec2,
    /// Så høyt over `pos` den henger før den senkes.
    pub hang: f32,
    /// Høyden på kjettingfestene i taket.
    pub top: f32,
    pub level: usize,
    pub switch: usize,
}

/// Heisen i sjakta (`kit_lift` med kjettinger opp til heistårnet
/// `kit_winch`): står oppe med bremsen på til bryteren (spaken) åpnes, og
/// kjører så mellom toppen og bunnen når du bruker styrestangen på den.
#[derive(Clone, Debug)]
pub struct Lift {
    /// Midt på dekket når heisen står oppe, og retningen.
    pub pos: Vec3,
    pub yaw: f32,
    /// Bredde og dybde på dekket.
    pub size: Vec2,
    /// Så langt ned den går.
    pub drop: f32,
    pub level: usize,
    pub switch: usize,
}

#[derive(Clone, Debug)]
pub struct Ladder {
    pub pos: Vec3,
    pub yaw: f32,
    pub top: Vec3,
    pub level: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrapKind {
    Arrows,
    Spikes,
    Rocks,
    Pendulum,
    Gas,
    Flood,
}

#[derive(Clone, Debug)]
pub struct Trap {
    pub kind: TrapKind,
    /// Trykkplata (midten på gulvet) og målene.
    pub pos: Vec3,
    pub size: Vec2,
    /// Retningen pilene flyr (flatt).
    pub dir: Vec3,
    pub level: usize,
}

#[derive(Clone, Debug)]
pub struct Rest {
    pub pos: Vec3,
    pub yaw: f32,
    pub level: usize,
}

#[derive(Clone, Debug)]
pub struct Lore {
    pub pos: Vec3,
    pub yaw: f32,
    pub title: String,
    pub text: String,
    pub level: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BonusKind {
    Life,
    Stamina,
}

#[derive(Clone, Debug)]
pub struct Bonus {
    pub pos: Vec3,
    pub yaw: f32,
    pub kind: BonusKind,
    pub level: usize,
    /// Navnet i lagringen (nivå og nummer), så den bare tas én gang.
    pub key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ambush {
    /// Vekkes når du kommer så nær.
    Near(f32),
    /// Vekkes når du går inn i rommet.
    Room(usize),
}

/// Fiender som står et sted: slaget (`label`, se `mobs::Kind::from_label`),
/// hvor mange, ruta de går, og om de ligger i bakhold.
#[derive(Clone, Debug)]
pub struct Spawn {
    pub label: String,
    pub pos: Vec3,
    pub yaw: f32,
    pub n: u32,
    pub patrol: Vec<Vec3>,
    pub ambush: Option<Ambush>,
    pub elite: bool,
    pub boss: bool,
    pub level: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintKind {
    Draft,
    Sound,
    Crack,
}

#[derive(Clone, Debug)]
pub struct Hint {
    pub kind: HintKind,
    pub pos: Vec3,
    pub level: usize,
}

/// Det som ennå ikke gjør noe (heiser, vannstand): tatt vare på, så det
/// er klart når mekanismen kommer.
#[derive(Clone, Debug)]
pub struct Pending {
    pub kind: String,
    pub id: Option<String>,
    pub pos: Vec3,
    pub level: usize,
}

/// Pynten merket `prop` kan sette ut (modellnavnet i `label`): byggesettets,
/// og møblene fra byen som hører hjemme i en gravkjeller (alteret,
/// lysestakene, benkene, kistene og hyllene i de ferdige rommene, del 21),
/// med kolliderne fra `<navn>_inside.json` (`town::carry`).
pub const PROPS: &[&str] = &[
    "kit_sarcophagus", "kit_strut", "kit_rail", "kit_cart", "kit_lamp", "kit_crystal", "kit_banner", "kit_armor", "kit_statue", "kit_rack", "kit_throne", "kit_chandelier",
    "altar", "candle_stand", "candle_stand_low", "candle", "bench_16", "bench_20", "scroll_chest", "chest_forms", "cellar_shelf", "cellar_crate", "sacks", "barrel_tall",
];
/// Pynten som henger på muren eller i taket, eller står oppå noe (tronen på
/// forhøyningen), ikke på gulvet: settes ut med `place_mounted`.
pub const MOUNTED_PROPS: &[&str] = &["kit_lamp", "kit_banner", "kit_chandelier", "kit_throne", "candle"];

/// Et rom eller en gang, i verden.
#[derive(Clone, Debug)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub poly: Vec<Vec2>,
    /// Gulvet og taket over det (absolutt y og høyde).
    pub y: f32,
    pub h: f32,
    pub secret: bool,
    pub min: Vec2,
    pub max: Vec2,
    pub(super) area: f32,
    /// Så langt under gulvet rommet når i lyset (sjakta til trappa ned).
    pub sink: f32,
    /// Så langt over taket (`h`) rommet når i lyset: hvelvet.
    pub rise: f32,
    /// Stemningen: tåka (0–1) og lyden (`doc::SOUNDS`).
    pub fog: f32,
    pub sound: String,
}

impl Room {
    /// Om punktet (x, z) er inne i omrisset.
    pub fn contains(&self, q: Vec2) -> bool {
        if q.x < self.min.x || q.y < self.min.y || q.x > self.max.x || q.y > self.max.y {
            return false;
        }
        let mut inside = false;
        let n = self.poly.len();
        for i in 0..n {
            let (a, b) = (self.poly[i], self.poly[(i + n - 1) % n]);
            if (a.y > q.y) != (b.y > q.y) && q.x < (b.x - a.x) * (q.y - a.y) / (b.y - a.y) + a.x {
                inside = !inside;
            }
        }
        inside
    }
}
