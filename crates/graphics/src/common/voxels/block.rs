#![allow(dead_code)]

use inox_math::*;

use crate::VertexData;

pub static mut CONFIG: Vec<BlockConfig> = Vec::new();

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BlockConfig {
    pub name: &'static str,
    pub is_active: bool,
    pub is_transparent: bool,
}

impl Default for BlockConfig {
    fn default() -> Self {
        Self {
            name: "",
            is_active: false,
            is_transparent: true,
        }
    }
}

#[allow(dead_code)]
impl BlockConfig {
    pub fn new(name: &'static str, is_active: bool, is_transparent: bool) -> BlockConfig {
        let config = BlockConfig {
            name,
            is_active,
            is_transparent,
        };
        if !BlockConfig::exists(name) {
            unsafe {
                CONFIG.push(config);
            }
        }
        config
    }

    pub fn exists(name: &'static str) -> bool {
        unsafe { matches!(CONFIG.iter().find(|el| el.name == name), Some(_)) }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Block(u32);

impl Block {
    pub const DEFAULT_BLOCK: Block = Block(0);
    pub const DEFAULT: &'static str = "default";

    const FACE_COUNT: usize = 6;
    const VERTICES_COUNT: usize = 8;

    const FACES: [[usize; 4]; Block::FACE_COUNT] = [
        [5, 4, 0, 1], // Close;   RTC, LTC, LBC, RBC
        [7, 6, 2, 3], // Far;     LTF, RTF, RBF, LBF
        [6, 5, 1, 2], // Right;   RTF, RTC, RBC, RBF
        [4, 7, 3, 0], // Left;    LTC, LTF, LBF, LBC
        [6, 7, 4, 5], // Top;     RTF, LTF, LTC, RTC
        [1, 0, 3, 2], // Bottom;  RBC, LBC, LBF, RBF
    ];

    const VERTICES: [[f32; 3]; Block::VERTICES_COUNT] = [
        [0., 0., 0.], // 0: LBC
        [1., 0., 0.], // 1: RBC
        [1., 0., 1.], // 2: RBF
        [0., 0., 1.], // 3: LBF
        [0., 1., 0.], // 4: LTC
        [1., 1., 0.], // 5: RTC
        [1., 1., 1.], // 6: RTF
        [0., 1., 1.], // 7: LTF
    ];

    const NORMALS: [[f32; 3]; Block::FACE_COUNT] = [
        [0., 0., -1.],
        [0., 0., 1.],
        [1., 0., 0.],
        [-1., 0., 0.],
        [0., 1., 0.],
        [0., -1., 0.],
    ];

    const FACE_ORDER: [usize; Block::FACE_COUNT] = [0, 3, 1, 1, 3, 2];

    const FACE_EDGES: [[usize; 4]; Block::FACE_COUNT] = [
        [0, 2, 1, 3],   // Close;   CT, CL, CB, CR
        [5, 6, 4, 7],   // Far;     FT, FR, FB, FL
        [4, 8, 0, 9],   // Right;   RT, CR, RB, FR
        [1, 10, 5, 11], // Left;    LT, FL, LB, CL
        [8, 6, 10, 2],  // Top;     FT, LT, CT, RT
        [9, 3, 11, 7],  // Bottom;  CB, LB, FB, RB
    ];

    pub fn from_name(name: &str) -> Block {
        unsafe {
            for (i, conf) in CONFIG.iter().enumerate() {
                if conf.name == name {
                    return Block(i as _);
                }
            }
        }
        panic!("Unknown block config name {}", name);
    }

    fn get_config(&self) -> &BlockConfig {
        unsafe { &CONFIG[self.0 as usize] }
    }

    pub fn is_transparent(&self) -> bool {
        self.get_config().is_transparent
    }

    pub fn is_active(&self) -> bool {
        self.get_config().is_active
    }

    pub fn is_invisible(&self) -> bool {
        !self.is_active()
    }

    pub fn generate_mesh(
        &self,
        vertices: &mut Vec<VertexData>,
        coord: Vector3,
        sides: u8,
        edges: u32,
        corners: u8,
    ) {
        if sides == 0b000000 {
            return;
        }

        for side in 0..Block::FACE_COUNT {
            if sides & (1 << side) == 0b000000 {
                continue;
            }

            let face_index = &Block::FACES[side];
            for &pos in &Block::FACE_ORDER {
                let vertex_index = face_index[pos]; // Also used as the corner index
                let mut position = Block::VERTICES[vertex_index];
                position[0] += coord.x;
                position[1] += coord.z; // Swap Y with Z
                position[2] += coord.y; // Swap Z with Y

                let has_edge_a = edges & (1 << Block::FACE_EDGES[side][pos]) != 0;
                let has_edge_b = edges & (1 << Block::FACE_EDGES[side][(pos + 1) % 4]) != 0;
                let has_corner = corners & (1 << vertex_index) != 0;
                let shade_corner = !has_edge_a || !has_edge_b || !has_corner;
                let darkness = 0.5;
                let color = if shade_corner {
                    [darkness, darkness, darkness, darkness]
                } else {
                    [1., 1., 1., 1.]
                };

                let normal = Block::NORMALS[side as usize];

                vertices.push(VertexData {
                    pos: position.into(),
                    color: color.into(),
                    normal: normal.into(),
                    ..VertexData::default()
                });
            }
        }
    }
}
