use inox_render::{
    PackedSceneBuilder, PackedSceneHeader, PackedTableRange, SceneTableKind, PACKED_BUFFER_MAGIC,
    PACKED_BUFFER_VERSION,
};

#[test]
fn packed_scene_header_has_an_exact_word_addressable_layout() {
    assert_eq!(std::mem::size_of::<PackedSceneHeader>(), 192);
    assert_eq!(std::mem::size_of::<PackedSceneHeader>() / 4, 48);
    assert_eq!(std::mem::size_of::<PackedTableRange>(), 16);
}

#[test]
fn packed_scene_stores_aligned_ranges_for_each_table() {
    let mut builder = PackedSceneBuilder::new();
    builder.append_typed(SceneTableKind::Indices, &[1u32, 2, 3], 1);
    builder.append_words(SceneTableKind::Bvh, &[4, 5, 6, 7], 4);
    let scene = builder.finish();

    assert_eq!(scene.words()[0], PACKED_BUFFER_MAGIC);
    assert_eq!(scene.words()[1], PACKED_BUFFER_VERSION);
    assert_eq!(scene.range(SceneTableKind::Indices).count_words, 3);
    assert_eq!(scene.range(SceneTableKind::Bvh).offset_words % 4, 0);
    assert!(scene.validate().is_ok());
}

#[test]
fn packed_scene_keeps_every_table_range_inside_the_payload() {
    let mut builder = PackedSceneBuilder::new();
    builder.append_words(SceneTableKind::Meshes, &[1, 2], 4);
    builder.append_words(SceneTableKind::Bvh, &[3, 4, 5], 4);
    let mut scene = builder.finish();
    assert!(scene.validate().is_ok());

    for kind in [
        SceneTableKind::Indices,
        SceneTableKind::VertexPositions,
        SceneTableKind::VertexAttributes,
        SceneTableKind::Instances,
        SceneTableKind::Transforms,
        SceneTableKind::Meshes,
        SceneTableKind::Meshlets,
        SceneTableKind::Bvh,
        SceneTableKind::Materials,
        SceneTableKind::Textures,
        SceneTableKind::Lights,
    ] {
        let range = scene.range(kind);
        assert!(range.offset_words as usize + range.count_words as usize <= scene.words().len());
        assert!(range.count_words == 0 || range.stride_words > 0);
    }

    scene.words_mut()[2] = 4;
    assert!(scene.validate().is_err());
}
