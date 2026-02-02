use std::hash::{DefaultHasher, Hash, Hasher};
/* Hash properties
- Array to store data
- A hash function to compute the index
- A collision resolution strategy
*/
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
    pub fn hash(k: &K, modulus: u64) -> u64 {
        let mut hash_function = DefaultHasher::new();
        k.hash(&mut hash_function);
        let result = hash_function.finish();
        result % modulus
    }
    pub fn insert(&mut self, key: K, value: V) -> bool {
        let mut index = Self::hash(&key, self.capacity as u64) as usize;
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
        let index = Self::hash(&key, self.capacity as u64) as usize;
        let delete_value = self.array[index].clone();
        self.array[index] = Slot::Deleted;
        delete_value
    }
    pub fn find(&self, key: K) -> Slot<&K, &V> {
        let mut index = Self::hash(&key, self.capacity as u64) as usize;
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

#[cfg(test)]
mod tests {
    use super::*;

    // Basic Operations
    #[test]
    fn test_new() {
        let map: OpenHashMap<i32, i32> = OpenHashMap::new(10);
        assert_eq!(map.capacity, 10);
        assert_eq!(map.array.len(), 10);
    }

    #[test]
    fn test_insert_single() {
        let mut map = OpenHashMap::new(10);
        let result = map.insert("key1", 100);
        assert!(result);
    }

    #[test]
    fn test_find_existing_key() {
        let mut map = OpenHashMap::new(10);
        map.insert("key1", 100);
        match map.find("key1") {
            Slot::Some((k, v)) => {
                assert_eq!(*k, "key1");
                assert_eq!(*v, 100);
            }
            _ => panic!("Expected Slot::Some"),
        }
    }

    #[test]
    fn test_find_nonexistent_key() {
        let map: OpenHashMap<&str, i32> = OpenHashMap::new(10);
        match map.find("missing") {
            Slot::Empty => {}
            _ => panic!("Expected Slot::Empty"),
        }
    }

    // Collision Handling
    #[test]
    fn test_insert_multiple() {
        let mut map = OpenHashMap::new(10);
        assert!(map.insert("a", 1));
        assert!(map.insert("b", 2));
        assert!(map.insert("c", 3));
    }

    #[test]
    fn test_find_multiple_keys() {
        let mut map = OpenHashMap::new(10);
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        match map.find("a") {
            Slot::Some((_, v)) => assert_eq!(*v, 1),
            _ => panic!("Expected Slot::Some for 'a'"),
        }
        match map.find("b") {
            Slot::Some((_, v)) => assert_eq!(*v, 2),
            _ => panic!("Expected Slot::Some for 'b'"),
        }
        match map.find("c") {
            Slot::Some((_, v)) => assert_eq!(*v, 3),
            _ => panic!("Expected Slot::Some for 'c'"),
        }
    }

    // Update Behavior
    #[test]
    fn test_insert_duplicate_updates_value() {
        let mut map = OpenHashMap::new(10);
        map.insert("key", 100);
        map.insert("key", 200);

        match map.find("key") {
            Slot::Some((_, v)) => assert_eq!(*v, 200),
            _ => panic!("Expected Slot::Some with updated value"),
        }
    }

    // Delete Operations
    #[test]
    fn test_delete_existing() {
        let mut map = OpenHashMap::new(10);
        map.insert("key", 100);
        let deleted = map.delete("key");

        match deleted {
            Slot::Some((k, v)) => {
                assert_eq!(k, "key");
                assert_eq!(v, 100);
            }
            _ => panic!("Expected Slot::Some for deleted item"),
        }
    }

    #[test]
    fn test_find_after_delete() {
        let mut map = OpenHashMap::new(10);
        map.insert("key", 100);
        map.delete("key");

        match map.find("key") {
            Slot::Empty | Slot::Deleted => {}
            Slot::Some(_) => panic!("Expected key to not be found after delete"),
        }
    }

    // Edge Cases
    #[test]
    fn test_full_map_returns_false() {
        let mut map = OpenHashMap::new(3);
        assert!(map.insert("a", 1));
        assert!(map.insert("b", 2));
        assert!(map.insert("c", 3));
        assert!(!map.insert("d", 4)); // Should return false when full
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_large_capacity() {
        let mut map = OpenHashMap::new(1000);
        for i in 0..500 {
            assert!(map.insert(i, i * 2));
        }
        for i in 0..500 {
            match map.find(i) {
                Slot::Some((_, v)) => assert_eq!(*v, i * 2),
                _ => panic!("Expected Slot::Some for key {}", i),
            }
        }
    }

    // ==========================================
    // Tests with different types
    // ==========================================

    // Integer keys and values
    #[test]
    fn test_i32_keys_and_values() {
        let mut map: OpenHashMap<i32, i32> = OpenHashMap::new(10);
        map.insert(1, 100);
        map.insert(2, 200);
        map.insert(-5, 500);

        match map.find(1) {
            Slot::Some((k, v)) => {
                assert_eq!(*k, 1);
                assert_eq!(*v, 100);
            }
            _ => panic!("Expected Slot::Some for key 1"),
        }
        match map.find(-5) {
            Slot::Some((k, v)) => {
                assert_eq!(*k, -5);
                assert_eq!(*v, 500);
            }
            _ => panic!("Expected Slot::Some for key -5"),
        }
    }

    // u64 keys
    #[test]
    fn test_u64_keys() {
        let mut map: OpenHashMap<u64, u64> = OpenHashMap::new(10);
        map.insert(999_999_999, 1);
        map.insert(0, 2);
        map.insert(u64::MAX, 3);

        match map.find(u64::MAX) {
            Slot::Some((_, v)) => assert_eq!(*v, 3),
            _ => panic!("Expected Slot::Some for u64::MAX"),
        }
    }

    // String keys (owned)
    #[test]
    fn test_string_keys() {
        let mut map: OpenHashMap<String, i32> = OpenHashMap::new(10);
        map.insert(String::from("hello"), 1);
        map.insert(String::from("world"), 2);
        map.insert(String::from("rust"), 3);

        match map.find(String::from("hello")) {
            Slot::Some((k, v)) => {
                assert_eq!(k, "hello");
                assert_eq!(*v, 1);
            }
            _ => panic!("Expected Slot::Some for 'hello'"),
        }
        match map.find(String::from("rust")) {
            Slot::Some((_, v)) => assert_eq!(*v, 3),
            _ => panic!("Expected Slot::Some for 'rust'"),
        }
    }

    // Char keys
    #[test]
    fn test_char_keys() {
        let mut map: OpenHashMap<char, i32> = OpenHashMap::new(10);
        map.insert('a', 1);
        map.insert('z', 26);
        map.insert('!', 100);

        match map.find('a') {
            Slot::Some((k, v)) => {
                assert_eq!(*k, 'a');
                assert_eq!(*v, 1);
            }
            _ => panic!("Expected Slot::Some for 'a'"),
        }
    }

    // Tuple keys
    #[test]
    fn test_tuple_keys() {
        let mut map: OpenHashMap<(i32, i32), i32> = OpenHashMap::new(10);
        map.insert((0, 0), 1);
        map.insert((1, 2), 3);
        map.insert((-1, -1), 100);

        match map.find((1, 2)) {
            Slot::Some((k, v)) => {
                assert_eq!(*k, (1, 2));
                assert_eq!(*v, 3);
            }
            _ => panic!("Expected Slot::Some for (1, 2)"),
        }
    }

    // Bool keys (only 2 possible keys)
    #[test]
    fn test_bool_keys() {
        let mut map: OpenHashMap<bool, i32> = OpenHashMap::new(5);
        map.insert(true, 1);
        map.insert(false, 0);

        match map.find(true) {
            Slot::Some((_, v)) => assert_eq!(*v, 1),
            _ => panic!("Expected Slot::Some for true"),
        }
        match map.find(false) {
            Slot::Some((_, v)) => assert_eq!(*v, 0),
            _ => panic!("Expected Slot::Some for false"),
        }
    }

    // Mixed: String key, char value
    #[test]
    fn test_string_key_char_value() {
        let mut map: OpenHashMap<String, char> = OpenHashMap::new(10);
        map.insert(String::from("first"), 'A');
        map.insert(String::from("second"), 'B');

        match map.find(String::from("first")) {
            Slot::Some((_, v)) => assert_eq!(*v, 'A'),
            _ => panic!("Expected Slot::Some"),
        }
    }

    // i32 key, i64 value
    #[test]
    fn test_i32_key_i64_value() {
        let mut map: OpenHashMap<i32, i64> = OpenHashMap::new(10);
        map.insert(1, 314);
        map.insert(2, 2718);

        match map.find(1) {
            Slot::Some((_, v)) => assert_eq!(*v, 314),
            _ => panic!("Expected Slot::Some for key 1"),
        }
    }
}
