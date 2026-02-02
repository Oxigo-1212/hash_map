# Hash Map Implementations in Rust

An educational project implementing hash maps with different collision resolution strategies.

## Overview

This library provides two hash map implementations:

1. **OpenHashMap** - Basic open addressing with linear probing
2. **RobinHashMap** - Robin Hood hashing with backward shift deletion

## Robin Hood Hashing

Robin Hood hashing is a variant of linear probing where elements that are "far from home" (high PSL) can **displace** elements that are "close to home" (low PSL). This reduces variance in probe lengths and improves worst-case lookup performance.

### Core Concept

Each element tracks its **Probe Sequence Length (PSL)** - the distance from its "home" position (where it would ideally be placed based on its hash).

- PSL = 1: Element is at its home position
- PSL = 2: Element is 1 slot away from home
- etc.

During insertion, if we encounter an element with a *lower* PSL than ours, we **swap** with it (steal from the rich) and continue inserting the displaced element.

### Data Structures

```rust
struct Bucket<K, V> {
    key: K,
    value: V,
    probe_length: usize,  // PSL: starts at 1 (home position)
}

struct RobinHashMap<K, V> {
    array: Vec<Option<Bucket<K, V>>>,  // None = empty slot
    max_psl: usize,                     // tracks maximum PSL for early termination
    capacity: usize,
}
```

### Algorithms

#### Insert

1. Hash the key to find home position
2. Linear probe from home
3. If empty slot: insert here
4. If same key: update value
5. If bucket has lower PSL: swap (Robin Hood) and continue with displaced element
6. Otherwise: keep probing, increment PSL

```rust
pub fn insert(&mut self, key: K, value: V) -> Option<Bucket<K, V>>
```

#### Lookup / Contains

1. Hash the key to find home position
2. Linear probe from home
3. Stop early if:
   - Found the key
   - Hit empty slot
   - Current PSL exceeds `max_psl` (key can't exist beyond this point)
   - Found element with lower PSL than current probe distance

```rust
pub fn get(&self, key: &K) -> Option<&V>
pub fn contains(&self, key: &K) -> bool
```

#### Delete (with Backward Shift)

1. Find the key using lookup algorithm
2. Remove the element
3. **Backward shift**: Move subsequent elements back to fill the gap
   - Only shift elements with PSL > 1 (not at home position)
   - Decrement PSL of shifted elements

```rust
pub fn delete(&mut self, key: &K) -> Option<V>
```

## API

### RobinHashMap

| Method | Description |
|--------|-------------|
| `new(capacity: usize)` | Creates empty map with given capacity |
| `insert(key, value) -> Option<Bucket>` | Inserts or updates; returns old bucket if key existed |
| `get(&key) -> Option<&V>` | Returns reference to value if key exists |
| `contains(&key) -> bool` | Returns true if key exists |
| `delete(&key) -> Option<V>` | Removes key and returns value |

### OpenHashMap

| Method | Description |
|--------|-------------|
| `new(capacity: usize)` | Creates empty map with given capacity |
| `insert(key, value) -> bool` | Inserts key-value pair |
| `find(&key) -> Option<&V>` | Returns reference to value if key exists |
| `delete(&key) -> bool` | Removes key, returns true if existed |

## Usage

```rust
use hash_map::{RobinHashMap, OpenHashMap};

fn main() {
    // Robin Hood Hash Map
    let mut robin = RobinHashMap::new(16);
    robin.insert("key1", 100);
    robin.insert("key2", 200);
    
    assert_eq!(robin.get(&"key1"), Some(&100));
    assert!(robin.contains(&"key2"));
    
    robin.delete(&"key1");
    assert!(!robin.contains(&"key1"));

    // Basic Open Addressing Hash Map
    let mut basic = OpenHashMap::new(16);
    basic.insert("hello", 42);
    assert_eq!(basic.find(&"hello"), Some(&42));
}
```

## Project Structure

```
src/
├── lib.rs                  # Library exports
├── robin_hood_hash_map.rs  # Robin Hood implementation + 9 tests
└── basic_hash_map.rs       # Basic open addressing + 19 tests
```

## Running Tests

```bash
cargo test
```

## References

- [Robin Hood Hashing (Wikipedia)](https://en.wikipedia.org/wiki/Hash_table#Robin_Hood_hashing)
- [Robin Hood Hashing: Backward Shift Deletion](https://codecapsule.com/2013/11/17/robin-hood-hashing-backward-shift-deletion/)
