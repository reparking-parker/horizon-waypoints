// The camera of this frame.
fn waypoints_pose(position: &[f64; 3], yaw: f32, pitch: f32) {
    crate::game::waypoints_glue::pose(position, yaw, pitch);
}
