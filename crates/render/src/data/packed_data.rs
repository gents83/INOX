pub const PACKED_BUFFER_MAGIC: u32 = 0x584F_4E49;
pub const PACKED_BUFFER_VERSION: u32 = 1;
pub const PACKED_BUFFER_HEADER_WORDS: usize = 4;
pub const PACKED_SCENE_HEADER_WORDS: usize = 52;
pub const PACKED_LIGHT_ALIAS_VERSION: u32 = 1;
pub const PACKED_LIGHT_ALIAS_HEADER_WORDS: usize = 4;
pub const PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS: usize = 4;
pub const PACKED_LIGHT_ALIAS_CAPACITY: usize = 1024;
pub const PACKED_LIGHT_ALIAS_FLAG_OVERFLOW: u32 = 1;
pub const WAVEFRONT_RECORD_VERSION: u32 = 1;
pub const WAVEFRONT_RAY_QUEUE_RECORD_STRIDE_WORDS: usize = 16;
pub const WAVEFRONT_HIT_RECORD_STRIDE_WORDS: usize = 8;
pub const WAVEFRONT_ACCUMULATION_RECORD_STRIDE_WORDS: usize = 8;
pub const WAVEFRONT_RESERVOIR_RECORD_STRIDE_WORDS: usize = 8;
pub const WAVEFRONT_AO_RECORD_STRIDE_WORDS: usize = 4;
pub const WAVEFRONT_TRANSPARENT_HIT_RECORD_STRIDE_WORDS: usize = 10;
pub const WAVEFRONT_DISPATCH_ARGS_STRIDE_WORDS: usize = 3;
pub const WAVEFRONT_QUEUE_HEADER_STRIDE_WORDS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedRecordLayout {
    pub name: &'static str,
    pub version: u32,
    pub stride_words: usize,
    pub alignment_bytes: usize,
}

pub const WAVEFRONT_RECORD_LAYOUTS: &[PackedRecordLayout] = &[
    PackedRecordLayout {
        name: "RayQueueRecord",
        version: WAVEFRONT_RECORD_VERSION,
        stride_words: WAVEFRONT_RAY_QUEUE_RECORD_STRIDE_WORDS,
        alignment_bytes: 4,
    },
    PackedRecordLayout {
        name: "HitRecord",
        version: WAVEFRONT_RECORD_VERSION,
        stride_words: WAVEFRONT_HIT_RECORD_STRIDE_WORDS,
        alignment_bytes: 4,
    },
    PackedRecordLayout {
        name: "AccumulationRecord",
        version: WAVEFRONT_RECORD_VERSION,
        stride_words: WAVEFRONT_ACCUMULATION_RECORD_STRIDE_WORDS,
        alignment_bytes: 4,
    },
    PackedRecordLayout {
        name: "ReservoirRecord",
        version: WAVEFRONT_RECORD_VERSION,
        stride_words: WAVEFRONT_RESERVOIR_RECORD_STRIDE_WORDS,
        alignment_bytes: 4,
    },
    PackedRecordLayout {
        name: "AoRecord",
        version: WAVEFRONT_RECORD_VERSION,
        stride_words: WAVEFRONT_AO_RECORD_STRIDE_WORDS,
        alignment_bytes: 4,
    },
    PackedRecordLayout {
        name: "TransparentHitRecord",
        version: WAVEFRONT_RECORD_VERSION,
        stride_words: WAVEFRONT_TRANSPARENT_HIT_RECORD_STRIDE_WORDS,
        alignment_bytes: 4,
    },
    PackedRecordLayout {
        name: "DispatchArgs",
        version: WAVEFRONT_RECORD_VERSION,
        stride_words: WAVEFRONT_DISPATCH_ARGS_STRIDE_WORDS,
        alignment_bytes: 4,
    },
    PackedRecordLayout {
        name: "QueueHeader",
        version: WAVEFRONT_RECORD_VERSION,
        stride_words: WAVEFRONT_QUEUE_HEADER_STRIDE_WORDS,
        alignment_bytes: 4,
    },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedBufferHeader {
    pub magic: u32,
    pub version: u32,
    pub word_count: u32,
    pub flags: u32,
}

impl PackedBufferHeader {
    pub const fn version() -> u32 {
        PACKED_BUFFER_VERSION
    }

    pub fn validate(words: &[u32]) -> Result<(), &'static str> {
        if words.len() < PACKED_BUFFER_HEADER_WORDS {
            return Err("packed buffer is missing its header");
        }
        if words[0] != PACKED_BUFFER_MAGIC {
            return Err("packed buffer magic does not match");
        }
        if words[1] != Self::version() {
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
    pub fn try_new(
        offset_words: usize,
        count_words: usize,
        stride_words: usize,
        flags: u32,
    ) -> Result<Self, &'static str> {
        if offset_words > u32::MAX as usize
            || count_words > u32::MAX as usize
            || stride_words > u32::MAX as usize
        {
            return Err("packed table range exceeds u32 offsets");
        }
        if count_words != 0 && stride_words == 0 {
            return Err("packed table range has a zero stride");
        }
        if stride_words != 0 && !count_words.is_multiple_of(stride_words) {
            return Err("packed table range is not stride aligned");
        }
        Ok(Self {
            offset_words: offset_words as u32,
            count_words: count_words as u32,
            stride_words: stride_words as u32,
            flags,
        })
    }

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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedLightAliasHeader {
    pub version: u32,
    pub count: u32,
    pub flags: u32,
    pub total_power: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PackedLightAliasRecord {
    pub alias_index: u32,
    pub selection_probability: f32,
    pub proposal_pdf: f32,
    pub _padding: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackedLightAliasData {
    words: Vec<u32>,
}

impl PackedLightAliasData {
    pub fn from_lights(lights: &[crate::GPULight]) -> Self {
        let powers = lights
            .iter()
            .map(crate::GPULight::sampling_power)
            .collect::<Vec<_>>();
        Self::from_powers(&powers)
    }

    pub fn from_powers(powers: &[f32]) -> Self {
        let count = powers.len().min(PACKED_LIGHT_ALIAS_CAPACITY);
        let overflowed = powers.len() > PACKED_LIGHT_ALIAS_CAPACITY;
        let sanitized = powers[..count]
            .iter()
            .map(|power| {
                if power.is_finite() && *power > 0.0 {
                    *power
                } else {
                    0.0
                }
            })
            .collect::<Vec<_>>();
        let total_power: f32 = sanitized.iter().sum();
        let probabilities = if count == 0 {
            Vec::new()
        } else if total_power.is_finite() && total_power > 0.0 {
            sanitized.iter().map(|power| *power / total_power).collect()
        } else {
            vec![1.0 / count as f32; count]
        };

        let mut scaled = probabilities
            .iter()
            .map(|probability| probability * count as f32)
            .collect::<Vec<_>>();
        let mut alias = (0..count).collect::<Vec<_>>();
        let mut small = (0..count)
            .filter(|index| scaled[*index] < 1.0)
            .collect::<Vec<_>>();
        let mut large = (0..count)
            .filter(|index| scaled[*index] >= 1.0)
            .collect::<Vec<_>>();
        while let (Some(small_index), Some(large_index)) = (small.pop(), large.pop()) {
            alias[small_index] = large_index;
            scaled[large_index] = (scaled[large_index] + scaled[small_index] - 1.0).max(0.0);
            if scaled[large_index] < 1.0 {
                small.push(large_index);
            } else {
                large.push(large_index);
            }
        }

        let mut words = Vec::with_capacity(
            PACKED_LIGHT_ALIAS_HEADER_WORDS + count * PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS,
        );
        words.extend_from_slice(&[
            PACKED_LIGHT_ALIAS_VERSION,
            count as u32,
            if overflowed {
                PACKED_LIGHT_ALIAS_FLAG_OVERFLOW
            } else {
                0
            },
            if total_power.is_finite() && total_power > 0.0 {
                total_power.to_bits()
            } else {
                0
            },
        ]);
        for index in 0..count {
            words.extend_from_slice(&[
                alias[index] as u32,
                scaled[index].clamp(0.0, 1.0).to_bits(),
                probabilities[index].to_bits(),
                0,
            ]);
        }
        Self { words }
    }

    pub fn words(&self) -> &[u32] {
        &self.words
    }

    pub fn header(&self) -> PackedLightAliasHeader {
        PackedLightAliasHeader {
            version: self.words[0],
            count: self.words[1],
            flags: self.words[2],
            total_power: self.words[3],
        }
    }

    pub fn record(&self, index: usize) -> Option<PackedLightAliasRecord> {
        if index >= self.header().count as usize {
            return None;
        }
        let base = PACKED_LIGHT_ALIAS_HEADER_WORDS + index * PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS;
        Some(PackedLightAliasRecord {
            alias_index: self.words[base],
            selection_probability: f32::from_bits(self.words[base + 1]),
            proposal_pdf: f32::from_bits(self.words[base + 2]),
            _padding: self.words[base + 3],
        })
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
                PackedBufferHeader::version(),
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
