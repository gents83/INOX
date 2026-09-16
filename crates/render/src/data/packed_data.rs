pub const PACKED_BUFFER_MAGIC: u32 = 0x584F_4E49;
pub const PACKED_BUFFER_VERSION: u32 = 1;
pub const PACKED_BUFFER_HEADER_WORDS: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedBufferHeader {
    pub magic: u32,
    pub version: u32,
    pub word_count: u32,
    pub flags: u32,
}

impl PackedBufferHeader {
    pub fn validate(words: &[u32]) -> Result<(), &'static str> {
        if words.len() < PACKED_BUFFER_HEADER_WORDS {
            return Err("packed buffer is missing its header");
        }
        if words[0] != PACKED_BUFFER_MAGIC {
            return Err("packed buffer magic does not match");
        }
        if words[1] != PACKED_BUFFER_VERSION {
            return Err("packed buffer version does not match");
        }
        let word_count = words[2] as usize;
        if word_count < PACKED_BUFFER_HEADER_WORDS || word_count > words.len() {
            return Err("packed buffer word count is invalid");
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedTableRange {
    pub offset_words: u32,
    pub count_words: u32,
    pub stride_words: u32,
    pub flags: u32,
}

impl PackedTableRange {
    pub const fn new(
        offset_words: usize,
        count_words: usize,
        stride_words: usize,
        flags: u32,
    ) -> Self {
        Self {
            offset_words: offset_words as u32,
            count_words: count_words as u32,
            stride_words: stride_words as u32,
            flags,
        }
    }
}

pub struct PackedBufferBuilder {
    words: Vec<u32>,
}

impl Default for PackedBufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PackedBufferBuilder {
    pub fn new() -> Self {
        Self {
            words: vec![
                PACKED_BUFFER_MAGIC,
                PACKED_BUFFER_VERSION,
                PACKED_BUFFER_HEADER_WORDS as u32,
                0,
            ],
        }
    }

    pub fn append_words(&mut self, words: &[u32], alignment_words: usize) -> PackedTableRange {
        let alignment = alignment_words.max(1);
        let aligned_offset = self.words.len().next_multiple_of(alignment);
        self.words.resize(aligned_offset, 0);
        let offset = self.words.len();
        self.words.extend_from_slice(words);
        let end = self.words.len().next_multiple_of(alignment);
        self.words.resize(end, 0);
        self.words[2] = self.words.len() as u32;
        PackedTableRange::new(offset, words.len(), 1, 0)
    }

    pub fn words(&self) -> &[u32] {
        &self.words
    }
}
