//! Minimal Rust fixture for T1 golden conformance.
//! Covers: fn defs, use import, same-file call, unresolved call, struct.

use std::collections::HashMap;

fn helper(x: i32) -> i32 {
    x + 1
}

struct Point {
    x: i32,
}

fn main() {
    let _ = helper(2);
    missing_fn();
}
