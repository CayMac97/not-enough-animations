// Not Enough Animations - Glue Module
// Connects native settings, frame updates, and keybinds to the animation runtime.

use crate::game::Game;
use crate::game::native_hooks::NativeMod;
use horizon_client::mod_not_enough_animations as rt;

pub(super) fn on_native_frame(game: &mut Game<'_>, _elapsed: std::time::Duration) {
    let Some(p) = game.player.as_ref() else {
        rt::set_climbing(false);
        rt::set_in_boat(false, false);
        return;
    };

    let feet = [
        p.position.x.floor() as i32,
        p.position.y.floor() as i32,
        p.position.z.floor() as i32,
    ];

    let on_climbable = game.block_in_tag("minecraft:climbable", &game.block_name(feet))
        || game.block_in_tag(
            "minecraft:climbable",
            &game.block_name([feet[0], feet[1] + 1, feet[2]]),
        )
        || p.last_climbable.is_some();

    let is_climbing = !p.flying && !p.ground.on_ground && on_climbable;
    rt::set_climbing(is_climbing);

    let in_boat = game.boats.iter().any(|b| Some(b.id) == game.riding);
    let moving = in_boat && (p.velocity.x.abs() > 0.01 || p.velocity.z.abs() > 0.01);
    rt::set_in_boat(in_boat, moving);
}

pub(super) fn on_native_mods(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: rt::ID.to_string(),
        title: format!("{} {}", rt::NAME, rt::VERSION),
        description: rt::DESCRIPTION.to_string(),
        keys: vec![
            ("toggle".to_string(), "Toggle Animations".to_string(), "key.keyboard.unknown".to_string()).into(),
        ],
    });
}

pub(super) fn on_native_settings(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    let _ = game;
    if mod_id != rt::ID {
        return;
    }
    buttons.push(format!("Animations: {}", if rt::is_enabled() { "Enabled" } else { "Disabled" }));
    buttons.push(format!("Eating & Drinking: {}", if rt::eating_enabled() { "ON" } else { "OFF" }));
    buttons.push(format!("Map Holding: {}", if rt::maps_enabled() { "ON" } else { "OFF" }));
    buttons.push(format!("Ladder Climbing: {}", if rt::climbing_enabled() { "ON" } else { "OFF" }));
    buttons.push(format!("Boat Rowing: {}", if rt::boats_enabled() { "ON" } else { "OFF" }));
}

pub(super) fn on_native_settings_click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    let _ = game;
    if mod_id != rt::ID {
        return;
    }
    match button {
        0 => { rt::toggle_enabled(); }
        1 => { rt::toggle_eating(); }
        2 => { rt::toggle_maps(); }
        3 => { rt::toggle_climbing(); }
        4 => { rt::toggle_boats(); }
        _ => {}
    }
}

pub(super) fn on_native_key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed {
        return;
    }
    if let Some((_, bound)) = game.native_mapping("toggle") {
        if key == bound {
            rt::toggle_enabled();
        }
    }
}
