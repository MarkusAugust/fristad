//! Paletten: fargene i byen som kommer fra Fristil-tokens og bytter med
//! temaet. Geometrien bærer bare et plassnummer, så husene og komponentene
//! kan bytte farge uten ny geometri. Plass 0 betyr «bruk vertexfargen».

use crate::domain::scene::PALETTE;
use crate::domain::tokens::Tokens;
use glam::Vec3;

/// Plassene som er bundet til ett token hver. Komponentene som trenger
/// variantavhengige farger bruker de frie plassene under.
pub const TOKEN_SLOTS: &[(u8, &str)] = &[
    (9, "neutral-surface"),
    (10, "neutral-canvas"),
    (11, "neutral-text-strong"),
    (12, "neutral-text-subtle"),
    (13, "neutral-border"),
    (14, "accent-fill"),
    (15, "accent-surface"),
    (16, "accent-text"),
    (17, "success-fill"),
    (18, "success-surface"),
    (19, "success-text"),
    (20, "warning-fill"),
    (21, "warning-surface"),
    (22, "warning-text"),
    (23, "danger-fill"),
    (24, "danger-surface"),
    (25, "danger-text"),
    (26, "accent-border"),
    (27, "neutral-fill"),
    (28, "neutral-raised"),
    (29, "neutral-border-subtle"),
    (30, "accent-content"),
    (31, "neutral-text"),
];

pub mod slot {
    pub const BUTTON_BODY: u8 = 1;
    pub const BUTTON_RIM: u8 = 2;
    pub const BUTTON_LABEL: u8 = 3;
    pub const BRAND1: u8 = 4;
    pub const BRAND2: u8 = 5;
    pub const BRAND3: u8 = 6;
    pub const ACCENT: u8 = 7;
    pub const WARNING: u8 = 8;
    pub const NEUTRAL_SURFACE: u8 = 9;
    pub const NEUTRAL_CANVAS: u8 = 10;
    pub const NEUTRAL_TEXT_STRONG: u8 = 11;
    pub const NEUTRAL_TEXT_SUBTLE: u8 = 12;
    pub const NEUTRAL_BORDER: u8 = 13;
    pub const ACCENT_FILL: u8 = 14;
    pub const ACCENT_SURFACE: u8 = 15;
    pub const ACCENT_TEXT: u8 = 16;
    pub const SUCCESS_FILL: u8 = 17;
    pub const SUCCESS_SURFACE: u8 = 18;
    pub const SUCCESS_TEXT: u8 = 19;
    pub const WARNING_FILL: u8 = 20;
    pub const WARNING_SURFACE: u8 = 21;
    pub const WARNING_TEXT: u8 = 22;
    pub const DANGER_FILL: u8 = 23;
    pub const DANGER_SURFACE: u8 = 24;
    pub const DANGER_TEXT: u8 = 25;
    pub const ACCENT_BORDER: u8 = 26;
    pub const NEUTRAL_FILL: u8 = 27;
    pub const NEUTRAL_RAISED: u8 = 28;
    pub const NEUTRAL_BORDER_SUBTLE: u8 = 29;
    pub const ACCENT_CONTENT: u8 = 30;
    pub const NEUTRAL_TEXT: u8 = 31;
    // Frie plasser til installasjonene, som setter dem i `palette()`.
    pub const TAG_FILL: u8 = 32;
    pub const TAG_BORDER: u8 = 33;
    pub const TAG_TEXT: u8 = 34;
    pub const BADGE_FILL: u8 = 35;
    pub const BADGE_TEXT: u8 = 36;
    pub const SWITCH_BODY: u8 = 37;
    pub const PROGRESS_FILL: u8 = 38;
    pub const SWITCH_KNOB: u8 = 39;
    pub const SWITCH_BORDER: u8 = 40;
    pub const DIALOG_OVERLAY: u8 = 41;
    pub const CARD_FILL: u8 = 42;
    pub const CARD_BORDER: u8 = 43;
    pub const ACC_BORDER: u8 = 44;
    pub const INPUT_FILL: u8 = 45;
    pub const INPUT_BORDER: u8 = 46;
    pub const INPUT_TEXT: u8 = 47;
    pub const SELECT_FILL: u8 = 48;
    pub const SELECT_BORDER: u8 = 49;
    pub const SELECT_TEXT: u8 = 50;
    pub const CHECK_FILL: u8 = 51;
    pub const CHECK_BORDER: u8 = 52;
    pub const CHECK_ON: u8 = 53;
    pub const CHECK_MARK: u8 = 54;
    pub const RADIO_BORDER: u8 = 55;
    pub const TA_FILL: u8 = 56;
    pub const TA_BORDER: u8 = 57;
    pub const TA_TEXT: u8 = 58;
}

/// Paletten for et øyeblikk mellom dag (0) og natt (1).
pub fn build(tokens: &Tokens, night: f32) -> [Vec3; PALETTE] {
    let mut p = [Vec3::ONE; PALETTE];
    for (slot, name) in TOKEN_SLOTS {
        p[*slot as usize] = tokens.blend(name, night);
    }
    p[slot::BRAND1 as usize] = tokens.blend("brand1-fill", night);
    p[slot::BRAND2 as usize] = tokens.blend("brand2-fill", night);
    p[slot::BRAND3 as usize] = tokens.blend("brand3-fill", night);
    p[slot::ACCENT as usize] = tokens.blend("accent-fill", night);
    p[slot::WARNING as usize] = tokens.blend("warning-fill", night);
    p
}
