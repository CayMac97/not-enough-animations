// Not Enough Animations - Runtime Module
// Provides realistic third-person animations for players:
// - Eating & Drinking: raises food/potion right to mouth with prominent chewing motion and head nods
// - Two-Handed Map: holds maps realistically with both hands in front of the chest
// - Ladder Climbing: hand-over-hand climbing animation on ladders and vines
// - Boat Rowing: fluid oar-rowing arm movement while riding boats

use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "not_enough_animations";
pub const NAME: &str = "Not Enough Animations";
pub const VERSION: &str = "1.0.1";
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

#[inline]
fn is_map_item(item: Option<&str>) -> bool {
    item.is_some_and(|id| id.contains("map"))
}

fn is_holding_map(m: &crate::mobs::Mob, s: &crate::mobs::anim::State) -> bool {
    if s.swing > 0.0 || m.extra.using_item > 0 {
        return false;
    }
    is_map_item(m.extra.main_hand.as_deref()) || is_map_item(m.extra.off_hand.as_deref())
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
        // Prominent chewing oscillation (~4 Hz)
        let chew_wave = (ticks * 1.2).sin();
        let chew_x = chew_wave * 0.16; // ~9.2 degrees
        let chew_y = chew_wave * 0.08; // ~4.6 degrees

        // Bring hand right to mouth:
        // xrot (pitch): lifted up towards face/mouth (-100 degrees) + head pitch tracking + chewing motion
        p.xrot(arm, -1.75 + head_pitch * 0.5 + chew_x);

        // yrot (yaw): angled inward towards mouth + head yaw tracking + chewing
        let inward_yaw = if is_left { 0.58 } else { -0.58 };
        let chew_side = if is_left { -chew_y } else { chew_y };
        p.yrot(arm, inward_yaw + head_yaw * 0.35 + chew_side);

        // zrot (roll): tilted so the item faces the mouth directly
        p.zrot(arm, if is_left { -0.32 } else { 0.32 });

        // Distinctive head nod in sync with each bite / sip
        p.get("head")[3] += chew_wave.max(0.0) * 0.09;
        return;
    }

    // 2. TWO-HANDED MAP HOLDING
    if maps_enabled() && is_holding_map(m, s) {
        let breathe = (s.age * 0.08).sin() * 0.025;
        let has_main_map = is_map_item(m.extra.main_hand.as_deref());
        let has_off_map = is_map_item(m.extra.off_hand.as_deref());
        let two_handed = (has_main_map && (m.extra.off_hand.is_none() || has_off_map))
            || (has_off_map && m.extra.main_hand.is_none());

        if two_handed || has_main_map {
            // Right arm holding map in front of chest
            p.xrot("right_arm", -1.25 + head_pitch * 0.4 + breathe);
            p.yrot("right_arm", -0.45 + head_yaw * 0.35);
            p.zrot("right_arm", 0.25);
        }

        if two_handed || has_off_map {
            // Left arm supporting map in front of chest
            p.xrot("left_arm", -1.25 + head_pitch * 0.4 + breathe);
            p.yrot("left_arm", 0.45 + head_yaw * 0.35);
            p.zrot("left_arm", -0.25);
        }
        return;
    }

    // 3. LADDER & VINE CLIMBING
    if climbing_enabled() && m.climbing {
        let is_moving = m.limb_speed > 0.01 || m.velocity.y.abs() > 0.01;
        let climb_cycle = if is_moving {
            // Alternating hand-over-hand cycle
            (m.position.y as f32 * 5.0 + s.age * 0.15).sin()
        } else {
            0.5
        };

        // Arms reach high up on the ladder rungs
        p.xrot("right_arm", -2.45 + climb_cycle * 0.65);
        p.yrot("right_arm", 0.18);
        p.zrot("right_arm", 0.12);

        p.xrot("left_arm", -2.45 - climb_cycle * 0.65);
        p.yrot("left_arm", -0.18);
        p.zrot("left_arm", -0.12);

        // Alternating leg climbing movement
        let leg_cycle = if is_moving { climb_cycle } else { 0.0 };
        p.xrot("right_leg", leg_cycle * 0.55);
        p.xrot("left_leg", -leg_cycle * 0.55);
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
