//! Hva innbyggerne sier. Replikkene er innhold, ikke plassering: byen som
//! data (`layout`) peker til dem med en nøkkel (`LINES`).

pub const SMITH: &[&str] = &[
    "Buttonsmeden, til tjeneste. Jeg smir det du bærer opp fra krypten.",
    "Et sverd trenger en klinge og et grep: en divider og en button. Resten er håndlag.",
    "Kom tilbake med komponenter, så får vi se hva stålet duger til.",
];

pub const CLERK: &[&str] = &[
    "Skriptoriet. Ingenting smis uten et korrekt utfylt skjema.",
    "Oppskriftene er skjemaer. Fyll ut feltene med komponentene du har vunnet, og send.",
    "Bøkene her er dokumentasjonen. Les om det du bærer, så leges sårene.",
];

pub const APOTHECARY: &[&str] = &[
    "Apoteket. Drikker mot det meste, tokens mot alt.",
    "Les i bøkene når du er skadet. Kunnskap leger.",
    "Hold deg unna lavaen nederst i dypet. Den står ikke i noen bok.",
];

pub const GORVEK: &[&str] = &[
    "Gorvek. Jeg lærer deg trekk, om du har tokens og tålmodighet.",
    "Blokk i det bittet kommer. Da står de lamslått, og du hugger dobbelt.",
    "Unna med Shift. Det koster pust, men sparer blod.",
];

pub const INNKEEPER: &[&str] = &[
    "Velkommen til Vertshuset. Øl er det nok av, nyheter også.",
    "De sier krypten under borgen går dypere enn noen har gått.",
    "Det kommer rare ting opp fra dypet om natta. Buttons med tenner.",
];

pub const TOWNSFOLK: &[&str] = &[
    "God dag. Vannet i brønnen er kaldt i dag, kaldt som krypten.",
    "Har du vært nede under borgen? Bror min gikk ned i fjor. Han snakker bare om buttons nå.",
    "Torgdag i dag. Alt er dyrere enn i går.",
    "Kirkeklokka ringte feil i natt. Tretten slag.",
];

pub const GUARD: &[&str] = &[
    "Hold deg unna borghaugen etter mørkets frembrudd.",
    "Portene stenges ikke, men jeg holder øye med hvem som går ut.",
    "Ser du noe som kryper opp fra krypten, rop.",
];

pub const PILGRIM: &[&str] = &[
    "Jeg har gått fra Nidaros for å se det tapte systemet. De sier det passer i alle rammeverk.",
    "Fristil. Navnet alene gjør meg rolig.",
];

/// Alle replikksettene med nøkkelen byen som data bruker.
pub const LINES: &[(&str, &[&str])] = &[
    ("smith", SMITH),
    ("clerk", CLERK),
    ("apothecary", APOTHECARY),
    ("gorvek", GORVEK),
    ("innkeeper", INNKEEPER),
    ("townsfolk", TOWNSFOLK),
    ("guard", GUARD),
    ("pilgrim", PILGRIM),
];

/// Replikkene med nøkkelen `key` (tomt om den ikke finnes).
pub fn lines(key: &str) -> &'static [&'static str] {
    LINES
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, l)| *l)
        .unwrap_or(&[])
}

/// Nøkkelen til et replikksett (samme liste i minnet).
pub fn key_of(lines: &'static [&'static str]) -> &'static str {
    LINES
        .iter()
        .find(|(_, l)| *l == lines)
        .map(|(k, _)| *k)
        .unwrap_or("")
}
