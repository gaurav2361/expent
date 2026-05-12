# BOLT: Performance Intelligence Journal

## 2026-05-11 - Optimized UPI Regex Compilation
- **Architecture/Bottleneck:** The `add_user_upi` function inside `crates/users/src/ops/upi.rs` was compiling a `Regex` on every single invocation. This is highly inefficient in Rust, as `Regex::new` incurs significant initialization overhead, especially under heavy load.
- **Optimization:** Refactored the codebase to use `std::sync::LazyLock` (now standard in Rust >=1.80) to evaluate and store the compiled regex exactly once. This eliminated repetitive parsing strings into automata graphs, directly improving the CPU footprint of the `add_user_upi` operation.
## 2026-05-11 - Optimized UPI Regex Compilation
- **Architecture/Bottleneck:** The `add_user_upi` function inside `crates/users/src/ops/upi.rs` was compiling a `Regex` on every single invocation. This is highly inefficient in Rust, as `Regex::new` incurs significant initialization overhead, especially under heavy load.
- **Optimization:** Refactored the codebase to use `std::sync::LazyLock` (now standard in Rust >=1.80) to evaluate and store the compiled regex exactly once. This eliminated repetitive parsing strings into automata graphs, directly improving the CPU footprint of the `add_user_upi` operation.
