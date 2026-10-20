//! Hvordan en fiende går mot et mål på navigasjonsnettet (del av `mobs.rs`).

use super::*;

pub(super) fn walk(m: &mut Mob, goal: Vec3, speed: f32, dt: f32, crypt: &Crypt, push: Vec3, stats: &Stats, clips: &Clips, out: &mut Outcome) {
    if climb(m, dt, clips, out) {
        return;
    }
    m.repath -= dt;
    if m.repath <= 0.0 || m.path.is_empty() || m.goal.distance(goal) > 1.0 {
        m.path = crypt.route(m.pos, goal).unwrap_or_default();
        m.goal = goal;
        m.repath = 0.5;
    }
    loop {
        // Ved foten av stigen eller kanten av heisen: veien går over lenken.
        if let Some(steps) = m.path.first().and_then(|next| crypt.hop_ahead(m.pos, *next, speed)) {
            m.climb = steps;
            m.path.clear();
            climb(m, dt, clips, out);
            return;
        }
        if !(m.path.len() > 1 && m.path[0].with_y(0.0).distance(m.pos.with_y(0.0)) < 0.3) {
            break;
        }
        // Rundt en indre krok (bro mot balkong): helt fram til hjørnet først.
        if m.path[0].with_y(0.0).distance(m.pos.with_y(0.0)) > 0.02 && !crypt.walkable(m.pos + (m.path[1] - m.pos).with_y(0.0).normalize_or_zero() * 0.15) {
            break;
        }
        m.path.remove(0);
    }
    let from = m.pos;
    let on = crypt.walkable(from);
    let next = if on { m.path.first().copied().unwrap_or(goal) } else { crypt.snap(from, 3.0).unwrap_or(goal) };
    let dir = (next - from).with_y(0.0).normalize_or_zero();
    let speed = if on && m.path.len() > 1 { speed.min((next - from).with_y(0.0).length() / dt.max(1e-4)) } else { speed };
    // Står en annen rett i veien (de går mot hverandre), går den til siden
    // i stedet for å bli stående og dytte.
    let mut push = push;
    if push.length_squared() > 1e-6 && push.dot(dir) < 0.0 {
        let side = Vec3::new(-dir.z, 0.0, dir.x);
        let away = if push.dot(side) >= 0.0 { side } else { -side };
        push += away * push.length() * 1.5;
    }
    let step = dir * speed * dt + push * dt * if on { 1.0 } else { 0.0 };
    let turn = |a: f32| Quat::from_rotation_y(a) * step;
    let tries = [step, Vec3::new(step.x, 0.0, 0.0), Vec3::new(0.0, 0.0, step.z), turn(0.9), turn(-0.9), turn(1.6), turn(-1.6)];
    if !on {
        m.pos = from + step;
    } else if let Some(t) = tries.into_iter().find(|t| t.length_squared() > (0.25 * step.length()).powi(2).max(1e-10) && crypt.walkable(from + *t)) {
        m.pos = from + t;
    } else if let Some(q) = crypt.snap(from + step, 0.3).filter(|q| q.with_y(0.0).distance(next.with_y(0.0)) < from.with_y(0.0).distance(next.with_y(0.0)) - 1e-4) {
        m.pos = q; // glir langs kanten av nettet i stedet for å bli stående
    }
    m.pos.y = crypt.height_at(m.pos + Vec3::Y * 0.3);
    let s = m.scale();
    // Skrittene: bossens tunge, de andres tassing (ikke tooltipene, som svever).
    if m.kind != Kind::Tooltip {
        m.stride += (m.pos - from).with_y(0.0).length();
        if m.stride > 1.4 * s {
            m.stride = 0.0;
            out.sounds.push((if m.boss { "boss_step" } else { "mob_step" }, m.pos));
        }
    }
    let moved = (m.pos - from).with_y(0.0).length() / dt.max(1e-4);
    let want_state = if moved > 0.3 { State::Walk } else { State::Idle };
    set_state(m, want_state, clips);
    if let Some(a) = m.anim.as_mut() {
        if want_state == State::Walk {
            // Gangen er laget for slagets fart (mob_motion.py), så føttene
            // står stille på gulvet når farten stemmer.
            a.speed = (moved / stats.speed.max(0.1)).clamp(0.4, 1.6);
        }
    }
    if dir.length_squared() > 0.0 {
        let want = dir.x.atan2(dir.z);
        let diff = (want - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        m.yaw += diff * (dt * 6.0).min(1.0);
    }
}

/// Fienden går langs stigen eller heisen (`Mob.climb`), utenom nettet, i
/// stigefarten eller heisfarten; heisen kjøres når den går om bord. I selve
/// stigen vender den ansiktet mot den og spiller klatreklippet (`climb`, med
/// fasen fra høyden), eller gangen om modellen ikke har det; på heisen står
/// den. Gir om den er på en lenke.
fn climb(m: &mut Mob, dt: f32, clips: &Clips, out: &mut Outcome) -> bool {
    if m.climb.is_empty() {
        return false;
    }
    let from = m.pos;
    let now = m.climb[0];
    let mut left = dt;
    while left > 0.0 {
        let Some(step) = m.climb.first_mut() else { break };
        if let Some(n) = step.ride.take() {
            out.lifts.push(n);
        }
        let d = step.to - m.pos;
        let reach = step.speed * left;
        if d.length() <= reach {
            m.pos = step.to;
            left -= d.length() / step.speed.max(1e-3);
            m.climb.remove(0);
        } else {
            m.pos += d.normalize_or_zero() * reach;
            left = 0.0;
        }
    }
    let flat = (m.pos - from).with_y(0.0);
    let riding = !now.climb && (now.to.y - from.y).abs() > (now.to - from).with_y(0.0).length();
    m.state = State::Walk;
    if now.climb {
        if now.face.length_squared() > 0.5 {
            let want = now.face.x.atan2(now.face.z);
            let diff = (want - m.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
            m.yaw += diff * (dt * 10.0).min(1.0);
        }
        if let Some(a) = m.anim.as_mut() {
            a.play(clips.climb.map_or(clips.walk, |c| c.0), true, 0.2);
            if let Some((_, len)) = clips.climb {
                // Klippet følger høyden på stigen, ikke klokka: fasen fra
                // høyden over foten delt på trinnavstanden (`climb_phase`),
                // så føttene står på trinnene opp og ned.
                let ph = crate::domain::world::crypt::hops::climb_phase(m.pos.y - now.foot, now.rung);
                a.time = ph * len;
                a.speed = if now.to.y >= from.y { 1.0 } else { -1.0 };
            } else {
                a.speed = 1.0;
            }
        }
    } else {
        if flat.length_squared() > 1e-6 {
            m.yaw = flat.x.atan2(flat.z);
        }
        if let Some(a) = m.anim.as_mut() {
            a.play(if riding { clips.idle } else { clips.walk }, true, 0.2);
            a.speed = 1.0;
        }
    }
    if m.climb.is_empty() {
        // Av lenken: ny vei herfra, og gangen igjen.
        m.repath = 0.0;
        if let Some(a) = m.anim.as_mut() {
            a.play(clips.walk, true, 0.2);
            a.speed = 1.0;
        }
    }
    true
}
