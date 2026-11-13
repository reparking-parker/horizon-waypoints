// The row in Options > Mods.
fn waypoints_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::waypoints_glue::listed(mods);
}
