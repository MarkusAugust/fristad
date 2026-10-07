//! Kistene i kjelleren: en usynlig prop over hver kiste byen satte ut
//! (`Dungeon.chests`), så F åpner den (`Env.open_chest`) og spillet gir
//! deg en komponent fra etasjens liste (`craft::LOOT`). Ingen komponent.

use super::{Action, Component, Env, Installation};
use crate::domain::city::Builder;
use crate::domain::geometry::Mesh;
use glam::{Affine3A, Vec3};

#[derive(Default)]
pub struct Chests {
    props: Vec<usize>,
    opened: Vec<bool>,
    aimed: Option<usize>,
}

impl Installation for Chests {
    fn component(&self) -> Component {
        Component { name: "chest", class: "" }
    }
    fn usable(&self) -> bool {
        self.aimed.is_some_and(|i| !self.opened[i])
    }
    fn inspectable(&self) -> bool {
        false
    }
    fn label(&self) -> &'static str {
        if self.aimed.is_some_and(|i| self.opened[i]) { "Tom kiste" } else { "Kiste" }
    }
    fn verb(&self) -> &'static str {
        "Åpne"
    }

    fn build(&mut self, b: &mut Builder) {
        let chests = b.dungeon.as_ref().map(|d| d.chests.clone()).unwrap_or_default();
        for (p, yaw, _) in chests {
            let i = b.prop("chest", Mesh::default(), Affine3A::from_rotation_translation(glam::Quat::from_rotation_y(yaw), p), Some((Vec3::new(0.0, 0.3, 0.0), Vec3::new(0.48, 0.33, 0.33))));
            self.props.push(i);
            self.opened.push(false);
        }
    }

    fn tick(&mut self, env: &mut Env) {
        self.aimed = env.aimed_prop.and_then(|p| self.props.iter().position(|q| *q == p));
    }

    fn act(&mut self, action: Action, env: &mut Env) {
        if action != Action::Use {
            return;
        }
        if let Some(i) = self.aimed {
            if !self.opened[i] {
                self.opened[i] = true;
                env.open_chest = Some(i);
            }
        }
    }
}
