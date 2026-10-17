//! Hva innbyggerne sier. Replikkene er innhold, ikke plassering: byen som
//! data (`layout`) peker til dem med en nøkkel (`LINES`).

pub const SMITH: &[&str] = &[
    "Buttonsmeden, til tjeneste. Jeg smir det du bærer opp fra krypten.",
    "Et sverd trenger en klinge og et grep: en divider og en button. Resten er håndlag.",
    "Komponentene der nede er de ekte, de fra mønsterboka. Bare vridd. Bær dem hit, så retter vi dem ut.",
];

pub const CLERK: &[&str] = &[
    "Skriptoriet. Her skrev mester Ansgar en gang. Nå smir vi med det han skrev.",
    "Oppskriftene er skjemaer. Fyll ut feltene med komponentene du har vunnet, og send.",
    "Bøkene her er dokumentasjonen. Les om det du bærer, så leges sårene.",
    "Ansgar gikk ned med de siste sidene. Finner du noe han skrev der nede, ta det med deg i hodet.",
];

pub const APOTHECARY: &[&str] = &[
    "Apoteket. Drikker mot det meste, og olje til lykta. Uten lys er krypten en grav.",
    "Les i bøkene når du er skadet. Kunnskap leger.",
    "Hvilestedene der nede fyller lykta. Gå ikke forbi et alter uten å tenne det.",
];

pub const GORVEK: &[&str] = &[
    "Torgils. Jeg lærer deg trekk, om du har tokens og tålmodighet.",
    "Blokk i det bittet kommer. Da står de lamslått, og du hugger dobbelt.",
    "Unna med Shift. Det koster pust, men sparer blod.",
    "Gorvek hogg det i stykker med én øks. Du har flere trekk enn han hadde. Bruk dem.",
];

pub const INNKEEPER: &[&str] = &[
    "Velkommen til Vertshuset. Øl er det nok av, nyheter også.",
    "De sier krypten under kirka går dypere enn noen har gått. Ti trapper, sier klokkeren.",
    "Det kommer rare ting opp fra dypet om natta. Buttons med tenner.",
    "Klokkeren så lys under kryptdøra i vinter. Ingen hadde tent det.",
];

pub const TOWNSFOLK: &[&str] = &[
    "God dag. Vannet i brønnen er kaldt i dag, kaldt som krypten.",
    "Har du vært nede i krypten? Bror min gikk ned i fjor. Han snakker bare om buttons nå.",
    "Torgdag i dag. Alt er dyrere enn i går.",
    "Kirkeklokka ringte feil i natt. Tretten slag.",
    "Bestefar sa at Gorvek hogg beistet i stykker over byen, natta gjennom i regnet. Og at ingen så ham siden.",
    "Krypten er på kirkegården, ut nordporten. Gå ikke ned uten olje på lykta.",
];

pub const GUARD: &[&str] = &[
    "Hold deg unna kirkegården etter mørkets frembrudd.",
    "Portene stenges ikke, men jeg holder øye med hvem som går ut.",
    "Ser du noe som kryper opp fra krypten, rop.",
    "Nordporten tar deg til kirkegården. Døra til krypten står mot veien.",
];

pub const PILGRIM: &[&str] = &[
    "Jeg har gått fra Nidaros for å se mønsterboka. De sier den aldri sto i veien for noen.",
    "Fristil. Navnet alene gjør meg rolig.",
    "Byene på veien hit skriver alle på hoffets tunge. Bare her husker noen den gamle boka.",
];

/// Alle replikksettene med nøkkelen byen som data bruker.
pub const LINES: &[(&str, &[&str])] = &[("smith", SMITH), ("clerk", CLERK), ("apothecary", APOTHECARY), ("gorvek", GORVEK), ("innkeeper", INNKEEPER), ("townsfolk", TOWNSFOLK), ("guard", GUARD), ("pilgrim", PILGRIM)];

/// Replikkene med nøkkelen `key` (tomt om den ikke finnes).
pub fn lines(key: &str) -> &'static [&'static str] {
    LINES.iter().find(|(k, _)| *k == key).map(|(_, l)| *l).unwrap_or(&[])
}

/// Nøkkelen til et replikksett (samme liste i minnet).
pub fn key_of(lines: &'static [&'static str]) -> &'static str {
    LINES.iter().find(|(_, l)| *l == lines).map(|(k, _)| *k).unwrap_or("")
}
