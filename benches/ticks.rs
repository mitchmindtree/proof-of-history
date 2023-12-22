use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn tick_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ticks");
    group.throughput(Throughput::Elements(1));
    group.bench_function("SHA2-256", |b| {
        let mut ticks = proof_of_history::ticks::<sha2::Sha256>();
        b.iter(|| {
            ticks.next().unwrap();
        });
    });
    group.bench_function("SHA3-256", |b| {
        let mut ticks = proof_of_history::ticks::<sha3::Sha3_256>();
        b.iter(|| {
            ticks.next().unwrap();
        });
    });
    group.bench_function("Keccak256", |b| {
        let mut ticks = proof_of_history::ticks::<sha3::Keccak256>();
        b.iter(|| {
            ticks.next().unwrap();
        });
    });
    group.bench_function("BLAKE-3", |b| {
        let mut ticks = proof_of_history::ticks::<blake3::Hasher>();
        b.iter(|| {
            ticks.next().unwrap();
        });
    });
    group.finish();

    let mut group = c.benchmark_group("verify-2^16-ticks");
    group.throughput(Throughput::Elements(1));
    group.bench_function("SHA2-256", |b| {
        type Hasher = sha2::Sha256;
        let ticks: Vec<_> = proof_of_history::ticks::<Hasher>()
            .take(2usize.pow(16))
            .collect();
        b.iter(|| {
            proof_of_history::verify::<Hasher>(&ticks).unwrap();
        });
    });
    group.bench_function("Keccak256", |b| {
        type Hasher = sha3::Keccak256;
        let ticks: Vec<_> = proof_of_history::ticks::<Hasher>()
            .take(2usize.pow(16))
            .collect();
        b.iter(|| {
            proof_of_history::verify::<Hasher>(&ticks).unwrap();
        });
    });
}

criterion_group!(tick_benches, tick_benchmark);
criterion_main!(tick_benches);
