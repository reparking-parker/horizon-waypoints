// /wp.
fn waypoints_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    crate::game::waypoints_glue::command(game, command, ran);
}
