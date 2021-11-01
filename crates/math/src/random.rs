use rand::Rng;

pub fn get_random_f32(min: f32, max: f32) -> f32 {
    rand::thread_rng().gen_range(min..max)
}
pub fn get_random_u32(min: u32, max: u32) -> u32 {
    rand::thread_rng().gen_range(min..max)
}
