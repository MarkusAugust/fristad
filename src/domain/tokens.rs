//! Det kjernen trenger fra Fristil: versjonen av designsystemet som
//! komponentene i spillet (byttet, bøkene i skriptoriet) hører til.
//! `assets/tokens.json` lages av `tools/export-tokens.ts` fra
//! `@fristil/designsystem`, og bygges inn i binæren.

use serde::Deserialize;

#[derive(Deserialize)]
struct Raw {
    fristil: String,
}

pub struct Tokens {
    pub version: String,
}

impl Tokens {
    pub fn load() -> Self {
        let raw: Raw = serde_json::from_str(include_str!("../../assets/tokens.json")).expect("assets/tokens.json er ugyldig. Kjør `bun tools/export-tokens.ts`.");
        Self { version: raw.fristil }
    }
}
