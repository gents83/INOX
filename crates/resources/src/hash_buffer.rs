use std::collections::HashMap;
use std::hash::Hash;

pub struct HashBuffer<Id, Data, const MAX_COUNT: usize>
where
    Id: Eq + Hash + Copy,
{
    map: HashMap<Id, usize>,
    buffer: Vec<Data>,
}

impl<Id, Data, const MAX_COUNT: usize> Default for HashBuffer<Id, Data, MAX_COUNT>
where
    Id: Eq + Hash + Copy,
    Data: Default,
{
    fn default() -> Self {
        let mut buffer = Vec::new();
        for _ in 0..MAX_COUNT {
            buffer.push(Data::default());
        }
        Self {
            map: HashMap::new(),
            buffer,
        }
    }
}

impl<Id, Data, const MAX_COUNT: usize> HashBuffer<Id, Data, MAX_COUNT>
where
    Id: Eq + Hash + Copy,
    Data: Default,
{
    pub fn insert(&mut self, id: &Id, data: Data) -> usize {
        if let Some(index) = self.map.get(id) {
            self.buffer[*index] = data;
            *index
        } else {
            let count = self.map.len();
            let mut new_index = count as i32;
            self.map.iter().for_each(|(_, index)| {
                let index = *index as i32;
                if index <= new_index {
                    new_index = index - 1;
                }
            });
            if new_index >= 0 {
                let index = new_index as usize;
                self.map.insert(*id, index);
                if index >= self.buffer.len() {
                    self.buffer.push(data);
                } else {
                    self.buffer[index] = data;
                }
                return index;
            }
            self.map.insert(*id, count);

            if MAX_COUNT == 0 {
                self.buffer.push(data);
            } else {
                debug_assert!(
                    count < MAX_COUNT,
                    "Trying to insert more than {} elements",
                    MAX_COUNT
                );
                self.buffer[count] = data;
            }

            count
        }
    }
    pub fn move_to(&mut self, id: &Id, index: usize) {
        let old_index = *self.map.get(id).unwrap();
        if old_index != index {
            if let Some(old_id) = self.id(index) {
                self.map.insert(old_id, old_index);
            }
            self.map.insert(*id, index);
            if old_index < self.buffer.len() && index < self.buffer.len() {
                self.buffer.swap(old_index, index);
            }
        }
    }
    pub fn remove(&mut self, id: &Id) -> Option<usize> {
        self.map.remove(id)
    }
    pub fn index(&self, id: &Id) -> Option<usize> {
        self.map.get(id).copied()
    }
    pub fn id(&self, index: usize) -> Option<Id> {
        self.map
            .iter()
            .find(|(_, i)| *i == &index)
            .map(|(id, _)| *id)
    }
    pub fn get(&self, id: &Id) -> Option<&Data> {
        self.map.get(id).map(|index| &self.buffer[*index])
    }
    pub fn get_mut(&mut self, id: &Id) -> Option<&mut Data> {
        self.map.get(id).map(|index| &mut self.buffer[*index])
    }
    pub fn data(&self) -> &[Data] {
        self.buffer.as_slice()
    }
    pub fn data_mut(&mut self) -> &mut [Data] {
        self.buffer.as_mut_slice()
    }
    pub fn item_count(&self) -> usize {
        self.map.len()
    }
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn for_each_item(&self, mut f: impl FnMut(&Id, usize, &Data)) {
        self.map
            .iter()
            .for_each(|(id, index)| f(id, *index, &self.buffer[*index]));
    }
}

#[allow(dead_code)]
fn test_resource_indexer<const SIZE: usize>() {
    let mut indexer = HashBuffer::<crate::ResourceId, u32, SIZE>::default();
    let id1 = inox_uid::generate_random_uid();
    let id2 = inox_uid::generate_random_uid();
    let id3 = inox_uid::generate_random_uid();
    indexer.insert(&id1, 100);
    assert_eq!(indexer.index(&id1), Some(0));
    assert_eq!(indexer.get(&id1), Some(&100));
    indexer.insert(&id2, 200);
    assert_eq!(indexer.index(&id2), Some(1));
    assert_eq!(indexer.get(&id2), Some(&200));
    assert_eq!(indexer.item_count(), 2);
    if SIZE == 0 {
        assert_eq!(indexer.buffer_len(), 2);
    } else {
        assert_eq!(indexer.buffer_len(), 3);
    }
    indexer.move_to(&id2, 0);
    assert_eq!(indexer.index(&id2), Some(0));
    assert_eq!(indexer.index(&id1), Some(1));
    indexer.remove(&id1);
    assert_eq!(indexer.item_count(), 1);
    if SIZE == 0 {
        assert_eq!(indexer.buffer_len(), 2);
    } else {
        assert_eq!(indexer.buffer_len(), 3);
    }
    indexer.move_to(&id2, 1);
    assert_eq!(indexer.index(&id2), Some(1));
    assert_eq!(indexer.get(&id2), Some(&200));
    indexer.insert(&id3, 300);
    indexer.insert(&id1, 100);
    assert_eq!(indexer.buffer_len(), 3);
    assert_eq!(indexer.item_count(), 3);
    assert_eq!(indexer.index(&id2), Some(1));
    assert_eq!(indexer.index(&id3), Some(0));
    assert_eq!(indexer.index(&id1), Some(2));
    assert_eq!(indexer.get(&id1), Some(&100));
    assert_eq!(indexer.get(&id2), Some(&200));
    assert_eq!(indexer.get(&id3), Some(&300));
    indexer.move_to(&id1, 0);
    indexer.move_to(&id2, 1);
    indexer.move_to(&id3, 2);
    assert_eq!(indexer.index(&id1), Some(0));
    assert_eq!(indexer.index(&id2), Some(1));
    assert_eq!(indexer.index(&id3), Some(2));
    assert_eq!(indexer.get(&id1), Some(&100));
    assert_eq!(indexer.get(&id2), Some(&200));
    assert_eq!(indexer.get(&id3), Some(&300));
}

#[test]
fn test() {
    test_resource_indexer::<3>();
    test_resource_indexer::<0>();
}
