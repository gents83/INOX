const PACKED_SCENE_TABLES: u32 = 12u;

fn scene_storage_words() -> u32 { return arrayLength(&scene_data); }
fn scene_word_count() -> u32 {
    if (scene_storage_words() <= 2u) { return 0u; }
    return scene_data[2u];
}
fn table_header_word(kind: u32) -> u32 { return 4u + kind * 4u; }
fn table_offset(kind: u32) -> u32 {
    if (kind >= PACKED_SCENE_TABLES || scene_storage_words() < PACKED_SCENE_HEADER_WORDS) { return 0u; }
    let header = table_header_word(kind);
    return scene_data[header];
}
fn table_count(kind: u32) -> u32 {
    if (kind >= PACKED_SCENE_TABLES || scene_storage_words() < PACKED_SCENE_HEADER_WORDS) { return 0u; }
    let header = table_header_word(kind);
    return scene_data[header + 1u];
}
fn table_words(kind: u32) -> u32 { return table_count(kind); }
fn table_stride(kind: u32) -> u32 {
    if (kind >= PACKED_SCENE_TABLES || scene_storage_words() < PACKED_SCENE_HEADER_WORDS) { return 0u; }
    return scene_data[table_header_word(kind) + 2u];
}
fn packed_table_expected_stride(kind: u32) -> u32 {
    switch kind {
        case 0u, 1u, 2u, 11u: { return 1u; }
        case 3u: { return 4u; }
        case 4u: { return 16u; }
        case 5u: { return 16u; }
        case 6u: { return 20u; }
        case 7u: { return 8u; }
        case 8u: { return 48u; }
        case 9u: { return 4u; }
        case 10u: { return 16u; }
        default: { return 0u; }
    }
}
fn packed_scene_valid() -> bool {
    let storage = scene_storage_words();
    if (storage < PACKED_SCENE_HEADER_WORDS || scene_data[0u] != PACKED_BUFFER_MAGIC
        || scene_data[1u] != PACKED_BUFFER_VERSION) { return false; }
    let words = scene_data[2u];
    if (words != storage || words < PACKED_SCENE_HEADER_WORDS) { return false; }
    for (var kind = 0u; kind < PACKED_SCENE_TABLES; kind += 1u) {
        let header = table_header_word(kind);
        let offset = scene_data[header];
        let count = scene_data[header + 1u];
        let stride = scene_data[header + 2u];
        if (offset > words || (count > 0u && offset < PACKED_SCENE_HEADER_WORDS)
            || (count > 0u && stride == 0u) || (stride > 0u && count % stride != 0u)
            || (count > 0u && stride != packed_table_expected_stride(kind))) { return false; }
        if (count > words - offset) { return false; }
    }
    for (var left = 0u; left < PACKED_SCENE_TABLES; left += 1u) {
        let left_offset = scene_data[table_header_word(left)];
        let left_count = scene_data[table_header_word(left) + 1u];
        if (left_count == 0u) { continue; }
        for (var right = left + 1u; right < PACKED_SCENE_TABLES; right += 1u) {
            let right_offset = scene_data[table_header_word(right)];
            let right_count = scene_data[table_header_word(right) + 1u];
            if (right_count > 0u) {
                let left_end = left_offset + left_count;
                let right_end = right_offset + right_count;
                if (left_offset < right_end && right_offset < left_end) { return false; }
            }
        }
    }
    return true;
}
fn range_has_words(kind: u32, offset: u32, count: u32) -> bool {
    if (kind >= PACKED_SCENE_TABLES) { return false; }
    let base = table_offset(kind);
    let length = table_count(kind);
    let words = scene_word_count();
    return base <= words && length <= words - base && offset <= length
        && count <= length - offset && count <= words - base - offset;
}
fn range_has_words_at(kind: u32, base_offset: u32, relative_offset: u32, count: u32) -> bool {
    if (kind >= PACKED_SCENE_TABLES) { return false; }
    let length = table_count(kind);
    if (base_offset > length || relative_offset > length - base_offset) { return false; }
    return count <= length - base_offset - relative_offset
        && range_has_words(kind, base_offset + relative_offset, count);
}
fn range_has_indexed_words(kind: u32, base_offset: u32, index: u32, stride: u32, count: u32) -> bool {
    if (stride == 0u || kind >= PACKED_SCENE_TABLES) { return false; }
    let length = table_count(kind);
    if (base_offset > length || count > length - base_offset
        || index >= (length - base_offset) / stride) { return false; }
    let relative_offset = index * stride;
    return count <= length - base_offset - relative_offset
        && range_has_words(kind, base_offset + relative_offset, count);
}
fn scene_relative_index_valid(kind: u32, base_index: u32, relative_index: u32, stride: u32) -> bool {
    if (stride == 0u || kind >= PACKED_SCENE_TABLES) { return false; }
    let length = table_count(kind);
    let records = length / stride;
    if (base_index > records || relative_index >= records - base_index) { return false; }
    let absolute_index = base_index + relative_index;
    return scene_index_valid(kind, absolute_index, stride);
}
fn scene_index_valid(kind: u32, index: u32, stride: u32) -> bool {
    if (stride == 0u || kind >= PACKED_SCENE_TABLES) { return false; }
    let length = table_count(kind);
    if (index >= length / stride) { return false; }
    let relative = index * stride;
    return range_has_words(kind, relative, stride);
}
fn packed_table_range_valid(kind: u32, offset: u32, count: u32, stride: u32) -> bool {
    return stride > 0u && count % stride == 0u && range_has_words(kind, offset, count);
}
fn load_scene_f32(offset: u32) -> f32 {
    if (offset >= scene_storage_words()) { return 0.0; }
    return bitcast<f32>(scene_data[offset]);
}
fn load_f32(offset: u32) -> f32 { return load_scene_f32(offset); }
fn load_vec3(offset: u32) -> vec3<f32> {
    if (offset > scene_storage_words() || scene_storage_words() - offset < 3u) { return vec3<f32>(0.0); }
    return vec3<f32>(load_scene_f32(offset), load_scene_f32(offset + 1u), load_scene_f32(offset + 2u));
}
