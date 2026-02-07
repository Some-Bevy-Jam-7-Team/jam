# block-pseudorand

This crate allows multi-threaded creation of pseudorandom `Vec<T>`'s of arbitrary length. It does this
by generating arbitrary byte vectors randomly and transmuting to the provided type.

<p style="background:rgba(255,181,77,0.16);padding:0.75em;">
<strong>Warning:</strong> This is wildly unsafe for some types as it does not uphold any invariants your type might expect.
Only use this crate if your type can be safely generated from completely arbitrary bytes.
Generally, this means your type should consist of nothing but primitive numbers such as u32, i64, or f32.
</p>

## Notes

- Generated data is not guaranteed to be cryptographically secure
- Output can be deterministic if you provide a seed
- This is very unsafe for certain types, see the warning above

## Use cases

If you need a lot of random numbers quickly for something non-production critical like unit tests, this may be
a good candidate. Otherwise, if you are planning to use this at runtime, or with types that are non-numeric
or otherwise cannot be created from arbitrary bytes, I would recommend you to choose another, safer crate.

## Usage

If you are certain the above warnings do not apply to the type you are generating, you can use this library like so:

### Without Seed

```rust
use block_pseudorand::block_rand;

let random_data: Vec<u64> = block_rand(128);

assert_eq!(random_data.len(), 128);
```

### With Seed

```rust
use block_pseudorand::block_rand_with_seed;

// Populate this seed as you wish
let seed = [0u8; 32];
let random_data: Vec<u64> = block_rand_with_seed(128, &seed);

assert_eq!(random_data.len(), 128);
```
