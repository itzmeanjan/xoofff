# xoofff
Farfalle with Xoodoo: Parallel Permutation-based Cryptography

## Overview

Farfalle is a keyed cryptographic function with extendable input and it's able to return an output of arbitrary length --- it offers a nice and flexible incremental property in both its input and output interfaces. For example, say we have two messages `X`, `Y` and we want to compute `F(X || Y)`, then the cost of processing it is only absoring `Y`, if `F(X)` is already processed. Once `X` is absorbed, you can finalize the state to squeeze arbitrary number of bytes from it. After that one can restart absorption phase, when `Y` is ready to be absorbed, then it can be finalized and arbitrary many bytes can again be squeezed. This way one can restart `absorb->finalize->squeeze` cycle again and again for processing arbitrary number of messages, while accumulator keeps the internal state intact over restarts. This idea is defined in https://ia.cr/2016/1188. And Xoofff is a farfalle contruction which is instantiated with Xoodoo permutation, which was described in https://ia.cr/2018/767. In this (later) paper, deck function name was proposed - which is a keyed function, that takes a sequence of input strings and returns a pseudorandom string of arbitrary length which can be incrementally computed s.t. the acronym **deck** stands for **D**oubly-**E**xtendable **C**ryptographic **K**eyed function. On top of Xoofff deck function, various modes of **S**ession **A**uthenticated **E**ncryption were proposed, *which are not yet implemented in this library*.

Here I'm developing and maintaining a Rust library crate, implementing Xoofff deck function, along with authenticated encryption modes defined on top of it. See [below](#usage) for API usage examples.

## Prerequisites

Rust stable toolchain, which you can obtain by following https://rustup.rs.

```bash
# When developing this library, I was using
rustc --version
rustc 1.68.2 (9eb3afe9e 2023-03-27)
```

## Testing

For ensuring that both 

- oneshot message absorption into/ squeezing from deck function
- incremental message absorption into/ squeezing from deck function

reach same state, I maintain few test cases. You can run those by issuing

> **Warning** I don't yet have any test vectors for ensuring that Xoofff - the deck function implementation itself is functionally correct. I hope that I'll be able to address this issue soon.

```bash
cargo test --lib
```

## Benchmarking

Issue following command for benchmarking deck function Xoofff for various input sizes.

```bash
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo bench xoofff
```

If interested in benchmarking underlying Xoodoo permutation, consider issuing following command.

```bash
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo bench xoodoo --features="dev"
```

### On Intel(R) Core(TM) i5-8279U CPU @ 2.40GHz

#### Xoodoo[{6, 12}] Permutation

```bash
xoodoo/xoodoo[6] (cached)
                        time:   [30.989 ns 31.113 ns 31.274 ns]
                        thrpt:  [1.4294 GiB/s 1.4368 GiB/s 1.4426 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
xoodoo/xoodoo[6] (random)
                        time:   [34.302 ns 34.748 ns 35.229 ns]
                        thrpt:  [1.2689 GiB/s 1.2865 GiB/s 1.3032 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

xoodoo/xoodoo[12] (cached)
                        time:   [60.590 ns 60.857 ns 61.209 ns]
                        thrpt:  [747.86 MiB/s 752.19 MiB/s 755.50 MiB/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
xoodoo/xoodoo[12] (random)
                        time:   [63.608 ns 64.406 ns 65.271 ns]
                        thrpt:  [701.33 MiB/s 710.75 MiB/s 719.66 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
```

#### Xoofff - Deck Function

```bash
xoofff/key = 32 | in = 32 | out = 32 | offset = 16 (cached)
                        time:   [258.24 ns 259.25 ns 260.52 ns]
                        thrpt:  [292.86 MiB/s 294.29 MiB/s 295.43 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
xoofff/key = 32 | in = 32 | out = 32 | offset = 16 (random)
                        time:   [321.99 ns 325.93 ns 330.51 ns]
                        thrpt:  [230.84 MiB/s 234.08 MiB/s 236.94 MiB/s]
Found 15 outliers among 100 measurements (15.00%)
  9 (9.00%) high mild
  6 (6.00%) high severe

xoofff/key = 32 | in = 64 | out = 32 | offset = 16 (cached)
                        time:   [295.70 ns 296.21 ns 296.77 ns]
                        thrpt:  [359.91 MiB/s 360.59 MiB/s 361.21 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
xoofff/key = 32 | in = 64 | out = 32 | offset = 16 (random)
                        time:   [362.85 ns 365.61 ns 368.72 ns]
                        thrpt:  [289.68 MiB/s 292.14 MiB/s 294.36 MiB/s]
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) high mild
  8 (8.00%) high severe

xoofff/key = 32 | in = 128 | out = 32 | offset = 16 (cached)
                        time:   [331.04 ns 331.81 ns 332.67 ns]
                        thrpt:  [504.55 MiB/s 505.85 MiB/s 507.03 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
xoofff/key = 32 | in = 128 | out = 32 | offset = 16 (random)
                        time:   [413.78 ns 418.28 ns 423.30 ns]
                        thrpt:  [396.52 MiB/s 401.28 MiB/s 405.64 MiB/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  3 (3.00%) high severe

xoofff/key = 32 | in = 256 | out = 32 | offset = 16 (cached)
                        time:   [437.37 ns 438.47 ns 439.69 ns]
                        thrpt:  [659.36 MiB/s 661.21 MiB/s 662.86 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
xoofff/key = 32 | in = 256 | out = 32 | offset = 16 (random)
                        time:   [549.86 ns 555.27 ns 560.91 ns]
                        thrpt:  [516.87 MiB/s 522.12 MiB/s 527.25 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe

xoofff/key = 32 | in = 512 | out = 32 | offset = 16 (cached)
                        time:   [614.95 ns 616.40 ns 617.97 ns]
                        thrpt:  [864.21 MiB/s 866.41 MiB/s 868.46 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
xoofff/key = 32 | in = 512 | out = 32 | offset = 16 (random)
                        time:   [806.63 ns 819.31 ns 831.76 ns]
                        thrpt:  [642.08 MiB/s 651.83 MiB/s 662.09 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

xoofff/key = 32 | in = 1024 | out = 32 | offset = 16 (cached)
                        time:   [1.0031 µs 1.0050 µs 1.0071 µs]
                        thrpt:  [1015.1 MiB/s 1017.2 MiB/s 1019.2 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
xoofff/key = 32 | in = 1024 | out = 32 | offset = 16 (random)
                        time:   [1.1083 µs 1.1182 µs 1.1297 µs]
                        thrpt:  [904.96 MiB/s 914.30 MiB/s 922.46 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe

xoofff/key = 32 | in = 2048 | out = 32 | offset = 16 (cached)
                        time:   [1.7482 µs 1.7524 µs 1.7570 µs]
                        thrpt:  [1.1110 GiB/s 1.1139 GiB/s 1.1166 GiB/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
xoofff/key = 32 | in = 2048 | out = 32 | offset = 16 (random)
                        time:   [1.8832 µs 1.8998 µs 1.9186 µs]
                        thrpt:  [1.0175 GiB/s 1.0275 GiB/s 1.0365 GiB/s]
Found 14 outliers among 100 measurements (14.00%)
  7 (7.00%) high mild
  7 (7.00%) high severe

xoofff/key = 32 | in = 4096 | out = 32 | offset = 16 (cached)
                        time:   [3.2770 µs 3.2899 µs 3.3051 µs]
                        thrpt:  [1.1677 GiB/s 1.1731 GiB/s 1.1777 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
xoofff/key = 32 | in = 4096 | out = 32 | offset = 16 (random)
                        time:   [3.4298 µs 3.4573 µs 3.4888 µs]
                        thrpt:  [1.1062 GiB/s 1.1163 GiB/s 1.1253 GiB/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe
```

## Usage

Getting started with using Xoofff - deck function API is fairly easy.

1) Add `xoofff` as dependency in your project's Cargo.toml file.

```toml
[dependencies]
# either
xoofff = { git = "https://github.com/itzmeanjan/xoofff" }
# or
xoofff = "0.1.0"
```

2) Create Xoofff deck function object.

```rust
use xoofff::Xoofff;

fn main() {
    let key = [0xff; 32]; // demo key

    // message sequence to be absorbed
    let msg0 = [0, 1, 2, 3];
    let msg1 = [4, 5, 6, 7];

    let mut dig = [0u8; 32]; // (to be) squeezed output bytes

    let mut deck = Xoofff::new(&key);
    // ...
}
```

3) Absorb arbitrary (>=0) bytes message into deck function state, by issuing `absorb` routine N (>0) -many times.

```rust
// either
deck.absorb(&msg0[..]);

// or
deck.absorb(&msg0[..1]);
deck.absorb(&msg0[1..]);

// this does no harm, but in most cases we can avoid doing it.
deck.absorb(&[]);
```

4) When all message bytes, of first message, are absorbed, we can finalize the state.

```rust
// (first arg) domain seperator can be at max 7 -bits wide
// (second arg) must be <= 7
// (third arg) byte offset, must be <= 48
deck.finalize(0, 0, 8);

// once finalized, calling `finalize` again should do nothing.
```

5) Now we're ready to squeeze arbitrary number of bytes from deck function state, by invoking `squeeze` routine arbitrary number of times.

```rust
// either
deck.squeeze(&mut dig[..]);

// or
deck.squeeze(&mut dig[..16]);
deck.squeeze(&mut dig[16..]);

// you can safely do it, though it's of not much help.
deck.squeeze(&mut []);
```

6) Deck functions support extending input message without paying the cost of processing historical messages in message sequence, once again. Accumulator keeps the absorbed message state intact when state is finalized and ready to be squeezed. When deck function state is restarted, once again, it's ready to go through `absorb->finalize->squeeze` cycle.

```rust
deck.restart();
```

7) Now one can absorb arbitrary number of bytes, from second message in this message sequence, by invoking `absorb` routine arbitrary number of times.

```rust
deck.absorb(&msg1);
```

8) Once all bytes of second message are absorbed, you can finalize the deck function state.

```rust
deck.finalize(0, 0, 8);
```

9) Finally squeeze arbitrary number of bytes from deck function state.

```rust
deck.squeeze(&mut dig);
```

10) As you understand, this way you can again restart by `absorb->finalize->squeeze` cycle, when new message is ready to be processed. Deck functions offer very flexible and extendable input/ output processing interfaces.

I maintain one example, in [deck_function.rs](./examples/deck_function.rs), which you may want to check out. You can also run it by issuing.

```bash
cargo run --example deck_function
```
