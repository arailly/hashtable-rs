# hashtable-rs

Implementation of OpenAddressing-based Hashtable with Rust.

## Usage
```
let mut hash_table = HashTable::new();
hash_table.insert("key1", 100);
println!("value: {}", hash_table.get("key1").unwrap()); // => "value: 100"
hash_table.delete("key1")
```
