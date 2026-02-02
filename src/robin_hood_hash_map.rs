use fxhash::FxHasher64;
use std::hash::{Hash, Hasher};
fn hash_with_fxhash<T: Hash>(t: &T) -> u64 {
    let mut s = FxHasher64::default();
    t.hash(&mut s);
    s.finish()
}

#[derive(Debug, Clone, Hash)]
pub struct Bucket<K, V> {
    pub key: K,
    pub value: V,
    pub probe_length: usize,
}
#[derive(Debug, Clone, Hash)]
pub struct RobinHashMap<K, V> {
    array: Vec<Option<Bucket<K, V>>>,
    max_psl: usize,
    capacity: usize,
}
impl<K, V> RobinHashMap<K, V>
where
    K: Eq + Clone + Hash,
    V: Eq + Clone,
{
    pub fn new(capacity: usize) -> Self {
        let mut array = Vec::with_capacity(capacity);
        let max_psl = 0;
        for _ in 0..capacity {
            array.push(None);
        }
        RobinHashMap {
            array,
            max_psl,
            capacity,
        }
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<Bucket<K, V>> {
        let mut index = (hash_with_fxhash(&key) as usize) % self.capacity;
        let mut incoming = Bucket {
            key,
            value,
            probe_length: 1,
        };
        loop {
            match &mut self.array[index] {
                None => {
                    self.max_psl = self.max_psl.max(incoming.probe_length);
                    self.array[index] = Some(incoming);
                    return None;
                }
                Some(bucket) if bucket.key == incoming.key => {
                    std::mem::swap(&mut bucket.value, &mut incoming.value);
                    return Some(incoming); // return old value wrapped in bucket
                }
                Some(bucket) if bucket.probe_length < incoming.probe_length => {
                    std::mem::swap(bucket, &mut incoming); // swap entire bucket
                }
                _ => {}
            }
            index = (index + 1) % self.capacity;
            incoming.probe_length += 1;
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        let mut index = (hash_with_fxhash(key) as usize) % self.capacity;
        let mut psl = 1;

        while psl <= self.max_psl {
            match &self.array[index] {
                None => return false,
                Some(bucket) if bucket.key == *key => return true,
                Some(bucket) if bucket.probe_length < psl => return false,
                _ => {}
            }
            index = (index + 1) % self.capacity;
            psl += 1;
        }
        false
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut index = (hash_with_fxhash(key) as usize) % self.capacity;
        let mut psl = 1;

        while psl <= self.max_psl {
            match &self.array[index] {
                None => return None,
                Some(bucket) if bucket.key == *key => return Some(&bucket.value),
                Some(bucket) if bucket.probe_length < psl => return None,
                _ => {}
            }
            index = (index + 1) % self.capacity;
            psl += 1;
        }
        None
    }

    pub fn delete(&mut self, key: &K) -> Option<V> {
        let mut index = (hash_with_fxhash(key) as usize) % self.capacity;
        let mut psl = 1;

        // Find the key
        while psl <= self.max_psl {
            match &self.array[index] {
                None => return None,
                Some(bucket) if bucket.key == *key => break,
                Some(bucket) if bucket.probe_length < psl => return None,
                _ => {}
            }
            index = (index + 1) % self.capacity;
            psl += 1;
        }

        if psl > self.max_psl {
            return None;
        }

        // Remove the element and get its value
        let removed = self.array[index].take().unwrap();
        let removed_value = removed.value;

        // Backward shift: move elements back to fill the gap
        let mut empty_index = index;
        loop {
            let next_index = (empty_index + 1) % self.capacity;

            match &self.array[next_index] {
                None => break,
                Some(bucket) if bucket.probe_length == 1 => break,
                Some(_) => {
                    // Move element back
                    self.array[empty_index] = self.array[next_index].take();
                    if let Some(ref mut bucket) = self.array[empty_index] {
                        bucket.probe_length -= 1;
                    }
                    empty_index = next_index;
                }
            }
        }

        Some(removed_value)
    }
}
