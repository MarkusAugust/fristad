//! Trubaduren i Vertshuset (eieren 18. oktober): han synger kvadet om
//! Gorvek bare når noen har kjøpt ham et krus mjød (1 token, E på ham), én
//! gang fra start til slutt (`assets/music/game/tavern.mp3`). Mens sangen
//! går, spiller han (`play`); etterpå sitter han stille (`sit`) og sier at
//! strupen er tørr.

use super::session::{sfx, Session};
use crate::domain::hero::Hero;
use crate::domain::npcs::Npcs;
use crate::domain::world::cast;
use crate::ports::driving::{Event, MeadView};

/// Et krus mjød koster så mange tokens.
pub const MEAD_PRICE: u32 = 1;
/// Så lenge kvadet varer (lengden på `tavern.mp3`, 302,4 s); nettlaget sier
/// fra når fila er ferdig (`Command::SongOver`), dette er reserven.
pub const SONG_SECS: f32 = 302.5;

/// Trubaduren: hvor langt i kvadet han er (`None` når han ikke synger).
/// Han eier bare klokka; innbyggerne, pungen og hendelseslista får han som
/// argument fra økta.
#[derive(Default)]
pub(super) struct Troubadour {
    song: Option<f32>,
}

/// Den som sitter og spiller (trubaduren), om han finnes.
fn seat(npcs: &Npcs) -> Option<usize> {
    npcs.npcs.iter().position(|n| n.seated)
}

impl Troubadour {
    pub fn singing(&self) -> bool {
        self.song.is_some()
    }

    /// E på trubaduren: tilbudet (knappen er av mens han synger).
    pub fn offer(&self, npcs: &Npcs, i: usize) -> Event {
        let name = npcs.npcs.get(i).map(|n| n.name).unwrap_or("Trubaduren");
        let singing = self.singing();
        Event::Mead(MeadView { npc: name.into(), text: (if singing { cast::MEAD_SINGING } else { cast::MEAD_OFFER }).into(), price: MEAD_PRICE, singing })
    }

    /// Kjøp et krus mjød: trekker en token og starter kvadet fra start.
    /// Synger han allerede, takker han og synger videre (ingen betaler to
    /// ganger). Gir om det gikk.
    pub fn buy_mead(&mut self, npcs: &mut Npcs, hero: &mut Hero, events: &mut Vec<Event>) -> bool {
        let Some(i) = seat(npcs) else { return false };
        if self.singing() {
            npcs.say(i, cast::MEAD_AGAIN);
            return false;
        }
        if hero.coins < MEAD_PRICE {
            events.extend([sfx("deny"), Event::Toast(format!("Et krus mjød koster {MEAD_PRICE} token."))]);
            return false;
        }
        hero.coins -= MEAD_PRICE;
        hero.dirty = true;
        events.extend([sfx("coins"), sfx("equip_wood")]);
        npcs.say(i, cast::MEAD_THANKS);
        self.start(npcs, events);
        true
    }

    /// Kvadet fra start.
    pub fn start(&mut self, npcs: &mut Npcs, events: &mut Vec<Event>) {
        self.song = Some(0.0);
        npcs.set_singing(true);
        events.push(Event::Song(true));
    }

    /// Kvadet er over (fila er ferdig, eller tiden er ute): han legger fra
    /// seg spillet og sier at strupen er tørr.
    pub fn end(&mut self, npcs: &mut Npcs, events: &mut Vec<Event>) {
        if self.song.take().is_none() {
            return;
        }
        npcs.set_singing(false);
        npcs.set_voice(0.0);
        if let Some(i) = seat(npcs) {
            npcs.say(i, cast::MEAD_DRY);
        }
        events.push(Event::Song(false));
    }

    /// Klokka i kvadet.
    pub fn tick(&mut self, dt: f32, npcs: &mut Npcs, events: &mut Vec<Event>) {
        if let Some(t) = &mut self.song {
            *t += dt;
            if *t >= SONG_SECS {
                self.end(npcs, events);
            }
        }
    }
}

impl Session {
    /// Om trubaduren synger nå.
    pub fn singing(&self) -> bool {
        self.tavern.singing()
    }

    pub fn buy_mead(&mut self) -> bool {
        self.tavern.buy_mead(&mut self.npcs, &mut self.hero, &mut self.pending)
    }

    /// Kvadet uten mjød (skjermbildene: `snapshot --voice V`).
    pub fn play_song(&mut self) {
        self.tavern.start(&mut self.npcs, &mut self.pending);
    }

    pub fn end_song(&mut self) {
        self.tavern.end(&mut self.npcs, &mut self.pending);
    }
}

#[cfg(any(test, feature = "testing"))]
impl Session {
    /// E på trubaduren, som om du siktet på ham.
    pub fn trade_troubadour(&self) -> Option<Event> {
        self.npc_panel(seat(&self.npcs)?)
    }

    /// Klippet trubaduren spiller nå (`sit` eller `play`); mangler figuren
    /// (`model::PLANNED`), klippet han ville spilt.
    pub fn troubadour_clip(&self) -> Option<&str> {
        let i = seat(&self.npcs)?;
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
