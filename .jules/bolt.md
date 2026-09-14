# Bolt Journal

## Performance Patterns

### Avoiding Unnecessary Database Writes
When tracking file changes using hashes, check if the calculated hash matches the previously stored hash before performing a database `INSERT` or `REPLACE` operation.

**Bottleneck Avoided:**
Without this check, the application would perform an expensive database transaction and disk I/O on every check iteration, even if the file hasn't changed.

**Expected Measurable Impact:**
*   **Reduced CPU Usage:** By skipping the database driver's query preparation and execution overhead.
*   **Lower Disk I/O:** Significantly reduces writes to the disk where the SQLite database is stored.
*   **Faster Loop Iterations:** The main processing loop can sleep or continue much quicker when there are no changes.
