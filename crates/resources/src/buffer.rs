use std::ops::Range;

use crate::ResourceId;
use inox_uid::{generate_random_uid, INVALID_UID};

#[derive(Clone)]
pub struct BufferData {
    pub id: ResourceId,
    pub range: Range<usize>,
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
    pub fn len(&self) -> usize {
        self.range.len()
    }
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }
}

#[derive(Clone)]
pub struct Buffer<T>
where
    T: Sized + Clone,
{
    occupied: Vec<BufferData>,
    free: Vec<BufferData>,
    data: Vec<T>,
}

impl<T> Default for Buffer<T>
where
    T: Sized + Clone,
{
    fn default() -> Self {
        Self {
            occupied: Vec::new(),
            free: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl<T> Buffer<T>
where
    T: Sized + Clone,
{
    pub fn allocate(&mut self, id: &ResourceId, data: &[T]) -> bool {
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
                ));
            }
            self.insert_at(id, free_data.range.start, data);
        } else {
            self.insert(id, data);
            need_realloc = true;
        }
        need_realloc
    }
    fn insert(&mut self, id: &ResourceId, data: &[T]) {
        let start = self.data.len();
        let end = start + data.len() - 1;
        //inox_log::debug_log!("[{:?}] added, [start {} : end {}]", id, start, end);

        self.data.extend_from_slice(data);
        self.occupied.push(BufferData::new(id, start, end));
    }

    fn insert_at(&mut self, id: &ResourceId, start: usize, data: &[T]) {
        debug_assert!(start <= self.data.len());
        let end = start + data.len() - 1;
        //inox_log::debug_log!("[{:?}] inserting at {}", id, start);
        self.update(start, data);
        if let Some(i) = self
            .occupied
            .iter()
            .position(|d| (d.range.end + 1) == start)
        {
            self.occupied.insert(i + 1, BufferData::new(id, start, end));
        } else if let Some(i) = self.occupied.iter().position(|d| d.range.start > end) {
            self.occupied.insert(i, BufferData::new(id, start, end));
        } else {
            self.occupied.push(BufferData::new(id, start, end));
        }
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
        self.data[start..(start + data.len())].clone_from_slice(&data[..data.len()]);
    }
    pub fn swap(&mut self, index: usize, other: usize) -> bool {
        if index == other {
            return false;
        }
        debug_assert!(index <= self.data.len());
        debug_assert!(other <= self.data.len());
        if let Some(index_a) = self.occupied.iter().position(|b| b.range.start == index) {
            if let Some(index_b) = self.occupied.iter().position(|b| b.range.start == other) {
                self.data.swap(index, other);
                self.occupied[index_a].range.end =
                    other + (self.occupied[index_a].range.end - self.occupied[index_a].range.start);
                self.occupied[index_a].range.start = other;
                self.occupied[index_b].range.end =
                    index + (self.occupied[index_b].range.end - self.occupied[index_b].range.start);
                self.occupied[index_b].range.start = index;
                return true;
            }
        }
        false
    }
    pub fn last(&self) -> Option<&BufferData> {
        self.occupied.last()
    }
    pub fn clear(&mut self) {
        self.occupied.clear();
        self.data.clear();
        self.free.clear();
    }
    pub fn len(&self) -> usize {
        let mut count = 0;
        self.occupied.iter().for_each(|b| {
            count += b.range.end + 1 - b.range.start;
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
    pub fn get_mut(&mut self, id: &ResourceId) -> Option<&mut [T]> {
        if let Some(buffer_data) = self.occupied.iter().find(|d| d.id == *id) {
            return Some(&mut self.data[buffer_data.range.start..(buffer_data.range.end + 1)]);
        }
        None
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
    pub fn for_each_data<F>(&self, mut f: F)
    where
        F: FnMut(usize, &T),
    {
        self.occupied.iter().for_each(|b| {
            let func = &mut f;
            self.data[b.range.start..(b.range.end + 1)]
                .iter()
                .enumerate()
                .for_each(|(i, d)| {
                    func(b.range.start + i, d);
                });
        });
    }
    pub fn data_at_index(&self, index: usize) -> &T {
        debug_assert!(index < self.data.len());
        self.data[index..(index + 1)].first().unwrap()
    }
    pub fn data_at_index_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < self.data.len());
        self.data[index..(index + 1)].first_mut().unwrap()
    }
    pub fn total_data(&self) -> &[T] {
        &self.data
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
        let mut new_data = Vec::<T>::new();
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

    let mut buffer = Buffer::<Data>::default();

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
    buffer.allocate(
        &meshes[NUM_MESHES - 1].id,
        meshes[NUM_MESHES - 1].data.as_slice(),
    );

    assert_eq!(
        buffer.len(),
        NUM_VERTICES as usize,
        "Allocator should hold a quad"
    );

    buffer.remove_with_id(&meshes[NUM_MESHES - 1].id);

    assert_eq!(buffer.len(), 0, "Allocator should be 0");
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
        buffer.len(),
        mesh.data.len() * NUM_MESHES,
        "Allocator should hold {} quad",
        NUM_MESHES
    );

    assert_eq!(
        buffer.total_data().len(),
        mesh.data.len() * NUM_MESHES,
        "Allocator should hold {} quad",
        NUM_MESHES
    );

    buffer.remove_with_id(&meshes[1].id);
    buffer.remove_with_id(&meshes[2].id);

    assert_eq!(
        buffer.total_len(),
        mesh.data.len() * NUM_MESHES,
        "Allocator should hold anyway {} quad",
        NUM_MESHES
    );
    assert_eq!(
        buffer.len(),
        mesh.data.len() * 2,
        "Allocator should hold only 2 quad",
    );

    buffer.allocate(&octo_mesh_1.id, &octo_mesh_1.data);

    assert_eq!(
        buffer.len(),
        mesh.data.len() * (NUM_MESHES / 2) + octo_mesh_1.data.len(),
        "Allocator should hold anyway {} quads + 1 octo",
        NUM_MESHES / 2
    );
    assert_eq!(
        buffer.total_len(),
        mesh.data.len() * (NUM_MESHES / 2) + octo_mesh_1.data.len(),
        "Allocator should hold anyway {} quads + 1 octo",
        NUM_MESHES / 2
    );
    assert!(buffer.is_full(), "Allocator should be full now");

    buffer.remove_with_id(&meshes[0].id);

    assert_eq!(
        buffer.total_len(),
        mesh.data.len() * (NUM_MESHES / 2) + octo_mesh_1.data.len(),
        "Allocator should hold anyway {} quads + 1 octo",
        NUM_MESHES / 2
    );
    assert_eq!(
        buffer.len(),
        mesh.data.len() + octo_mesh_1.data.len(),
        "Allocator should have some space {} vs {}",
        buffer.len(),
        mesh.data.len() + octo_mesh_1.data.len(),
    );

    buffer.allocate(&octo_mesh_2.id, &octo_mesh_2.data);

    assert_eq!(
        buffer.total_len(),
        mesh.data.len() + octo_mesh_1.data.len() + octo_mesh_2.data.len(),
        "Allocator should hold anyway 1 quads + 2 octos",
    );
    assert_eq!(
        buffer.len(),
        mesh.data.len() + octo_mesh_1.data.len() + octo_mesh_2.data.len(),
        "Allocator should hold anyway 1 quads + 2 octos",
    );

    assert!(buffer.is_full(), "Allocator should be full now");
}

#[test]
fn test() {
    test_buffer();
}
