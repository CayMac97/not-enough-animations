fn not_enough_animations_frame(
    game: &mut crate::game::Game<'_>,
    elapsed: std::time::Duration,
) {
    crate::game::mod_not_enough_animations_glue::on_native_frame(game, elapsed);
}
