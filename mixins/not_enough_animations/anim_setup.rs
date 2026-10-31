fn not_enough_animations_anim_setup(
    m: &crate::mobs::Mob,
    s: &crate::mobs::anim::State,
    p: &mut crate::mobs::anim::Poses,
    w_anims: &std::collections::HashMap<String, crate::mobs::keyframes::Animation>,
) -> core::ops::ControlFlow<()> {
    let _ = w_anims;
    if !matches!(m.kind.anim, Anim::Humanoid { zombie_arms: false }) {
        return core::ops::ControlFlow::Continue(());
    }

    // Run standard humanoid setup for player
    let player = true;
    let using_main = m.extra.using_item > 0 && !m.extra.using_off_hand;
    let spear_in = |hand: &Option<std::sync::Arc<str>>| {
        hand.as_deref()
            .filter(|_| player)
            .and_then(crate::spear::Kinetic::of)
    };
    let (main_spear, off_spear) =
        (spear_in(&m.extra.main_hand), spear_in(&m.extra.off_hand));
    let pose = match m.extra.main_hand.as_deref() {
        Some(_) if using_main && m.extra.blocking => ArmPose::Block,
        Some("minecraft:bow") if using_main => ArmPose::Bow,
        Some("minecraft:trident") if using_main => ArmPose::ThrowTrident,
        Some("minecraft:crossbow") if using_main && !m.extra.charged => {
            ArmPose::CrossbowCharge
        }
        Some("minecraft:crossbow") if m.extra.charged && s.swing == 0.0 => {
            ArmPose::CrossbowHold
        }
        Some(_) if main_spear.is_some() => ArmPose::Spear,
        Some(_) => ArmPose::Item,
        None => ArmPose::Empty,
    };
    let off = if m.extra.off_hand.is_some() && !pose.affects_offhand() {
        if off_spear.is_some() {
            ArmPose::Spear
        } else if m.extra.using_item > 0 && m.extra.using_off_hand && m.extra.blocking {
            ArmPose::Block
        } else {
            ArmPose::Item
        }
    } else {
        ArmPose::Empty
    };
    let mut h = if m.main_arm_left() {
        Hum::new(off, pose)
    } else {
        Hum::new(pose, off)
    };
    h.charge = (m.extra.ints[5] as f32, m.extra.using_item as f32);
    h.fall_flying = m.extra.fall_flying;
    if h.fall_flying {
        let v = m.velocity;
        h.fly_speed_scale =
            (((v.x * v.x + v.y * v.y + v.z * v.z) as f32 / 0.2).powi(3)).max(1.0);
    }
    h.crouching = m.extra.crouching;
    h.passenger = m.extra.passenger;
    h.swim = m.extra.swim.amount(s.partial);
    h.using_item = m.extra.using_item > 0;
    h.swing_right_arm = m.extra.swing_right_arm;
    h.stab = m.extra.swing_stab;
    h.left_main = m.main_arm_left();
    let main_side = usize::from(m.main_arm_left());
    h.spear[main_side] = main_spear;
    h.spear[1 - main_side] = off_spear;
    if m.extra.using_item > 0 {
        let using_side = if m.extra.using_off_hand {
            1 - main_side
        } else {
            main_side
        };
        h.spear_ticks[using_side] = m.extra.using_item as f32 + s.partial;
    }
    humanoid(p, s, &h);
    let hidden = m.extra.hidden_model_parts;
    for (bit, part) in [
        (1, "jacket"),
        (2, "left_sleeve"),
        (3, "right_sleeve"),
        (4, "left_pants"),
        (5, "right_pants"),
        (6, "hat"),
    ] {
        p.visible(part, hidden & (1 << bit) == 0);
    }

    // Apply Not Enough Animations
    crate::mod_not_enough_animations::apply_nea(p, s, m);

    core::ops::ControlFlow::Break(())
}
