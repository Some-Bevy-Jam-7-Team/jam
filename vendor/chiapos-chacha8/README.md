# chiapos-chacha8

This is a byte-for-byte compatible implementation of chiapos' ChaCha8 in Rust. It has been manually verified in a custom implementation of chiapos in Rust to produce identical output.

On top of producing identical output, this implementation is also multi-threaded using Rayon producing data extremely fast.

## Usage

This crate has some strict expectations that match the original use-case:

 - Output data must be in 512bit blocks
 - The input key must be 256 bits long (ChaCha8 takes either a 128bit or 256bit input in the original implementation)
 - The IV will be zeroed

```rust
// Randomize this key however you would like
let mut key = [0u8; 32];
let chacha8 = ChaCha8::new_from_256bit_key(&key);

// Output data goes in this vec, note that 64 * 8 = 512bits
let num_blocks = 128;
let mut chacha_blocks = vec![0u8; 64 * num_blocks];

// pos is the offset in the output keystream
let pos = 0;
chacha8.get_keystream(pos, &mut chacha_blocks);
```

## Benefits

This crate generates pseudorandom data extremely fast in 512bit blocks. The output is also deterministic for the same inputs.

## Caveats

This crate makes no guarantees about:

 - Correctness with regards to the original ChaCha8 implementation / specification
 - Cryptographical security

