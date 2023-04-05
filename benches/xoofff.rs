use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use rand::{thread_rng, RngCore};
use xoofff::Xoofff;

fn xoofff<
    const KLEN: usize,   // deck function key byte length
    const MLEN: usize,   // to be absorbed message byte length
    const DLEN: usize,   // to be squeezed output byte length
    const OFFSET: usize, // deck function offset byte length
>(
    c: &mut Criterion,
) {
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("xoofff");
    group.throughput(Throughput::Bytes((MLEN + DLEN + OFFSET) as u64));

    group.bench_function(
        format!(
            "key = {} | in = {} | out = {} | offset = {} (cached)",
            KLEN, MLEN, DLEN, OFFSET
        ),
        |bench| {
            let mut key = vec![0u8; KLEN];
            let mut msg = vec![0u8; MLEN];
            let mut dig = vec![0u8; DLEN];

            rng.fill_bytes(&mut key);
            rng.fill_bytes(&mut msg);
            rng.fill_bytes(&mut dig);

            bench.iter(|| {
                let mut deck = Xoofff::new(black_box(&key));
                deck.absorb(black_box(&msg));
                deck.finalize(black_box(0), black_box(0), black_box(OFFSET));
                deck.squeeze(black_box(&mut dig));
            });
        },
    );

    group.bench_function(
        format!(
            "key = {} | in = {} | out = {} | offset = {} (random)",
            KLEN, MLEN, DLEN, OFFSET
        ),
        |bench| {
            let mut key = vec![0u8; KLEN];
            let mut msg = vec![0u8; MLEN];
            let mut dig = vec![0u8; DLEN];

            rng.fill_bytes(&mut key);
            rng.fill_bytes(&mut msg);
            rng.fill_bytes(&mut dig);

            bench.iter_batched(
                || (key.clone(), msg.clone(), dig.clone()),
                |(key, msg, mut dig)| {
                    let mut deck = Xoofff::new(black_box(&key));
                    deck.absorb(black_box(&msg));
                    deck.finalize(black_box(0), black_box(0), black_box(OFFSET));
                    deck.squeeze(black_box(&mut dig));
                },
                BatchSize::SmallInput,
            );
        },
    );
}

criterion_group!(
    deck_function,
    xoofff::<32, 32, 32, 16>,
    xoofff::<32, 64, 32, 16>,
    xoofff::<32, 128, 32, 16>,
    xoofff::<32, 256, 32, 16>,
    xoofff::<32, 512, 32, 16>,
    xoofff::<32, 1024, 32, 16>,
    xoofff::<32, 2048, 32, 16>,
    xoofff::<32, 4096, 32, 16>,
);
criterion_main!(deck_function);
