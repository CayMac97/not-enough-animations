fn not_enough_animations_settings(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    crate::game::mod_not_enough_animations_glue::on_native_settings(game, mod_id, buttons);
}
