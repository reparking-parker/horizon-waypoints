// Each client tick: the death waypoint.
fn waypoints_tick(game: &mut crate::game::Game<'_>) {
    crate::game::waypoints_glue::tick(game);
}
