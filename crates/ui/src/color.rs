use std::str::from_utf8;

use sabi_math::{Vector3, Vector4};

const HASH_CHAR: u8 = b'#';

pub trait Color {
    fn add_color(&mut self, other_color: Vector4) -> Self;
    fn remove_color(&mut self, other_color: Vector4) -> Self;
}

impl Color for Vector4 {
    fn add_color(&mut self, other_color: Vector4) -> Self {
        self.x = (self.x + other_color.x).min(1.);
        self.y = (self.y + other_color.y).min(1.);
        self.z = (self.z + other_color.z).min(1.);
        *self
    }
    fn remove_color(&mut self, other_color: Vector4) -> Self {
        self.x = (self.x - other_color.x).max(0.);
        self.y = (self.y - other_color.y).max(0.);
        self.z = (self.z - other_color.z).max(0.);
        *self
    }
}

pub fn hex_to_rgb(s: &str) -> Vector3 {
    from_hex(s.as_bytes())
}
pub fn hex_to_rgba(s: &str) -> Vector4 {
    let rgb = from_hex(s.as_bytes());
    Vector4::new(rgb.x, rgb.y, rgb.z, 1.)
}

fn from_hex(s: &[u8]) -> Vector3 {
    let mut buff: [u8; 6] = [0; 6];
    let mut buff_len = 0;

    for b in s {
        debug_assert!(b.is_ascii() && buff_len < 6);

        let bl = b.to_ascii_lowercase();
        if bl == HASH_CHAR {
            continue;
        }
        if bl.is_ascii_hexdigit() {
            buff[buff_len] = bl;
            buff_len += 1;
        }
    }

    if buff_len == 3 {
        buff = [buff[0], buff[0], buff[1], buff[1], buff[2], buff[2]];
    }

    let hex_str = from_utf8(&buff).unwrap();
    let hex_digit = u32::from_str_radix(hex_str, 16).unwrap();

    hex_digit_to_rgb(hex_digit)
}

fn hex_digit_to_rgb(num: u32) -> Vector3 {
    Vector3::new(
        (num >> 16) as f32 / 255.,
        ((num >> 8) & 0x00FF) as f32 / 255.,
        (num & 0x0000_00FF) as f32 / 255.,
    )
}
