use std::hash::{DefaultHasher, Hash, Hasher};
fn hash<K: Hash>(k: &K, modulus: u64) -> u64 {
    let mut hash_function = DefaultHasher::new();
    k.hash(&mut hash_function);
    let result = hash_function.finish();
    result % modulus
}
#[derive(Debug, Hash, Clone)]
pub enum Slot<K, V> {
    Empty,
    Deleted,
    Some((K, V)),
}

#[derive(Debug)]
pub struct OpenHashMap<K, V> {
    array: Vec<Slot<K, V>>,
    capacity: usize,
}

impl<K, V> OpenHashMap<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone + Copy,
{
    pub fn new(capacity: usize) -> Self {
        let mut array = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            array.push(Slot::Empty);
        }
        OpenHashMap { array, capacity }
    }
    pub fn insert(&mut self, key: K, value: V) -> bool {
        let mut index = hash(&key, self.capacity as u64) as usize;
        let start_index = index;
        loop {
            match &self.array[index] {
                Slot::Empty | Slot::Deleted => {
                    self.array[index] = Slot::Some((key, value));
                    return true;
                }
                Slot::Some((existing_key, _)) if existing_key == &key => {
                    self.array[index] = Slot::Some((key, value));
                    return true;
                }
                _ => {
                    index = (index + 1) % self.capacity;
                    if index == start_index {
                        return false;
                    }
                }
            }
        }
    }
    pub fn delete(&mut self, key: K) -> Slot<K, V> {
        let index = hash(&key, self.capacity as u64) as usize;
        let delete_value = self.array[index].clone();
        self.array[index] = Slot::Deleted;
        delete_value
    }
    pub fn find(&self, key: K) -> Slot<&K, &V> {
        let mut index = hash(&key, self.capacity as u64) as usize;
        loop {
            match &self.array[index] {
                Slot::Some((k, v)) => {
                    if k == &key {
                        return Slot::Some((k, v));
                    }
                    index = (index + 1) % self.capacity;
                }
                Slot::Empty => return Slot::Empty,
                Slot::Deleted => {
                    index = (index + 1) % self.capacity;
                }
            }
        }
    }
}
