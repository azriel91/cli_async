# CLI Async

Stub application to demonstrate benefits of concurrency.

Uses async Rust with a `tokio` executor.

### Examples

```bash
git worktree add ./sync sync
git worktree add ./async_one async_one

# === In separate terminals === #
# synchronous
cd sync && cargo build --release
time ./target/release/cli_async
time ./target/release/cli_async -c 1000 --delay-retrieve 10 --delay-rate-limit 0

# async single threaded
cd sync && cargo build --release
time ./target/release/cli_async
time ./target/release/cli_async -c 1000 --delay-retrieve 10 --delay-rate-limit 0

# async multiple threads
cargo build --release
time ./target/release/cli_async
time ./target/release/cli_async -c 1000 --delay-retrieve 10 --delay-rate-limit 0
```
