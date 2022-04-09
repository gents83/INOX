use std::ops::Range;

use crate::ResourceId;
use inox_uid::{generate_random_uid, INVALID_UID};

pub fn from_u8_slice<T: Sized>(a: &[u8]) -> &[T] {
    unsafe {
        let len = a.len() / ::std::mem::size_of::<T>();
        ::std::slice::from_raw_parts((&a[0] as *const u8) as *const T, len)
    }
}
pub fn from_u8_slice_mut<T: Sized>(a: &mut [u8]) -> &mut [T] {
    unsafe {
        let len = a.len() / ::std::mem::size_of::<T>();
        ::std::slice::from_raw_parts_mut((&mut a[0] as *mut u8) as *mut T, len)
    }
}

pub fn to_u8_slice<T: Sized>(a: &[T]) -> &[u8] {
    if a.is_empty() {
        inox_log::debug_log!("to_u8_slice: empty slice");
    }
    unsafe {
        let len = a.len() * ::std::mem::size_of::<T>();
        ::std::slice::from_raw_parts((&a[0] as *const T) as *const u8, len)
    }
}

#[derive(Clone)]
pub struct BufferData {
    id: ResourceId,
    range: Range<usize>,
    item_size: usize,
}

impl Default for BufferData {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            range: 0..0,
            item_size: std::mem::size_of::<u8>(),
        }
    }
}

impl BufferData {
    pub fn new(id: &ResourceId, start: usize, end: usize, item_size: usize) -> Self {
        Self {
            id: *id,
            range: start..end,
            item_size,
        }
    }
    #[inline]
    pub fn is_adjacent(&self, buffer: &BufferData) -> bool {
        if buffer.range.start > 0 && self.range.end == (buffer.range.start - 1) {
            return true;
        }
        if self.range.start > 0 && buffer.range.end == (self.range.start - 1) {
            return true;
        }
        false
    }
    #[inline]
    pub fn combine(&mut self, buffer: &BufferData) -> bool {
        if buffer.range.start > 0 && self.range.end == (buffer.range.start - 1) {
            self.range.end = buffer.range.end;
            return true;
        }
        if self.range.start > 0 && buffer.range.end == (self.range.start - 1) {
            self.range.start = buffer.range.start;
            return true;
        }
        false
    }
    pub fn range(&self) -> &Range<usize> {
        &self.range
    }
    pub fn item_range(&self) -> Range<usize> {
        (self.range.start / self.item_size)
            ..((self.range.start / self.item_size) + (self.range.len() / self.item_size))
    }
    pub fn item_count(&self) -> usize {
        self.range.len() / self.item_size
    }
    pub fn total_len(&self) -> usize {
        self.range.len()
    }
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }
}

#[derive(Clone, Default)]
pub struct Buffer {
    occupied: Vec<BufferData>,
    free: Vec<BufferData>,
    data: Vec<u8>,
}

impl Buffer {
    pub fn allocate_with_size(&mut self, id: &ResourceId, data: &[u8], item_size: usize) -> bool {
        self.collapse_free();
        let mut need_realloc = false;
        let size = data.len();
        if let Some(index) = self
            .free
            .iter()
            .position(|d| (d.range.end + 1 - d.range.start) >= size)
        {
            let free_data = self.free.remove(index);
            if (free_data.range.end + 1 - free_data.range.start) > size {
                self.free.push(BufferData::new(
                    &generate_random_uid(),
                    free_data.range.start + size,
                    free_data.range.end,
                    item_size,
                ));
            }
            self.insert_at(id, free_data.range.start, data, item_size);
        } else {
            self.insert(id, data, item_size);
            need_realloc = true;
        }
        need_realloc
    }
    pub fn allocate<T>(&mut self, id: &ResourceId, data: &[T]) -> bool {
        self.allocate_with_size(id, to_u8_slice(data), std::mem::size_of::<T>())
    }
    fn insert(&mut self, id: &ResourceId, data: &[u8], item_size: usize) {
        let start = self.data.len();
        let size = data.len();
        let end = start + size - 1;
        //inox_log::debug_log!("[{:?}] added, [start {} : end {}]", id, start, end);

        self.data.extend_from_slice(data);
        self.occupied
            .push(BufferData::new(id, start, end, item_size));
    }
    fn insert_at(&mut self, id: &ResourceId, start: usize, data: &[u8], item_size: usize) {
        debug_assert!(start <= self.data.len());
        let size = data.len();
        let end = start + size - 1;
        //inox_log::debug_log!("[{:?}] inserting at {}", id, start);
        self.update(start, data);
        if let Some(i) = self
            .occupied
            .iter()
            .position(|d| (d.range.end + 1) == start)
        {
            self.occupied
                .insert(i + 1, BufferData::new(id, start, end, item_size));
        } else if let Some(i) = self.occupied.iter().position(|d| d.range.start > end) {
            self.occupied
                .insert(i, BufferData::new(id, start, end, item_size));
        } else {
            self.occupied
                .push(BufferData::new(id, start, end, item_size));
        }
    }
    pub fn update(&mut self, start: usize, data: &[u8]) {
        debug_assert!(start <= self.data.len());
        /*
        inox_log::debug_log!(
            "owerwriting, [start {} : end {}]",
            start,
            start + data.len() - 1
        );
        */
        let data = to_u8_slice(data);
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
            count += (b.range.end + 1 - b.range.start) / b.item_size;
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
    pub fn get(&self, id: &ResourceId) -> Option<&BufferData> {
        self.occupied.iter().find(|d| d.id == *id)
    }
    pub fn remove_with_id(&mut self, id: &ResourceId) -> bool {
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
    pub fn for_each_data<F, T>(&self, mut f: F)
    where
        F: FnMut(usize, &ResourceId, &T),
    {
        self.occupied.iter().for_each(|b| {
            let func = &mut f;
            self.data[b.range.start..(b.range.end + 1)]
                .chunks(std::mem::size_of::<T>())
                .enumerate()
                .for_each(|(i, d)| {
                    func(b.range.start + i, &b.id, from_u8_slice(d)[0]);
                });
        });
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn total_data<T>(&self) -> &[T] {
        from_u8_slice(&self.data)
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
    fn defrag(&mut self) {
        self.free.clear();
        let mut new_data = Vec::<u8>::new();
        let mut last_index = 0;
        self.occupied.iter_mut().for_each(|d| {
            new_data.extend_from_slice(&self.data[d.range.start..=d.range.end]);
            d.range.start = last_index;
            last_index = new_data.len();
            d.range.end = last_index - 1;
        });
        self.data = new_data;
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

    let mut buffer = Buffer::default();

    let mut meshes = Vec::new();
    let mut mesh = Mesh::new();
    mesh.add(NUM_VERTICES);
    let mut octo_mesh_1 = Mesh::new();
    octo_mesh_1.add(2 * NUM_VERTICES);
    let mut octo_mesh_2 = Mesh::new();
    octo_mesh_2.add(2 * NUM_VERTICES);
    for _ in 0..NUM_MESHES {
        meshes.push(mesh.clone());
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
        NUM_VERTICES as usize * std::mem::size_of::<Data>(),
        "Allocator should hold a quad"
    );

    buffer.remove_with_id(&mesh.id);

    assert_eq!(buffer.item_count(), 0, "Allocator should be 0");
    assert_eq!(
        buffer.total_len(),
        NUM_VERTICES as usize * std::mem::size_of::<Data>(),
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
        "Allocator should hold {} quad",
        NUM_MESHES
    );

    assert_eq!(
        buffer.total_data::<Data>().len(),
        mesh.data.len() * NUM_MESHES,
        "Allocator should hold {} quad",
        NUM_MESHES
    );

    buffer.remove_with_id(&meshes[1].id);
    buffer.remove_with_id(&meshes[2].id);

    assert_eq!(
        buffer.total_len(),
        mesh.data.len() * NUM_MESHES * std::mem::size_of::<Data>(),
        "Allocator should hold anyway {} quad",
        NUM_MESHES
    );
    assert_eq!(
        buffer.item_count(),
        mesh.data.len() * 2,
        "Allocator should hold only 2 quad",
    );

    buffer.allocate(&octo_mesh_1.id, &octo_mesh_1.data);

    assert_eq!(
        buffer.item_count(),
        mesh.data.len() * 2 + octo_mesh_1.data.len(),
        "Allocator should hold anyway {} quads + 1 octo",
        NUM_MESHES / 2
    );
    assert_eq!(
        buffer.total_len(),
        (mesh.data.len() * (NUM_MESHES / 2) + octo_mesh_1.data.len()) * std::mem::size_of::<Data>(),
        "Allocator should hold anyway {} quads + 1 octo",
        NUM_MESHES / 2
    );
    assert!(buffer.is_full(), "Allocator should be full now");

    buffer.remove_with_id(&meshes[0].id);

    assert_eq!(
        buffer.total_len(),
        (mesh.data.len() * (NUM_MESHES / 2) + octo_mesh_1.data.len()) * std::mem::size_of::<Data>(),
        "Allocator should hold anyway {} quads + 1 octo",
        NUM_MESHES / 2
    );
    assert_eq!(
        buffer.item_count(),
        mesh.data.len() + octo_mesh_1.data.len(),
        "Allocator should have some space {} vs {}",
        buffer.item_count(),
        mesh.data.len() + octo_mesh_1.data.len(),
    );

    buffer.allocate_with_size(
        &octo_mesh_2.id,
        to_u8_slice(&octo_mesh_2.data),
        std::mem::size_of::<Data>(),
    );

    assert_eq!(
        buffer.total_len(),
        (mesh.data.len() * NUM_MESHES / 2 + octo_mesh_1.data.len() + octo_mesh_2.data.len())
            * std::mem::size_of::<Data>(),
        "Allocator should hold anyway {} quads + 2 octos",
        NUM_MESHES / 2
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

#[test]
fn test() {
    test_buffer();
}
