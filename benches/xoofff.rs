use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use rand::{thread_rng, RngCore};
use xoofff::Xoofff;

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "loongarch64"
))]
use criterion_cycles_per_byte::CyclesPerByte;

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "loongarch64"
))]
type CriterionHandler = Criterion<CyclesPerByte>;

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "loongarch64"
)))]
type CriterionHandler = Criterion;

const FIXED_KEY_LEN: usize = 32;
const MIN_MSG_LEN: usize = 32;
const MAX_MSG_LEN: usize = 8192;
const FIXED_DIG_LEN: usize = 32;
const FIXED_OFFSET: usize = 16;

fn xoofff(c: &mut CriterionHandler) {
    let mut rng = thread_rng();

    let mut mlen = MIN_MSG_LEN;
    while mlen <= MAX_MSG_LEN {
        let mut group = c.benchmark_group("xoofff");
        group.throughput(Throughput::Bytes(
            (mlen + FIXED_DIG_LEN + FIXED_OFFSET) as u64,
        ));

        group.bench_function(
            format!(
                "key = {}B | in = {}B | out = {}B | offset = {}B (cached)",
                FIXED_KEY_LEN, mlen, FIXED_DIG_LEN, FIXED_OFFSET
            ),
            |bench| {
                let mut key = vec![0u8; FIXED_KEY_LEN];
                let mut msg = vec![0u8; mlen];
                let mut dig = vec![0u8; FIXED_DIG_LEN];

                rng.fill_bytes(&mut key);
                rng.fill_bytes(&mut msg);
                rng.fill_bytes(&mut dig);

                bench.iter(|| {
                    let mut deck = Xoofff::new(black_box(&key));
                    deck.absorb(black_box(&msg));
                    deck.finalize(black_box(0), black_box(0), black_box(FIXED_OFFSET));
                    deck.squeeze(black_box(&mut dig));
                });
            },
        );

        group.bench_function(
            format!(
                "key = {}B | in = {}B | out = {}B | offset = {}B (random)",
                FIXED_KEY_LEN, mlen, FIXED_DIG_LEN, FIXED_OFFSET
            ),
            |bench| {
                let mut key = vec![0u8; FIXED_KEY_LEN];
                let mut msg = vec![0u8; mlen];
                let mut dig = vec![0u8; FIXED_DIG_LEN];

                rng.fill_bytes(&mut key);
                rng.fill_bytes(&mut msg);
                rng.fill_bytes(&mut dig);

                bench.iter_batched(
                    || (key.clone(), msg.clone(), dig.clone()),
                    |(key, msg, mut dig)| {
                        let mut deck = Xoofff::new(black_box(&key));
                        deck.absorb(black_box(&msg));
                        deck.finalize(black_box(0), black_box(0), black_box(FIXED_OFFSET));
                        deck.squeeze(black_box(&mut dig));
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        mlen *= 4;
    }
}

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "loongarch64"
))]
criterion_group!(name = deck_function; config = Criterion::default().with_measurement(CyclesPerByte); targets = xoofff);

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "loongarch64"
)))]
criterion_group!(deck_function, xoofff);

criterion_main!(deck_function);
