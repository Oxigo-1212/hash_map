use fxhash::FxHasher64;
use std::hash::{Hash, Hasher};
fn hash_with_fxhash<T: Hash>(t: &T) -> u64 {
    let mut s = FxHasher64::default();
    t.hash(&mut s);
    s.finish()
}

#[derive(Debug, Clone, Hash)]
pub struct Bucket<K, V> {
    key: K,
    value: V,
    probe_length: usize,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_empty_map() {
        let map: RobinHashMap<String, i32> = RobinHashMap::new(16);
        assert_eq!(map.capacity, 16);
        assert_eq!(map.max_psl, 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut map = RobinHashMap::new(16);
        map.insert("key1", 100);
        map.insert("key2", 200);

        assert_eq!(map.get(&"key1"), Some(&100));
        assert_eq!(map.get(&"key2"), Some(&200));
        assert_eq!(map.get(&"key3"), None);
    }

    #[test]
    fn test_insert_returns_old_value_on_update() {
        let mut map = RobinHashMap::new(16);

        let result = map.insert("key", 100);
        assert!(result.is_none());

        let result = map.insert("key", 200);
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, 100);

        assert_eq!(map.get(&"key"), Some(&200));
    }

    #[test]
    fn test_contains() {
        let mut map = RobinHashMap::new(16);
        map.insert("exists", 42);

        assert!(map.contains(&"exists"));
        assert!(!map.contains(&"not_exists"));
    }

    #[test]
    fn test_delete() {
        let mut map = RobinHashMap::new(16);
        map.insert("key1", 100);
        map.insert("key2", 200);

        let deleted = map.delete(&"key1");
        assert_eq!(deleted, Some(100));
        assert!(!map.contains(&"key1"));
        assert!(map.contains(&"key2"));
    }

    #[test]
    fn test_delete_nonexistent_key() {
        let mut map: RobinHashMap<&str, i32> = RobinHashMap::new(16);
        let result = map.delete(&"nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_inserts_with_collisions() {
        let mut map = RobinHashMap::new(8);

        for i in 0..5 {
            map.insert(i, i * 10);
        }

        for i in 0..5 {
            assert_eq!(map.get(&i), Some(&(i * 10)));
        }
    }

    #[test]
    fn test_delete_maintains_lookup_integrity() {
        let mut map = RobinHashMap::new(16);

        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        map.delete(&"b");

        assert_eq!(map.get(&"a"), Some(&1));
        assert_eq!(map.get(&"b"), None);
        assert_eq!(map.get(&"c"), Some(&3));
    }

    #[test]
    fn test_update_existing_key() {
        let mut map = RobinHashMap::new(16);

        map.insert("key", 1);
        map.insert("key", 2);
        map.insert("key", 3);

        assert_eq!(map.get(&"key"), Some(&3));
    }
}
