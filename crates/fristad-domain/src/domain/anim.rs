//! Skjelettanimasjon: et skjelett av ledd med foreldre, klipp med
//! nøkkelbilder per ledd (flytting, dreining, skalering), posering og
//! overtoning mellom klipp. Rene tall: lasteren (glTF) fyller `Rig`,
//! spillet driver en `Animator` per figur, og rendereren får
//! leddmatrisene som `Frame.skins` og vekter dem inn per hjørne.

use glam::{Mat4, Quat, Vec3};

#[derive(Clone, Debug)]
pub struct Joint {
    pub name: String,
    /// Indeksen til forelderen i `Rig.joints`; alltid lavere enn egen indeks.
    pub parent: Option<usize>,
    /// Hvileposen, lokalt i forelderen.
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    /// Fra modellrommet (hjørnene slik de ligger i meshen) til leddet.
    pub inverse_bind: Mat4,
}

#[derive(Clone, Debug)]
pub enum Keys {
    Translation(Vec<Vec3>),
    Rotation(Vec<Quat>),
    Scale(Vec<Vec3>),
}

#[derive(Clone, Debug)]
pub struct Channel {
    pub joint: usize,
    pub times: Vec<f32>,
    pub keys: Keys,
    /// Trappetrinn i stedet for glidende overgang.
    pub step: bool,
}

#[derive(Clone, Debug)]
pub struct Clip {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<Channel>,
}

/// En lokal pose: (flytting, dreining, skalering) per ledd.
#[derive(Clone, Debug, PartialEq)]
pub struct Pose(pub Vec<(Vec3, Quat, Vec3)>);

#[derive(Clone, Debug, Default)]
pub struct Rig {
    pub joints: Vec<Joint>,
    pub clips: Vec<Clip>,
    /// Transformen lasteren la på meshen etter at hjørnene var lest
    /// (dreining, skalering, flytting til bakken). Leddmatrisene
    /// konjugeres med den, så skinningen skjer i det samme rommet.
    pub root: Mat4,
}

fn lerp3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}

impl Rig {
    pub fn rest(&self) -> Pose {
        Pose(self.joints.iter().map(|j| (j.translation, j.rotation, j.scale)).collect())
    }

    pub fn clip(&self, name: &str) -> Option<usize> {
        self.clips.iter().position(|c| c.name == name)
    }

    pub fn joint(&self, name: &str) -> Option<usize> {
        self.joints.iter().position(|j| j.name == name)
    }

    /// Posen i klipp `clip` ved tida `t` (sekunder). Ledd uten kanal i
    /// klippet står i hvileposen.
    pub fn sample(&self, clip: usize, t: f32, looping: bool) -> Pose {
        let mut pose = self.rest();
        let Some(c) = self.clips.get(clip) else { return pose };
        let t = if c.duration <= 0.0 {
            0.0
        } else if looping {
            t.rem_euclid(c.duration)
        } else {
            t.clamp(0.0, c.duration)
        };
        for ch in &c.channels {
            let Some(slot) = pose.0.get_mut(ch.joint) else { continue };
            let n = ch.times.len();
            if n == 0 {
                continue;
            }
            // Nøkkelbildet før og etter, og hvor langt mellom dem.
            let (i, f) = if t <= ch.times[0] {
                (0, 0.0)
            } else if t >= ch.times[n - 1] {
                (n - 1, 0.0)
            } else {
                let i = ch.times.partition_point(|x| *x <= t) - 1;
                let span = ch.times[i + 1] - ch.times[i];
                (i, if span > 1e-6 && !ch.step { (t - ch.times[i]) / span } else { 0.0 })
            };
            let j = (i + 1).min(n - 1);
            match &ch.keys {
                Keys::Translation(v) => slot.0 = lerp3(v[i], v[j], f),
                Keys::Rotation(v) => slot.1 = v[i].slerp(v[j], f),
                Keys::Scale(v) => slot.2 = lerp3(v[i], v[j], f),
            }
        }
        pose
    }

    /// Leddenes globale matriser (i modellrommet) for en pose.
    pub fn globals(&self, pose: &Pose) -> Vec<Mat4> {
        let mut out = Vec::with_capacity(self.joints.len());
        for (i, j) in self.joints.iter().enumerate() {
            let (t, r, s) = pose.0.get(i).copied().unwrap_or((j.translation, j.rotation, j.scale));
            let local = Mat4::from_scale_rotation_translation(s, r, t);
            let m = match j.parent {
                Some(p) => out[p] * local,
                None => local,
            };
            out.push(m);
        }
        out
    }

    /// Skinnmatrisene rendereren trenger: hjørne i meshen → hjørne posert,
    /// i samme rom som meshen ligger i (med `root`).
    pub fn skin(&self, pose: &Pose) -> Vec<Mat4> {
        let inv_root = self.root.inverse();
        self.globals(pose).iter().zip(&self.joints).map(|(g, j)| self.root * *g * j.inverse_bind * inv_root).collect()
    }

    /// Hvor et ledd står i meshens rom (til å henge et våpen i hånda).
    pub fn joint_transform(&self, pose: &Pose, joint: usize) -> Mat4 {
        self.root * self.globals(pose)[joint]
    }
}

impl Pose {
    /// Blanding mellom to poser, `w` = 0 gir `self`.
    pub fn lerp(&self, other: &Pose, w: f32) -> Pose {
        Pose(
            self.0
                .iter()
                .zip(&other.0)
                .map(|(a, b)| (lerp3(a.0, b.0, w), a.1.slerp(b.1, w), lerp3(a.2, b.2, w)))
                .collect(),
        )
    }
}

/// Avspillingen for én figur: klippet som går, med overtoning fra det
/// forrige. Spillet kaller `play` når tilstanden endrer seg og `update`
/// hvert bilde.
#[derive(Clone, Debug)]
pub struct Animator {
    pub clip: usize,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
    /// Forrige klipp (klipp, tid, løkke) mens overtoningen varer.
    prev: Option<(usize, f32, bool)>,
    fade: f32,
    fade_len: f32,
    /// Et klipp uten løkke har nådd slutten.
    pub finished: bool,
}

impl Animator {
    pub fn new(clip: usize, looping: bool) -> Self {
        Self { clip, time: 0.0, speed: 1.0, looping, prev: None, fade: 0.0, fade_len: 0.0, finished: false }
    }

    /// Bytt klipp med overtoning på `fade` sekunder. Samme løkke som alt
    /// går fortsetter uavbrutt.
    pub fn play(&mut self, clip: usize, looping: bool, fade: f32) {
        if clip == self.clip && looping && self.looping && !self.finished {
            return;
        }
        if fade > 0.0 {
            self.prev = Some((self.clip, self.time, self.looping));
            self.fade = 0.0;
            self.fade_len = fade;
        } else {
            self.prev = None;
        }
        self.clip = clip;
        self.time = 0.0;
        self.looping = looping;
        self.finished = false;
    }

    pub fn update(&mut self, dt: f32, rig: &Rig) {
        self.time += dt * self.speed;
        if let Some((_, t, _)) = &mut self.prev {
            *t += dt * self.speed;
        }
        if self.prev.is_some() {
            self.fade += dt;
            if self.fade >= self.fade_len {
                self.prev = None;
            }
        }
        if !self.looping {
            let d = rig.clips.get(self.clip).map(|c| c.duration).unwrap_or(0.0);
            if self.time >= d {
                self.time = d;
                self.finished = true;
            }
        }
    }

    /// Hvor langt klippet er kommet, 0..1.
    pub fn phase(&self, rig: &Rig) -> f32 {
        let d = rig.clips.get(self.clip).map(|c| c.duration).unwrap_or(0.0);
        if d <= 0.0 {
            1.0
        } else if self.looping {
            self.time.rem_euclid(d) / d
        } else {
            (self.time / d).clamp(0.0, 1.0)
        }
    }

    pub fn pose(&self, rig: &Rig) -> Pose {
        let cur = rig.sample(self.clip, self.time, self.looping);
        match self.prev {
            Some((c, t, l)) if self.fade_len > 0.0 => {
                let w = (self.fade / self.fade_len).clamp(0.0, 1.0);
                // Myk overgang.
                let w = w * w * (3.0 - 2.0 * w);
                rig.sample(c, t, l).lerp(&cur, w)
            }
            _ => cur,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rig() -> Rig {
        // To ledd: rota i origo, barnet 1 m opp. Hvileposen er identitet,
        // så inverse bind er motsatt av den globale hvileposen.
        let root = Joint { name: "root".into(), parent: None, translation: Vec3::ZERO, rotation: Quat::IDENTITY, scale: Vec3::ONE, inverse_bind: Mat4::IDENTITY };
        let child = Joint { name: "arm".into(), parent: Some(0), translation: Vec3::Y, rotation: Quat::IDENTITY, scale: Vec3::ONE, inverse_bind: Mat4::from_translation(-Vec3::Y) };
        let clip = Clip {
            name: "bend".into(),
            duration: 1.0,
            channels: vec![Channel { joint: 1, times: vec![0.0, 1.0], keys: Keys::Rotation(vec![Quat::IDENTITY, Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)]), step: false }],
        };
        Rig { joints: vec![root, child], clips: vec![clip], root: Mat4::IDENTITY }
    }

    #[test]
    fn hvileposen_gir_identitet() {
        let r = rig();
        for m in r.skin(&r.rest()) {
            assert!(m.abs_diff_eq(Mat4::IDENTITY, 1e-5), "{m}");
        }
    }

    #[test]
    fn barnet_dreier_om_sitt_eget_ledd() {
        let r = rig();
        let pose = r.sample(0, 1.0, false);
        let m = r.skin(&pose)[1];
        // Et punkt 1 m over leddet (2 m opp) skal svinge mot −X, i høyde 1.
        let p = m.transform_point3(Vec3::new(0.0, 2.0, 0.0));
        assert!(p.abs_diff_eq(Vec3::new(-1.0, 1.0, 0.0), 1e-4), "{p}");
        // Halvveis er dreiningen 45°.
        let half = r.sample(0, 0.5, false);
        let p = r.skin(&half)[1].transform_point3(Vec3::new(0.0, 2.0, 0.0));
        assert!((p.x + std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-3, "{p}");
    }

    #[test]
    fn overtoningen_ender_i_det_nye_klippet() {
        let r = rig();
        let mut a = Animator::new(0, true);
        a.update(1.0, &r);
        a.play(0, false, 0.2);
        for _ in 0..25 {
            a.update(0.05, &r);
        }
        assert!(a.finished);
        assert_eq!(a.pose(&r), r.sample(0, 1.0, false));
    }
}
