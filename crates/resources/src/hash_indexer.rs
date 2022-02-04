use std::collections::HashMap;
use std::hash::Hash;

use inox_serialize::generate_random_uid;

use crate::ResourceId;

#[derive(Default)]
pub struct HashIndexer<T>
where
    T: Eq + Hash + Copy,
{
    map: HashMap<T, usize>,
}

impl<T> HashIndexer<T>
where
    T: Eq + Hash + Copy,
{
    pub fn insert_at(&mut self, id: &T, index: usize) {
        self.remove(id);
        self.map.insert(*id, index);
    }
    pub fn insert(&mut self, id: &T) -> usize {
        if let Some(index) = self.map.get(id) {
            *index
        } else {
            let index = self.len();
            self.map.insert(*id, index);
            index
        }
    }
    pub fn remove(&mut self, id: &T) {
        if self.map.contains_key(id) {
            self.map.remove(id);
        }
    }
    pub fn remove_and_update(&mut self, id: &T) {
        if let Some(index) = self.map.remove(id) {
            self.map.iter_mut().for_each(|(_, v)| {
                if *v > index {
                    *v -= 1;
                }
            });
        }
    }
    pub fn get(&self, id: &T) -> Option<usize> {
        self.map.get(id).copied()
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[allow(dead_code)]
fn test_resource_indexer() {
    let mut indexer = HashIndexer::<ResourceId>::default();
    let id1 = generate_random_uid();
    let id2 = generate_random_uid();
    let id3 = generate_random_uid();
    indexer.insert(&id1);
    assert_eq!(indexer.get(&id1), Some(0));
    indexer.insert(&id2);
    assert_eq!(indexer.get(&id2), Some(1));
    indexer.remove(&id1);
    assert_eq!(indexer.get(&id2), Some(0));
    indexer.insert(&id1);
    indexer.insert(&id3);
    assert_eq!(indexer.len(), 3);
    assert_eq!(indexer.get(&id2), Some(0));
    assert_eq!(indexer.get(&id1), Some(1));
    assert_eq!(indexer.get(&id3), Some(2));
}

#[test]
fn test() {
    test_resource_indexer();
}
