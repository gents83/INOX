use inox_resources::to_slice;

use super::packed_data::{
    PackedBufferHeader, PackedLightAliasData, PackedLightAliasHeader, PackedTableRange,
    PACKED_BUFFER_MAGIC, PACKED_LIGHT_ALIAS_CAPACITY, PACKED_LIGHT_ALIAS_HEADER_WORDS,
    PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS, PACKED_LIGHT_ALIAS_VERSION, PACKED_SCENE_HEADER_WORDS,
};

pub const PACKED_SCENE_TABLE_COUNT: usize = 12;

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
    LightAliases = 11,
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

    pub fn validate_words(words: &[u32]) -> Result<(), &'static str> {
        if words.len() < PACKED_SCENE_HEADER_WORDS {
            return Err("packed scene is missing its header");
        }
        if words[0] != PACKED_BUFFER_MAGIC {
            return Err("packed scene magic does not match");
        }
        if words[1] != PackedBufferHeader::version() {
            return Err("packed scene version does not match");
        }
        let word_count = words[2] as usize;
        if word_count != words.len() {
            return Err("packed scene word count does not match payload");
        }
        let mut non_empty_ranges = Vec::with_capacity(PACKED_SCENE_TABLE_COUNT);
        for index in 0..PACKED_SCENE_TABLE_COUNT {
            let base = 4 + index * 4;
            let offset = words[base] as usize;
            let count = words[base + 1] as usize;
            let stride = words[base + 2] as usize;
            let end = offset
                .checked_add(count)
                .ok_or("packed scene table range is invalid")?;
            if end > word_count
                || (count > 0 && offset < PACKED_SCENE_HEADER_WORDS)
                || (count > 0 && stride == 0)
                || (stride > 0 && !count.is_multiple_of(stride))
            {
                return Err("packed scene table range is invalid");
            }
            if count > 0 {
                non_empty_ranges.push((offset, end));
            }
        }
        non_empty_ranges.sort_unstable();
        if non_empty_ranges
            .windows(2)
            .any(|ranges| ranges[0].1 > ranges[1].0)
        {
            return Err("packed scene table ranges overlap");
        }
        let alias_range = PackedTableRange {
            offset_words: words[4 + SceneTableKind::LightAliases as usize * 4],
            count_words: words[5 + SceneTableKind::LightAliases as usize * 4],
            stride_words: words[6 + SceneTableKind::LightAliases as usize * 4],
            flags: words[7 + SceneTableKind::LightAliases as usize * 4],
        };
        if alias_range.count_words > 0 {
            let alias_offset = alias_range.offset_words as usize;
            let alias_words = alias_range.count_words as usize;
            if alias_words < PACKED_LIGHT_ALIAS_HEADER_WORDS
                || alias_offset + alias_words > word_count
                || words[alias_offset] != PACKED_LIGHT_ALIAS_VERSION
            {
                return Err("packed light alias header is invalid");
            }
            let header = PackedLightAliasHeader {
                version: words[alias_offset],
                count: words[alias_offset + 1],
                flags: words[alias_offset + 2],
                total_power: words[alias_offset + 3],
            };
            let record_words = (header.count as usize)
                .checked_mul(PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS)
                .ok_or("packed light alias count is invalid")?;
            if header.count as usize > PACKED_LIGHT_ALIAS_CAPACITY
                || PACKED_LIGHT_ALIAS_HEADER_WORDS + record_words > alias_words
            {
                return Err("packed light alias count is invalid");
            }
            for index in 0..header.count as usize {
                let record = alias_offset
                    + PACKED_LIGHT_ALIAS_HEADER_WORDS
                    + index * PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS;
                let alias_index = words[record] as usize;
                let selection_probability = f32::from_bits(words[record + 1]);
                let proposal_pdf = f32::from_bits(words[record + 2]);
                if alias_index >= header.count as usize
                    || !selection_probability.is_finite()
                    || !(0.0..=1.0).contains(&selection_probability)
                    || !proposal_pdf.is_finite()
                    || !(0.0..=1.0).contains(&proposal_pdf)
                {
                    return Err("packed light alias record is invalid");
                }
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        Self::validate_words(&self.words)
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

    pub fn try_append_typed<T: Sized>(
        &mut self,
        kind: SceneTableKind,
        values: &[T],
        stride_words: usize,
    ) -> Result<(), &'static str> {
        if !std::mem::size_of::<T>().is_multiple_of(4) {
            return Err("packed table element is not word aligned");
        }
        self.try_append_words(kind, to_slice::<T, u32>(values), stride_words)
    }

    pub fn append_typed<T: Sized>(
        &mut self,
        kind: SceneTableKind,
        values: &[T],
        stride_words: usize,
    ) {
        self.try_append_typed(kind, values, stride_words)
            .expect("invalid packed table");
    }

    pub fn try_append_words(
        &mut self,
        kind: SceneTableKind,
        words: &[u32],
        stride_words: usize,
    ) -> Result<(), &'static str> {
        let alignment = stride_words.max(1);
        let aligned_words = self
            .words
            .len()
            .checked_add(alignment - 1)
            .ok_or("packed scene offset overflow")?
            / alignment;
        let offset = aligned_words
            .checked_mul(alignment)
            .ok_or("packed scene offset overflow")?;
        let end = offset
            .checked_add(words.len())
            .ok_or("packed scene word count overflow")?;
        if end > u32::MAX as usize {
            return Err("packed scene exceeds u32 word offsets");
        }
        let range = PackedTableRange::try_new(offset, words.len(), alignment, 0)?;
        self.words.resize(offset, 0);
        self.words.extend_from_slice(words);
        self.ranges[kind as usize] = range;
        Ok(())
    }

    pub fn append_words(&mut self, kind: SceneTableKind, words: &[u32], stride_words: usize) {
        self.try_append_words(kind, words, stride_words)
            .expect("invalid packed table");
    }

    pub fn append_light_aliases(&mut self, aliases: &PackedLightAliasData) {
        self.append_words(SceneTableKind::LightAliases, aliases.words(), 1);
    }

    pub fn try_finish(self) -> Result<PackedSceneData, &'static str> {
        if self.words.len() > u32::MAX as usize {
            return Err("packed scene exceeds u32 word offsets");
        }
        let mut data = PackedSceneData {
            words: self.words,
            ranges: self.ranges,
        };
        let header = PackedSceneHeader {
            magic: PACKED_BUFFER_MAGIC,
            version: PackedBufferHeader::version(),
            word_count: data.words.len() as u32,
            flags: 0,
            ranges: data.ranges,
        };
        let header_words = to_slice::<PackedSceneHeader, u32>(std::slice::from_ref(&header));
        data.words[..header_words.len()].copy_from_slice(header_words);
        data.validate()?;
        Ok(data)
    }

    pub fn finish(self) -> PackedSceneData {
        self.try_finish().expect("invalid packed scene")
    }
}
