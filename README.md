# xoofff
Farfalle with Xoodoo: Parallel Permutation-based Cryptography

## Overview

Farfalle is a keyed cryptographic function with extendable input and it's able to return an output of arbitrary length --- it offers nice and flexible incremental property in both of its input and output interfaces. For example, say we have two messages `X`, `Y` and we want to compute `F(X || Y)`, then the cost of processing it is only absorbing `Y`, if `F(X)` is already processed. Once `X` is absorbed, you can finalize the state to squeeze arbitrary number of bytes from it. After that one can restart absorption phase, when `Y` is ready to be absorbed, then state can again be finalized and arbitrary many bytes can again be squeezed. This way one can restart `absorb` -> `finalize` -> `squeeze` cycle again and again for processing arbitrary number of messages, while accumulator keeps the internal state intact over restarts. This idea is defined in https://ia.cr/2016/1188. And Xoofff is a farfalle contruction which is instantiated with Xoodoo permutation, which was described in https://ia.cr/2018/767. In this (later) paper, deck function name was proposed - which is a keyed function, that takes a sequence of input strings ( of arbitrary length ) and returns a pseudorandom string of arbitrary length which can be incrementally computed s.t. the acronym **deck** stands for **D**oubly-**E**xtendable **C**ryptographic **K**eyed function.

Here I'm developing and maintaining a Rust library crate, implementing Xoofff deck function. See [below](#usage) for API usage examples.

## Prerequisites

Rust stable toolchain, which you can obtain by following https://rustup.rs.

```bash
# When developing this library, I was using
rustc --version
rustc 1.74.0 (79e9716c9 2023-11-13)
```

> [!TIP]
> I advise you to also use `cargo-criterion` for running benchmark executable. Read more about it @ https://crates.io/crates/cargo-criterion. You can install it system-wide by issuing `$ cargo install cargo-criterion`.

## Testing

For ensuring that Xoofff deck function is correctly implemented and both

- oneshot message absorption into/ squeezing from deck function
- incremental message absorption into/ squeezing from deck function

reach same state, I maintain few test cases. You can run those by issuing

> [!NOTE]
> For ensuring functional correctness of Xoofff implementation, I use known answer tests, generated using reference implementation by Xoofff authors, following instructions specified on https://gist.github.com/itzmeanjan/504113021dec30a0909e5f5b47a5bde5.

```bash
cargo test --lib
```

## Benchmarking

Issue following command for benchmarking deck function Xoofff for various input sizes.

> [!CAUTION]
> When benchmarking make sure you've disabled CPU frequency scaling, otherwise numbers you see can be pretty misleading. I found https://github.com/google/benchmark/blob/b40db869/docs/reducing_variance.md helpful.

```bash
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo criterion xoofff
```

If interested in benchmarking underlying Xoodoo permutation, consider issuing following command.

```bash
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo criterion xoodoo --features="dev"
```

> [!NOTE]
> In case you're running benchmarks on `aarch64` target, consider reading https://github.com/itzmeanjan/criterion-cycles-per-byte/blob/63edc6b46/src/lib.rs#L63-L70.

> [!IMPORTANT]
> In case you didn't install `cargo-criterion`, you've to build and execute benchmark binary with `$ RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo bench ...`.

### On 12th Gen Intel(R) Core(TM) i7-1260P

#### Xoofff - Deck Function

```bash
xoofff/key = 32B | in = 32B | out = 32B | offset = 16B (cached)                                                                            
                        time:   [386.0719 cycles 386.7368 cycles 388.0084 cycles]
                        thrpt:  [4.8501 cpb 4.8342 cpb 4.8259 cpb]
xoofff/key = 32B | in = 32B | out = 32B | offset = 16B (random)                                                                            
                        time:   [463.2273 cycles 468.2415 cycles 472.7826 cycles]
                        thrpt:  [5.9098 cpb 5.8530 cpb 5.7903 cpb]

xoofff/key = 32B | in = 128B | out = 32B | offset = 16B (cached)                                                                            
                        time:   [510.5196 cycles 510.6597 cycles 510.8342 cycles]
                        thrpt:  [2.9025 cpb 2.9015 cpb 2.9007 cpb]
xoofff/key = 32B | in = 128B | out = 32B | offset = 16B (random)                                                                            
                        time:   [597.6074 cycles 598.6353 cycles 599.5923 cycles]
                        thrpt:  [3.4068 cpb 3.4013 cpb 3.3955 cpb]

xoofff/key = 32B | in = 512B | out = 32B | offset = 16B (cached)                                                                            
                        time:   [993.0588 cycles 994.1031 cycles 995.7628 cycles]
                        thrpt:  [1.7781 cpb 1.7752 cpb 1.7733 cpb]
xoofff/key = 32B | in = 512B | out = 32B | offset = 16B (random)                                                                             
                        time:   [1136.8571 cycles 1139.8106 cycles 1143.5883 cycles]
                        thrpt:  [2.0421 cpb 2.0354 cpb 2.0301 cpb]

xoofff/key = 32B | in = 2048B | out = 32B | offset = 16B (cached)                                                                             
                        time:   [2924.0033 cycles 2931.8707 cycles 2941.8708 cycles]
                        thrpt:  [1.4036 cpb 1.3988 cpb 1.3950 cpb]
xoofff/key = 32B | in = 2048B | out = 32B | offset = 16B (random)                                                                             
                        time:   [3010.5796 cycles 3014.4697 cycles 3018.8273 cycles]
                        thrpt:  [1.4403 cpb 1.4382 cpb 1.4363 cpb]

xoofff/key = 32B | in = 8192B | out = 32B | offset = 16B (cached)                                                                             
                        time:   [10624.0893 cycles 10626.9955 cycles 10630.3327 cycles]
                        thrpt:  [1.2901 cpb 1.2897 cpb 1.2893 cpb]
xoofff/key = 32B | in = 8192B | out = 32B | offset = 16B (random)                                                                             
                        time:   [10801.9904 cycles 10817.4401 cycles 10835.1726 cycles]
                        thrpt:  [1.3149 cpb 1.3128 cpb 1.3109 cpb]
```

#### Xoodoo[{6, 12}] Permutation

```bash
xoodoo/6 (cached)       time:   [71.5661 cycles 71.7006 cycles 71.9454 cycles]                
                        thrpt:  [1.4989 cpb 1.4938 cpb 1.4910 cpb]
xoodoo/6 (random)       time:   [82.5140 cycles 82.5797 cycles 82.6480 cycles]                
                        thrpt:  [1.7218 cpb 1.7204 cpb 1.7190 cpb]

xoodoo/12 (cached)      time:   [138.7907 cycles 138.8587 cycles 138.9365 cycles]             
                        thrpt:  [2.8945 cpb 2.8929 cpb 2.8915 cpb]
xoodoo/12 (random)      time:   [150.6641 cycles 152.5450 cycles 154.7112 cycles]             
                        thrpt:  [3.2231 cpb 3.1780 cpb 3.1388 cpb]
```

### On ARM Cortex-A72 (i.e. Raspberry Pi 4B)

#### Xoofff - Deck Function

```bash
xoofff/key = 32B | in = 32B | out = 32B | offset = 16B (cached)                                                                             
                        time:   [1475.7031 cycles 1475.7816 cycles 1475.8610 cycles]
                        thrpt:  [18.4483 cpb 18.4473 cpb 18.4463 cpb]
xoofff/key = 32B | in = 32B | out = 32B | offset = 16B (random)                                                                             
                        time:   [1614.8130 cycles 1617.2083 cycles 1619.2197 cycles]
                        thrpt:  [20.2402 cpb 20.2151 cpb 20.1852 cpb]

xoofff/key = 32B | in = 128B | out = 32B | offset = 16B (cached)                                                                             
                        time:   [2155.4275 cycles 2155.6064 cycles 2155.8013 cycles]
                        thrpt:  [12.2489 cpb 12.2478 cpb 12.2467 cpb]
xoofff/key = 32B | in = 128B | out = 32B | offset = 16B (random)                                                                             
                        time:   [2358.8773 cycles 2366.7201 cycles 2373.2365 cycles]
                        thrpt:  [13.4843 cpb 13.4473 cpb 13.4027 cpb]

xoofff/key = 32B | in = 512B | out = 32B | offset = 16B (cached)                                                                             
                        time:   [4846.7515 cycles 4847.1301 cycles 4847.5460 cycles]
                        thrpt:  [8.6563 cpb 8.6556 cpb 8.6549 cpb]
xoofff/key = 32B | in = 512B | out = 32B | offset = 16B (random)                                                                             
                        time:   [5129.3271 cycles 5141.8396 cycles 5152.6335 cycles]
                        thrpt:  [9.2011 cpb 9.1819 cpb 9.1595 cpb]

xoofff/key = 32B | in = 2048B | out = 32B | offset = 16B (cached)                                                                             
                        time:   [15637.2770 cycles 15638.4849 cycles 15639.8726 cycles]
                        thrpt:  [7.4618 cpb 7.4611 cpb 7.4605 cpb]
xoofff/key = 32B | in = 2048B | out = 32B | offset = 16B (random)                                                                             
                        time:   [16086.6226 cycles 16098.8318 cycles 16109.5064 cycles]
                        thrpt:  [7.6858 cpb 7.6807 cpb 7.6749 cpb]

xoofff/key = 32B | in = 8192B | out = 32B | offset = 16B (cached)                                                                             
                        time:   [58704.8839 cycles 58707.4874 cycles 58710.3427 cycles]
                        thrpt:  [7.1250 cpb 7.1247 cpb 7.1244 cpb]
xoofff/key = 32B | in = 8192B | out = 32B | offset = 16B (random)                                                                             
                        time:   [59828.4330 cycles 59853.6571 cycles 59876.0584 cycles]
                        thrpt:  [7.2665 cpb 7.2638 cpb 7.2607 cpb]
```

#### Xoodoo[{6, 12}] Permutation

```bash
xoodoo/6 (cached)       time:   [293.7426 cycles 293.8227 cycles 293.9663 cycles]            
                        thrpt:  [6.1243 cpb 6.1213 cpb 6.1196 cpb]
xoodoo/6 (random)       time:   [312.2511 cycles 312.3959 cycles 312.5695 cycles]            
                        thrpt:  [6.5119 cpb 6.5082 cpb 6.5052 cpb]

xoodoo/12 (cached)      time:   [618.6418 cycles 619.0397 cycles 619.4066 cycles]             
                        thrpt:  [12.9043 cpb 12.8967 cpb 12.8884 cpb]
xoodoo/12 (random)      time:   [638.8219 cycles 638.9825 cycles 639.1287 cycles]             
                        thrpt:  [13.3152 cpb 13.3121 cpb 13.3088 cpb]
```

## Usage

Getting started with using Xoofff - deck function API is fairly easy.

1) Add `xoofff` as dependency in your project's Cargo.toml file.

```toml
[dependencies]
# either
xoofff = { git = "https://github.com/itzmeanjan/xoofff" }
# or
xoofff = "=0.1.2"
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

6) Deck functions support extending input message without paying the cost of processing historical messages in message sequence, once again. Accumulator keeps the absorbed message state intact when state is finalized and ready to be squeezed. When deck function state is restarted, once again, it's ready to go through `absorb` -> `finalize` -> `squeeze` cycle.

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

10) As you understand, this way you can again restart by `absorb` -> `finalize` -> `squeeze` cycle, when new message is ready to be processed. Deck functions offer very flexible and extendable input/ output processing interfaces.

I maintain one example, in [deck_function.rs](./examples/deck_function.rs), which you may want to check out. You can also run it by issuing.

```bash
cargo run --example deck_function
```
