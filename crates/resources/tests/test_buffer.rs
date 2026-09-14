use inox_resources::{to_slice, Buffer, BufferData};
use inox_uid::generate_random_uid;

#[test]
fn combines_adjacent_buffer_ranges() {
    let id = generate_random_uid();
    let mut first = BufferData::new(&id, 0, 2);
    let second = BufferData::new(&id, 2, 5);

    assert!(first.is_adjacent(&second));
    assert!(first.combine(&second));
    assert_eq!(first.range(), &(0..5));
    assert_eq!(first.item_count(), 5);
}

#[test]
fn allocates_reads_and_reuses_preallocated_storage() {
    let mut buffer = Buffer::<u32>::default();
    buffer.prealloc::<4>();
    let first_id = generate_random_uid();
    let second_id = generate_random_uid();

    assert_eq!(buffer.allocate(&first_id, &[10, 20]), (false, 0..2));
    assert_eq!(buffer.get(&first_id), Some(&[10, 20][..]));
    assert!(buffer.remove(&first_id));
    assert_eq!(buffer.allocate(&second_id, &[30, 40]), (false, 0..2));
    assert_eq!(buffer.get(&second_id), Some(&[30, 40][..]));
    assert_eq!(buffer.item_count(), 2);
}

#[test]
fn converts_typed_slices_without_changing_the_values() {
    let values = [0x0102_u16, 0x0304_u16];
    let bytes: &[u8] = to_slice(&values);
    let restored: &[u16] = to_slice(bytes);

    assert_eq!(bytes.len(), 4);
    assert_eq!(restored, &values);
}
