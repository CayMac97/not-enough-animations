// Not Enough Animations - Runtime Module
// Brings realistic third-person animations for players:
// - Eating & Drinking: smoothly lifts food/potion to mouth with chewing motion and head nods
// - Two-Handed Map: holds maps realistically in front of the chest
// - Ladder Climbing: hand-over-hand climbing animation on ladders and vines
// - Boat Rowing: fluid oar-rowing arm movement while riding boats

use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "not_enough_animations";
pub const NAME: &str = "Not Enough Animations";
pub const VERSION: &str = "1.0.2";
pub const DESCRIPTION: &str =
    "Brings realistic third-person animations for eating, drinking, map holding, ladder climbing, and boat rowing.";

static ENABLED: AtomicBool = AtomicBool::new(true);
static EATING: AtomicBool = AtomicBool::new(true);
static MAPS: AtomicBool = AtomicBool::new(true);
static CLIMBING: AtomicBool = AtomicBool::new(true);
static BOATS: AtomicBool = AtomicBool::new(true);

static IS_CLIMBING: AtomicBool = AtomicBool::new(false);
static IN_BOAT: AtomicBool = AtomicBool::new(false);
static BOAT_MOVING: AtomicBool = AtomicBool::new(false);

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

#[inline]
pub fn set_climbing(val: bool) {
    IS_CLIMBING.store(val, Ordering::Relaxed);
}

#[inline]
pub fn is_climbing() -> bool {
    IS_CLIMBING.load(Ordering::Relaxed)
}

#[inline]
pub fn set_in_boat(in_boat: bool, moving: bool) {
    IN_BOAT.store(in_boat, Ordering::Relaxed);
    BOAT_MOVING.store(moving, Ordering::Relaxed);
}

#[inline]
pub fn in_boat() -> bool {
    IN_BOAT.load(Ordering::Relaxed)
}

#[inline]
pub fn boat_moving() -> bool {
    BOAT_MOVING.load(Ordering::Relaxed)
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
        || id.ends_with(":goat_horn")
        || id.ends_with(":spear"))
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

/// Applies enabled Not Enough Animations to the humanoid pose
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
        let is_left = if m.main_arm_left() {
            !m.extra.using_off_hand
        } else {
            m.extra.using_off_hand
        };
        let arm = if is_left { "left_arm" } else { "right_arm" };

        let ticks = m.extra.using_item as f32 + s.partial;
        // Smooth ease-in lift over first 4 ticks
        let raise = (ticks / 4.0).min(1.0);
        let ease = raise * raise * (3.0 - 2.0 * raise);

        // Chewing / drinking motion (~4 Hz)
        let chew_wave = (ticks * 1.2).sin();
        let chew_x = chew_wave * 0.12 * ease;
        let chew_y = chew_wave * 0.06 * ease;

        // Target lifted up towards mouth
        let target_x = -1.65 + head_pitch * 0.6 + chew_x;
        let inward_yaw = if is_left { 0.52 } else { -0.52 };
        let target_y = inward_yaw + head_yaw * 0.35 + (if is_left { -chew_y } else { chew_y });
        let target_z = if is_left { -0.28 } else { 0.28 };

        let cur_x = p.get(arm)[3];
        let cur_y = p.get(arm)[4];
        let cur_z = p.get(arm)[5];

        p.xrot(arm, cur_x + (target_x - cur_x) * ease);
        p.yrot(arm, cur_y + (target_y - cur_y) * ease);
        p.zrot(arm, cur_z + (target_z - cur_z) * ease);

        // Subtle head nod in sync with chewing
        p.get("head")[3] += chew_wave.max(0.0) * 0.08 * ease;
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
    if climbing_enabled() && (is_climbing() || m.climbing) {
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
    if boats_enabled() && (in_boat() || m.extra.passenger) {
        let is_moving = boat_moving() || m.limb_speed > 0.01;
        let row_phase = if is_moving {
            s.age * 0.35 + s.partial * 0.35
        } else {
            0.0
        };
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
