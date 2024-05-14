use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone, Debug)]
struct Bucket {
    hashed_key: u64,
    value: i32,
}

type BucketChain = Vec<Bucket>;

type BucketChains = Vec<Option<BucketChain>>;

#[derive(Debug)]
struct HashTable {
    chains: BucketChains,
}

impl HashTable {
    const INITIAL_SIZE: usize = 16;

    pub fn new() -> Self {
        let mut chains = Vec::new();
        chains.resize(Self::INITIAL_SIZE, None);
        HashTable{chains}
    }

    fn len(&self) -> usize {
        self.chains.len()
    }

    fn compute_hash(&self, key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn compute_bucket_index(&self, hashed_key: u64, len: usize) -> usize {
        (hashed_key % (len as u64)) as usize
    }

    pub fn upsert(&mut self, key: String, value: i32) {
        let hashed_key = self.compute_hash(key.as_str());
        let idx = self.compute_bucket_index(hashed_key, self.len());

        match &mut self.chains[idx] {
            // insert value if the bucket is empty
            None => {
                let mut chain = Vec::new();
                let bucket = Bucket {
                    hashed_key: hashed_key,
                    value: value,
                };
                chain.push(bucket);
                self.chains[idx] = Some(chain);
            },
            Some(chain) => {
                let bucket = Bucket {
                    hashed_key: hashed_key,
                    value: value,
                };
                // update value if the hashed key collides
                for bucket in chain.iter_mut() {
                    if bucket.hashed_key == hashed_key {
                        bucket.value = value;
                        return;
                    }
                }
                // isnert value into the tail of chain
                chain.push(bucket);
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<i32> {
        let hashed_key = self.compute_hash(key);
        let idx = self.compute_bucket_index(hashed_key, self.len());

        if self.chains[idx].is_none() {
            return None;
        }

        let chain = self.chains[idx].as_ref().unwrap();
        for bucket in chain {
            if bucket.hashed_key == hashed_key {
                return Some(bucket.value);
            }
        }

        None
    }

    pub fn delete(&mut self, key: &str) {
        let hashed_key = self.compute_hash(key);
        let idx = self.compute_bucket_index(hashed_key, self.len());

        if self.chains[idx].is_none() {
            return;
        }

        let chain = self.chains[idx].as_mut().unwrap();
        let mut delete_idx = None;
        
        for (i, bucket) in chain.iter().enumerate() {
            if bucket.hashed_key == hashed_key {
                delete_idx = Some(i);
                break;
            }
        }

        if delete_idx.is_some() {
            chain.remove(delete_idx.unwrap());
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
