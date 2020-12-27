use std::collections::HashMap;
use nrg_math::*;

use super::block::*;
use super::chunk::*;



#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ChunkCoordinate {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl ChunkCoordinate {
    #[inline]
    pub fn new(x: u32, y: u32, z: u32) -> ChunkCoordinate {
        ChunkCoordinate { x, y, z }
    }
}

#[derive(Debug, Clone)]
pub struct WorldBlockCoordinate {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl WorldBlockCoordinate {
    #[inline]
    pub fn new(x: u32, y: u32, z: u32) -> WorldBlockCoordinate {
        WorldBlockCoordinate { x, y, z }
    }

    #[inline]
    pub fn convert_into_chunk(&self) -> ChunkCoordinate {
        ChunkCoordinate::new(self.x / Chunk::SIZE_X as u32, self.y / Chunk::SIZE_Y as u32, self.z / Chunk::SIZE_Z as u32)
    }

    #[inline]
    pub fn convert_into_block(&self) -> BlockCoordinate {
        let chunk_coords = self.convert_into_chunk();
        BlockCoordinate::new(
            self.x as usize - chunk_coords.x as usize * Chunk::SIZE_X,
            self.y as usize - chunk_coords.y as usize * Chunk::SIZE_Y,
            self.z as usize - chunk_coords.z as usize * Chunk::SIZE_Z
        )
    }
}


pub struct World {
    pub chunks: HashMap<ChunkCoordinate, Chunk>,
    pub visible_chunks: HashMap<ChunkCoordinate, ChunkMesh>
}

impl Default for World {
    fn default() -> World {
        BlockConfig::new(Block::DEFAULT, false, true);
        Self {
            chunks: HashMap::new(),
            visible_chunks: HashMap::new(),
        }
    }
}

impl World {
    pub fn create_sphere(&mut self, block_type: Block, lower: &WorldBlockCoordinate, upper: &WorldBlockCoordinate) {
        
        let rx = (upper.x as f64 - lower.x as f64) / 2. + 1.;
        let ry = (upper.y as f64 - lower.y as f64) / 2. + 1.;
        let rz = (upper.z as f64 - lower.z as f64) / 2. + 1.;

        let cx = (lower.x as f64 + upper.x as f64) / 2.;
        let cy = (lower.y as f64 + upper.y as f64) / 2.;
        let cz = (lower.z as f64 + upper.z as f64) / 2.;

        for x in lower.x..=upper.x {
            for y in lower.y..=upper.y {
                for z in lower.z..=upper.z {
                    
                    let dist =
                        ((x as f64 - cx as f64) / rx).powi(2) +
                        ((y as f64 - cy as f64) / ry).powi(2) +
                        ((z as f64 - cz as f64) / rz).powi(2);

                    if dist <= 1 as _ {
                        self.set_block(&WorldBlockCoordinate::new(x, y, z), block_type);
                    }
                }
            }
        }
    }

    pub fn get_chunk(&mut self, index: &ChunkCoordinate) -> &mut Chunk {
        if !self.chunks.contains_key(&index) {
            let chunk = Chunk::default();
            self.chunks.insert(index.clone(), chunk);
        }
        self.chunks.get_mut(&index).unwrap()
    }

    pub fn set_block(&mut self, coords: &WorldBlockCoordinate, block_type: Block) {
        let chunk = self.get_chunk(&coords.convert_into_chunk());
        chunk.set_block(&coords.convert_into_block(), block_type);
    }

    pub fn update(&mut self, view_distance: u32, cam_pos: Vector3f) {
        let nearest_chunk = ChunkCoordinate::new(
            (cam_pos[0] / Chunk::SIZE_X as f32).chunk_clamp_x() as u32,
            (cam_pos[2] / Chunk::SIZE_Y as f32).chunk_clamp_y() as u32,  // Flip Y with Z
            (cam_pos[1] / Chunk::SIZE_Z as f32).chunk_clamp_z() as u32,  // Flip Z with Y
        );
        
        let x_range = nearest_chunk.x.saturating_sub(view_distance)..=nearest_chunk.x.saturating_add(view_distance);
        let y_range = nearest_chunk.y.saturating_sub(view_distance)..=nearest_chunk.y.saturating_add(view_distance);
        let z_range = nearest_chunk.z.saturating_sub(view_distance)..=nearest_chunk.z.saturating_add(view_distance);

        let mut chunks_to_remove = Vec::new();
        for (chunk_index, _) in self.visible_chunks.iter() {
            if !x_range.contains(&chunk_index.x) || !y_range.contains(&chunk_index.y) || !z_range.contains(&chunk_index.z) {
                chunks_to_remove.push(chunk_index.clone());
            }
        }
        for chunk_index in chunks_to_remove {
            self.visible_chunks.remove(&chunk_index);
        }

        for chunk_x in *x_range.start()..=*x_range.end(){
            for chunk_y in *y_range.start()..=*y_range.end() {
                for chunk_z in *z_range.start()..=*z_range.end() {
                    
                    let chunk_index = ChunkCoordinate::new(chunk_x, chunk_y, chunk_z);
                    if self.visible_chunks.contains_key(&chunk_index) {
                        continue; 
                    }
                    
                    let chunk = self.get_chunk(&chunk_index);
                    chunk.clean_sides();
                   
                    let mut vertices = Vec::new();
                    chunk.generate_mesh(&mut vertices);
                   
                    let transform= Matrix4f::from_translation(
                        [
                            chunk_x as f32 * Chunk::SIZE_X as f32,
                            chunk_z as f32 * Chunk::SIZE_Z as f32,  // Flip Y with Z
                            chunk_y as f32 * Chunk::SIZE_Y as f32,  // Flip Z with Y
                        ].into());
                    self.visible_chunks.insert(chunk_index.clone(), ChunkMesh { transform, vertices });
                }
            }
        }
    }
}