# Real-Time Garbage Collector

[![Documentation](https://docs.rs/rtgc/badge.svg)](https://docs.rs/rtgc/)
[![Crates.io](https://img.shields.io/crates/v/rtgc.svg)](https://crates.io/crates/rtgc)
[![License](https://img.shields.io/crates/l/rtgc.svg)](https://codeberg.org/BillyDM/rtgc/src/branch/main/LICENSE)

A simple garbage collector which collects resources dropped on a realtime thread and safely deallocates them on another thread.

The performance characteristics of the provided smart pointers are equivalant to `Arc` when reading (but constructing them is a bit more expensive).

This crate also contains optional triple buffer types for syncing data (enable with the `triple_buffer` feature).

Optional support for `no_std` is provided through the use of [`bevy_platform`](https://crates.io/crates/bevy_platform).

> This crate is similar to [basedrop](https://crates.io/crates/basedrop), except that it uses a simpler algorithm which makes use of standard library types as much as possible to greatly reduce the amount of internal unsafe code. (It is also not susceptible to memory leaks.) The drawback is that the collection pass is a bit more expensive than basedrop's implementation.

## Example

```rust
use std::time::Duration;
use rtgc::*;

let value_1 = ArcGc::new(String::from("foo"));
// Same as `ArcGc` but for `!Sync` data.
let value_2 = OwnedGc::new(String::from("bar"));

// A simulated "realtime thread"
let rt_thread = std::thread::spawn(move || {
    std::thread::sleep(Duration::from_millis(15));

    // Dropping the values on the realtime thread is realtime-safe
    // because the contents are automatically collected and
    // deallocated on a separate non-realtime thread.
    let _ = value_1;
    let _ = value_2;
});

// A simulated update loop on the main thread
for _ in 0..4 {
    // Call `GlobalRtGc::collect()` periodically to deallocate
    // any resources that were dropped on the realtime thread.
    GlobalRtGc::collect();

    std::thread::sleep(Duration::from_millis(15));
}
```

You can also use a non-static collector with `LocalRtGc` (enabled in the `local_collector` feature).

## Bevy support table

| bevy | rtgc |
|------|------|
| 0.18 | 0.3  |
| 0.17 | 0.2  |
