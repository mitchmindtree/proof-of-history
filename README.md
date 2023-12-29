# Proof of History

A naive, minimalist, demo implementation of the
[Proof of History (PoH) concept](https://solana.com/solana-whitepaper.pdf).
It allows for the creation of a continuous, cryptographically verifiable
sequence of hashed data (ticks).

The library is generic over the hashing implementation using the RustCrypto
group's [`Digest`](https://docs.rs/digest/latest/digest/) trait, making it easy
to play around with different hashing implementations as the basis for ticks.

The crate provides functions for the following:

- **Tick Generation**: Compute cryptographic hashes (ticks) using any hashing
  algorithm that implements the `Digest` trait from the `digest` crate. Supports
  associating arbitrary data with each tick.
- **Continuous Hash Sequence**: Generate an infinite sequence of ticks, each
  based on the hash of the previous tick, forming a chained hash sequence.
- **Parallel Verification**: Efficiently verify a sequence of ticks using
  parallel processing. Verification is significantly faster than producing new
  ticks ensuring that verifiers can always catch up to the tick producer. That
  said, the multicore implementation here is still orders of magnitude off the
  speedup that could be achieved using the GPU.

## Examples

### Generating Ticks
```rust
type Hasher = sha2::Sha256;
let seed = <_>::default();
let mut ticks = proof_of_history::ticks::<Hasher>(seed);
for i in 0..10 {
    let tick = ticks.next();
    println!("Tick {}: {:x}", i, tick);
}
```

### Verifying Tick Sequences
```rust
type Hasher = sha2::Sha256;
let seed = <_>::default();
let mut ticks = proof_of_history::ticks::<Hasher>(seed);
let ticks: Vec<_> = std::iter::from_fn(|| Some(ticks.next())).take(2usize.pow(16)).collect();
proof_of_history::verify::<Hasher, _>(&ticks, |_, _| <_>::default()).unwrap();
```

See [examples/demo.rs](examples/demo.rs) for a demonstration of a tick producer
and a verifier running side by side.

## Benches
Some benches are provided to get an idea of the rate of tick generation per
second for different hashing methods, along with an idea of the margin of error.
E.g. my M2 Macbook Air produced ~5.5M sha256 ticks per second, and verified
~30.7M ticks per second.
