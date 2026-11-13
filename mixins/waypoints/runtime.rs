// Waypoints, a Horizon native mod: named spots kept per group and dimension, small
// see-through markers on screen with the name and distance of the one nearest the
// crosshair, a death waypoint, B to add one where the player stands and /wp to manage
// them. It was a WebAssembly mod first: it keeps that mod's id, so its two files in
// `horizon-mods/.horizon-mod-data/waypoints/` (`waypoints`, `settings`) load as they are.
// This file is both the mod's Cargo library (its tests) and the module the bundle adds
// to the game; it knows nothing of the game's types.

pub const ID: &str = "waypoints";
pub const NAME: &str = "Waypoints";
pub const VERSION: &str = "2.0.0";
pub const DESCRIPTION: &str = "Named waypoints with markers on screen, a death waypoint, B to add one where you stand and /wp to manage them.";
/// The two files in the mod's folder.
pub const WAYPOINTS_FILE: &str = "waypoints";
pub const SETTINGS_FILE: &str = "settings";
/// The mod's keys: name, label and default key.
pub const ADD_KEY: [&str; 3] = ["add", "Add a waypoint here", "key.keyboard.b"];
pub const LIST_KEY: [&str; 3] = ["menu", "List the waypoints in chat", "key.keyboard.u"];

/// Vanilla's chat colours.
pub const COLORS: [(&str, u32); 16] = [
    ("black", 0x000000),
    ("dark_blue", 0x0000AA),
    ("dark_green", 0x00AA00),
    ("dark_aqua", 0x00AAAA),
    ("dark_red", 0xAA0000),
    ("dark_purple", 0xAA00AA),
    ("gold", 0xFFAA00),
    ("gray", 0xAAAAAA),
    ("dark_gray", 0x555555),
    ("blue", 0x5555FF),
    ("green", 0x55FF55),
    ("aqua", 0x55FFFF),
    ("red", 0xFF5555),
    ("light_purple", 0xFF55FF),
    ("yellow", 0xFFFF55),
    ("white", 0xFFFFFF),
];
/// Colours new waypoints take in turn.
const NEW_COLORS: [u8; 7] = [12, 9, 10, 14, 13, 11, 6];
pub const DEATH_NAME: &str = "Death";
const DEATH_COLOR: u8 = 4;
/// A marker's half size in GUI pixels, by the settings page's choice.
pub const MARKER_SIZES: [(i32, &str); 3] = [(2, "Small"), (3, "Medium"), (5, "Large")];
/// Which markers show their name and distance.
pub const LABELS: [&str; 3] = ["Near crosshair", "All", "Off"];
/// Markers are drawn this see-through (alpha of 0xAARRGGBB).
pub const MARKER_ALPHA: u32 = 0xB0;
/// Only the marker nearest the crosshair, within this many GUI pixels, shows its name.
pub const LABEL_RADIUS: f32 = 40.0;
/// The settings page's buttons before the waypoints' rows.
const BUTTONS: usize = 5;

#[derive(Clone, Debug, PartialEq)]
pub struct Waypoint {
    pub group: String,
    pub dim: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub color: u8,
    pub shown: bool,
}

impl Waypoint {
    /// One line of the `waypoints` file: group, dimension, name, x, y, z, colour, shown.
    pub fn line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.group,
            self.dim,
            self.name,
            self.x,
            self.y,
            self.z,
            COLORS[usize::from(self.color)].0,
            self.shown
        )
    }
    pub fn parse(line: &str) -> Option<Self> {
        let f: Vec<&str> = line.split('\t').collect();
        let [group, dim, name, x, y, z, color, shown] = f.as_slice() else {
            return None;
        };
        Some(Self {
            group: (*group).to_owned(),
            dim: (*dim).to_owned(),
            name: (*name).to_owned(),
            x: x.trim().parse().ok()?,
            y: y.trim().parse().ok()?,
            z: z.trim().parse().ok()?,
            color: color_index(color).unwrap_or(15),
            shown: shown.trim() != "false",
        })
    }
    pub fn rgb(&self) -> u32 {
        COLORS[usize::from(self.color)].1
    }
    fn coords(&self) -> String {
        format!("{} {} {}", self.x, self.y, self.z)
    }
}

/// The `settings` file, `key=value` lines. `fov` and `zoom` were the WebAssembly mod's
/// guesses at the camera, which the native mod reads instead; they stay in the file.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub group: String,
    fov: f32,
    zoom: f32,
    pub markers: bool,
    pub death: bool,
    /// Indexes into `MARKER_SIZES` and `LABELS`.
    pub size: usize,
    pub labels: usize,
}

impl Settings {
    pub fn parse(text: &str) -> Self {
        let mut s = Self {
            group: "default".to_owned(),
            fov: 70.0,
            zoom: 4.0,
            markers: true,
            death: true,
            size: 1,
            labels: 0,
        };
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "group" if !value.is_empty() => s.group = clean(value),
                "fov" => s.fov = value.parse().unwrap_or(s.fov),
                "zoom" => s.zoom = value.parse().unwrap_or(s.zoom),
                "markers" => s.markers = value != "false",
                "death" => s.death = value != "false",
                "size" => s.size = value.parse().map_or(s.size, |i: usize| i.min(MARKER_SIZES.len() - 1)),
                "labels" => s.labels = value.parse().map_or(s.labels, |i: usize| i.min(LABELS.len() - 1)),
                _ => {}
            }
        }
        s
    }
    pub fn text(&self) -> String {
        format!(
            "group={}\nfov={}\nzoom={}\nmarkers={}\ndeath={}\nsize={}\nlabels={}\n",
            self.group, self.fov, self.zoom, self.markers, self.death, self.size, self.labels
        )
    }
}

/// Where the player is, for the commands.
pub struct Here {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub dim: String,
}

impl Here {
    pub fn block(&self) -> (i32, i32, i32) {
        (block(self.x), block(self.y), block(self.z))
    }
}

/// What a command did: lines for chat, whether the files change, and a command for the server.
#[derive(Default, Debug)]
pub struct Outcome {
    pub chat: Vec<String>,
    pub changed: bool,
    pub send: Option<String>,
}

impl Outcome {
    fn say(text: impl Into<String>) -> Self {
        Self { chat: vec![text.into()], ..Self::default() }
    }
    fn saved(text: impl Into<String>) -> Self {
        Self { chat: vec![text.into()], changed: true, send: None }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Waypoints {
    pub list: Vec<Waypoint>,
    pub settings: Settings,
}

/// A name or group as stored: no tabs or line breaks, at most 64 characters.
pub fn clean(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(64)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn color_index(name: &str) -> Option<u8> {
    let name = name.trim().to_ascii_lowercase().replace(' ', "_");
    COLORS
        .iter()
        .position(|c| c.0 == name)
        .or_else(|| name.parse::<usize>().ok().filter(|&i| i < COLORS.len()))
        .and_then(|i| u8::try_from(i).ok())
}

pub fn dim_label(dim: &str) -> &str {
    match dim {
        "minecraft:overworld" => "Overworld",
        "minecraft:the_nether" => "Nether",
        "minecraft:the_end" => "End",
        other => other.strip_prefix("minecraft:").unwrap_or(other),
    }
}

/// A coordinate typed after a name: a whole number, `~` for the player's, or `~5`.
fn coordinate(token: &str, here: i32) -> Option<i32> {
    match token {
        "~" => Some(here),
        t => t
            .strip_prefix('~')
            .map_or_else(|| t.parse().ok(), |d| d.parse::<i32>().ok().map(|d| here + d)),
    }
}

pub fn block(v: f64) -> i32 {
    // Block coordinates are well inside i32.
    v.floor() as i32
}

const HELP: [&str; 11] = [
    "/wp add [name] [x y z]: add a waypoint (here, or at x y z)",
    "/wp remove <name>: delete one",
    "/wp rename <name> <new name>: rename one",
    "/wp hide <name> / show <name>: hide or show its marker",
    "/wp color <name> <red|blue|green|gold|...>",
    "/wp list: the group's waypoints",
    "/wp tp <name>: teleport there (needs permission)",
    "/wp group [name]: show or switch to another set",
    "/wp markers: markers on screen on or off",
    "/wp death: the death waypoint on or off",
    "B adds a waypoint where you stand, U lists them",
];

impl Waypoints {
    /// The mod's two files, `None` when one isn't there yet.
    pub fn load(waypoints: Option<&str>, settings: Option<&str>) -> Self {
        Self {
            list: waypoints.unwrap_or_default().lines().filter_map(Waypoint::parse).collect(),
            settings: Settings::parse(settings.unwrap_or_default()),
        }
    }
    /// What the two files hold.
    pub fn files(&self) -> (String, String) {
        let lines = self.list.iter().map(|w| w.line() + "\n").collect();
        (lines, self.settings.text())
    }

    fn groups(&self) -> Vec<String> {
        let mut groups = vec![self.settings.group.clone()];
        for w in &self.list {
            if !groups.contains(&w.group) {
                groups.push(w.group.clone());
            }
        }
        groups
    }

    /// The active group's waypoints, by index into `list`.
    pub fn in_group(&self) -> Vec<usize> {
        (0..self.list.len()).filter(|&i| self.list[i].group == self.settings.group).collect()
    }

    /// The active group's waypoint called `name` (any case), the player's dimension's first.
    fn find(&self, name: &str, dim: Option<&str>) -> Option<usize> {
        let matches = |w: &Waypoint| w.group == self.settings.group && w.name.eq_ignore_ascii_case(name.trim());
        self.list
            .iter()
            .position(|w| matches(w) && Some(w.dim.as_str()) == dim)
            .or_else(|| self.list.iter().position(matches))
    }

    fn named(&self, name: &str, dim: Option<&str>) -> Result<usize, String> {
        if name.trim().is_empty() {
            return Err("Name a waypoint: /wp list shows them".to_owned());
        }
        self.find(name, dim)
            .ok_or_else(|| format!("No waypoint \"{}\" in group \"{}\"", name.trim(), self.settings.group))
    }

    /// Adds `name` (or moves it) to `at` in `dim`.
    pub fn put(&mut self, name: &str, at: (i32, i32, i32), dim: &str, color: Option<u8>) -> String {
        let name = if name.is_empty() {
            let n = self.in_group().len();
            format!("Waypoint {}", n + 1)
        } else {
            clean(name)
        };
        let group = self.settings.group.clone();
        let here = self
            .list
            .iter()
            .position(|w| w.group == group && w.dim == dim && w.name.eq_ignore_ascii_case(&name));
        let (x, y, z) = at;
        if let Some(i) = here {
            let w = &mut self.list[i];
            (w.x, w.y, w.z) = at;
            w.shown = true;
            if let Some(c) = color {
                w.color = c;
            }
        } else {
            let color = color.unwrap_or(NEW_COLORS[self.list.len() % NEW_COLORS.len()]);
            self.list.push(Waypoint { group, dim: dim.to_owned(), name: name.clone(), x, y, z, color, shown: true });
        }
        format!(
            "{} \"{name}\" at {x} {y} {z} ({})",
            if here.is_some() { "Moved" } else { "Added" },
            dim_label(dim)
        )
    }

    /// The death waypoint, where the player died.
    pub fn died(&mut self, here: &Here) -> Option<String> {
        if !self.settings.death {
            return None;
        }
        let at = here.block();
        self.put(DEATH_NAME, at, &here.dim, Some(DEATH_COLOR));
        Some(format!("Death waypoint at {} {} {}", at.0, at.1, at.2))
    }

    pub fn list_lines(&self, dim: Option<&str>) -> Vec<String> {
        let mut lines = vec![format!("Waypoints in group \"{}\":", self.settings.group)];
        let mut mine: Vec<&Waypoint> = self.in_group().into_iter().map(|i| &self.list[i]).collect();
        // The player's dimension first.
        mine.sort_by_key(|w| Some(w.dim.as_str()) != dim);
        if mine.is_empty() {
            lines.push("none yet: press B or /wp add <name>".to_owned());
        }
        for w in mine {
            let hidden = if w.shown { "" } else { " (hidden)" };
            lines.push(format!("{}: {} ({}){hidden}", w.name, w.coords(), dim_label(&w.dim)));
        }
        lines
    }

    /// `/wp <args>`.
    pub fn run(&mut self, args: &str, here: Option<&Here>) -> Outcome {
        let args = args.trim();
        let (verb, rest) = args.split_once(char::is_whitespace).unwrap_or((args, ""));
        let rest = rest.trim();
        let dim = here.map(|h| h.dim.as_str());
        let result = match verb.to_ascii_lowercase().as_str() {
            "" | "help" => Ok(Outcome { chat: HELP.iter().map(|l| (*l).to_owned()).collect(), ..Outcome::default() }),
            "add" | "set" => self.add(rest, here),
            "remove" | "delete" | "del" | "rm" => self.named(rest, dim).map(|i| {
                let w = self.list.remove(i);
                Outcome::saved(format!("Removed \"{}\"", w.name))
            }),
            "hide" | "show" => self.named(rest, dim).map(|i| {
                self.list[i].shown = verb.eq_ignore_ascii_case("show");
                let w = &self.list[i];
                Outcome::saved(format!("\"{}\" is {}", w.name, if w.shown { "shown" } else { "hidden" }))
            }),
            "color" | "colour" => self.color(rest, dim),
            "list" | "ls" => Ok(Outcome { chat: self.list_lines(dim), ..Outcome::default() }),
            "tp" => self.named(rest, dim).and_then(|i| {
                let w = &self.list[i];
                if dim.is_some_and(|d| d != w.dim) {
                    return Err(format!("\"{}\" is in the {}", w.name, dim_label(&w.dim)));
                }
                Ok(Outcome { send: Some(format!("/tp @s {}.5 {} {}.5", w.x, w.y, w.z)), ..Outcome::default() })
            }),
            "group" => Ok(if rest.is_empty() {
                Outcome::say(format!("Group \"{}\"; groups: {}", self.settings.group, self.groups().join(", ")))
            } else {
                self.settings.group = clean(rest);
                Outcome::saved(format!("Now showing group \"{}\"", self.settings.group))
            }),
            "rename" | "mv" => self.rename(rest, dim),
            "markers" => {
                self.settings.markers = !self.settings.markers;
                Ok(Outcome::saved(format!("Markers {}", if self.settings.markers { "shown" } else { "hidden" })))
            }
            "death" => {
                self.settings.death = !self.settings.death;
                Ok(Outcome::saved(format!("Death waypoint {}", if self.settings.death { "on" } else { "off" })))
            }
            other => Err(format!("Unknown /wp {other}: /wp help lists the commands")),
        };
        result.unwrap_or_else(Outcome::say)
    }

    fn add(&mut self, args: &str, here: Option<&Here>) -> Result<Outcome, String> {
        let here = here.ok_or("Join a world first")?;
        let at = here.block();
        let tokens: Vec<&str> = args.split_whitespace().collect();
        let mut target = at;
        let mut name = tokens.as_slice();
        if let [rest @ .., x, y, z] = tokens.as_slice()
            && let (Some(x), Some(y), Some(z)) = (coordinate(x, at.0), coordinate(y, at.1), coordinate(z, at.2))
        {
            target = (x, y, z);
            name = rest;
        }
        Ok(Outcome::saved(self.put(&name.join(" "), target, &here.dim, None)))
    }

    fn color(&mut self, args: &str, dim: Option<&str>) -> Result<Outcome, String> {
        let (name, color) = args.rsplit_once(char::is_whitespace).ok_or("Usage: /wp color <name> <colour>")?;
        let c = color_index(color).ok_or_else(|| {
            let names: Vec<&str> = COLORS.iter().map(|c| c.0).collect();
            format!("Unknown colour \"{color}\": {}", names.join(", "))
        })?;
        let i = self.named(name, dim)?;
        self.list[i].color = c;
        Ok(Outcome::saved(format!("\"{}\" is {}", self.list[i].name, COLORS[usize::from(c)].0)))
    }

    /// `/wp rename <old name> <new name>`: the longest start of the words that names a
    /// waypoint is the old name, the rest the new one (`>` between them also works).
    fn rename(&mut self, args: &str, dim: Option<&str>) -> Result<Outcome, String> {
        const USAGE: &str = "Usage: /wp rename <old name> <new name>";
        let (i, new) = if let Some((old, new)) = args.split_once('>') {
            (self.find(old, dim).ok_or_else(|| format!("No waypoint \"{}\"", old.trim()))?, new.to_owned())
        } else {
            let words: Vec<&str> = args.split_whitespace().collect();
            (1..words.len())
                .rev()
                .find_map(|n| Some((self.find(&words[..n].join(" "), dim)?, words[n..].join(" "))))
                .ok_or(USAGE)?
        };
        let new = clean(&new);
        if new.is_empty() {
            return Err(USAGE.to_owned());
        }
        let old = std::mem::replace(&mut self.list[i].name, new.clone());
        Ok(Outcome::saved(format!("Renamed \"{old}\" to \"{new}\"")))
    }

    /// The settings page: its buttons, then a row per waypoint of the group (a click shows
    /// or hides it).
    pub fn page(&self) -> Vec<String> {
        let on = |b: bool| if b { "ON" } else { "OFF" };
        let s = &self.settings;
        let mut buttons = vec![
            format!("Markers on screen: {}", on(s.markers)),
            format!("Marker size: {}", MARKER_SIZES[s.size].1),
            format!("Names shown: {}", LABELS[s.labels]),
            format!("Death waypoint: {}", on(s.death)),
            format!("Group: {}", s.group),
        ];
        for i in self.in_group() {
            let w = &self.list[i];
            let dim = match w.dim.as_str() {
                "minecraft:overworld" => "O",
                "minecraft:the_nether" => "N",
                "minecraft:the_end" => "E",
                _ => "?",
            };
            buttons.push(format!("{} {dim} {} {}", if w.shown { "[x]" } else { "[ ]" }, w.name, w.coords()));
        }
        buttons
    }

    /// A click on the page's button `i`; true when the files change.
    pub fn click(&mut self, i: usize) -> bool {
        let s = &mut self.settings;
        match i {
            0 => s.markers = !s.markers,
            1 => s.size = (s.size + 1) % MARKER_SIZES.len(),
            2 => s.labels = (s.labels + 1) % LABELS.len(),
            3 => s.death = !s.death,
            4 => {
                let groups = self.groups();
                let at = groups.iter().position(|g| *g == self.settings.group).unwrap_or(0);
                self.settings.group = groups[(at + 1) % groups.len()].clone();
            }
            row => match self.in_group().get(row - BUTTONS) {
                Some(&w) => self.list[w].shown = !self.list[w].shown,
                None => return false,
            },
        }
        true
    }

    /// The markers to draw in `dim`: the shown waypoints of the group.
    pub fn markers<'a>(&'a self, dim: &'a str) -> impl Iterator<Item = &'a Waypoint> {
        let group = &self.settings.group;
        self.list.iter().filter(move |w| w.shown && w.group == *group && w.dim == dim)
    }
}

/// Where world offset `d` from the camera lands on a `w` × `h` GUI, through the camera's
/// camera-relative `view` and `projection` matrices (column-major); none behind it or off
/// screen.
pub fn project(view: &[f32; 16], projection: &[f32; 16], d: [f64; 3], w: f32, h: f32) -> Option<(f32, f32)> {
    let mul = |m: &[f32; 16], v: [f64; 4]| -> [f64; 4] {
        let mut out = [0.0; 4];
        for (row, o) in out.iter_mut().enumerate() {
            *o = (0..4).map(|col| f64::from(m[col * 4 + row]) * v[col]).sum();
        }
        out
    };
    let clip = mul(projection, mul(view, [d[0], d[1], d[2], 1.0]));
    if clip[3] <= 0.05 {
        return None;
    }
    let (nx, ny) = (clip[0] / clip[3], clip[1] / clip[3]);
    let x = (nx + 1.0) / 2.0 * f64::from(w);
    let y = (1.0 - ny) / 2.0 * f64::from(h);
    let inside = (0.0..=f64::from(w)).contains(&x) && (0.0..=f64::from(h)).contains(&y);
    inside.then_some((x as f32, y as f32))
}

/// Where world offset `d` from a camera turned `yaw`, `pitch` (degrees; yaw 0 faces south,
/// +Z) with a vertical field of view of `fov` degrees lands on a `w` × `h` GUI; none behind it
/// or off screen. For when the game gives no matrices: the frame's own camera pose.
pub fn project_pose(yaw: f32, pitch: f32, fov: f32, d: [f64; 3], w: f32, h: f32) -> Option<(f32, f32)> {
    let (yaw, pitch) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    let ahead = [-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos()];
    let right = [-yaw.cos(), 0.0, -yaw.sin()];
    let up = [-yaw.sin() * pitch.sin(), pitch.cos(), yaw.cos() * pitch.sin()];
    let dot = |a: [f64; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
    let (r, u, a) = (dot(right), dot(up), dot(ahead));
    if a <= 0.05 {
        return None;
    }
    let (cx, cy) = (f64::from(w) / 2.0, f64::from(h) / 2.0);
    let tan = (f64::from(fov).to_radians() / 2.0).tan();
    let aspect = f64::from(w) / f64::from(h).max(1.0);
    let x = cx + r / (a * tan * aspect) * cx;
    let y = cy - u / (a * tan) * cy;
    let inside = (0.0..=f64::from(w)).contains(&x) && (0.0..=f64::from(h)).contains(&y);
    inside.then_some((x as f32, y as f32))
}

/// A diamond of half size `s` as rows of fill: (left, top, width) from its centre.
pub fn diamond(s: i32) -> Vec<(f32, f32, f32)> {
    (-s..=s)
        .map(|dy| {
            let half = s - dy.abs();
            (-half as f32, dy as f32, (2 * half + 1) as f32)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD_WAYPOINTS: &str = "default\tminecraft:overworld\tStronghold\t-4491\t-42\t605\tgreen\ttrue\n\
default\tminecraft:the_nether\tfortress\t-777\t61\t-222\tdark_green\tfalse\n";
    const OLD_SETTINGS: &str = "group=default\nfov=105\nzoom=4\nmarkers=true\ndeath=true\nsize=1\nlabels=0\n";

    fn here() -> Here {
        Here { x: 10.7, y: 64.0, z: -3.2, dim: "minecraft:overworld".into() }
    }

    #[test]
    fn the_webassembly_mods_files_load_and_save_as_they_were() {
        let wp = Waypoints::load(Some(OLD_WAYPOINTS), Some(OLD_SETTINGS));
        assert_eq!(wp.list.len(), 2);
        assert_eq!(wp.list[0].name, "Stronghold");
        assert_eq!((wp.list[0].x, wp.list[0].y, wp.list[0].z), (-4491, -42, 605));
        assert!(!wp.list[1].shown);
        let (lines, settings) = wp.files();
        assert_eq!(lines, OLD_WAYPOINTS);
        assert_eq!(settings, OLD_SETTINGS);
    }

    #[test]
    fn no_files_yet_is_an_empty_default() {
        let wp = Waypoints::load(None, None);
        assert!(wp.list.is_empty());
        assert_eq!(wp.settings.group, "default");
        assert!(wp.settings.markers && wp.settings.death);
    }

    #[test]
    fn commands_add_rename_colour_hide_and_remove() {
        let mut wp = Waypoints::load(None, None);
        let out = wp.run("add Home", Some(&here()));
        assert!(out.changed);
        assert_eq!((wp.list[0].x, wp.list[0].y, wp.list[0].z), (10, 64, -4));
        wp.run("add Far 100 ~5 ~-1", Some(&here()));
        assert_eq!((wp.list[1].x, wp.list[1].y, wp.list[1].z), (100, 69, -5));
        wp.run("rename Far Base Camp", Some(&here()));
        assert_eq!(wp.list[1].name, "Base Camp");
        wp.run("color Base Camp gold", Some(&here()));
        assert_eq!(wp.list[1].color, 6);
        wp.run("hide home", Some(&here()));
        assert!(!wp.list[0].shown);
        assert_eq!(wp.run("tp Base Camp", Some(&here())).send.as_deref(), Some("/tp @s 100.5 69 -5.5"));
        wp.run("remove Base Camp", Some(&here()));
        assert_eq!(wp.list.len(), 1);
        assert!(!wp.run("remove nothing", Some(&here())).changed);
        assert!(wp.run("add", None).chat[0].contains("Join a world"));
    }

    #[test]
    fn dying_puts_the_death_waypoint_unless_turned_off() {
        let mut wp = Waypoints::load(None, None);
        assert!(wp.died(&here()).is_some());
        assert_eq!(wp.list[0].name, DEATH_NAME);
        wp.died(&Here { x: 0.0, y: 70.0, z: 0.0, dim: "minecraft:overworld".into() });
        assert_eq!(wp.list.len(), 1, "moved, not added again");
        wp.run("death", None);
        assert!(wp.died(&here()).is_none());
    }

    #[test]
    fn the_settings_page_toggles_and_its_rows_show_or_hide() {
        let mut wp = Waypoints::load(Some(OLD_WAYPOINTS), Some(OLD_SETTINGS));
        assert_eq!(wp.page().len(), BUTTONS + 2);
        assert!(wp.click(0));
        assert!(!wp.settings.markers);
        assert!(wp.click(BUTTONS + 1));
        assert!(wp.list[1].shown);
        assert!(!wp.click(BUTTONS + 5));
    }

    #[test]
    fn straight_ahead_is_the_middle_and_behind_is_nothing() {
        // Identity view, a plain perspective projection looking down -Z.
        let view = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let f = 1.0 / (35f32.to_radians()).tan();
        let projection = [f, 0.0, 0.0, 0.0, 0.0, f, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.05, 0.0];
        let (x, y) = project(&view, &projection, [0.0, 0.0, -10.0], 400.0, 200.0).unwrap();
        assert!((x - 200.0).abs() < 0.01 && (y - 100.0).abs() < 0.01, "{x} {y}");
        let (_, up) = project(&view, &projection, [0.0, 2.0, -10.0], 400.0, 200.0).unwrap();
        assert!(up < 100.0, "higher is up the screen");
        assert!(project(&view, &projection, [0.0, 0.0, 10.0], 400.0, 200.0).is_none());
    }

    #[test]
    fn a_pose_looking_south_sees_south_in_the_middle_and_east_to_the_left() {
        let (x, y) = project_pose(0.0, 0.0, 70.0, [0.0, 0.0, 10.0], 400.0, 200.0).unwrap();
        assert!((x - 200.0).abs() < 0.01 && (y - 100.0).abs() < 0.01);
        // Facing south (+Z), east (+X) is on the left.
        let (east, _) = project_pose(0.0, 0.0, 70.0, [2.0, 0.0, 10.0], 400.0, 200.0).unwrap();
        assert!(east < 200.0);
        assert!(project_pose(0.0, 0.0, 70.0, [0.0, 0.0, -10.0], 400.0, 200.0).is_none());
        // Looking down (pitch 90), a point below is in the middle.
        let (_, below) = project_pose(0.0, 89.0, 70.0, [0.0, -10.0, 0.1], 400.0, 200.0).unwrap();
        assert!((below - 100.0).abs() < 5.0);
    }

    #[test]
    fn a_diamond_is_widest_in_the_middle() {
        let rows = diamond(2);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[2], (-2.0, 0.0, 5.0));
        assert_eq!(rows[0].2, 1.0);
    }
}
