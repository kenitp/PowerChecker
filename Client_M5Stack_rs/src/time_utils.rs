use core::sync::atomic::Ordering;

use esp_hal::time::Instant;

use crate::{NTP_EPOCH_SECS, NTP_SYNC_MS};

pub const DOW_STR: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// Current JST time as (year, month 1-12, day 1-31, hour, min, sec, weekday 0=Sun).
pub fn current_time() -> (i32, u8, u8, u8, u8, u8, u8) {
    let base_secs = NTP_EPOCH_SECS.load(Ordering::Relaxed);
    if base_secs == 0 {
        return (2000, 1, 1, 0, 0, 0, 0);
    }
    let base_ms = NTP_SYNC_MS.load(Ordering::Relaxed) as u64;
    let now_ms = Instant::now().duration_since_epoch().as_millis();
    // Guard against wrap-around (NTP_SYNC_MS is u32; now_ms is u64).
    let elapsed_secs = if now_ms >= base_ms {
        ((now_ms - base_ms) / 1000) as i32
    } else {
        0
    };
    epoch_to_parts(base_secs + elapsed_secs)
}

/// Convert a Unix timestamp to (year, month, day, hour, min, sec, weekday 0=Sun).
pub fn epoch_to_parts(t: i32) -> (i32, u8, u8, u8, u8, u8, u8) {
    let sec = (t % 60) as u8;
    let min = ((t / 60) % 60) as u8;
    let hour = ((t / 3600) % 24) as u8;
    let days = t / 86400;

    let dow = ((days + 4).rem_euclid(7)) as u8;

    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y, m as u8, d as u8, hour, min, sec, dow)
}
