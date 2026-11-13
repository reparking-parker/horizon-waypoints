// B and U.
fn waypoints_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::waypoints_glue::key(game, key, pressed);
}
