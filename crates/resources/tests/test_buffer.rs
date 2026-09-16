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
fn clear_resets_preallocation_state_before_reallocation() {
    let id = generate_random_uid();
    let mut buffer = Buffer::<u32>::default();
    buffer.prealloc::<2>();
    assert_eq!(buffer.allocate(&id, &[1, 2]), (false, 0..2));
    buffer.clear();
    assert_eq!(buffer.allocate(&id, &[3, 4, 5]), (true, 0..3));
    assert_eq!(buffer.get(&id), Some(&[3, 4, 5][..]));
}

#[test]
fn reallocating_an_id_shrinks_grows_and_releases_its_logical_range() {
    let id = generate_random_uid();
    let replacement_id = generate_random_uid();
    let mut buffer = Buffer::<u32>::default();

    assert_eq!(buffer.allocate(&id, &[1, 2, 3, 4]), (true, 0..4));
    assert_eq!(buffer.allocate(&id, &[5, 6]), (false, 0..2));
    assert_eq!(buffer.get(&id), Some(&[5, 6][..]));
    assert_eq!(buffer.item_count(), 2);

    let (need_realloc, grown_range) = buffer.allocate(&id, &[7, 8, 9, 10, 11]);
    assert!(need_realloc);
    assert_eq!(grown_range.len(), 5);
    assert_eq!(buffer.get(&id), Some(&[7, 8, 9, 10, 11][..]));
    assert!(buffer.remove(&id));
    assert!(buffer.get(&id).is_none());
    assert!(buffer.is_empty());

    assert_eq!(buffer.allocate(&replacement_id, &[12, 13]), (false, 0..2));
    assert_eq!(buffer.get(&replacement_id), Some(&[12, 13][..]));
}

#[test]
fn empty_typed_slices_are_safe_to_reinterpret() {
    let values: &[u16] = &[];
    let bytes: &[u8] = to_slice(values);
    assert!(bytes.is_empty());
}

#[test]
fn converts_typed_slices_without_changing_the_values() {
    let values = [0x0102_u16, 0x0304_u16];
    let bytes: &[u8] = to_slice(&values);
    let restored: &[u16] = to_slice(bytes);

    assert_eq!(bytes.len(), 4);
    assert_eq!(restored, &values);
}
