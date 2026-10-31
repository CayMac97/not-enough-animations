fn not_enough_animations_key(
    game: &mut crate::game::Game<'_>,
    key: &str,
    pressed: bool,
) {
    crate::game::mod_not_enough_animations_glue::on_native_key(game, key, pressed);
}
