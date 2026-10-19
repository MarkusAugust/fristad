//! Trubaduren i Vertshuset (eieren 18. oktober): han synger kvadet om
//! Gorvek bare når noen har kjøpt ham et krus mjød (1 token, E på ham), én
//! gang fra start til slutt (`assets/music/game/tavern.mp3`). Mens sangen
//! går, spiller han (`play`); etterpå sitter han stille (`sit`) og sier at
//! strupen er tørr.

use super::session::Session;
use crate::domain::note::Note;
use crate::domain::world::cast;
use crate::ports::driving::{Event, MeadView};

/// Et krus mjød koster så mange tokens.
pub const MEAD_PRICE: u32 = 1;
/// Så lenge kvadet varer (lengden på `tavern.mp3`, 302,4 s); nettlaget sier
/// fra når fila er ferdig (`Command::SongOver`), dette er reserven.
pub const SONG_SECS: f32 = 302.5;

/// Trubaduren: hvor langt i kvadet han er (`None` når han ikke synger).
#[derive(Default)]
pub(super) struct Troubadour {
    pub song: Option<f32>,
}

impl Session {
    /// Den som sitter og spiller (trubaduren), om han finnes.
    fn troubadour(&self) -> Option<usize> {
        self.npcs.npcs.iter().position(|n| n.seated)
    }

    /// Om trubaduren synger nå.
    pub fn singing(&self) -> bool {
        self.tavern.song.is_some()
    }

    /// E på trubaduren: tilbudet (knappen er av mens han synger).
    pub(super) fn mead_view(&self, i: usize) -> Event {
        let name = self.npcs.npcs.get(i).map(|n| n.name).unwrap_or("Trubaduren");
        let singing = self.singing();
        Event::Mead(MeadView { npc: name.into(), text: (if singing { cast::MEAD_SINGING } else { cast::MEAD_OFFER }).into(), price: MEAD_PRICE, singing })
    }

    /// Kjøp et krus mjød: trekker en token og starter kvadet fra start.
    /// Synger han allerede, takker han og synger videre (ingen betaler to
    /// ganger). Gir om det gikk.
    pub fn buy_mead(&mut self) -> bool {
        let Some(i) = self.troubadour() else { return false };
        let mut events = vec![];
        if self.singing() {
            self.npcs.say(i, cast::MEAD_AGAIN);
            return false;
        }
        if self.hero.coins < MEAD_PRICE {
            self.say([Note::Sfx("deny"), Note::toast(format!("Et krus mjød koster {MEAD_PRICE} token."))], &mut events);
            self.pending.extend(events);
            return false;
        }
        self.hero.coins -= MEAD_PRICE;
        self.hero.dirty = true;
        self.say([Note::Sfx("coins"), Note::Sfx("equip_wood")], &mut events);
        self.npcs.say(i, cast::MEAD_THANKS);
        self.start_song(&mut events);
        self.pending.extend(events);
        true
    }

    /// Kvadet uten mjød (skjermbildene: `snapshot --voice V`).
    pub fn play_song(&mut self) {
        let mut events = vec![];
        self.start_song(&mut events);
        self.pending.extend(events);
    }

    fn start_song(&mut self, events: &mut Vec<Event>) {
        self.tavern.song = Some(0.0);
        self.npcs.set_singing(true);
        events.push(Event::Song(true));
    }

    /// Kvadet er over (fila er ferdig, eller tiden er ute): han legger fra
    /// seg spillet og sier at strupen er tørr.
    pub fn end_song(&mut self) {
        if self.tavern.song.take().is_none() {
            return;
        }
        self.npcs.set_singing(false);
        self.npcs.set_voice(0.0);
        if let Some(i) = self.troubadour() {
            self.npcs.say(i, cast::MEAD_DRY);
        }
        self.pending.push(Event::Song(false));
    }

    /// Klokka i kvadet.
    pub(super) fn tick_song(&mut self, dt: f32) {
        if let Some(t) = &mut self.tavern.song {
            *t += dt;
            if *t >= SONG_SECS {
                self.end_song();
            }
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl Session {
    /// E på trubaduren, som om du siktet på ham.
    pub fn trade_troubadour(&self) -> Option<Event> {
        self.npc_panel(self.troubadour()?)
    }

    /// Klippet trubaduren spiller nå (`sit` eller `play`); mangler figuren
    /// (`model::PLANNED`), klippet han ville spilt.
    pub fn troubadour_clip(&self) -> Option<&str> {
        let i = self.troubadour()?;
        let n = &self.npcs.npcs[i];
        self.npcs.clip_of(i).or(Some(if self.npcs.singing && n.speaking.is_none() { "play" } else { "sit" }))
    }

    pub fn tokens(&self) -> u32 {
        self.hero.coins
    }

    pub fn set_tokens(&mut self, n: u32) {
        self.hero.coins = n;
    }
}
