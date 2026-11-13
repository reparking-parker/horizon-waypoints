// The markers on the HUD.
fn waypoints_hud(game: &mut crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    crate::game::waypoints_glue::hud(game, batch);
}
