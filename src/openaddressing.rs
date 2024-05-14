use std::hash::{DefaultHasher, Hash, Hasher};
use rand::prelude::*;
use std::cmp;

#[derive(Debug, Clone)]
struct Bucket {
    key: String,
    hashed_key: u64,
    value: i32,
    deleted: bool,
}

// Implementation of OpenAddressing
#[derive(Debug)]
struct HashTable {
    buckets: Vec<Option<Bucket>>,
}

impl HashTable {
    const INITIAL_SIZE: usize = 16;
    const MAX_PROBE: usize = 4;

    pub fn new() -> Self {
        let mut buckets = Vec::new();
        buckets.resize(Self::INITIAL_SIZE, None);
        Self { buckets }
    }

    pub fn upsert(&mut self, key: String, value: i32) {
        let hashed_key = self.compute_hash(&key);
        loop {
            match self.compute_insertable_index(hashed_key, &self.buckets) {
                // rehash if no insertable index
                // then re-compute insertable index
                None => {
                    self.rehash();
                },
                // insert value when found insertable index
                Some(idx) => {
                    self.buckets[idx] = Some(Bucket{
                        key: key.clone(),
                        hashed_key: hashed_key,
                        value: value,
                        deleted: false,
                    });
                    return;
                }
            }
        }
    }

    fn compute_insertable_index(&self, hashed_key: u64, buckets: &Vec<Option<Bucket>>) -> Option<usize> {
        let idx = self.compute_bucket_index(hashed_key, buckets.len());
        
        for i in idx..cmp::min(idx + Self::MAX_PROBE, buckets.len()) {
            match &buckets[i] {
                // insert value when the bucket is empty
                None => {
                    return Some(i);
                },
                // update value when same key is specified
                Some(bucket) if bucket.hashed_key == hashed_key || bucket.deleted => {
                    return Some(i)
                },
                // insert value to the first empty bucket when hash value collides
                Some(_) => {}
            }
        }
        return None;
    }

    pub fn get(&self, key: &str) -> Option<i32> {
        let hashed_key = self.compute_hash(key);
        let idx = self.compute_bucket_index(hashed_key, self.len());
        
        for i in idx..cmp::min(idx + Self::MAX_PROBE, self.len()) {
            match &self.buckets[i] {
                // return None when reach empty bucket
                None => {
                    return None;
                },
                // return Some when reach non empty bucket and hashed key is identical
                Some(bucket) if bucket.hashed_key == hashed_key && !bucket.deleted => {
                    return Some(bucket.value);
                },
                // continue when reach non empty bucket but hashed key is not identical
                Some(_) => {}
            }
        }
        return None;
    }

    fn len(&self) -> usize {
        self.buckets.len()
    }

    fn compute_hash(&self, key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn compute_bucket_index(&self, hashed_key: u64, len: usize) -> usize {
        (hashed_key % (len as u64)) as usize
    }

    fn rehash(&mut self) {
        let mut rng = rand::thread_rng();
        let mut next_len = self.len() + rng.gen::<usize>() % self.len();
        loop {
            match self.make_rehashed_buckets(next_len) {
                None => {
                    next_len += rng.gen::<usize>() % self.len();
                },
                Some(new_buckets) => {
                    self.buckets = new_buckets;
                    return;
                }
            }
        }
    }

    fn make_rehashed_buckets(&self, next_len: usize) -> Option<Vec<Option<Bucket>>> {
        let mut new_buckets: Vec<Option<Bucket>> = Vec::new();
        new_buckets.resize(next_len, None);
        
        for bucket in &self.buckets {
            match bucket {
                None => {},
                Some(bucket) => {
                    match self.compute_insertable_index(bucket.hashed_key, &new_buckets) {
                        None => {
                            return None;
                        },
                        Some(idx) => {
                            let cloned = bucket.clone();
                            new_buckets[idx] = Some(Bucket{
                                key: cloned.key,
                                hashed_key: cloned.hashed_key,
                                value: cloned.value,
                                deleted: false,
                            })
                        }
                    }
                }
            }
        }
        Some(new_buckets)
    }

    pub fn delete(&mut self, key: &str) {
        let hashed_key = self.compute_hash(key);
        let idx = self.compute_bucket_index(hashed_key, self.len());
        
        for i in idx..cmp::min(idx + Self::MAX_PROBE, self.len()) {
            match &mut self.buckets[i] {
                // do nothing when reach empty bucket
                None => { return; },
                // delete value when reach non empty bucket and hashed key is identical
                Some(bucket) if bucket.hashed_key == hashed_key => {
                    bucket.deleted = true;
                },
                // continue when reach non empty bucket but hashed key is not identical
                Some(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Setup
        let mut hash_table = HashTable::new();
        
        // Exercise: insert
        for i in 1..100 {
            let key = format!("key{}", i);
            let value = i;
            hash_table.upsert(key, value)
        }

        // Verify: get (found)
        for i in 1..100 {
            let key = format!("key{}", i);
            let expected_value = i;
            let actual = hash_table.get(key.as_str());
            assert!(actual.is_some());
            assert_eq!(expected_value, actual.unwrap());
        }

        // Exercise: update
        for i in 1..100 {
            let key = format!("key{}", i);
            let value = i * 2;
            hash_table.upsert(key, value)
        }

        // Verify: update
        for i in 1..100 {
            let key = format!("key{}", i);
            let expected_value = i * 2;
            let actual = hash_table.get(key.as_str());
            assert!(actual.is_some());
            assert_eq!(expected_value, actual.unwrap());
        }

        // Verify: get (not found)
        {
            let actual = hash_table.get("key100");
            assert!(actual.is_none());
        }

        // Exercise: delete and get (not found)
        for i in 1..50 {
            let key = format!("key{}", i);
            hash_table.delete(key.as_str());
            let actual = hash_table.get(key.as_str());
            assert!(actual.is_none());
        }

        // Exercise: insert and get (found)
        for expected_value in 1..50 {
            let key = format!("key{}", expected_value);
            hash_table.upsert(key.clone(), expected_value);
            let actual = hash_table.get(key.as_str());
            assert!(actual.is_some());
            assert_eq!(expected_value, actual.unwrap());
        }
    }
}
