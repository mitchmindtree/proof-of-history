//! A demo implementation of Proof of History, generic over the hashing implementation.

use digest::{Digest, Output};

/// Computes a cryptographic hash of the given data using the specified digest algorithm.
/// This pure function represents a tick operation in the proof of history.
///
/// Returns an `Output<D>` that contains the resulting hash of the data.
pub fn tick<D: Digest>(data: &[u8]) -> Output<D> {
    let mut digest = D::new();
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
/// # Examples
/// ```
/// for tick_hash in proof_of_history::ticks::<sha2::Sha256>().take(10) {
///     // Do something with `tick_hash`
/// }
/// ```
pub fn ticks<D: Digest>() -> impl Iterator<Item = Output<D>> {
    let mut data = Output::<D>::default();
    std::iter::from_fn(move || {
        let output = tick::<D>(&data);
        data.copy_from_slice(&output);
        Some(output.clone())
    })
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
///
/// # Returns
/// A `Result` which is `Ok(())` if the sequence is valid, or `Err(usize)` indicating the index of
/// the first invalid tick.
///
/// # Examples
/// ```
/// type Hasher = sha2::Sha256;
/// let ticks: Vec<_> = proof_of_history::ticks::<Hasher>().take(2usize.pow(16)).collect();
/// proof_of_history::verify::<Hasher>(&ticks).unwrap();
/// ```
pub fn verify<D: Digest>(ticks: &[Output<D>]) -> Result<(), usize> {
    use rayon::prelude::*;
    let ix = ticks
        .par_windows(2)
        .enumerate()
        .filter(|(_, window)| tick::<D>(&window[0]) != window[1])
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
        let ticks: Vec<_> = ticks::<Hasher>().take(2usize.pow(16)).collect();
        dbg!(timer.elapsed());
        let timer = std::time::Instant::now();
        verify::<Hasher>(&ticks).unwrap();
        dbg!(timer.elapsed());
    }
}
