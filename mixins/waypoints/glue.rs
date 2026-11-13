// The game side of Waypoints: its state, the files in its folder, and what the hooks do.
// A child of `game`, so it reads the game through the native mod helpers only.
use super::Game;
use super::waypoints_mod as m;
use std::cell::{Cell, RefCell};

thread_local! {
    /// The waypoints and settings, read once from the mod's folder.
    static STATE: RefCell<Option<m::Waypoints>> = const { RefCell::new(None) };
    /// This frame's camera, as vanilla placed it: position, yaw and pitch (F5's views
    /// included), and the world's field of view (zoom included).
    static POSE: Cell<Option<([f64; 3], f32, f32)>> = const { Cell::new(None) };
    static FOV: Cell<f32> = const { Cell::new(70.0) };
    /// Where the player last stood alive, and whether the death screen was up last tick.
    static ALIVE: RefCell<Option<m::Here>> = const { RefCell::new(None) };
    static DEAD: Cell<bool> = const { Cell::new(false) };
}

fn with<T>(game: &Game<'_>, f: impl FnOnce(&mut m::Waypoints) -> T) -> T {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let wp = s.get_or_insert_with(|| {
            let list = game.native_setting(m::ID, m::WAYPOINTS_FILE);
            let settings = game.native_setting(m::ID, m::SETTINGS_FILE);
            m::Waypoints::load(list.as_deref(), settings.as_deref())
        });
        f(wp)
    })
}

/// Writes both files.
fn save(game: &Game<'_>) {
    let files = STATE.with(|s| s.borrow().as_ref().map(m::Waypoints::files));
    if let Some((list, settings)) = files {
        game.native_set_setting(m::ID, m::WAYPOINTS_FILE, &list);
        game.native_set_setting(m::ID, m::SETTINGS_FILE, &settings);
    }
}

fn here(game: &Game<'_>) -> Option<m::Here> {
    let p = game.native_player()?;
    Some(m::Here { x: p.x, y: p.y, z: p.z, dim: p.dimension.to_owned() })
}

/// The mod's row in Options > Mods, with its keys.
pub(super) fn listed(mods: &mut Vec<super::native_hooks::NativeMod>) {
    mods.push(super::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: vec![m::ADD_KEY.map(String::from), m::LIST_KEY.map(String::from)],
    });
}

/// B adds a waypoint where the player stands, U lists them in chat.
pub(super) fn key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(m::ID) || !game.native_playing() {
        return;
    }
    let [add, _, add_default] = m::ADD_KEY;
    let [list, _, list_default] = m::LIST_KEY;
    let Some(h) = here(game) else { return };
    let lines = if key == game.native_mod_key(m::ID, add, add_default) {
        let line = with(game, |w| w.put("", h.block(), &h.dim, None));
        save(game);
        vec![line]
    } else if key == game.native_mod_key(m::ID, list, list_default) {
        with(game, |w| w.list_lines(Some(&h.dim)))
    } else {
        return;
    };
    for line in lines {
        game.native_chat(&line);
    }
}

/// `/wp …`.
pub(super) fn command(game: &mut Game<'_>, command: &str, ran: &mut bool) {
    let (name, args) = command.split_once(' ').unwrap_or((command, ""));
    if name != "wp" || !game.native_on(m::ID) {
        return;
    }
    *ran = true;
    let h = here(game);
    let out = with(game, |w| w.run(args, h.as_ref()));
    if out.changed {
        save(game);
    }
    for line in &out.chat {
        game.native_chat(line);
    }
    if let Some(send) = out.send
        && let Err(e) = game.native_send_chat(&send)
    {
        game.native_chat(&format!("Can't teleport: {e}"));
    }
}

pub(super) fn commands(commands: &mut Vec<&'static str>) {
    commands.push("wp");
}

pub(super) fn page(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == m::ID {
        buttons.extend(with(game, |w| w.page()));
    }
}

pub(super) fn click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if mod_id == m::ID && with(game, |w| w.click(button)) {
        save(game);
    }
}

/// Each client tick: the death screen coming up puts the death waypoint where the player
/// last stood alive.
pub(super) fn tick(game: &mut Game<'_>) {
    if !game.native_on(m::ID) {
        return;
    }
    let state = game.native_client_state();
    let dead = state.is_some_and(|s| matches!(s.screen, super::native_client_services::NativeScreenKind::Death));
    if dead && !DEAD.get() {
        let last = ALIVE.with(|a| a.borrow_mut().take());
        if let Some(line) = last.and_then(|h| with(game, |w| w.died(&h))) {
            save(game);
            game.native_chat(&line);
        }
    } else if !dead {
        let h = here(game);
        ALIVE.with(|a| *a.borrow_mut() = h);
    }
    DEAD.set(dead);
}

pub(super) fn pose(position: &[f64; 3], yaw: f32, pitch: f32) {
    POSE.set(Some((*position, yaw, pitch)));
}

pub(super) fn fov(degrees: f32) {
    if degrees.is_finite() {
        FOV.set(degrees);
    }
}

/// The markers of the player's dimension, and the name and distance of the one nearest the
/// crosshair (or of every one, as the settings say).
pub(super) fn hud(game: &mut Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    if !game.native_on(m::ID) {
        return;
    }
    let (Some(p), Some(state)) = (game.native_player(), game.native_client_state()) else {
        return;
    };
    let [w, h] = state.gui_size;
    let camera = game.native_camera();
    let (eye, project): ([f64; 3], Box<dyn Fn([f64; 3]) -> Option<(f32, f32)>>) = match camera {
        Some(c) => (c.position, Box::new(move |d| m::project(&c.view, &c.projection, d, w, h))),
        None => {
            let Some((at, yaw, pitch)) = POSE.get() else { return };
            let fov = FOV.get();
            (at, Box::new(move |d| m::project_pose(yaw, pitch, fov, d, w, h)))
        }
    };
    let (size, labels, mut marks) = with(game, |wp| {
        let s = &wp.settings;
        if !s.markers {
            return (0, m::LABELS[s.labels], Vec::new());
        }
        let marks: Vec<(f32, f32, f64, String, u32)> = wp
            .markers(p.dimension)
            .filter_map(|m| {
                let d = [f64::from(m.x) + 0.5 - eye[0], f64::from(m.y) + 1.0 - eye[1], f64::from(m.z) + 0.5 - eye[2]];
                let (x, y) = project(d)?;
                let far = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                Some((x, y, far, m.name.clone(), m.rgb()))
            })
            .collect();
        (m::MARKER_SIZES[s.size].0, m::LABELS[s.labels], marks)
    });
    // Farthest first, so the nearest draw on top.
    marks.sort_by(|a, b| b.2.total_cmp(&a.2));
    let marks = &marks[marks.len().saturating_sub(500)..];
    for (x, y, _, _, rgb) in marks {
        for (left, top, width) in m::diamond(size) {
            let rect = [(x + left).round(), (y + top).round(), width, 1.0];
            game.native_fill(batch, rect, m::MARKER_ALPHA << 24 | rgb);
        }
    }
    let off = |x: f32, y: f32| (x - w / 2.0).hypot(y - h / 2.0);
    let named: Vec<&(f32, f32, f64, String, u32)> = match labels {
        "All" => marks.iter().collect(),
        "Off" => Vec::new(),
        _ => marks
            .iter()
            .filter(|m| off(m.0, m.1) <= m::LABEL_RADIUS)
            .min_by(|a, b| off(a.0, a.1).total_cmp(&off(b.0, b.1)))
            .into_iter()
            .collect(),
    };
    for (x, y, far, name, rgb) in named {
        let text = format!("{name} {far:.0} m");
        let tw = game.native_text_width(&text);
        game.native_text(batch, &text, [(x - tw / 2.0).round(), (y + size as f32 + 3.0).round()], *rgb, true);
    }
}
