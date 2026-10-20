//! Slagene av fiender (`Kind`) og tallene bak dem (`Stats`): modellen,
//! navnet, liv, fart, skade og rekkevidde. Oppførselen står i `mobs.rs`.

use crate::domain::mobs::{FPS, MOB_HP};

/// Alle fiendene er seigere enn tallene i tabellen (eieren 18. oktober:
/// «spillet er for lett»): 50 % mer liv for hvert slag, også eliten og
/// bossene, så forholdet mellom dem er det samme.
pub const TOUGHNESS: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Button,
    Tooltip,
    Input,
    /// Frosken i kloakken (Toast Frog): hopper og spytter.
    Toast,
    /// Eliten i kloakken (Select Broodmother).
    Select,
    Modal,
    /// Eliten i gravkjelleren (Datepicker Priest).
    Priest,
    Table,
    Framework,
    /// Gravkongens hall (nivå 2): ruller som et hjul og hopper opp og biter.
    Avatar,
    /// Gravkongens hall: en kjede av kuler som kryper etter deg.
    Breadcrumb,
    /// Gravkongens hall: ligger som en kiste og folder seg ut i bakhold.
    Accordion,
    /// Bossen i Ridderhallen (nivå 8): ridderkongen med åket og de fem
    /// skjoldene.
    Navbar,
    /// Bossen i Krystallgrotta (nivå 6): steinkjempen med krystallene.
    Dropdown,
    /// Eliten i Tvillingsalen (nivå 9): halvt stein, halvt plast.
    TwinWarden,
    /// Bueskytteren på brystvernene i nivå 7–8: skyter som inputen.
    Archer,
    /// Nivå 9–10: et tynt skjelett av grå staver som rykker fram.
    SkeletonLoader,
    /// Nivå 9: bygd av hule hvite celler.
    GridWarden,
    /// Nivå 5–6: steinhjulet på fire bein.
    Spinner,
    /// Eliten i Gruva (nivå 5) med hakka og glasstolpen i brystet.
    Foreman,
    /// Nivå 7–8: knekten med kroppen som et kort.
    CardSquire,
    /// Nivå 9: irrbloset som svever.
    Wisp,
}

/// Tallene bak et slag.
#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub hp: f32,
    pub speed: f32,
    pub damage: f32,
    /// Så nær må du være før den går til angrep, og så langt rekker slaget.
    pub range: f32,
    pub reach: f32,
    pub scale: f32,
    pub sight: f32,
    pub cooldown: f32,
    /// Sekundet i attack-klippet der slaget lander, og klippets lengde.
    pub attack_hit: f32,
    pub attack_len: f32,
    /// Skyter på avstand: (nærmeste, lengste avstand, skade).
    pub bolt: Option<(f32, f32, f32)>,
    /// Så mye av skaden slipper gjennom forfra når den ikke angriper.
    pub armor: f32,
    /// Slag under dette avbryter den ikke.
    pub poise: f32,
    pub tokens: u32,
}

impl Kind {
    pub const ALL: [Kind; 22] = [Kind::Button, Kind::Tooltip, Kind::Input, Kind::Toast, Kind::Select, Kind::Modal, Kind::Priest, Kind::Table, Kind::Framework, Kind::Avatar, Kind::Breadcrumb, Kind::Accordion, Kind::Navbar, Kind::Dropdown, Kind::TwinWarden, Kind::Archer, Kind::SkeletonLoader, Kind::GridWarden, Kind::Spinner, Kind::Foreman, Kind::CardSquire, Kind::Wisp];

    pub fn model(self) -> &'static str {
        match self {
            Kind::Button => "mutant_button",
            Kind::Tooltip => "mutant_tooltip",
            Kind::Input => "mutant_input",
            Kind::Toast => "mutant_toast",
            Kind::Select => "mutant_select",
            Kind::Modal => "mutant_modal",
            Kind::Priest => "datepicker_priest",
            Kind::Table => "mutant_table",
            Kind::Framework => "framework",
            Kind::Avatar => "avatar_skull",
            Kind::Breadcrumb => "breadcrumb_crawler",
            Kind::Accordion => "accordion_coffin",
            Kind::Navbar => "navbar",
            Kind::Dropdown => "dropdown",
            Kind::TwinWarden => "twin_warden",
            Kind::Archer => "breadcrumb_archer",
            Kind::SkeletonLoader => "skeleton_loader",
            Kind::GridWarden => "grid_warden",
            Kind::Spinner => "spinner",
            Kind::Foreman => "progress_foreman",
            Kind::CardSquire => "card_squire",
            Kind::Wisp => "popover_wisp",
        }
    }

    /// Navnet på slaget i et fiendemerke (`label`), det redigeringen
    /// foreslår: `from_label` gir slaget tilbake.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Button => "button",
            Kind::Tooltip => "tooltip",
            Kind::Input => "input",
            Kind::Toast => "toast",
            Kind::Select => "select",
            Kind::Modal => "modal",
            Kind::Priest => "datepicker",
            Kind::Table => "table",
            Kind::Framework => "rammeverket",
            Kind::Avatar => "avatar",
            Kind::Breadcrumb => "breadcrumb",
            Kind::Accordion => "accordion",
            Kind::Navbar => "navbar",
            Kind::Dropdown => "dropdown",
            Kind::TwinWarden => "tvillingvokter",
            Kind::Archer => "bueskytter",
            Kind::SkeletonLoader => "skeleton loader",
            Kind::GridWarden => "grid-vokter",
            Kind::Spinner => "spinner",
            Kind::Foreman => "progress-formann",
            Kind::CardSquire => "kortknekt",
            Kind::Wisp => "popover",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Button => "Korrupt button",
            Kind::Tooltip => "Korrupt tooltip",
            Kind::Input => "Korrupt input",
            Kind::Toast => "Korrupt toast",
            Kind::Select => "Select-moren",
            Kind::Modal => "Korrupt modal",
            Kind::Priest => "Datepicker-presten",
            Kind::Table => "Korrupt tabell",
            Kind::Framework => "Rammeverket",
            Kind::Avatar => "Korrupt avatar",
            Kind::Breadcrumb => "Korrupt breadcrumb",
            Kind::Accordion => "Korrupt accordion",
            Kind::Navbar => "Navbaren",
            Kind::Dropdown => "Dropdownen",
            Kind::TwinWarden => "Tvillingvokteren",
            Kind::Archer => "Breadcrumb-bueskytter",
            Kind::SkeletonLoader => "Skeleton Loader",
            Kind::GridWarden => "Grid-vokteren",
            Kind::Spinner => "Spinneren",
            Kind::Foreman => "Progress-formannen",
            Kind::CardSquire => "Kortknekten",
            Kind::Wisp => "Popover-irrbloss",
        }
    }

    /// Det infopanelet sier om slaget (kanon: de korrupte komponentene,
    /// fanget i krypten siden forseglingen), og hva du bør passe deg for.
    pub fn info(self) -> &'static str {
        match self {
            Kind::Button => "En knapp som ikke lenger bare kunne være en knapp. Kontorets regler er stemplet på den. Svak alene, farlig i flokk.",
            Kind::Tooltip => "Liten og rask, med en boble der hodet skulle vært. Hopper fram og biter før du rekker å se den, men tåler lite.",
            Kind::Input => "Lusker lavt med spydene fram og spytter på avstand. Gå tett på den.",
            Kind::Toast => "Frosken fra kloakken. Dukker opp når du minst venter det, hopper og spytter.",
            Kind::Select => "Moren i kloakken, med alle valgene sine i buken. Bærer nøkkelen og har tykt skall forfra.",
            Kind::Modal => "Stor og tung, og stenger veien til du er ferdig med den. Slår i bakken: blokk, eller gå til siden.",
            Kind::Priest => "Vokter alteret i gravkjelleren og teller dager i en kalender ingen andre kan lese. Bærer nøkkelen.",
            Kind::Table => "Kryper på alle fire med radene sine, tung som en eikekiste. Tåler mye; hugg der leddene sitter.",
            Kind::Framework => "Kongens rammeverk, levende, med seglet og krona. Gorvek delte det, og nå har det funnet delene sine igjen. Det kan ikke dø, men det kan deles.",
            Kind::Avatar => "En hodeskalle med et ansikt fanget i seg. Ruller som et hjul, hopper opp og biter.",
            Kind::Breadcrumb => "En kjede av kuler som kryper etter deg, ledd for ledd, og alltid vet veien tilbake.",
            Kind::Accordion => "Ligger stille som en kiste til du kommer nær. Så folder den seg ut, og lokket er kjeven.",
            Kind::Navbar => "Ridderkongen i tronsalen, med åket og de fem skjoldene. Lenkene er smidd av Rådets regler, ett ledd for hver regel. Den skulle vise alle veien, men bare den ene. Neven slår hardt: gå til siden.",
            Kind::Dropdown => "Steinkjempen i Krystallgrotta, med krystallene som gror ut av skuldrene. Valgene ligger lagvis på brystet, men den viser bare ett om gangen. Langsom og tung: slabbene treffer hardt.",
            Kind::TwinWarden => "Halvt stein og halvt hvit plast, sydd sammen med en glødende søm under kongens segl. Steinsida er tung, den hvite stiv og rykkete. Bærer nøkkelen.",
            Kind::Archer => "En smal kjede av steinledd med bue og kogger på ryggen. Står på murene og skyter. Gå tett på den.",
            Kind::SkeletonLoader => "Et skjelett av grå staver som later som noe er på vei. Rykker fram ledd for ledd, stopper opp, og kommer igjen.",
            Kind::GridWarden => "Bygd av hule hvite celler i rader og kolonner, stiv som en hylle. Hver celle har sin plass, og den går dit den skal.",
            Kind::Spinner => "Et steinhjul med jerntenner på fire korte bein. Det snurrer og snurrer, og hullet i navet er munnen.",
            Kind::Foreman => "Formannen i gruva, med hakka i neven og en glasstolpe i brystet som fylles og aldri blir full. Tung og bred. Bærer nøkkelen.",
            Kind::CardSquire => "En knekt av stein med kroppen som et spillekort. Plata tar imot slagene forfra: gå rundt ham.",
            Kind::Wisp => "Et lite hvitt irrbloss med ett øye, tre glødende kuber bak seg og en pil under. Svever, og stikker ned fra lufta.",
        }
    }

    /// Kroppen å sikte på, i modellens målestokk: (toppen, radius, halv
    /// lengde). Uten lengde er det en stående kapsel fra beina til toppen;
    /// med lengde ligger den langs ryggen i halve høyden (de lange).
    pub fn body(self) -> (f32, f32, f32) {
        match self {
            Kind::Input => (0.5, 0.32, 0.3),
            Kind::Table => (0.5, 0.36, 1.3),
            Kind::Toast => (0.55, 0.36, 0.0),
            Kind::Priest => (2.05, 0.45, 0.0),
            Kind::Select => (1.55, 0.6, 0.0),
            Kind::Avatar => (0.72, 0.34, 0.0),
            Kind::Breadcrumb => (0.55, 0.3, 0.55),
            Kind::Accordion => (1.2, 0.36, 0.0),
            // Rammeverket fra TRELLIS: 1,74 høy i modellen (4 m i spillet).
            Kind::Framework => (1.7, 0.5, 0.0),
            // Navbaren fra TRELLIS: 2,0 høy i modellen (5 m i spillet).
            Kind::Navbar => (1.95, 0.45, 0.0),
            // Dropdownen 2,0 høy i modellen (5 m), Tvillingvokteren 1,85
            // (2,4 m som elite), bueskytteren 1,6.
            Kind::Dropdown => (1.95, 0.5, 0.0),
            Kind::TwinWarden => (1.8, 0.42, 0.0),
            Kind::Archer => (1.55, 0.3, 0.0),
            // Høydene i modellen: 1,8, 2,2, 0,8 (hjulet), 1,69 (2,2 m som
            // elite), 1,5 og 0,9 (svever).
            Kind::SkeletonLoader => (1.75, 0.3, 0.0),
            Kind::GridWarden => (2.1, 0.4, 0.0),
            Kind::Spinner => (0.78, 0.4, 0.0),
            Kind::Foreman => (1.6, 0.45, 0.0),
            Kind::CardSquire => (1.45, 0.4, 0.0),
            Kind::Wisp => (0.95, 0.3, 0.0),
            _ => (1.45, 0.5, 0.0),
        }
    }

    pub fn boss(self) -> bool {
        self == Kind::Framework
    }

    /// Slaget som står i stedet når slagets egen modell mangler: om
    /// Rammeverkets modell (`framework`, fra `tools/models/mob3d.py`) ikke
    /// lastes, står modalen der, større (`Stats.scale`).
    pub fn stand_in(self) -> Option<Kind> {
        match self {
            Kind::Framework | Kind::Navbar | Kind::TwinWarden => Some(Kind::Modal),
            Kind::Dropdown => Some(Kind::Select),
            Kind::Archer | Kind::SkeletonLoader => Some(Kind::Breadcrumb),
            Kind::GridWarden => Some(Kind::Table),
            Kind::Spinner => Some(Kind::Avatar),
            Kind::Foreman => Some(Kind::Accordion),
            Kind::CardSquire => Some(Kind::Button),
            Kind::Wisp => Some(Kind::Tooltip),
            _ => None,
        }
    }

    pub fn stats(self) -> Stats {
        let bite = (15.0 / FPS, 30.0 / FPS);
        let slam = (28.0 / FPS, 44.0 / FPS);
        let fist = (22.0 / FPS, 36.0 / FPS);
        let s = match self {
            Kind::Button => Stats { hp: MOB_HP, speed: 1.4, damage: 12.0, range: 1.7, reach: 2.3, scale: 1.0, sight: 26.0, cooldown: 0.9, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Tooltip => Stats { hp: 38.0, speed: 3.6, damage: 6.0, range: 1.3, reach: 1.7, scale: 0.55, sight: 30.0, cooldown: 0.7, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 1 },
            Kind::Input => Stats { hp: 80.0, speed: 1.9, damage: 10.0, range: 1.8, reach: 2.2, scale: 1.15, sight: 34.0, cooldown: 1.6, attack_hit: bite.0, attack_len: bite.1, bolt: Some((3.5, 14.0, 14.0)), armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::Toast => Stats { hp: 45.0, speed: 2.4, damage: 8.0, range: 1.4, reach: 1.9, scale: 1.0, sight: 28.0, cooldown: 1.4, attack_hit: bite.0, attack_len: bite.1, bolt: Some((3.0, 12.0, 10.0)), armor: 1.0, poise: 0.0, tokens: 2 },
            Kind::Select => Stats { hp: 150.0, speed: 1.6, damage: 18.0, range: 2.0, reach: 2.6, scale: 1.0, sight: 26.0, cooldown: 1.4, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 0.7, poise: 20.0, tokens: 6 },
            Kind::Modal => Stats { hp: 320.0, speed: 1.3, damage: 30.0, range: 2.2, reach: 3.0, scale: 1.5, sight: 24.0, cooldown: 1.8, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.3, poise: 40.0, tokens: 6 },
            Kind::Priest => Stats { hp: 160.0, speed: 1.2, damage: 22.0, range: 2.0, reach: 2.7, scale: 0.8, sight: 28.0, cooldown: 1.5, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.6, poise: 25.0, tokens: 6 },
            Kind::Table => Stats { hp: 420.0, speed: 2.4, damage: 22.0, range: 2.2, reach: 3.0, scale: 1.4, sight: 30.0, cooldown: 1.1, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 30.0, tokens: 10 },
            // Rullingen er 2πR (1,9 m) på 18 bilder: 2,6 m/s.
            Kind::Avatar => Stats { hp: 40.0, speed: 2.6, damage: 9.0, range: 1.3, reach: 1.8, scale: 1.0, sight: 28.0, cooldown: 1.0, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 2 },
            Kind::Breadcrumb => Stats { hp: 55.0, speed: 1.9, damage: 8.0, range: 1.5, reach: 2.0, scale: 1.0, sight: 30.0, cooldown: 1.0, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 2 },
            Kind::Accordion => Stats { hp: 90.0, speed: 1.3, damage: 14.0, range: 1.6, reach: 2.2, scale: 1.0, sight: 18.0, cooldown: 1.4, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 0.8, poise: 10.0, tokens: 3 },
            // Mellom Tabellen (nivå 4) og Rammeverket (nivå 10): platen tar
            // av forfra, og neveslaget (`attack_fist`) lander i bilde 22.
            Kind::Navbar => Stats { hp: 800.0, speed: 1.5, damage: 30.0, range: 2.8, reach: 3.8, scale: 2.5, sight: 36.0, cooldown: 1.3, attack_hit: fist.0, attack_len: fist.1, bolt: None, armor: 0.6, poise: 50.0, tokens: 24 },
            // Mellom Tabellen (nivå 4) og Navbaren (nivå 8), langsom og tung.
            Kind::Dropdown => Stats { hp: 620.0, speed: 1.2, damage: 28.0, range: 2.6, reach: 3.6, scale: 2.5, sight: 32.0, cooldown: 1.5, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.7, poise: 45.0, tokens: 18 },
            // Eliten i nivå 9 (`ELITE_HP` og `ELITE_SCALE` kommer i tillegg).
            Kind::TwinWarden => Stats { hp: 170.0, speed: 1.4, damage: 22.0, range: 2.0, reach: 2.7, scale: 1.0, sight: 30.0, cooldown: 1.4, attack_hit: fist.0, attack_len: fist.1, bolt: None, armor: 0.6, poise: 30.0, tokens: 8 },
            // Skyter som inputen; pila går i bilde 15 av skuddet (`attack_shoot`).
            Kind::Archer => Stats { hp: 50.0, speed: 2.0, damage: 9.0, range: 1.6, reach: 2.1, scale: 1.0, sight: 34.0, cooldown: 1.6, attack_hit: bite.0, attack_len: bite.1, bolt: Some((4.0, 16.0, 12.0)), armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::SkeletonLoader => Stats { hp: 60.0, speed: 1.8, damage: 11.0, range: 1.6, reach: 2.2, scale: 1.0, sight: 30.0, cooldown: 1.1, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 3 },
            Kind::GridWarden => Stats { hp: 140.0, speed: 1.4, damage: 16.0, range: 1.9, reach: 2.6, scale: 1.0, sight: 28.0, cooldown: 1.3, attack_hit: fist.0, attack_len: fist.1, bolt: None, armor: 0.8, poise: 20.0, tokens: 5 },
            // Hjulet (`mob_rigid.spinner`): bittet med navet i bilde 15.
            Kind::Spinner => Stats { hp: 45.0, speed: 2.6, damage: 9.0, range: 1.3, reach: 1.8, scale: 1.0, sight: 28.0, cooldown: 1.0, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 0.8, poise: 0.0, tokens: 2 },
            // Eliten i nivå 5 (`ELITE_HP` og `ELITE_SCALE` i tillegg).
            Kind::Foreman => Stats { hp: 180.0, speed: 1.4, damage: 26.0, range: 2.1, reach: 2.8, scale: 1.0, sight: 28.0, cooldown: 1.6, attack_hit: slam.0, attack_len: slam.1, bolt: None, armor: 0.6, poise: 35.0, tokens: 8 },
            // Plata tar av forfra.
            Kind::CardSquire => Stats { hp: 70.0, speed: 1.7, damage: 13.0, range: 1.7, reach: 2.3, scale: 1.0, sight: 28.0, cooldown: 1.2, attack_hit: fist.0, attack_len: fist.1, bolt: None, armor: 0.5, poise: 10.0, tokens: 3 },
            // Svever (`mob_rigid.wisp`): stikket i bilde 15.
            Kind::Wisp => Stats { hp: 35.0, speed: 2.8, damage: 8.0, range: 1.4, reach: 1.9, scale: 1.0, sight: 32.0, cooldown: 1.0, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 0.0, tokens: 2 },
            Kind::Framework => Stats { hp: 1400.0, speed: 1.6, damage: 34.0, range: 3.0, reach: 4.0, scale: 2.3, sight: 40.0, cooldown: 1.2, attack_hit: bite.0, attack_len: bite.1, bolt: None, armor: 1.0, poise: 55.0, tokens: 40 },
        };
        Stats { hp: s.hp * TOUGHNESS, ..s }
    }

    /// Slaget etter navnet i nivåfila (`label` på merket): norsk eller
    /// engelsk, og «knapp i bakhold» er en knapp. Ukjente blir knapper.
    pub fn from_label(label: &str) -> Kind {
        let l = label.to_lowercase();
        let has = |w: &[&str]| w.iter().any(|x| l.contains(x));
        if has(&["rammeverk", "framework"]) {
            Kind::Framework
        } else if has(&["navbar"]) {
            Kind::Navbar
        } else if has(&["dropdown"]) {
            Kind::Dropdown
        } else if has(&["tvilling", "twin"]) {
            Kind::TwinWarden
        } else if has(&["bueskytter", "archer"]) {
            Kind::Archer
        } else if has(&["skeleton"]) {
            Kind::SkeletonLoader
        } else if has(&["grid"]) {
            Kind::GridWarden
        } else if has(&["spinner"]) {
            Kind::Spinner
        } else if has(&["progress", "formann"]) {
            Kind::Foreman
        } else if has(&["kortknekt", "card"]) {
            Kind::CardSquire
        } else if has(&["popover", "irrbloss"]) {
            Kind::Wisp
        } else if has(&["tabell", "table"]) {
            Kind::Table
        } else if has(&["datepicker", "prest", "priest"]) {
            Kind::Priest
        } else if has(&["modal"]) {
            Kind::Modal
        } else if has(&["select", "brood"]) {
            Kind::Select
        } else if has(&["avatar"]) {
            Kind::Avatar
        } else if has(&["breadcrumb", "brødsmule", "smule"]) {
            Kind::Breadcrumb
        } else if has(&["accordion", "trekkspill", "coffin"]) {
            Kind::Accordion
        } else if has(&["toast", "frosk"]) {
            Kind::Toast
        } else if has(&["input"]) {
            Kind::Input
        } else if has(&["tooltip"]) {
            Kind::Tooltip
        } else {
            if !has(&["knapp", "button"]) {
                log::warn!("Ukjent fiende «{label}»: blir en knapp");
            }
            Kind::Button
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Navnet redigeringen foreslår for hvert slag (`Kind::key`) gir det
    /// samme slaget tilbake, og hvert slag har sin egen modell.
    #[test]
    fn hvert_slag_har_et_navn_redigeringen_kan_bruke() {
        for k in Kind::ALL {
            assert_eq!(Kind::from_label(k.key()), k, "«{}»", k.key());
            assert_eq!(Kind::ALL.iter().filter(|o| o.model() == k.model()).count(), 1, "{}", k.model());
        }
    }
}
