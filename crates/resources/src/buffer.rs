use crate::ResourceId;
use sabi_serialize::{generate_random_uid, INVALID_UID};

#[derive(Clone)]
pub struct BufferData {
    pub id: ResourceId,
    pub start: usize,
    pub end: usize,
}

impl Default for BufferData {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            start: 0,
            end: 0,
        }
    }
}

impl BufferData {
    pub fn new(id: &ResourceId, start: usize, end: usize) -> Self {
        Self {
            id: *id,
            start,
            end,
        }
    }
    #[inline]
    pub fn is_adjacent(&self, buffer: &BufferData) -> bool {
        if buffer.start > 0 && self.end == (buffer.start - 1) {
            return true;
        }
        if self.start > 0 && buffer.end == (self.start - 1) {
            return true;
        }
        false
    }
    #[inline]
    pub fn combine(&mut self, buffer: &BufferData) -> bool {
        if buffer.start > 0 && self.end == (buffer.start - 1) {
            self.end = buffer.end;
            return true;
        }
        if self.start > 0 && buffer.end == (self.start - 1) {
            self.start = buffer.start;
            return true;
        }
        false
    }
    pub fn len(&self) -> usize {
        self.end + 1 - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
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
        if let Some(index) = self.free.iter().position(|d| (d.end + 1 - d.start) >= size) {
            let free_data = self.free.remove(index);
            self.insert_at(id, free_data.start, data);

            if (free_data.end - free_data.start) > (size) {
                self.free.push(BufferData::new(
                    &generate_random_uid(),
                    free_data.end - (size),
                    free_data.end,
                ));
            }
        } else {
            self.defrag();
            self.insert(id, data);
            need_realloc = true;
        }
        need_realloc
    }
    fn insert(&mut self, id: &ResourceId, data: &[T]) {
        let start = self.data.len();
        let end = start + data.len() - 1;
        self.data.extend_from_slice(data);
        if let Some(i) = self.occupied.iter().position(|d| d.end > start) {
            self.occupied.insert(i, BufferData::new(id, start, end));
        } else {
            self.occupied.push(BufferData::new(id, start, end));
        }
    }
    fn insert_at(&mut self, id: &ResourceId, start: usize, data: &[T]) {
        debug_assert!(start <= self.data.len());
        let end = start + data.len() - 1;
        self.update(start, data);
        if let Some(i) = self.occupied.iter().position(|d| d.end > start) {
            self.occupied.insert(i, BufferData::new(id, start, end));
        } else {
            self.occupied.push(BufferData::new(id, start, end));
        }
    }
    pub fn update(&mut self, start: usize, data: &[T]) {
        debug_assert!(start <= self.data.len());
        self.data[start..(start + data.len())]
            .clone_from_slice(&data[..((start + data.len()) - start)]);
    }
    pub fn swap(&mut self, index: usize, other: usize) -> bool {
        if index == other {
            return false;
        }
        debug_assert!(index <= self.data.len());
        debug_assert!(other <= self.data.len());
        if let Some(index_a) = self.occupied.iter().position(|b| b.start == index) {
            if let Some(index_b) = self.occupied.iter().position(|b| b.start == other) {
                self.data.swap(index, other);
                self.occupied[index_a].end =
                    other + (self.occupied[index_a].end - self.occupied[index_a].start);
                self.occupied[index_a].start = other;
                self.occupied[index_b].end =
                    index + (self.occupied[index_b].end - self.occupied[index_b].start);
                self.occupied[index_b].start = index;
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
        self.data.len()
    }
    pub fn find(&self, size: usize) -> Option<usize> {
        self.occupied.iter().position(|b| (b.end - b.start) >= size)
    }
    pub fn get(&self, id: &ResourceId) -> Option<&BufferData> {
        self.occupied.iter().find(|d| d.id == *id)
    }
    pub fn get_mut(&mut self, id: &ResourceId) -> Option<&mut [T]> {
        if let Some(buffer_data) = self.occupied.iter().find(|d| d.id == *id) {
            return Some(&mut self.data[buffer_data.start..(buffer_data.end + 1)]);
        }
        None
    }
    pub fn remove_with_id(&mut self, id: &ResourceId) -> bool {
        if let Some(index) = self.occupied.iter().position(|d| d.id == *id) {
            let data = self.occupied.remove(index);
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
    pub fn occupied_data(&self) -> Vec<&T> {
        self.occupied
            .iter()
            .map(|b| self.data[b.start..(b.end + 1)].iter().collect::<Vec<_>>())
            .flatten()
            .collect::<Vec<&T>>()
    }
    pub fn data_at_index(&self, index: usize) -> &T {
        debug_assert!(index < self.data.len());
        self.data[index..(index + 1)].first().unwrap()
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
                    .position(|d| d.end > first_buffer.start)
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
            new_data.extend_from_slice(&self.data[d.start..=d.end]);
            d.start = last_index;
            last_index = new_data.len();
            d.end = last_index - 1;
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

    #[derive(Default, Clone)]
    struct Mesh {
        id: ResourceId,
        data: Vec<Data>,
    }
    impl Mesh {
        fn add(&mut self, count: u32) {
            (0..count).for_each(|_| self.data.push(Data::default()));
        }
    }

    const NUM_VERTICES: u32 = 4;

    let mut buffer = Buffer::<Data>::default();

    let mut meshes = Vec::new();
    let mut mesh = Mesh::default();
    mesh.add(NUM_VERTICES);
    let mut octo_mesh = Mesh::default();
    octo_mesh.add(2 * NUM_VERTICES);
    for _ in 0..4 {
        meshes.push(mesh.clone());
    }

    assert!(buffer.is_empty(), "Allocator should be empty");
    buffer.allocate(
        &meshes.last().unwrap().id,
        meshes.last().unwrap().data.as_slice(),
    );

    assert_eq!(
        buffer.len(),
        NUM_VERTICES as usize,
        "Allocator should hold a quad"
    );

    buffer.remove_with_id(&meshes.last().unwrap().id);

    assert_eq!(
        buffer.len(),
        NUM_VERTICES as usize,
        "Allocator should have an empty space for a quad"
    );
    assert!(buffer.is_empty(), "Allocator should be empty");

    buffer.defrag();
    assert_eq!(
        buffer.len(),
        0,
        "Allocator should be defragged and completely empty"
    );
    assert!(buffer.is_empty(), "Allocator should be empty");

    meshes.iter().for_each(|m| {
        buffer.allocate(&m.id, &m.data);
    });

    assert_eq!(
        buffer.len(),
        mesh.data.len() * meshes.len(),
        "Allocator should hold {} quad",
        meshes.len()
    );

    assert_eq!(
        buffer.total_data().len(),
        mesh.data.len() * meshes.len(),
        "Allocator should hold {} quad",
        meshes.len()
    );

    buffer.remove_with_id(&meshes[1].id);
    buffer.remove_with_id(&meshes[2].id);

    assert_eq!(
        buffer.len(),
        mesh.data.len() * meshes.len(),
        "Allocator should hold anyway {} quad",
        meshes.len()
    );

    buffer.defrag();

    assert_eq!(
        buffer.len(),
        mesh.data.len() * 2,
        "Allocator should hold only 2 quad",
    );

    buffer.allocate(&octo_mesh.id, &octo_mesh.data);

    assert_eq!(
        buffer.len(),
        mesh.data.len() * meshes.len(),
        "Allocator should hold anyway {} quad",
        meshes.len()
    );
    assert!(buffer.is_full(), "Allocator should be full now");
}

#[test]
fn test() {
    test_buffer();
}
