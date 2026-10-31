fn not_enough_animations_list(
    mods: &mut Vec<crate::game::native_hooks::NativeMod>,
) {
    crate::game::mod_not_enough_animations_glue::on_native_mods(mods);
}
