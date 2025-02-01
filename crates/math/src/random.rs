use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_random_f32(min: f32, max: f32) -> f32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let nanos = now.as_nanos() as u64; // Get nanoseconds as u64
    let normalized = (nanos % 1_000_000_000) as f32 / 1_000_000_000.0; // Normalize to [0,1)

    min + normalized * (max - min) // Scale to range [min, max]
}
pub fn get_random_u32(min: u32, max: u32) -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let nanos = now.as_nanos() as u64; // Get nanoseconds as u64
    (nanos % ((max - min + 1) as u64)) as u32 + min
}
