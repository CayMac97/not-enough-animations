fn not_enough_animations_click(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    button: usize,
) {
    crate::game::mod_not_enough_animations_glue::on_native_settings_click(game, mod_id, button);
}
