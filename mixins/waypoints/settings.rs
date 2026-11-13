// The settings page.
fn waypoints_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    crate::game::waypoints_glue::page(game, mod_id, buttons);
}
