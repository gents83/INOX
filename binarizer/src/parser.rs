use std::{fs::File, io::Read, mem::size_of};

use nrg_math::{VecBase, Vector2, Vector3, Vector4};

pub trait Parser {
    fn size() -> usize;
    fn parse(file: &mut File) -> Self;
}

impl Parser for f32 {
    fn size() -> usize {
        size_of::<f32>()
    }
    fn parse(file: &mut File) -> f32 {
        debug_assert!(Self::size() == 4);
        let mut bytes = [0u8; 4];
        if file.read_exact(&mut bytes).is_ok() {
            return f32::from_le_bytes(bytes);
        }
        0.
    }
}

impl Parser for u8 {
    fn size() -> usize {
        size_of::<u8>()
    }
    fn parse(file: &mut File) -> u8 {
        debug_assert!(Self::size() == 1);
        let mut bytes = [0u8; 1];
        if file.read_exact(&mut bytes).is_ok() {
            return u8::from_le_bytes(bytes);
        }
        0
    }
}

impl Parser for u16 {
    fn size() -> usize {
        size_of::<u16>()
    }
    fn parse(file: &mut File) -> u16 {
        debug_assert!(Self::size() == 2);
        let mut bytes = [0u8; 2];
        if file.read_exact(&mut bytes).is_ok() {
            return u16::from_le_bytes(bytes);
        }
        0
    }
}

impl Parser for u32 {
    fn size() -> usize {
        size_of::<u32>()
    }
    fn parse(file: &mut File) -> u32 {
        debug_assert!(Self::size() == 4);
        let mut bytes = [0u8; 4];
        if file.read_exact(&mut bytes).is_ok() {
            return u32::from_le_bytes(bytes);
        }
        0
    }
}

impl Parser for Vector2 {
    fn size() -> usize {
        2 * size_of::<f32>()
    }
    fn parse(file: &mut File) -> Vector2 {
        let mut v = Vector2::default_zero();
        v.x = f32::parse(file);
        v.y = f32::parse(file);
        v
    }
}

impl Parser for Vector3 {
    fn size() -> usize {
        3 * size_of::<f32>()
    }
    fn parse(file: &mut File) -> Vector3 {
        let mut v = Vector3::default_zero();
        v.x = f32::parse(file);
        v.y = f32::parse(file);
        v.z = f32::parse(file);
        v
    }
}

impl Parser for Vector4 {
    fn size() -> usize {
        4 * size_of::<f32>()
    }
    fn parse(file: &mut File) -> Vector4 {
        let mut v = Vector4::default_zero();
        v.x = f32::parse(file);
        v.y = f32::parse(file);
        v.z = f32::parse(file);
        v.w = f32::parse(file);
        v
    }
}
