use inox_render::{DrawIndexedCommand, DRAW_INDEXED_COMMAND_OFFSET, DRAW_INDEXED_COMMAND_STRIDE};

#[test]
fn indirect_indexed_draw_stride_matches_the_gpu_record_layout() {
    assert_eq!(
        DRAW_INDEXED_COMMAND_STRIDE,
        std::mem::size_of::<DrawIndexedCommand>() as u64
    );
    assert_eq!(DRAW_INDEXED_COMMAND_STRIDE, 20);
}

#[test]
fn indirect_indexed_draw_starts_after_the_vector_count_prefix() {
    assert_eq!(DRAW_INDEXED_COMMAND_OFFSET, 16);
    assert_eq!(DRAW_INDEXED_COMMAND_OFFSET, 16);
}
