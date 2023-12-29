//! A demo implementation of Proof of History, generic over the hashing implementation.

#[doc(inline)]
pub use digest;
use digest::{Digest, Output};

/// A simple wrapper around the `tick` function that stores the output of the
/// previous tick and automatically supplies it to the `tick` function on each call
/// to `next` or `next_with_data`.
#[derive(Clone, Debug)]
pub struct Ticks<D: Digest> {
    seed: Output<D>,
}

impl<D: Digest> Ticks<D> {
    /// Calls [`tick`] providing the stored `seed` (the hash of the previous
    /// tick, or the initial seed) and default data.
    pub fn next(&mut self) -> Output<D> {
        let data = Output::<D>::default();
        self.next_with_data(&data)
    }

    /// Calls [`tick`] providing the stored `seed` (the hash of the previous
    /// tick, or the initial seed) and the given `data`.
    pub fn next_with_data(&mut self, data: &Output<D>) -> Output<D> {
        self.seed = tick::<D>(&self.seed, data);
        self.seed.clone()
    }
}

/// Computes a cryptographic hash of the given data using the specified digest algorithm.
/// This pure function represents a tick operation in the proof of history.
///
/// Returns an `Output<D>` that contains the resulting hash of the `data` and `extra` combined.
///
/// # Arguments
/// * `seed` - Either the seed hash, or the output of the previous tick.
/// * `data` - The hash of any extra input data that is to be associated with the tick.
///   If no extra data is required, `Output::<D>::default()` should be used.
///
/// Both arguments are fixed size in order to ensure a best-effort consistent
/// timing for the digest, and in turn the tick rate.
pub fn tick<D: Digest>(seed: &Output<D>, data: &Output<D>) -> Output<D> {
    let mut digest = D::new();
    digest.update(seed);
    digest.update(data);
    digest.finalize()
}

/// Creates an infinite iterator that produces a continuous sequence of ticks.
/// Each tick is generated based on the output of the previous tick, creating a chained hash
/// sequence.
///
/// Returns an implementation of `Iterator` that yields `Output<D>` items, each representing a
/// hashed tick.
///
/// # Arguments
/// * `seed` - If no ticks have been emitted, this is the seed hash. Otherwise, this is
///   the output of the previous tick.
///
/// # Examples
/// ```
/// type Hasher = sha2::Sha256;
/// let seed = <_>::default();
/// let mut ticks = proof_of_history::ticks::<Hasher>(seed);
/// for i in 0..10 {
///     let tick = ticks.next();
///     println!("Tick {}: {:x}", i, tick);
/// }
/// ```
pub fn ticks<D: Digest>(seed: Output<D>) -> Ticks<D> {
    Ticks { seed }
}

/// Verifies the validity of a given sequence of ticks using parallel processing.
/// This function checks if each tick correctly follows from its predecessor in the sequence.
///
/// Verification is approximately `number-of-cores` times faster than producing new ticks, due to
/// parallel computation. It is important that verification of existing tick sequences is
/// significantly faster than producing new ticks in order to allow new verifiers to catch up to
/// the tick producer. In practise, verification of large sequences should be performed on the GPU.
///
/// # Arguments
/// * `ticks` - A slice of `Output<D>` representing the sequence of ticks to be verified.
/// * `data` - A function that maps a tick's index and output hash to its input data.
///
/// # Returns
/// A `Result` which is `Ok(())` if the sequence is valid, or `Err(usize)` indicating the index of
/// the first invalid tick.
///
/// # Examples
/// ```
/// type Hasher = sha2::Sha256;
/// let seed = <_>::default();
/// let mut ticks = proof_of_history::ticks::<Hasher>(seed);
/// let ticks: Vec<_> = std::iter::from_fn(|| Some(ticks.next())).take(2usize.pow(16)).collect();
/// proof_of_history::verify::<Hasher, _>(&ticks, |_, _| <_>::default()).unwrap();
/// ```
pub fn verify<D, F>(ticks: &[Output<D>], data: F) -> Result<(), usize>
where
    D: Digest,
    F: Sync + Fn(usize, &Output<D>) -> Output<D>,
{
    use rayon::prelude::*;
    let ix = ticks
        .par_windows(2)
        .enumerate()
        .filter(|&(ix, window)| {
            let seed = &window[0];
            let hash = &window[1];
            let data = data(ix, hash);
            tick::<D>(seed, &data) != *hash
        })
        .map(|(ix, _)| ix)
        .reduce(|| ticks.len(), |a, b| a.min(b));
    if ix < ticks.len() {
        Err(ix)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_verify() {
        type Hasher = sha2::Sha256;
        let timer = std::time::Instant::now();
        let default_data = <_>::default();
        let mut ticks = ticks::<Hasher>(default_data);
        let iter = std::iter::from_fn(|| Some(ticks.next()));
        let ticks: Vec<_> = iter.take(2usize.pow(16)).collect();
        dbg!(timer.elapsed());
        let timer = std::time::Instant::now();
        verify::<Hasher, _>(&ticks, |_i, _h| default_data).unwrap();
        dbg!(timer.elapsed());
    }
}
