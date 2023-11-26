use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use rand::{thread_rng, Rng};
use xoofff::xoodoo;

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

fn xoodoo<const ROUNDS: usize>(c: &mut CriterionHandler) {
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("xoodoo");
    group.throughput(Throughput::Bytes(48)); // Xoodoo permutation works on 384 -bit wide state

    group.bench_function(format!("{} (cached)", ROUNDS), |bench| {
        let mut state = [0u32; 12];
        rng.fill(&mut state);

        bench.iter(|| xoodoo::permute::<{ ROUNDS }>(black_box(&mut state)))
    });
    group.bench_function(format!("{} (random)", ROUNDS), |bench| {
        let mut state = [0u32; 12];
        rng.fill(&mut state);

        bench.iter_batched(
            || state.clone(),
            |mut state| xoodoo::permute::<{ ROUNDS }>(black_box(&mut state)),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "loongarch64"
))]
criterion_group!(name = permutation; config = Criterion::default().with_measurement(CyclesPerByte); targets = xoodoo::<6>, xoodoo::<12>);

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    target_arch = "aarch64",
    target_arch = "loongarch64"
)))]
criterion_group!(permutation, xoodoo::<6>, xoodoo::<12>);

criterion_main!(permutation);
