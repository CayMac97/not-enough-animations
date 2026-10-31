// Not Enough Animations - Runtime Module
// Provides realistic third-person animations for players:
// - Eating & Drinking: raises food to mouth with natural chewing motion
// - Two-Handed Map: holds maps realistically with both hands
// - Ladder Climbing: hand-over-hand climbing animation on ladders and vines
// - Boat Rowing: fluid oar-rowing arm movement while riding boats

use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "not_enough_animations";
pub const NAME: &str = "Not Enough Animations";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str =
    "Brings realistic third-person animations for eating, drinking, map holding, ladder climbing, and boat rowing.";

static ENABLED: AtomicBool = AtomicBool::new(true);
static EATING: AtomicBool = AtomicBool::new(true);
static MAPS: AtomicBool = AtomicBool::new(true);
static CLIMBING: AtomicBool = AtomicBool::new(true);
static BOATS: AtomicBool = AtomicBool::new(true);

#[inline]
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn toggle_enabled() -> bool {
    let prev = ENABLED.fetch_xor(true, Ordering::Relaxed);
    !prev
}

#[inline]
pub fn eating_enabled() -> bool {
    is_enabled() && EATING.load(Ordering::Relaxed)
}

pub fn toggle_eating() -> bool {
    let prev = EATING.fetch_xor(true, Ordering::Relaxed);
    !prev
}

#[inline]
pub fn maps_enabled() -> bool {
    is_enabled() && MAPS.load(Ordering::Relaxed)
}

pub fn toggle_maps() -> bool {
    let prev = MAPS.fetch_xor(true, Ordering::Relaxed);
    !prev
}

#[inline]
pub fn climbing_enabled() -> bool {
    is_enabled() && CLIMBING.load(Ordering::Relaxed)
}

pub fn toggle_climbing() -> bool {
    let prev = CLIMBING.fetch_xor(true, Ordering::Relaxed);
    !prev
}

#[inline]
pub fn boats_enabled() -> bool {
    is_enabled() && BOATS.load(Ordering::Relaxed)
}

pub fn toggle_boats() -> bool {
    let prev = BOATS.fetch_xor(true, Ordering::Relaxed);
    !prev
}

fn is_eating_or_drinking(m: &crate::mobs::Mob) -> bool {
    if m.extra.using_item <= 0 || m.extra.blocking {
        return false;
    }
    let hand = if m.extra.using_off_hand {
        m.extra.off_hand.as_deref()
    } else {
        m.extra.main_hand.as_deref()
    };
    let Some(id) = hand else { return false };
    !(id.ends_with(":bow")
        || id.ends_with(":crossbow")
        || id.ends_with(":trident")
        || id.ends_with(":shield")
        || id.ends_with(":brush")
        || id.ends_with(":spyglass")
        || id.ends_with(":goat_horn"))
}

fn is_holding_map(m: &crate::mobs::Mob, s: &crate::mobs::anim::State) -> bool {
    if s.swing > 0.0 || m.extra.using_item > 0 {
        return false;
    }
    let check = |item: Option<&str>| {
        item.is_some_and(|id| id.ends_with("_map") || id == "minecraft:map")
    };
    check(m.extra.main_hand.as_deref()) || check(m.extra.off_hand.as_deref())
}

/// Applies all enabled animations to the given poses
pub fn apply_nea(
    p: &mut crate::mobs::anim::Poses,
    s: &crate::mobs::anim::State,
    m: &crate::mobs::Mob,
) {
    if !is_enabled() {
        return;
    }

    let head_pitch = p.get("head")[3];
    let head_yaw = p.get("head")[4];

    // 1. EATING & DRINKING
    if eating_enabled() && is_eating_or_drinking(m) {
        let is_left = m.extra.using_off_hand;
        let arm = if is_left { "left_arm" } else { "right_arm" };

        let ticks = m.extra.using_item as f32 + s.partial;
        // Natural chewing oscillation (cycles every ~7.5 frames / ~2.6 Hz)
        let chew_wave = (ticks * 0.85).sin();
        let chew = chew_wave * 0.055;

        // Bring hand to mouth:
        // xrot (pitch): lifted up towards face (-75 degrees) + head pitch tracking
        p.xrot(arm, -1.32 + head_pitch * 0.5 + chew);

        // yrot (yaw): angled inward towards mouth + head yaw tracking
        let inward_yaw = if is_left { 0.44 } else { -0.44 };
        p.yrot(arm, inward_yaw + head_yaw * 0.5);

        // zrot (roll): tilted slightly so the item faces the mouth
        p.zrot(arm, if is_left { -0.28 } else { 0.28 });

        // Subtle eating nod on head in sync with chewing
        p.get("head")[3] += chew_wave.max(0.0) * 0.025;
        return;
    }

    // 2. TWO-HANDED MAP HOLDING
    if maps_enabled() && is_holding_map(m, s) {
        // Both arms raised and holding the map in front of chest
        p.xrot("right_arm", -0.78 + head_pitch * 0.35);
        p.yrot("right_arm", -0.28 + head_yaw * 0.35);
        p.zrot("right_arm", 0.08);

        p.xrot("left_arm", -0.78 + head_pitch * 0.35);
        p.yrot("left_arm", 0.28 + head_yaw * 0.35);
        p.zrot("left_arm", -0.08);
        return;
    }

    // 3. LADDER & VINE CLIMBING
    if climbing_enabled() && m.climbing {
        let climb_speed = if m.limb_speed > 0.01 { m.limb_speed } else { 0.25 };
        let climb_cycle = (s.pos * 0.75 + s.age * 0.1).sin();

        // Arms reach high up on the ladder rungs
        p.xrot("right_arm", -2.35 + climb_cycle * 0.4 * climb_speed);
        p.yrot("right_arm", 0.15);
        p.zrot("right_arm", 0.1);

        p.xrot("left_arm", -2.35 - climb_cycle * 0.4 * climb_speed);
        p.yrot("left_arm", -0.15);
        p.zrot("left_arm", -0.1);

        // Alternating leg climbing movement
        p.xrot("right_leg", climb_cycle * 0.45 * climb_speed);
        p.xrot("left_leg", -climb_cycle * 0.45 * climb_speed);
        return;
    }

    // 4. BOAT ROWING
    if boats_enabled() && m.extra.passenger {
        let row_phase = s.age * 0.28 + s.partial * 0.28;
        let row_sin = row_phase.sin();
        let row_cos = row_phase.cos();

        // Arms reach forward and pull back with the oars
        p.xrot("right_arm", -0.78 + row_cos * 0.32);
        p.yrot("right_arm", 0.38 + row_sin * 0.18);
        p.zrot("right_arm", 0.12);

        p.xrot("left_arm", -0.78 + row_cos * 0.32);
        p.yrot("left_arm", -0.38 - row_sin * 0.18);
        p.zrot("left_arm", -0.12);
    }
}
