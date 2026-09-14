## 2024-03-24 - Rust String Allocation during Parsing
**Learning:** The log parsing functionality used heavy string allocation via `.chars().collect::<String>()` and mutation via `.split_off()`, making it very inefficient.
**Action:** Use zero-copy string slicing (`&str`), iterators, and `find()` for string processing, especially in high-throughput loops like parsing log files, to avoid unnecessary memory allocations.
## 2024-03-24 - Rust File Read Performance
**Learning:** `BufReader::lines()` allocates a new `String` for every line. Iterating a file to count lines and then iterating it again to process it is a major I/O anti-pattern.
**Action:** Use `BufReader::read_line(&mut buffer)` with a single, reused `String` buffer (and `.clear()`) to avoid massive per-line memory allocations, and combine operations (like counting and parsing) into a single pass to minimize I/O overhead. Remember that `read_line` includes the newline character, so you may need to `.trim()` the resulting string before parsing it.
