#![allow(dead_code)]

use sabi_math::*;

use crate::VertexData;

use super::block::*;

const N: DeltaDir = DeltaDir::Negative;
const Z: DeltaDir = DeltaDir::Zero;
const P: DeltaDir = DeltaDir::Positive;

#[derive(Eq, PartialEq, Copy, Clone)]
enum DeltaDir {
    Negative,
    Zero,
    Positive,
}

impl DeltaDir {
    fn checked_add_usize(&self, base: usize) -> Option<usize> {
        match self {
            DeltaDir::Negative => base.checked_sub(1),
            DeltaDir::Zero => Some(base),
            DeltaDir::Positive => base.checked_add(1),
        }
    }
}

#[derive(Debug)]
pub struct BlockCoordinate {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
impl BlockCoordinate {
    #[inline]
    pub fn new(x: usize, y: usize, z: usize) -> BlockCoordinate {
        BlockCoordinate { x, y, z }
    }
}

pub type BlockSides = u8; // 0b000000 flags for each side
pub type BlockEdges = u32; // 0b00000000000 flags for each edge
pub type BlockCorners = u8; // 0b0000000 flags for each corner

type BlockData<T> = [[[T; Chunk::SIZE_Z]; Chunk::SIZE_Y]; Chunk::SIZE_X];
type Blocks = BlockData<Block>;
type Sides = BlockData<BlockSides>;
type Edges = BlockData<BlockEdges>;
type Corners = BlockData<BlockCorners>;

pub struct Chunk {
    blocks: Box<Blocks>,
    sides: Box<Sides>,
    edges: Box<Edges>,
    corners: Box<Corners>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: Box::new(
                [[[Block::DEFAULT_BLOCK; Chunk::SIZE_Z]; Chunk::SIZE_Y]; Chunk::SIZE_X],
            ),
            sides: Box::new([[[0b000000; Chunk::SIZE_Z]; Chunk::SIZE_Y]; Chunk::SIZE_X]),
            edges: Box::new([[[0b00000000000; Chunk::SIZE_Z]; Chunk::SIZE_Y]; Chunk::SIZE_X]),
            corners: Box::new([[[0b0000000; Chunk::SIZE_Z]; Chunk::SIZE_Y]; Chunk::SIZE_X]),
        }
    }
}

impl Chunk {
    pub const SIZE_X: usize = 32;
    pub const SIZE_Y: usize = 32;
    pub const SIZE_Z: usize = 32;
    pub const FACE_COUNT: usize = 6;
    pub const VERTICES_COUNT: usize = 8;
    pub const EDGE_COUNT: usize = 12;

    const SIDE_DIRS: [[DeltaDir; 3]; Chunk::FACE_COUNT] = [
        [Z, N, Z], // Close
        [Z, P, Z], // Far
        [P, Z, Z], // Right
        [N, Z, Z], // Left
        [Z, Z, P], // Top
        [Z, Z, N], // Bottom
    ];

    const EDGE_DIRS: [[DeltaDir; 3]; Chunk::EDGE_COUNT] = [
        [P, N, Z], //  0: CR
        [N, N, Z], //  1: CL
        [Z, N, P], //  2: CT
        [Z, N, N], //  3: CB
        [P, P, Z], //  4: FR
        [N, P, Z], //  5: FL
        [Z, P, P], //  6: FT
        [Z, P, N], //  7: FB
        [P, Z, P], //  8: RT
        [P, Z, N], //  9: RB
        [N, Z, P], // 10: LT
        [N, Z, N], // 11: LB
    ];

    const CORNER_DIRS: [[DeltaDir; 3]; Chunk::VERTICES_COUNT] = [
        // Y and Z are flipped
        [N, N, N], // 0: LBC
        [P, N, N], // 1: RBC
        [P, P, N], // 2: RBF
        [N, P, N], // 3: LBF
        [N, N, P], // 4: LTC
        [P, N, P], // 5: RTC
        [P, P, P], // 6: RTF
        [N, P, P], // 7: LTF
    ];

    pub fn visible_count(&self) -> u32 {
        let mut count = 0;
        for x in 0..Chunk::SIZE_X {
            for y in 0..Chunk::SIZE_Y {
                for z in 0..Chunk::SIZE_Z {
                    if !self.blocks[x][y][z].is_invisible() {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    pub fn set_block(&mut self, position: &BlockCoordinate, block: Block) {
        self.blocks[position.x][position.y][position.z] = block;
    }

    pub fn clean_sides(&mut self) {
        for x in 0..Chunk::SIZE_X {
            for y in 0..Chunk::SIZE_Y {
                for z in 0..Chunk::SIZE_Z {
                    self.process_sides_for_index(x, y, z);
                }
            }
        }
    }

    fn process_sides_for_index(&mut self, x: usize, y: usize, z: usize) {
        let mut sides = 0b000000;
        let mut edges = 0b00000000000;
        let mut corners = 0b00000000;

        if !self.blocks[x][y][z].is_invisible() {
            for side in 0..Chunk::FACE_COUNT {
                let dir = &Chunk::SIDE_DIRS[side];

                if let Some(block) = self.get_block_from_dir(x, y, z, dir) {
                    if block.is_transparent() {
                        sides |= 1 << side;
                    }
                } else {
                    sides |= 1 << side;
                }
            }

            for edge in 0..Chunk::EDGE_COUNT {
                let dir = &Chunk::EDGE_DIRS[edge];

                if let Some(block) = self.get_block_from_dir(x, y, z, dir) {
                    if block.is_transparent() {
                        edges |= 1 << edge;
                    }
                } else {
                    edges |= 1 << edge;
                }
            }

            for corner in 0..Chunk::VERTICES_COUNT {
                let dir = &Chunk::CORNER_DIRS[corner];

                if let Some(block) = self.get_block_from_dir(x, y, z, dir) {
                    if block.is_transparent() {
                        corners |= 1 << corner;
                    }
                } else {
                    corners |= 1 << corner;
                }
            }
        }

        self.sides[x][y][z] = sides as _;
        self.edges[x][y][z] = edges as _;
        self.corners[x][y][z] = corners as _;
    }

    fn get_block_from_dir(
        &self,
        x: usize,
        y: usize,
        z: usize,
        dir: &[DeltaDir; 3],
    ) -> Option<Block> {
        let dx = match dir[0].checked_add_usize(x) {
            Some(x) => x,
            None => return None,
        };
        let dy = match dir[1].checked_add_usize(y) {
            Some(y) => y,
            None => return None,
        };
        let dz = match dir[2].checked_add_usize(z) {
            Some(z) => z,
            None => return None,
        };

        if dx >= Chunk::SIZE_X || dy >= Chunk::SIZE_Y || dz >= Chunk::SIZE_Z {
            return None;
        }

        Some(self.blocks[dx][dy][dz])
    }

    pub fn generate_mesh(&self, vertices: &mut Vec<VertexData>) {
        for x in 0..Chunk::SIZE_X {
            for y in 0..Chunk::SIZE_Y {
                for z in 0..Chunk::SIZE_Z {
                    self.blocks[x][y][z].generate_mesh(
                        vertices,
                        Vector3::new(x as f32, y as f32, z as f32),
                        self.sides[x][y][z],
                        self.edges[x][y][z],
                        self.corners[x][y][z],
                    );
                }
            }
        }
    }
}

pub struct ChunkMesh {
    pub transform: Matrix4,
    pub vertices: Vec<VertexData>,
}

pub trait ChunkClamp {
    fn chunk_clamp_x(self) -> Self;
    fn chunk_clamp_y(self) -> Self;
    fn chunk_clamp_z(self) -> Self;
}

impl ChunkClamp for f32 {
    fn chunk_clamp_x(self) -> Self {
        self.min(Chunk::SIZE_X as f32 - 1.).max(0.)
    }

    fn chunk_clamp_y(self) -> Self {
        self.min(Chunk::SIZE_Y as f32 - 1.).max(0.)
    }

    fn chunk_clamp_z(self) -> Self {
        self.min(Chunk::SIZE_Z as f32 - 1.).max(0.)
    }
}

impl ChunkClamp for u32 {
    fn chunk_clamp_x(self) -> Self {
        self.min(Chunk::SIZE_X as u32 - 1)
    }

    fn chunk_clamp_y(self) -> Self {
        self.min(Chunk::SIZE_Y as u32 - 1)
    }

    fn chunk_clamp_z(self) -> Self {
        self.min(Chunk::SIZE_Z as u32 - 1)
    }
}

impl ChunkClamp for usize {
    fn chunk_clamp_x(self) -> Self {
        self.min(Chunk::SIZE_X - 1)
    }

    fn chunk_clamp_y(self) -> Self {
        self.min(Chunk::SIZE_Y - 1)
    }

    fn chunk_clamp_z(self) -> Self {
        self.min(Chunk::SIZE_Z - 1)
    }
}
