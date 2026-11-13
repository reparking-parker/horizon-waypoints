# Waypoints (Horizon native mod)

Named waypoints for [Horizon](https://github.com/horizon-rust/horizon), with small markers on screen.

- **B** adds a waypoint where you stand, **U** lists them in chat (change both in Options > Mods > Waypoints).
- Markers show your waypoints in the dimension you're in, with the name and distance of the one nearest the crosshair.
- When you die, a **Death** waypoint marks the spot.
- `/wp help` lists the commands: `add`, `remove`, `rename`, `hide`, `show`, `color`, `list`, `tp`, `group`, `markers`, `death`.
- Options > Mods > Waypoints > Settings: markers on or off, marker size, which names show, the death waypoint, the
  group, and a row per waypoint to show or hide it.

## Without the launcher window

`windows/le_horizon.bat` runs Horizon with no launcher window, for slow computers. Put it next to
`horizon-launcher.exe` (or install the launcher from playhorizon.dev) and double-click it every time. The first run
signs in, installs Horizon, offers to install this mod and applies fast settings. Each run updates the launcher and
Horizon, then shows a menu: play, switch Horizon version (it rebuilds the mods on the new one), mods, servers,
worlds, resource and shader packs, accounts, screenshots, logs and settings.

The commands it uses for this mod:

```bat
horizon-launcher.exe mods add-github https://github.com/reparking-parker/horizon-waypoints --trust-native-code
horizon-launcher.exe native-build --ignore-pins --trust-native-code
horizon-launcher.exe mods update-github waypoints --trust-native-code
```

`--ignore-pins` builds the mod on the Horizon you play rather than the Nightly it was tested on; it needs Nightly
2026-11-08 or newer.

## Your old waypoints

This is the native version of the WebAssembly Waypoints mod, which Horizon stopped running on 2026-10-29. It keeps that
mod's id, so it reads the same files: `horizon-mods/.horizon-mod-data/waypoints/waypoints` and `.../settings` in your
game folder (on Windows, `%APPDATA%\horizon\horizon-mods\.horizon-mod-data\waypoints\`). If those files are still
there, your waypoints come back as they were.

## Install

Native mods are Rust code the Horizon launcher compiles into the game on your computer:

1. In the launcher, turn on Settings > Game > Native mods.
2. On the launcher's Mods page, choose **Add from GitHub** and paste this repository's link, then press **Build**. The first build downloads Rust (about 450 MB)
   and compiles the game, which takes 10 to 30 minutes. Later builds are faster.
3. Play the **Native mods** instance the build makes, and turn Waypoints on in Options > Mods.

## Made on the hook points

It only uses Horizon's native hook points (`compatibility.json`: `hooks`), so it builds on newer Nightlies without a
new release. Tested on Nightly `2026-11-08-fdc87ae`.

| Hook | What the mod does there |
|---|---|
| `native_hud` | draws the markers and names |
| `native_key` | B and U |
| `native_command`, `native_commands` | `/wp` |
| `native_mods`, `native_settings`, `native_settings_click` | the Options > Mods row and settings page |
| `native_client_tick_post` | the death waypoint, when the death screen comes up |
| `native_camera_pose`, `native_compute_fov` | the frame's camera and field of view, to place the markers |

`cargo test` runs the mod's own tests (`mixins/waypoints/runtime.rs`).

Tested on Linux: old waypoint files loading, `/wp add` and `list`, markers on screen and the death waypoint.

Not carried over from the WebAssembly mod: the U menu screen (U lists in chat instead; edit with `/wp` and the settings
page), and the field of view, zoom and F5 settings (the native mod reads the real camera).
