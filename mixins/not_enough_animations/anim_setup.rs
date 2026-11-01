fn not_enough_animations_anim_setup(
    value: (),
    m: &crate::mobs::Mob,
    s: &crate::mobs::anim::State,
    p: &mut crate::mobs::anim::Poses,
) {
    let _ = value;
    if !matches!(m.kind.anim, Anim::Humanoid { .. }) {
        return;
    }
    crate::mod_not_enough_animations::apply_nea(p, s, m);
}
