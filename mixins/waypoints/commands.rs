// /wp in the chat box's list of commands.
fn waypoints_commands(commands: &mut Vec<&'static str>) {
    crate::game::waypoints_glue::commands(commands);
}
