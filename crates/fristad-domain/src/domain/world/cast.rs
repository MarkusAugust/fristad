//! Hva innbyggerne sier. Replikkene er innhold, ikke plassering: byen som
//! data (`layout`) peker til dem med en nøkkel (`LINES`). Hver innbygger har
//! sine egne replikker etter rollen sin (eieren 18. oktober), og noen kommer
//! først når du har nådd et visst nivå i krypten (`Line.from`), eller bare
//! før du har vært nede (`Line.until`). Alt bygger på `docs/kanon.md`.

/// En replikk og når den kan sies: når du har nådd minst `from` nivåer i
/// krypten og færre enn `until` (0 er aldri vært nede).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub text: &'static str,
    pub from: u8,
    pub until: u8,
}

impl Line {
    /// Om replikken passer når du har nådd `depth` nivåer.
    pub fn fits(&self, depth: u8) -> bool {
        self.from <= depth && depth < self.until
    }
}

/// Alltid.
const fn a(text: &'static str) -> Line {
    Line { text, from: 0, until: u8::MAX }
}

/// Når du har nådd nivå `n` (eller dypere).
const fn at(n: u8, text: &'static str) -> Line {
    Line { text, from: n, until: u8::MAX }
}

/// Bare før du har vært nede i krypten.
const fn new(text: &'static str) -> Line {
    Line { text, from: 0, until: 1 }
}

pub const SMITH: &[Line] = &[
    a("Buttonsmeden, til tjeneste. Jeg smir det du bærer opp fra krypten."),
    a("Et sverd trenger en klinge og et grep: en Divider og en Button. Resten er håndlag."),
    a("Komponentene der nede er de ekte, de fra mønsterboka. Bare vridd. Bær dem hit, så retter vi dem ut."),
    a("Kontorets regler sitter som slagg i dem. Slå slagget løs, og det som blir igjen er rent."),
    a("Far min sa at en Button skulle være en Button. Ikke en Button som også fører regnskap for Kontoret."),
    a("Ambolten er gammel, men den har aldri spurt hvilket rammeverk jeg hamrer etter."),
    new("Du har ikke vært nede ennå, ser jeg. Hendene dine er for rene."),
    at(1, "Gravkjelleren først? Kom tilbake med det presten bar, så ser vi hva stålet tåler."),
    at(3, "Kloakken lukter jeg herfra. Vask det du bærer i brønnen før du legger det på ambolten min."),
    at(5, "Malm fra gruva? Nei, det er tokens. Men de klinger som godt jern."),
    at(8, "Du har vært i Ridderhallen. Jeg ser det på skjoldet. Ingen kommer derfra uten bulker."),
];

pub const CLERK: &[Line] = &[
    a("Skriptoriet. Her skrev mester Ansgar en gang. Nå smir vi med det han skrev."),
    a("Oppskriftene er skjemaer. Fyll ut feltene med komponentene du har vunnet, og send."),
    a("Bøkene her er mønsterboka, side for side. Les om det du bærer, så leges sårene."),
    a("På hoffets tunge måtte hver side godkjennes av Rådet. Her skriver vi det vi trenger."),
    a("Omnia una compage, sto det på budet over døra. Vi skrapte det av med en kniv."),
    a("Ansgar gikk ned med de siste sidene. Finner du noe han skrev der nede, ta det med deg i hodet."),
    new("Krypten under kirka har ti trapper, står det i kirkeboka. Ingen har skrevet ned hva som står nederst."),
    at(2, "Fant du noe skrevet der nede? Munkene skrev på norsk; seglene er på kanselliets latin."),
    at(4, "Cisternen. Ansgar skrev om den: vann som husker hver regel det har rent forbi."),
    at(7, "Den sunkne borgen sto en gang over jorda, sier de gamle skriftene. Noe dro den ned."),
    at(9, "Du har sett sømmene. Ansgar kalte dem lappene, omvei på omvei. Ingen skrev seg ut av dem."),
];

pub const APOTHECARY: &[Line] = &[
    a("Apoteket. Drikker mot det meste, og olje til lykta. Uten lys er krypten en grav."),
    a("Les i bøkene når du er skadet. Kunnskap leger."),
    a("Lykta brenner bare nede. Fyll den her før du går, ikke når du står i mørket."),
    a("Hvilestedene der nede fyller lykta. Gå ikke forbi et alter uten å tenne det."),
    a("Urtene på loftet tørker best i røyken fra smia. Spør ikke hvordan jeg vet det."),
    new("Du ser frisk ut. Det varer ikke. Kjøp olje."),
    at(1, "Svidd i ansiktet og øyne som er vant til mørket. Du har vært nede."),
    at(3, "Kloakken? Vask hendene. Det Toastene der nede spytter, rår ingen urt med."),
    at(6, "Krystallene i grotta lyser av seg selv, sier du? Ta med en bit. Jeg betaler i olje."),
];

pub const GORVEK: &[Line] = &[
    a("Torgils. Jeg lærer deg trekk, om du har tokens og tålmodighet."),
    a("Blokk i det bittet kommer. Da står de lamslått, og du hugger dobbelt."),
    a("Unna med Shift. Det koster pust, men sparer blod."),
    a("Parér i siste øyeblikk, ikke før. Den som er for tidlig, får bittet i armen."),
    a("Det tunge hugget krever pust. Lad det, og la det falle når de vakler."),
    a("Dukkene slår ikke tilbake. Det gjør det som venter under kirka."),
    a("Gorvek hogg det i stykker med én øks. Du har flere trekk enn han hadde. Bruk dem."),
    at(2, "Modalen i Gravkongens hall stenger døra bak deg. Gå ikke inn uten pust og olje."),
    at(4, "Tabellen kryper på alle fire. Hugg beina, ett etter ett."),
    at(6, "En Select skifter ansikt når du tror du har den. Vent til den har valgt, og slå da."),
    at(8, "Navbaren holder hele hallen i ett grep. Kom deg bak den."),
];

pub const INNKEEPER: &[Line] = &[
    a("Velkommen til Vertshuset. Øl er det nok av, nyheter også."),
    a("De sier krypten under kirka går dypere enn noen har gått. Ti trapper, sier klokkeren."),
    a("Det kommer rare ting opp fra dypet om natta. Buttons med tenner."),
    a("Klokkeren så lys under kryptdøra i vinter. Ingen hadde tent det."),
    a("Pilegrimen betaler i bønner. Vaktmannen betaler ikke i det hele tatt."),
    new("En fremmed uten fortid? Velkommen. Her spør vi ikke hvor folk kommer fra."),
    at(1, "Du har støv fra gravkjelleren i håret. Sett deg. Første krus er på huset."),
    at(3, "Folk sier du har gått under bekkene. Hold deg ved døra med de støvlene der."),
    at(6, "Halve byen snakker om deg nå. Den andre halvparten tror ikke på et ord."),
    at(10, "Kjernen? Så dypt har ingen gått. Ikke engang i visene."),
];

pub const GUARD: &[Line] = &[
    a("Sørporten er min vakt. Det som kommer inn her, ser jeg først."),
    a("Hold deg unna kirkegården etter mørkets frembrudd."),
    a("Portene stenges ikke, men jeg holder øye med hvem som går ut."),
    a("Nordporten tar deg til kirkegården. Døra til krypten står mot veien."),
    a("Inkvisitorene kom inn gjennom denne porten en gang. Røde kapper. Farfar åpnet for dem."),
    new("En fremmed. Gå rett fram til torget. Krypten er ut nordporten, men derfra kommer få opp igjen."),
    at(1, "Den steinringen som stiger opp av bakken bak meg når du kommer hjem. Jeg har sett mye, men aldri det."),
    at(3, "Ser du noe som kryper opp fra kloakken, rop. Jeg har bare ett spyd."),
    at(5, "Du kommer opp med kullstøv på hendene. Gruva? Under kirka? Jeg vil ikke vite det."),
];

/// Trubaduren på krakken ved ildstedet i Vertshuset: synger sagaen om
/// Gorvek (`assets/music/game/tavern.mp3`).
pub const TROUBADOUR: &[Line] = &[
    a("Sett deg ved ilden, fremmede. Visa om Gorvek er lang, og ølet er kort."),
    a("Gorvek ba aldri og spurte aldri. Han tok livet i strupen. Derfor synger vi om ham."),
    a("Jeg kan alle versene om forseglingen. Mester Ansgar får det siste, men ingen vet hvordan det slutter."),
    a("Kvadet om Gorvek koster et krus mjød. En tørr strupe synger ikke."),
    new("Et nytt ansikt ved porten. Det blir kanskje et vers av deg også."),
    at(1, "Du kommer nedenfra. Fortell hva du så, så setter jeg det i visa."),
    at(5, "Gruva, sier du? Det rimer ikke på noe. Jeg får finne på noe."),
    at(10, "Kjernen. Det finnes ikke noe vers om den ennå. Kanskje det blir ditt."),
];

/// Trubaduren og mjøden (E, `application/tavern.rs`): det han sier når du
/// står ved krakken, når du har kjøpt et krus, om du vil kjøpe mens han
/// synger, og når kvadet er sunget og strupen er tørr.
pub const MEAD_OFFER: &str = "Et krus mjød, fremmede, så synger jeg kvadet om Gorvek: han som kom gjennom regnet med øksa over skulderen og hogg beistet der leddene satt, natta gjennom. Men strupen er tørr, og kvadet er langt.";
pub const MEAD_SINGING: &str = "Han er midt i kvadet, og kruset ditt står ennå ved krakken.";
pub const MEAD_THANKS: &str = "Skål, fremmede! Så hør da om Gorvek og regnet.";
pub const MEAD_AGAIN: &str = "Takk, men kruset står ennå fullt. Hør heller etter.";
pub const MEAD_DRY: &str = "Der er Gorvek borte i regnet igjen. Strupen er tørr som pergament. Et krus til, så synger jeg igjen.";

pub const PILGRIM: &[Line] = &[
    a("Jeg har gått fra Nidaros for å se mønsterboka. De sier den aldri sto i veien for noen."),
    a("Fristil. Navnet alene gjør meg rolig."),
    a("Byene på veien hit skriver alle på hoffets tunge. Bare her husker noen den gamle boka."),
    a("Ved hver kirke på veien sto budet slått opp: Nulla exceptio. Her har noen revet det ned."),
    a("I hovedstaden sier de at Gorvek aldri fantes. Her sier de at han hogg beistet i stykker. Jeg tror bøndene."),
    a("Jeg ber ved kryptdøra hver kveld. Det er noe der nede som ber tilbake."),
    at(2, "Du har gått ned. Fant du mester Ansgar? Nei. Ingen finner ham, sier de."),
    at(9, "Overgangen, sier du. Der boka slutter og rammeverket begynner. Be for deg selv."),
];

pub const FARMER: &[Line] = &[
    a("Kom inn med kjerra før det lysnet. Tollen ved porten er den samme, men kålen er mindre."),
    a("Kuene vil ikke beite nær kirkegården lenger. De står og stirrer mot krypten."),
    a("Bestefar sa at Gorvek hogg beistet i stykker over byen, natta gjennom i regnet. Og at ingen så ham siden."),
    a("Kontorets menn ville ha kålen min skrevet inn på hoffets tunge. Jeg kan ikke skrive på noen tunge."),
    a("Torgdag i dag. Alt er dyrere enn i går."),
    at(1, "Er det sant at du går ned dit frivillig? Ta med deg et kålhode. For sikkerhets skyld."),
    at(5, "Det romler under åkeren min om natta. Som om noen graver seg oppover."),
];

pub const MAID: &[Line] = &[
    a("God dag. Vannet i brønnen er kaldt i dag, kaldt som krypten."),
    a("Vertinnen sender meg etter vann tjue ganger om dagen. Brønnen og jeg er gamle venner."),
    a("Kirkeklokka ringte feil i natt. Tretten slag."),
    a("Gjestene på loftet snorker på hoffets tunge. Det høres ut som latin med forkjølelse."),
    new("Skal du ned i krypten? Ta med deg en ekstra lykt. Eller to."),
    at(2, "Du kom tilbake! Vertinnen skyldte meg et krus om du gjorde det."),
    at(7, "Er det sant at det står en hel borg der nede? Under oss? Jeg sover dårlig nok fra før."),
];

pub const MERCHANT: &[Line] = &[
    a("Ull fra gårdene, salt fra kysten. Kontorets pris, men ikke Kontorets regler."),
    a("Kjøpmennene i hovedstaden ville ha alle varer i samme kasse. Jeg selger fra mine egne."),
    a("Har du tokens? Da er du velkommen hos Buttonsmeden. Jeg tar bare sølv."),
    a("Alt er dyrere enn i går. Det sier jeg hver dag, og hver dag er det sant."),
    at(1, "Det du bærer opp fra krypten, kan ikke selges på torget. Men folk vil høre om det, og det er også en vare."),
    at(4, "En Table med åtte bein, sier du? Jeg har solgt dårligere bord."),
];

pub const FISHWIFE: &[Line] = &[
    a("Fersk sild! I går var den ferskere, men i dag er den billigere."),
    a("Mannen min ror ut før lyset. Han sier fisken går dypere hvert år, som om noe trekker den ned."),
    a("Har du vært nede i krypten? Bror min gikk ned i fjor. Han snakker bare om Buttons nå."),
    a("Kloakken renner ut i bekken nedenfor byen. Der er det ingen fisk lenger."),
    at(3, "Du har vært i kloakken. Ikke kom nær silda mi."),
    at(6, "Selg meg en av de lysende krystallene, så får du sild hele vinteren."),
];

pub const BOY: &[Line] = &[
    a("Jeg skal bli våpenmester som Torgils! Han lar meg holde skjoldet når ingen ser."),
    a("Har du sett en Tooltip? Bror min sier de svever over hodet på deg og hvisker."),
    a("Mor sier Gorvek bare er en vise. Men hvorfor synger alle den da?"),
    a("Jeg kastet en stein i brønnen og telte til fem før den plasket."),
    new("Tør du gå ned i krypten? Jeg tør. Men ikke i dag."),
    at(1, "Har du drept en Button? Hvordan lød den? Klikket den?"),
    at(2, "Er Modalen virkelig så stor at den stenger hele rommet? Fortell!"),
    at(10, "Du har sett Rammeverket! Var det høyere enn kirketårnet? Mor sier det er umulig."),
];

pub const WASHERWOMAN: &[Line] = &[
    a("Lin og ull, lin og ull. Hendene mine har ikke vært tørre siden jeg var tolv."),
    a("Kappa til Vaktmannen var rød en gang, som inkvisitorenes. Den er bleket nå. Det sørget jeg for."),
    a("Krypten er på kirkegården, ut nordporten. Gå ikke ned uten olje på lykta."),
    a("Munkene vasket sine egne kutter. Det var den eneste regelen de holdt."),
    at(1, "Gi meg kappa di når du kommer opp. Den lukter grav."),
    at(3, "Kloakkvann? Kom ikke hit med det. Brønnen er til drikke, bekken er til sånt."),
    at(5, "Kullstøv og blod. Det går ut med lut, men ikke med én vask."),
];

/// Alle replikksettene med nøkkelen byen som data bruker.
pub const LINES: &[(&str, &[Line])] = &[
    ("smith", SMITH),
    ("clerk", CLERK),
    ("apothecary", APOTHECARY),
    ("gorvek", GORVEK),
    ("innkeeper", INNKEEPER),
    ("troubadour", TROUBADOUR),
    ("guard", GUARD),
    ("pilgrim", PILGRIM),
    ("farmer", FARMER),
    ("maid", MAID),
    ("merchant", MERCHANT),
    ("fishwife", FISHWIFE),
    ("boy", BOY),
    ("washerwoman", WASHERWOMAN),
];

/// Replikkene med nøkkelen `key` (tomt om den ikke finnes).
pub fn lines(key: &str) -> &'static [Line] {
    LINES.iter().find(|(k, _)| *k == key).map(|(_, l)| *l).unwrap_or(&[])
}
