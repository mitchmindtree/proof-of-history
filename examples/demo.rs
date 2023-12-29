//! The main thread produces history by creating new ticks.
//! A second thread runs the verifier.
//! The tick producer sends the verifier blocks of ticks (1M) for verification.
//! After 10 blocks, the producer stops, the threads synchronise, and the
//! producer and verifier sanity-check their history.
//! This demo needs to run on a system with at least 2 cores, or the verifier will fall behind.

use digest::Digest;
use std::collections::HashMap;

type Hasher = sha3::Keccak256;
type Hash = digest::Output<Hasher>;

struct Block {
    ticks: Vec<Hash>,
    input_data: HashMap<Hash, String>,
}

// Number of ticks in a block.
const BLOCK: usize = 1_000_000;

fn main() {
    let (tx, rx) = std::sync::mpsc::sync_channel::<Block>(1);
    let seed_data = "Hello World!";
    let seed = Hasher::digest(seed_data.as_bytes());
    let start = std::time::Instant::now();

    // Run the verifier on a separate thread.
    let verifier = std::thread::spawn(move || {
        let mut history: Vec<Hash> = vec![seed];
        let mut data: HashMap<Hash, String> = HashMap::default();
        for block in rx {
            let block_ts = std::time::Instant::now();
            // The closure used to map the tick hash to its input data.
            let data_fn = |_ix, hash: &Hash| {
                block
                    .input_data
                    .get(hash)
                    .map(Hasher::digest)
                    .unwrap_or_default()
            };
            // Verify the start of the block is valid given the last tick.
            if let (Some(last), Some(first)) = (history.last(), block.ticks.first()) {
                let ticks = &[last.clone(), first.clone()];
                proof_of_history::verify::<Hasher, _>(ticks, &data_fn).unwrap();
            }
            // Verify the block.
            proof_of_history::verify::<Hasher, _>(&block.ticks, &data_fn).unwrap();
            println!(
                "{:?}: Verified block {}..{} in {:?}",
                start.elapsed(),
                history.len() - 1,
                history.len() - 1 + BLOCK,
                block_ts.elapsed(),
            );
            history.extend(block.ticks);
            data.extend(block.input_data);
        }
        history
    });

    // Produce 10 blocks of ticks.
    let mut history = vec![seed];
    let mut ticks = proof_of_history::ticks::<Hasher>(seed);
    let mut block_ts = std::time::Instant::now();
    let mut input_data: HashMap<Hash, String> = HashMap::default();
    for i in 0..(BLOCK * 10) {
        // Add some data to every 16th tick.
        let tick = match (i + 1) % 16 {
            0 => {
                // When running PoH in production, the tick thread should be
                // doing as little work as possible besides producing ticks,
                // but for now we construct data on the same thread just for
                // the demo!
                let tick_data = format!("Extra data for tick {i}");
                let tick_data_hash = Hasher::digest(&tick_data);
                let tick = ticks.next_with_data(&tick_data_hash);
                input_data.insert(tick, tick_data);
                tick
            }
            _ => ticks.next(),
        };
        history.push(tick);
        if i > 0 && (i + 1) % BLOCK == 0 {
            println!(
                "{:?}: Produced block {}..{} in {:?}",
                start.elapsed(),
                history.len() - 1 - BLOCK,
                history.len() - 1,
                block_ts.elapsed(),
            );
            let ticks = history[history.len() - BLOCK..].to_vec();
            let block = Block { ticks, input_data };
            tx.send(block).expect("Verifier fell behind!");
            block_ts = std::time::Instant::now();
            input_data = HashMap::default();
        }
    }

    // Let the verifier begin processing the last block before dropping the channel.
    std::thread::sleep(std::time::Duration::from_secs(1));
    std::mem::drop(tx);
    let verifier_history = verifier.join().unwrap();

    assert_eq!(history, verifier_history);
}
