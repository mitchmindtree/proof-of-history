# Toy Proof of History (PoH) Implementation

A naive, minimalist, demo implementation of the
[Proof of History (PoH) concept](https://solana.com/solana-whitepaper.pdf).
It allows for the creation of a continuous, cryptographically verifiable
sequence of hashed data (ticks).

The library is generic over the hashing implementation using the RustCrypto
group's [`Digest`](https://docs.rs/digest/latest/digest/) trait, making it easy
to play around with different hashing implementations as the basis for ticks.

The crate provides functions for the following:

- **Tick Generation**: Compute cryptographic hashes (ticks) using any hashing
  algorithm that implements the `Digest` trait from the `digest` crate.
- **Continuous Hash Sequence**: Generate an infinite sequence of ticks, each
  based on the hash of the previous tick, forming a chained hash sequence.
- **Parallel Verification**: Efficiently verify a sequence of ticks using
  parallel processing. Verification is significantly faster than producing new
  ticks ensuring that verifiers can always catch up to the tick producer. That
  said, the multicore implementation here is still likely orders of magnitude
  off the speedup that could be achieved using the GPU.

## Examples

### Generating Ticks
```rust
for tick_hash in proof_of_history::ticks::<Sha256>() {
    // Never-ending `tick_hash`es...
}
```

### Verifying Tick Sequences
```rust
let ticks: Vec<_> = proof_of_history::ticks::<Sha256>().take(1024).collect();
proof_of_history::verify::<Sha256>(&ticks).unwrap();
```

See [examples/demo.rs](examples/demo.rs) for a demonstration of a tick producer
and a verifier running side by side.

## Benches
Some benches are provided to get an idea of the rate of tick generation per
second for different hashing methods, along with an idea of the margin of error.
E.g. my M2 Macbook Air produced ~5.5M sha256 ticks per second, and verified
~30.7M ticks per second.
