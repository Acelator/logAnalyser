## 2024-03-24 - Rust String Allocation during Parsing
**Learning:** The log parsing functionality used heavy string allocation via `.chars().collect::<String>()` and mutation via `.split_off()`, making it very inefficient.
**Action:** Use zero-copy string slicing (`&str`), iterators, and `find()` for string processing, especially in high-throughput loops like parsing log files, to avoid unnecessary memory allocations.
