use std::hash::Hash;

use lru::LruCache;

pub struct LruLikeDictionary<K: Hash + Eq, V> {
    target_size: usize,
    lru: LruCache<K, V>,
}

impl<K: Hash + Eq, V> LruLikeDictionary<K, V> {
    pub fn new(target_size: usize) -> Self {
        Self {
            target_size,
            lru: LruCache::unbounded(),
        }
    }

    pub fn len(&self) -> usize {
        self.lru.len()
    }

    pub fn put(&mut self, key: K, value: V) {
        self.lru.put(key, value);
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.lru.get(key)
    }

    pub fn evict<F>(&mut self, predicate: F) -> usize where
        F: Fn(&V) -> bool {
        let mut evicted = 0;

        loop {
            if self.lru.len() <= self.target_size {
                break;
            }

            match self.lru.peek_lru() {
                None => break,
                Some(lru) => {
                    if predicate(lru.1) {
                        self.lru.pop_lru();
                        evicted += 1;
                    } else {
                        break;
                    }
                }
            }
        }

        evicted
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct SomeType {
        from_state_version: i64,
    }

    impl SomeType {
        fn new(from_state_version: i64) -> Self {
            Self {
                from_state_version,
            }
        }
    }

    #[test]
    fn run() -> Result<(), dyn Error> {
        let mut dict: LruLikeDictionary<&str, SomeType> = LruLikeDictionary::new(3);
        dict.put("a", SomeType::new(2));
        dict.put("b", SomeType::new(2));
        dict.put("c", SomeType::new(2));
        dict.put("d", SomeType::new(3));
        dict.put("a", SomeType::new(3));
        dict.put("e", SomeType::new(4));
        dict.put("a", SomeType::new(4));
        dict.put("f", SomeType::new(5));
        assert_eq!(6, dict.len());
        assert_eq!(2, dict.evict(|x| x.from_state_version < 3));
        assert_eq!(4, dict.len());
        assert_eq!(SomeType { from_state_version: 4 }, *dict.get(&"a")?);

        Ok()
    }
}