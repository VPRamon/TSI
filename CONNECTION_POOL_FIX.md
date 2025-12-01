# Connection Pool Timeout Fix

## Problem
The Rust backend was failing with the error:
```
RuntimeError - Failed to get connection: Timed out in bb8 (method=py_list_schedules, args_count=0)
```

## Root Cause
The bb8 connection pool was exhausting all available connections due to:

1. **Small pool size**: Only 5 connections maximum
2. **No idle timeout**: Connections could remain in the pool indefinitely without validation
3. **No max lifetime**: Connections never expired, leading to potential stale connections
4. **Connection exhaustion**: When all 5 connections were in use, new requests would wait up to 30 seconds before timing out

## Solution
Updated the connection pool configuration in `/workspace/rust_backend/src/db/pool.rs`:

### Before:
```rust
let pool = Pool::builder()
    .max_size(5)
    .connection_timeout(Duration::from_secs(30))
    .build(manager)
```

### After:
```rust
let pool = Pool::builder()
    .max_size(15)                                           // Tripled pool size
    .connection_timeout(Duration::from_secs(30))            // Time to wait for connection
    .idle_timeout(Some(Duration::from_secs(300)))          // Close idle connections after 5 min
    .max_lifetime(Some(Duration::from_secs(1800)))         // Recycle connections after 30 min
    .build(manager)
```

## Changes Made

### 1. Increased Pool Size (5 → 15)
- Allows more concurrent database operations
- Reduces likelihood of connection exhaustion
- Suitable for concurrent web requests and API calls

### 2. Added Idle Timeout (5 minutes)
- Closes connections that are idle for more than 5 minutes
- Frees up resources when load decreases
- Prevents accumulation of unused connections

### 3. Added Max Lifetime (30 minutes)
- Forces connections to be recycled after 30 minutes
- Prevents stale connections from causing issues
- Ensures fresh connections with up-to-date authentication tokens (important for Azure AD)

### 4. Kept Connection Timeout (30 seconds)
- Time to wait for an available connection from the pool
- Prevents indefinite waiting when pool is exhausted
- Provides clear error messages when timeouts occur

## Benefits

1. **Reduced Timeouts**: More connections available = less waiting
2. **Better Resource Management**: Idle connections are cleaned up automatically
3. **Stale Connection Prevention**: Connections are recycled regularly
4. **Improved Concurrency**: Can handle up to 15 concurrent database operations
5. **Azure AD Token Refresh**: Max lifetime ensures tokens stay fresh

## Testing

To verify the fix works:

```bash
python /workspace/test_connection_pool.py
```

This script tests:
- Single call to `list_schedules_db()`
- 10 concurrent calls to verify pool handles load

## Rebuild Instructions

After making changes to the Rust backend:

```bash
cd /workspace/rust_backend
maturin develop --release
```

## Related Files

- `/workspace/rust_backend/src/db/pool.rs` - Connection pool configuration
- `/workspace/rust_backend/src/db/operations.rs` - Database operations using the pool
- `/workspace/rust_backend/src/python/database.rs` - Python bindings for database operations

## Additional Recommendations

If timeout issues persist, consider:

1. **Monitoring**: Add pool metrics to track connection usage
2. **Further Increase**: Raise max_size to 20-25 if needed
3. **Query Optimization**: Review slow queries that hold connections longer
4. **Connection Validation**: Enable connection health checks before use
5. **Retry Logic**: Add exponential backoff retry for transient failures

## Prevention

To avoid similar issues in the future:

1. Monitor pool usage metrics in production
2. Load test with concurrent requests before deployment
3. Set up alerts for connection pool exhaustion
4. Review long-running database operations
5. Ensure connections are always returned to the pool (Rust's RAII helps with this)
