use inox_render::{
    PackedBufferBuilder, PackedBufferHeader, PackedTableRange, PACKED_BUFFER_MAGIC,
    PACKED_SCENE_HEADER_WORDS, WAVEFRONT_RECORD_LAYOUTS, WAVEFRONT_RECORD_VERSION,
};

#[test]
fn packed_builder_aligns_tables_and_records_word_ranges() {
    let mut builder = PackedBufferBuilder::new();
    let first = builder.append_words(&[1, 2, 3], 4);
    let second = builder.append_words(&[4, 5], 4);

    assert_eq!(first, PackedTableRange::new(4, 3, 1, 0));
    assert_eq!(second, PackedTableRange::new(8, 2, 1, 0));
    assert_eq!(
        builder.words(),
        &[PACKED_BUFFER_MAGIC, 1, 12, 0, 1, 2, 3, 0, 4, 5, 0, 0]
    );
}

#[test]
fn packed_headers_and_ranges_keep_their_gpu_layout() {
    assert_eq!(std::mem::size_of::<PackedBufferHeader>(), 4 * 4);
    assert_eq!(std::mem::size_of::<PackedTableRange>(), 4 * 4);
    assert_eq!(PACKED_BUFFER_MAGIC, u32::from_le_bytes(*b"INOX"));
}

#[test]
fn packed_record_descriptors_are_versioned_and_word_addressable() {
    let expected = [
        ("RayQueueRecord", 16),
        ("HitRecord", 8),
        ("AccumulationRecord", 8),
        ("ReservoirRecord", 8),
        ("AoRecord", 4),
        ("TransparentHitRecord", 10),
        ("DispatchArgs", 3),
        ("QueueHeader", 4),
    ];
    assert_eq!(WAVEFRONT_RECORD_LAYOUTS.len(), expected.len());
    for (layout, (name, stride)) in WAVEFRONT_RECORD_LAYOUTS.iter().zip(expected) {
        assert_eq!(layout.name, name);
        assert_eq!(layout.version, WAVEFRONT_RECORD_VERSION);
        assert_eq!(layout.stride_words, stride);
        assert_eq!(layout.alignment_bytes, 4);
    }
    assert_eq!(PACKED_SCENE_HEADER_WORDS, 48);
}

#[test]
fn packed_builder_header_rejects_invalid_or_truncated_data() {
    let mut builder = PackedBufferBuilder::new();
    builder.append_words(&[10, 20, 30, 40], 4);
    let words = builder.words();
    assert!(PackedBufferHeader::validate(words).is_ok());

    let mut invalid_magic = words.to_vec();
    invalid_magic[0] = 0;
    assert!(PackedBufferHeader::validate(&invalid_magic).is_err());

    let mut invalid_version = words.to_vec();
    invalid_version[1] = 0;
    assert!(PackedBufferHeader::validate(&invalid_version).is_err());

    let mut invalid_count = words.to_vec();
    invalid_count[2] = (words.len() as u32) + 1;
    assert!(PackedBufferHeader::validate(&invalid_count).is_err());

    assert!(PackedBufferHeader::validate(&words[..3]).is_err());
}
