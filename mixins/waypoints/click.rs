// A click on the settings page.
fn waypoints_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    crate::game::waypoints_glue::click(game, mod_id, button);
}
