use std::{any::Any, ops::Range, sync::Arc};

use crate::ResourceId;
use inox_uid::{generate_random_uid, INVALID_UID};

pub fn to_slice_mut<T: Sized, U: Sized>(a: &mut [T]) -> &mut [U] {
    if a.is_empty() {
        inox_log::debug_log!("to_chunk_slice: empty slice");
    }
    unsafe {
        let len = ::std::mem::size_of_val(a) / ::std::mem::size_of::<U>();
        ::std::slice::from_raw_parts_mut((&a[0] as *const T) as *mut U, len)
    }
}

pub fn to_slice<T: Sized, U: Sized>(a: &[T]) -> &[U] {
    if a.is_empty() {
        inox_log::debug_log!("to_chunk_slice: empty slice");
    }
    unsafe {
        let len = ::std::mem::size_of_val(a) / ::std::mem::size_of::<U>();
        ::std::slice::from_raw_parts((&a[0] as *const T) as *const U, len)
    }
}

pub fn as_slice<T: Sized, U: Sized>(a: &T) -> &[U] {
    unsafe {
        let len = ::std::mem::size_of::<T>() / ::std::mem::size_of::<U>();
        ::std::slice::from_raw_parts((a as *const T) as *const U, len)
    }
}

#[derive(Clone)]
pub struct BufferData {
    id: ResourceId,
    range: Range<usize>,
}

impl Default for BufferData {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            range: 0..0,
        }
    }
}

impl BufferData {
    pub fn new(id: &ResourceId, start: usize, end: usize) -> Self {
        Self {
            id: *id,
            range: start..end,
        }
    }
    #[inline]
    pub fn is_adjacent(&self, buffer: &BufferData) -> bool {
        if buffer.range.start > 0 && self.range.end == buffer.range.start {
            return true;
        }
        if self.range.start > 0 && buffer.range.end == self.range.start {
            return true;
        }
        false
    }
    #[inline]
    pub fn combine(&mut self, buffer: &BufferData) -> bool {
        if buffer.range.start > 0 && self.range.end == buffer.range.start {
            self.range.end = buffer.range.end;
            return true;
        }
        if self.range.start > 0 && buffer.range.end == self.range.start {
            self.range.start = buffer.range.start;
            return true;
        }
        false
    }
    pub fn range(&self) -> &Range<usize> {
        &self.range
    }
    pub fn item_count(&self) -> usize {
        self.range.len()
    }
    pub fn total_len(&self) -> usize {
        self.range.len()
    }
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }
}

#[derive(Default)]
pub struct Buffer<T> {
    occupied: Vec<BufferData>,
    free: Vec<BufferData>,
    data: Vec<T>,
    max_size: usize,
}

unsafe impl<T> Sync for Buffer<T> {}
unsafe impl<T> Send for Buffer<T> {}

impl<T> Buffer<T>
where
    T: Sized + Clone + 'static,
{
    pub fn prealloc<const PREALLOCATED_SIZE: usize>(&mut self)
    where
        T: Default,
    {
        self.free = vec![BufferData::new(
            &generate_random_uid(),
            0,
            PREALLOCATED_SIZE,
        )];
        self.data = vec![T::default(); PREALLOCATED_SIZE];
        self.max_size = PREALLOCATED_SIZE;
    }
    pub fn as_any(&self) -> &dyn Any {
        self
    }
    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    pub fn push(&mut self, id: &ResourceId, data: T) -> (bool, Range<usize>) {
        self.remove(id);
        self.collapse_free();
        let mut need_realloc = false;
        let size = 1;
        let range;
        if let Some(index) = self
            .free
            .iter()
            .position(|d| (d.range.end - d.range.start) >= size)
        {
            let free_data = self.free.remove(index);
            if (free_data.range.end - free_data.range.start) > size {
                self.free.push(BufferData::new(
                    &generate_random_uid(),
                    free_data.range.start + size,
                    free_data.range.end,
                ));
            }
            range = self.insert_at(id, free_data.range.start, &[data]);
        } else {
            range = self.insert(id, &[data]);
            need_realloc = true;
        }
        (need_realloc, range)
    }
    pub fn allocate(&mut self, id: &ResourceId, data: &[T]) -> (bool, Range<usize>) {
        self.remove(id);
        self.collapse_free();
        if data.is_empty() {
            return (false, 0..0);
        }
        let mut need_realloc = false;
        let size = data.len();
        let range;
        if let Some(index) = self
            .free
            .iter()
            .position(|d| (d.range.end - d.range.start) >= size)
        {
            let free_data = self.free.remove(index);
            if (free_data.range.end - free_data.range.start) > size {
                self.free.push(BufferData::new(
                    &generate_random_uid(),
                    free_data.range.start + size,
                    free_data.range.end,
                ));
            }
            range = self.insert_at(id, free_data.range.start, data);
        } else {
            range = self.insert(id, data);
            need_realloc = true;
        }
        (need_realloc, range)
    }
    fn insert(&mut self, id: &ResourceId, data: &[T]) -> Range<usize> {
        debug_assert!(
            self.max_size == 0,
            "Trying to add in a buffer with preallocated size!!!"
        );
        let start = self.data.len();
        let size = data.len();
        let end = start + size;
        //inox_log::debug_log!("[{:?}] added, [start {} : end {}]", id, start, end);

        self.data.extend_from_slice(data);
        self.occupied.push(BufferData::new(id, start, end));
        start..end
    }
    fn insert_at(&mut self, id: &ResourceId, start: usize, data: &[T]) -> Range<usize> {
        debug_assert!(start <= self.data.len());
        let size = data.len();
        let end = start + size;

        //inox_log::debug_log!("[{:?}] inserting at {}", id, start);
        self.update(start, data);
        if let Some(i) = self.occupied.iter().position(|d| d.range.end == start) {
            self.occupied.insert(i + 1, BufferData::new(id, start, end));
        } else if let Some(i) = self.occupied.iter().position(|d| d.range.start > end) {
            self.occupied.insert(i, BufferData::new(id, start, end));
        } else {
            self.occupied.push(BufferData::new(id, start, end));
        }
        start..end
    }
    pub fn update(&mut self, start: usize, data: &[T]) {
        debug_assert!(start <= self.data.len());
        /*
        inox_log::debug_log!(
            "owerwriting, [start {} : end {}]",
            start,
            start + data.len() - 1
        );
        */
        let data = to_slice(data);
        self.data[start..(start + data.len())].clone_from_slice(&data[..data.len()]);
    }
    pub fn last(&self) -> Option<&BufferData> {
        self.occupied.last()
    }
    pub fn clear(&mut self) {
        self.occupied.clear();
        self.data.clear();
        self.free.clear();
    }
    pub fn item_count(&self) -> usize {
        let mut count = 0;
        self.occupied.iter().for_each(|b| {
            count += b.range.end - b.range.start;
        });
        count
    }
    pub fn total_len(&self) -> usize {
        self.data.len()
    }
    pub fn find(&self, size: usize) -> Option<usize> {
        self.occupied
            .iter()
            .position(|b| (b.range.end - b.range.start) >= size)
    }
    pub fn indices(&self, id: &ResourceId) -> Option<&BufferData> {
        self.occupied.iter().find(|d| d.id == *id)
    }
    pub fn get(&self, id: &ResourceId) -> Option<&[T]> {
        if let Some(buffer) = self.occupied.iter().find(|d| d.id == *id) {
            return Some(&self.data[buffer.range.start..buffer.range.end]);
        }
        None
    }
    pub fn get_mut(&mut self, id: &ResourceId) -> Option<&mut [T]> {
        if let Some(buffer) = self.occupied.iter().find(|d| d.id == *id) {
            return Some(&mut self.data[buffer.range.start..buffer.range.end]);
        }
        None
    }
    pub fn get_first_with_index(&self, id: &ResourceId) -> Option<(&T, u32)> {
        if let Some(buffer) = self.occupied.iter().find(|d| d.id == *id) {
            return Some((&self.data[buffer.range.start], buffer.range.start as u32));
        }
        None
    }
    pub fn get_first_with_index_mut(&mut self, id: &ResourceId) -> Option<(&mut T, u32)> {
        if let Some(buffer) = self.occupied.iter().find(|d| d.id == *id) {
            return Some((
                &mut self.data[buffer.range.start],
                buffer.range.start as u32,
            ));
        }
        None
    }
    pub fn get_first(&self, id: &ResourceId) -> Option<&T> {
        if let Some(buffer) = self.occupied.iter().find(|d| d.id == *id) {
            return Some(&self.data[buffer.range.start]);
        }
        None
    }
    pub fn get_first_mut(&mut self, id: &ResourceId) -> Option<&mut T> {
        if let Some(buffer) = self.occupied.iter().find(|d| d.id == *id) {
            return Some(&mut self.data[buffer.range.start]);
        }
        None
    }
    pub fn remove(&mut self, id: &ResourceId) -> bool {
        if let Some(index) = self.occupied.iter().position(|d| d.id == *id) {
            let data = self.occupied.remove(index);
            /*
            inox_log::debug_log!(
                "[{:?}] has been removed, [start {} : end {}]",
                id,
                data.start,
                data.end
            );
            */
            self.free.push(data);
            return true;
        }
        false
    }
    pub fn pop(&mut self, index: usize) -> BufferData {
        self.occupied.remove(index)
    }
    pub fn is_empty(&self) -> bool {
        self.occupied.is_empty()
    }
    pub fn is_full(&self) -> bool {
        !self.occupied.is_empty() && self.free.is_empty()
    }
    pub fn for_each_occupied<F>(&self, f: &mut F)
    where
        F: FnMut(&ResourceId, &Range<usize>),
    {
        self.occupied.iter().for_each(|b| {
            f(&b.id, &b.range);
        });
    }
    pub fn for_each_free<F>(&self, f: &mut F)
    where
        F: FnMut(&ResourceId, &Range<usize>),
    {
        self.free.iter().for_each(|b| {
            f(&b.id, &b.range);
        });
    }
    pub fn for_each_data<F>(&self, mut f: F)
    where
        F: FnMut(usize, &ResourceId, &T),
    {
        self.occupied.iter().for_each(|b| {
            let func = &mut f;
            self.data[b.range.start..b.range.end]
                .iter()
                .enumerate()
                .for_each(|(i, d)| {
                    func(b.range.start + i, &b.id, d);
                });
        });
    }
    pub fn for_each_data_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, &ResourceId, &mut T) -> bool,
    {
        self.occupied.iter().for_each(|b| {
            let func = &mut f;
            self.data[b.range.start..b.range.end]
                .iter_mut()
                .enumerate()
                .for_each(|(i, d)| {
                    func(b.range.start + i, &b.id, d);
                });
        });
    }
    pub fn data(&self) -> &[T] {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
    pub fn total_data<U>(&self) -> &[U] {
        to_slice(&self.data)
    }
    pub fn collapse_free(&mut self) {
        if self.free.len() <= 1 {
            return;
        }
        loop {
            let mut collapsed = 0;
            let mut collapsed_list = Vec::<BufferData>::new();
            while !self.free.is_empty() {
                let mut first_buffer: BufferData = BufferData::default();
                let mut rest_buffers: Vec<BufferData> = Vec::new();
                if let Some((first, rest)) = self.free.split_first_mut() {
                    first_buffer = first.clone();
                    rest_buffers = rest.to_vec();
                }
                while !rest_buffers.is_empty() {
                    if let Some(index) = rest_buffers
                        .iter()
                        .position(|a| a.is_adjacent(&first_buffer))
                    {
                        first_buffer.combine(&rest_buffers[index]);
                        rest_buffers.remove(index);
                        self.free.remove(index + 1);
                        collapsed += 1;
                    } else {
                        break;
                    }
                }
                if let Some(i) = collapsed_list
                    .iter()
                    .position(|d| d.range.end > first_buffer.range.start)
                {
                    collapsed_list.insert(i, first_buffer);
                } else {
                    collapsed_list.push(first_buffer);
                }
                self.free.remove(0);
            }
            self.free = collapsed_list;
            if collapsed == 0 {
                break;
            }
        }
    }
    pub fn defrag(&mut self) {
        if !self.free.is_empty() {
            self.free.clear();
            let mut new_data = Vec::<T>::new();
            let mut last_index = 0;
            self.occupied.iter_mut().for_each(|d| {
                new_data.extend_from_slice(&self.data[d.range.start..d.range.end]);
                d.range.start = last_index;
                last_index = new_data.len();
                d.range.end = last_index;
            });
            self.data = new_data;
        }
    }
}

#[allow(dead_code)]
fn test_buffer() {
    #[derive(Default, Clone)]
    struct Data {
        pos_x: f32,
        pos_y: f32,
    }

    #[derive(Clone)]
    struct Mesh {
        id: ResourceId,
        data: Vec<Data>,
    }
    impl Mesh {
        fn new() -> Self {
            Self {
                id: generate_random_uid(),
                data: Vec::new(),
            }
        }
        fn add(&mut self, count: u32) {
            (0..count).for_each(|_| self.data.push(Data::default()));
        }
    }

    const NUM_VERTICES: u32 = 4;
    const NUM_MESHES: usize = 4;

    let mut buffer = Buffer::<Data>::default();

    let mut meshes = Vec::new();
    let mut mesh = Mesh::new();
    mesh.add(NUM_VERTICES);
    let mut octo_mesh_1 = Mesh::new();
    octo_mesh_1.add(2 * NUM_VERTICES);
    let mut octo_mesh_2 = Mesh::new();
    octo_mesh_2.add(2 * NUM_VERTICES);
    for _ in 0..NUM_MESHES {
        let mut mesh = Mesh::new();
        mesh.add(NUM_VERTICES);
        meshes.push(mesh);
    }

    assert!(buffer.is_empty(), "Allocator should be empty");
    buffer.allocate(&mesh.id, mesh.data.as_slice());

    assert_eq!(
        buffer.item_count(),
        NUM_VERTICES as usize,
        "Allocator should hold a quad"
    );
    assert_eq!(
        buffer.total_len(),
        NUM_VERTICES as usize,
        "Allocator should hold a quad"
    );

    buffer.remove(&mesh.id);

    assert_eq!(buffer.item_count(), 0, "Allocator should be 0");
    assert_eq!(
        buffer.total_len(),
        NUM_VERTICES as usize,
        "Allocator should have an empty space for a quad"
    );
    assert!(buffer.is_empty(), "Allocator should be empty");

    buffer.defrag();

    assert_eq!(
        buffer.total_len(),
        0,
        "Allocator should be defragged and completely empty"
    );
    assert!(buffer.is_empty(), "Allocator should be empty");

    meshes.iter().for_each(|m| {
        buffer.allocate(&m.id, &m.data);
    });

    assert_eq!(
        buffer.item_count(),
        mesh.data.len() * NUM_MESHES,
        "Allocator should hold {NUM_MESHES} quad"
    );

    assert_eq!(
        buffer.total_len(),
        mesh.data.len() * NUM_MESHES,
        "Allocator should hold {NUM_MESHES} quad"
    );

    assert_eq!(
        buffer.total_data::<Data>().len(),
        mesh.data.len() * NUM_MESHES,
        "Allocator should hold {NUM_MESHES} quad"
    );

    buffer.remove(&meshes[1].id);
    buffer.remove(&meshes[2].id);

    assert_eq!(
        buffer.total_len(),
        mesh.data.len() * NUM_MESHES,
        "Allocator should hold anyway {NUM_MESHES} quad"
    );
    assert_eq!(
        buffer.item_count(),
        mesh.data.len() * (NUM_MESHES - 2),
        "Allocator should hold only {} quad",
        NUM_MESHES - 2,
    );

    buffer.allocate(&octo_mesh_1.id, &octo_mesh_1.data);

    assert_eq!(
        buffer.item_count(),
        mesh.data.len() * (NUM_MESHES - 2) + octo_mesh_1.data.len(),
        "Allocator should hold anyway {} quads + 1 octo",
        NUM_MESHES - 2
    );
    assert_eq!(
        buffer.total_len(),
        (mesh.data.len() * (NUM_MESHES - 2) + octo_mesh_1.data.len()),
        "Allocator should hold anyway {} quads + 1 octo",
        NUM_MESHES - 2
    );
    assert!(buffer.is_full(), "Allocator should be full now");

    buffer.remove(&meshes[0].id);

    assert_eq!(
        buffer.total_len(),
        (mesh.data.len() * (NUM_MESHES - 2) + octo_mesh_1.data.len()),
        "Allocator should hold anyway {} quads + 1 octo",
        NUM_MESHES - 2
    );
    assert_eq!(
        buffer.item_count(),
        mesh.data.len() + octo_mesh_1.data.len(),
        "Allocator should have some space {} vs {}",
        buffer.item_count(),
        mesh.data.len() + octo_mesh_1.data.len(),
    );

    buffer.allocate(&octo_mesh_2.id, &octo_mesh_2.data);

    assert_eq!(
        buffer.total_len(),
        (mesh.data.len() * (NUM_MESHES - 2) + octo_mesh_1.data.len() + octo_mesh_2.data.len()),
        "Allocator should hold anyway {} quads + 2 octos",
        NUM_MESHES - 2
    );
    assert_eq!(
        buffer.item_count(),
        mesh.data.len() + octo_mesh_1.data.len() + octo_mesh_2.data.len(),
        "Allocator should hold anyway 1 quads + 2 octos",
    );

    buffer.allocate(&meshes[0].id, &mesh.data);

    assert_eq!(
        buffer.item_count(),
        mesh.data.len() * 2 + octo_mesh_1.data.len() + octo_mesh_2.data.len(),
        "Allocator should hold anyway 1 quads + 2 octos",
    );
    assert!(buffer.is_full(), "Allocator should be full now");
}

#[allow(dead_code)]
fn test_buffer_fixed() {
    #[derive(Default, Clone)]
    struct Data {
        integer: i32,
    }
    const MAX_NUM_DATA: usize = 2;
    let buffer = Arc::new(std::sync::RwLock::new(Buffer::<Data>::default()));
    buffer.write().unwrap().prealloc::<MAX_NUM_DATA>();
    let id_0 = generate_random_uid();
    let id_1 = generate_random_uid();
    buffer.write().unwrap().allocate(&id_0, &[Data::default()]);
    buffer.write().unwrap().allocate(&id_1, &[Data::default()]);
    let id_2 = generate_random_uid();
    let b = buffer.clone();

    buffer.write().unwrap().remove(&id_0);
    buffer.write().unwrap().allocate(&id_2, &[Data::default()]);
    let id_3 = generate_random_uid();

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(move || {
        b.write().unwrap().allocate(&id_3, &[Data::default()]);
    });
    std::panic::set_hook(prev_hook);

    assert!(result.is_err());
}

#[test]
fn test() {
    test_buffer();
    test_buffer_fixed();
}
