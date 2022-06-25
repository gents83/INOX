use std::collections::HashMap;
use std::hash::Hash;

pub struct HashBuffer<Id, Data, const MAX_COUNT: usize>
where
    Id: Eq + Hash + Copy,
{
    map: HashMap<Id, usize>,
    buffer: Vec<Data>,
    is_changed: bool,
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
            is_changed: false,
        }
    }
}

impl<Id, Data, const MAX_COUNT: usize> HashBuffer<Id, Data, MAX_COUNT>
where
    Id: Eq + Hash + Copy,
    Data: Default,
{
    pub fn is_changed(&self) -> bool {
        self.is_changed
    }
    pub fn mark_as_unchanged(&mut self) {
        self.is_changed = false;
    }
    pub fn collapse(&mut self) {
        let old_map = self.map.clone();
        self.map.clear();
        let mut index = 0;
        old_map.iter().for_each(|(id, &old_index)| {
            self.map.insert(*id, index);
            self.buffer.swap(index, old_index);
            index += 1;
        });
        self.buffer.truncate(index);
    }
    fn new_index(&self) -> usize {
        if self.map.is_empty() {
            return 0;
        }
        for i in 0..self.buffer.len() {
            if self.map.iter().any(|(_id, &index)| index == i) {
                continue;
            } else {
                return i;
            }
        }
        self.buffer.len()
    }
    pub fn insert(&mut self, id: &Id, data: Data) -> usize {
        self.is_changed = true;
        if let Some(index) = self.map.get(id) {
            //inox_log::debug_log!("Trying to reinsert {:?} at {}", id, index);
            //inox_log::debug_log!("Buffer len is {}", self.buffer.len());
            self.buffer[*index] = data;
            *index
        } else {
            let index = self.new_index();
            self.map.insert(*id, index);
            //inox_log::debug_log!("Inserting [{:?}] = {} ", *id, index);
            if MAX_COUNT == 0 && index >= self.buffer.len() {
                self.buffer.push(data);
            } else {
                self.buffer[index] = data;
            }
            //inox_log::debug_log!("Buffer len is {}", self.buffer.len());

            index
        }
    }
    pub fn move_to(&mut self, id: &Id, index: usize) {
        let old_index = *self.map.get(id).unwrap();
        if old_index != index {
            //inox_log::debug_log!("Trying to swap {} in {}", old_index, index);
            if old_index < self.buffer.len() {
                if let Some(old_id) = self.id_at(index) {
                    self.map.insert(old_id, old_index);
                    //inox_log::debug_log!("Moving old [{:?}] = {} ", old_id, old_index);
                }
            }
            if index < self.buffer.len() {
                //inox_log::debug_log!("Moving new [{:?}] = {} ", *id, index);
                self.map.insert(*id, index);
            }
            if old_index < self.buffer.len() && index < self.buffer.len() {
                self.is_changed = true;
                self.buffer.swap(old_index, index);
            }
            //inox_log::debug_log!("Buffer len is {}", self.buffer.len());
        }
    }
    pub fn remove(&mut self, id: &Id) -> Option<&Data> {
        //inox_log::debug_log!("Removing [{:?}]", *id);
        //inox_log::debug_log!("Buffer len is {}", self.buffer.len());
        if let Some(index) = self.map.remove(id) {
            self.is_changed = true;
            return Some(&self.buffer[index]);
        }
        None
    }
    pub fn clear(&mut self) {
        self.map.clear();
        if MAX_COUNT == 0 {
            self.buffer.clear();
        }
    }
    pub fn index_of(&self, id: &Id) -> Option<usize> {
        self.map.get(id).copied()
    }
    pub fn id_at(&self, index: usize) -> Option<Id> {
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
    pub fn at(&self, index: usize) -> &Data {
        &self.buffer[index]
    }
    pub fn at_mut(&mut self, index: usize) -> &mut Data {
        &mut self.buffer[index]
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
    pub fn for_each_item_mut(&mut self, mut f: impl FnMut(&Id, usize, &mut Data)) {
        self.map
            .iter()
            .for_each(|(id, index)| f(id, *index, &mut self.buffer[*index]));
    }
}

#[allow(dead_code)]
fn test_resource_indexer<const SIZE: usize>() {
    let mut indexer = HashBuffer::<crate::ResourceId, u32, SIZE>::default();
    let id1 = inox_uid::generate_random_uid();
    let id2 = inox_uid::generate_random_uid();
    let id3 = inox_uid::generate_random_uid();
    indexer.insert(&id1, 100);
    assert_eq!(indexer.index_of(&id1), Some(0));
    assert_eq!(indexer.get(&id1), Some(&100));
    indexer.insert(&id2, 200);
    assert_eq!(indexer.index_of(&id2), Some(1));
    assert_eq!(indexer.get(&id2), Some(&200));
    assert_eq!(indexer.item_count(), 2);
    if SIZE == 0 {
        assert_eq!(indexer.buffer_len(), 2);
    } else {
        assert_eq!(indexer.buffer_len(), 3);
    }
    indexer.move_to(&id2, 0);
    assert_eq!(indexer.index_of(&id2), Some(0));
    assert_eq!(indexer.index_of(&id1), Some(1));
    indexer.remove(&id1);
    assert_eq!(indexer.item_count(), 1);
    if SIZE == 0 {
        assert_eq!(indexer.buffer_len(), 2);
    } else {
        assert_eq!(indexer.buffer_len(), 3);
    }
    indexer.move_to(&id2, 1);
    assert_eq!(indexer.index_of(&id2), Some(1));
    assert_eq!(indexer.get(&id2), Some(&200));
    indexer.insert(&id3, 300);
    indexer.insert(&id1, 100);
    assert_eq!(indexer.buffer_len(), 3);
    assert_eq!(indexer.item_count(), 3);
    assert_eq!(indexer.index_of(&id2), Some(1));
    assert_eq!(indexer.index_of(&id3), Some(0));
    assert_eq!(indexer.index_of(&id1), Some(2));
    assert_eq!(indexer.get(&id1), Some(&100));
    assert_eq!(indexer.get(&id2), Some(&200));
    assert_eq!(indexer.get(&id3), Some(&300));
    indexer.move_to(&id1, 0);
    indexer.move_to(&id2, 1);
    indexer.move_to(&id3, 2);
    assert_eq!(indexer.index_of(&id1), Some(0));
    assert_eq!(indexer.index_of(&id2), Some(1));
    assert_eq!(indexer.index_of(&id3), Some(2));
    assert_eq!(indexer.get(&id1), Some(&100));
    assert_eq!(indexer.get(&id2), Some(&200));
    assert_eq!(indexer.get(&id3), Some(&300));
}

#[test]
fn test() {
    test_resource_indexer::<3>();
    test_resource_indexer::<0>();
}
