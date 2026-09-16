use inox_resources::to_slice;

use super::packed_data::{PackedTableRange, PACKED_BUFFER_MAGIC, PACKED_BUFFER_VERSION};

pub const PACKED_SCENE_TABLE_COUNT: usize = 11;

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneTableKind {
    Indices = 0,
    VertexPositions = 1,
    VertexAttributes = 2,
    Instances = 3,
    Transforms = 4,
    Meshes = 5,
    Meshlets = 6,
    Bvh = 7,
    Materials = 8,
    Textures = 9,
    Lights = 10,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedSceneHeader {
    pub magic: u32,
    pub version: u32,
    pub word_count: u32,
    pub flags: u32,
    pub ranges: [PackedTableRange; PACKED_SCENE_TABLE_COUNT],
}

pub struct PackedSceneData {
    words: Vec<u32>,
    ranges: [PackedTableRange; PACKED_SCENE_TABLE_COUNT],
}

impl PackedSceneData {
    pub fn words(&self) -> &[u32] {
        &self.words
    }

    pub fn words_mut(&mut self) -> &mut [u32] {
        &mut self.words
    }

    pub fn range(&self, kind: SceneTableKind) -> PackedTableRange {
        self.ranges[kind as usize]
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.words.len() < std::mem::size_of::<PackedSceneHeader>() / 4 {
            return Err("packed scene is missing its header");
        }
        if self.words[0] != PACKED_BUFFER_MAGIC {
            return Err("packed scene magic does not match");
        }
        if self.words[1] != PACKED_BUFFER_VERSION {
            return Err("packed scene version does not match");
        }
        let word_count = self.words[2] as usize;
        if word_count != self.words.len() {
            return Err("packed scene word count does not match payload");
        }
        for range in self.ranges {
            let end = range.offset_words as usize + range.count_words as usize;
            if end > word_count || (range.count_words > 0 && range.stride_words == 0) {
                return Err("packed scene table range is invalid");
            }
        }
        Ok(())
    }
}

pub struct PackedSceneBuilder {
    words: Vec<u32>,
    ranges: [PackedTableRange; PACKED_SCENE_TABLE_COUNT],
}

impl Default for PackedSceneBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PackedSceneBuilder {
    pub fn new() -> Self {
        Self {
            words: vec![0; std::mem::size_of::<PackedSceneHeader>() / 4],
            ranges: [PackedTableRange::default(); PACKED_SCENE_TABLE_COUNT],
        }
    }

    pub fn append_typed<T: Sized>(
        &mut self,
        kind: SceneTableKind,
        values: &[T],
        stride_words: usize,
    ) {
        debug_assert_eq!(std::mem::size_of::<T>() % 4, 0);
        self.append_words(kind, to_slice::<T, u32>(values), stride_words);
    }

    pub fn append_words(&mut self, kind: SceneTableKind, words: &[u32], stride_words: usize) {
        let alignment = stride_words.max(1);
        let offset = self.words.len().next_multiple_of(alignment);
        self.words.resize(offset, 0);
        self.words.extend_from_slice(words);
        self.ranges[kind as usize] = PackedTableRange::new(offset, words.len(), alignment, 0);
    }

    pub fn finish(self) -> PackedSceneData {
        let mut data = PackedSceneData {
            words: self.words,
            ranges: self.ranges,
        };
        let header = PackedSceneHeader {
            magic: PACKED_BUFFER_MAGIC,
            version: PACKED_BUFFER_VERSION,
            word_count: data.words.len() as u32,
            flags: 0,
            ranges: data.ranges,
        };
        let header_words = to_slice::<PackedSceneHeader, u32>(std::slice::from_ref(&header));
        data.words[..header_words.len()].copy_from_slice(header_words);
        data
    }
}
