use inox_render::{
    GPULight, PackedLightAliasData, PackedSceneBuilder, PackedSceneHeader, PackedTableRange,
    SceneTableKind, PACKED_BUFFER_MAGIC, PACKED_BUFFER_VERSION, PACKED_LIGHT_ALIAS_CAPACITY,
    PACKED_LIGHT_ALIAS_FLAG_OVERFLOW, PACKED_SCENE_HEADER_WORDS,
};

#[test]
fn packed_scene_header_has_an_exact_word_addressable_layout() {
    assert_eq!(std::mem::size_of::<PackedSceneHeader>(), 208);
    assert_eq!(std::mem::size_of::<PackedSceneHeader>() / 4, 52);
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
fn packed_scene_rejects_ranges_that_overlap_header() {
    let mut builder = PackedSceneBuilder::new();
    builder.append_words(SceneTableKind::Indices, &[1, 2, 3, 4], 1);
    let mut scene = builder.finish();
    scene.words_mut()[4] = (PACKED_SCENE_HEADER_WORDS - 1) as u32;
    assert!(scene.validate().is_err());
}

#[test]
fn packed_scene_rejects_counts_that_are_not_stride_aligned() {
    let mut builder = PackedSceneBuilder::new();
    builder.append_words(SceneTableKind::Indices, &[1, 2, 3, 4], 2);
    let mut scene = builder.finish();
    scene.words_mut()[5] = 3;
    assert!(scene.validate().is_err());
}

#[test]
fn packed_scene_keeps_every_table_range_inside_the_payload() {
    let mut builder = PackedSceneBuilder::new();
    builder.append_words(SceneTableKind::Meshes, &[1, 2, 3, 4], 4);
    builder.append_words(SceneTableKind::Bvh, &[3, 4, 5, 6], 4);
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
        SceneTableKind::LightAliases,
    ] {
        let range = scene.range(kind);
        assert!(range.offset_words as usize + range.count_words as usize <= scene.words().len());
        assert!(range.count_words == 0 || range.stride_words > 0);
    }

    scene.words_mut()[2] = 4;
    assert!(scene.validate().is_err());
}

#[test]
fn light_alias_table_handles_empty_and_single_power() {
    let empty = PackedLightAliasData::from_powers(&[]);
    assert_eq!(empty.header().count, 0);
    assert_eq!(empty.words().len(), 4);

    let single = PackedLightAliasData::from_powers(&[3.0]);
    assert_eq!(single.header().count, 1);
    let record = single.record(0).unwrap();
    assert_eq!(record.alias_index, 0);
    assert_eq!(record.selection_probability, 1.0);
    assert_eq!(record.proposal_pdf, 1.0);
}

#[test]
fn light_alias_table_sanitizes_zero_nonfinite_and_marks_overflow() {
    let values = [0.0, f32::NAN, f32::INFINITY, 2.0, 1.0];
    let aliases = PackedLightAliasData::from_powers(&values);
    assert_eq!(aliases.header().count, values.len() as u32);
    assert!(aliases.record(3).unwrap().proposal_pdf > 0.0);
    assert_eq!(aliases.record(0).unwrap().proposal_pdf, 0.0);
    assert_eq!(aliases.record(1).unwrap().proposal_pdf, 0.0);
    assert_eq!(aliases.record(2).unwrap().proposal_pdf, 0.0);

    let oversized = PackedLightAliasData::from_powers(&vec![1.0; PACKED_LIGHT_ALIAS_CAPACITY + 1]);
    assert_eq!(
        oversized.header().count as usize,
        PACKED_LIGHT_ALIAS_CAPACITY
    );
    assert_ne!(
        oversized.header().flags & PACKED_LIGHT_ALIAS_FLAG_OVERFLOW,
        0
    );
}

#[test]
fn packed_scene_validates_light_alias_header_and_range() {
    let aliases = PackedLightAliasData::from_powers(&[1.0, 2.0, 4.0]);
    let mut builder = PackedSceneBuilder::new();
    builder.append_light_aliases(&aliases);
    let scene = builder.finish();
    assert_eq!(
        scene.range(SceneTableKind::LightAliases).count_words,
        aliases.words().len() as u32
    );
    assert!(scene.validate().is_ok());
}

#[test]
fn packed_scene_rejects_alias_indices_outside_the_table() {
    let aliases = PackedLightAliasData::from_powers(&[1.0, 2.0]);
    let mut builder = PackedSceneBuilder::new();
    builder.append_light_aliases(&aliases);
    let mut scene = builder.finish();
    let offset = scene.range(SceneTableKind::LightAliases).offset_words as usize;
    scene.words_mut()[offset + 4] = 2;
    assert_eq!(
        scene.validate(),
        Err("packed light alias record is invalid")
    );
}

#[test]
fn packed_scene_rejects_nonfinite_or_out_of_range_alias_probabilities() {
    for (word_offset, value) in [(5, f32::NAN.to_bits()), (6, 2.0f32.to_bits())] {
        let aliases = PackedLightAliasData::from_powers(&[1.0, 2.0]);
        let mut builder = PackedSceneBuilder::new();
        builder.append_light_aliases(&aliases);
        let mut scene = builder.finish();
        let offset = scene.range(SceneTableKind::LightAliases).offset_words as usize;
        scene.words_mut()[offset + word_offset] = value;
        assert_eq!(
            scene.validate(),
            Err("packed light alias record is invalid")
        );
    }
}

#[test]
fn packed_scene_preserves_finite_representative_light_power() {
    let light = GPULight {
        position: [0.0, 1.0, 0.0],
        color: [1.0, 0.25, 0.1],
        intensity: 8.0,
        ..Default::default()
    };
    let power = light.sampling_power();
    assert!(power.is_finite() && power > 0.0);
    let aliases = PackedLightAliasData::from_lights(&[light]);
    assert_eq!(aliases.header().count, 1);
    assert!(aliases.record(0).unwrap().proposal_pdf > 0.0);
}
