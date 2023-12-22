//! The main thread produces history by creating new ticks.
//! A second thread runs the verifier.
//! The tick producer sends the verifier blocks of ticks (1M) for verification.
//! After 10 blocks, the producer stops, the threads synchronise, and the
//! producer and verifier sanity-check their history.
//! This demo needs to run on a system with at least 2 cores, or the verifier will fall behind.

type Hasher = sha3::Keccak256;
type Hash = digest::Output<Hasher>;

fn main() {
    // Number of ticks in a block.
    const BLOCK: usize = 1_000_000;

    let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<Hash>>(1);
    let start = std::time::Instant::now();

    // Run the verifier on a separate thread.
    let verifier = std::thread::spawn(move || {
        let mut history: Vec<digest::Output<Hasher>> = vec![];
        for block in rx {
            // Verify the start of the block is valid given the last tick.
            if let (Some(last), Some(first)) = (history.last(), block.first()) {
                proof_of_history::verify::<Hasher>(&[last.clone(), first.clone()]).unwrap();
            }
            // Verify the block.
            proof_of_history::verify::<Hasher>(&block).unwrap();
            println!(
                "{:?}: Verified block {}..{}",
                start.elapsed(),
                history.len(),
                history.len() + BLOCK
            );
            history.extend(block);
        }
        history
    });

    // Produce some ticks.
    let mut history = vec![];
    for (i, tick) in proof_of_history::ticks::<Hasher>()
        .enumerate()
        .take(BLOCK * 10)
    {
        history.push(tick);
        if i > 0 && (i + 1) % BLOCK == 0 {
            println!(
                "{:?}: Produced block {}..{}",
                start.elapsed(),
                history.len() - BLOCK,
                history.len()
            );
            let block = history[history.len() - BLOCK..].to_vec();
            tx.send(block).expect("Verifier fell behind!");
        }
    }

    // Let the verifier begin processing the last block before dropping the channel.
    std::thread::sleep(std::time::Duration::from_secs(1));
    std::mem::drop(tx);
    let verifier_history = verifier.join().unwrap();

    assert_eq!(history, verifier_history);
}
