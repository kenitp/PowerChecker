use crate::PowerLevel;

pub const POWER_LOW_MAX: u32 = 300;
pub const POWER_MID_MAX: u32 = 1200;
pub const POWER_MAX_DISPLAY: u32 = 2000;

pub fn watts_to_level(w: u32) -> PowerLevel {
    if w == 0 {
        PowerLevel::Unknown
    } else if w < POWER_LOW_MAX {
        PowerLevel::Low
    } else if w < POWER_MID_MAX {
        PowerLevel::Mid
    } else {
        PowerLevel::High
    }
}

pub fn watts_to_ratio(w: u32) -> f32 {
    (w as f32 / POWER_MAX_DISPLAY as f32).min(1.0)
}
