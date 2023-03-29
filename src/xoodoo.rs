use crunchy::unroll;

/// Maximum number of rounds one can request to have when applying Xoodoo\[n_r\] permutation i.e. n_r <= MAX_ROUNDS
///
/// See table 2 of https://ia.cr/2018/767
const MAX_ROUNDS: usize = 12;

/// Xoodoo\[n_r\] round constants, taken from table 2 of https://ia.cr/2018/767
const RC: [u32; MAX_ROUNDS] = [
    0x00000058, 0x00000038, 0x000003c0, 0x000000d0, 0x00000120, 0x00000014, 0x00000060, 0x0000002c,
    0x00000380, 0x000000f0, 0x000001a0, 0x00000012,
];

/// Given a plane of Xoodoo permutation state ( each plane has 4 lanes, each lane 32 -bit wide ),
/// this routine function cyclically shifts the plane such that bit at position (x, z) is
/// moved to (x+T, z+V).
///
/// Note, at bit index z = 0, least significant bit of each lane lives.
/// See row 2 of table 1 of https://ia.cr/2018/767.
#[inline(always)]
pub fn cyclic_shift<const T: usize, const V: u32>(plane: &[u32]) -> [u32; 4] {
    debug_assert!(
        plane.len() == 4,
        "Each lane of Xoodoo permutation state must have four lanes !"
    );

    let mut shifted = [0u32; 4];
    unroll! {
        for i in 0..4 {
            shifted[(T + i) & 3usize] = plane[i].rotate_left(V);
        }
    }
    shifted
}

/// θ step mapping of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn theta(state: &mut [u32]) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    let mut p = [0u32; 4];
    unroll! {
        for i in (0..12).step_by(4) {
            p[0] ^= state[i + 0];
            p[1] ^= state[i + 1];
            p[2] ^= state[i + 2];
            p[3] ^= state[i + 3];
        }
    }

    let t0 = cyclic_shift::<1, 5>(&p);
    let t1 = cyclic_shift::<1, 14>(&p);

    let mut e = [0u32; 4];
    unroll! {
        for i in 0..4 {
            e[i] = t0[i] ^ t1[i];
        }
    }

    unroll! {
        for i in (0..12).step_by(4) {
            state[i + 0] ^= e[0];
            state[i + 1] ^= e[1];
            state[i + 2] ^= e[2];
            state[i + 3] ^= e[3];
        }
    }
}

/// ρ_west step mapping function of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn rho_west(state: &mut [u32]) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    let t0 = cyclic_shift::<1, 0>(&state[4..8]);
    let t1 = cyclic_shift::<0, 11>(&state[8..12]);

    state[4..8].copy_from_slice(&t0);
    state[8..12].copy_from_slice(&t1);
}

/// ρ_east step mapping function of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn rho_east(state: &mut [u32]) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    let t0 = cyclic_shift::<0, 1>(&state[4..8]);
    let t1 = cyclic_shift::<2, 8>(&state[8..12]);

    state[4..8].copy_from_slice(&t0);
    state[8..12].copy_from_slice(&t1);
}

/// ι step mapping function of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn iota(state: &mut [u32], ridx: usize) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    state[0] ^= RC[ridx]
}

/// χ step mapping function of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn chi(state: &mut [u32]) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    let mut b0 = [0u32; 4];
    unroll! {
        for i in 0..4 {
            b0[i] = !state[4 + i] & state[8 + i];
        }
    }

    let mut b1 = [0u32; 4];
    unroll! {
        for i in 0..4 {
            b1[i] = !state[8 + i] & state[i];
        }
    }

    let mut b2 = [0u32; 4];
    unroll! {
        for i in 0..4 {
            b2[i] = !state[i] & state[4 + i];
        }
    }

    unroll! {
        for i in 0..4 {
            state[i] ^= b0[i];
            state[4 + i] ^= b1[i];
            state[8 + i] ^= b2[i];
        }
    }
}

/// Round function of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn round(state: &mut [u32], ridx: usize) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );
    debug_assert!(ridx < MAX_ROUNDS, "Round index must ∈ [0, MAX_ROUNDS) !");

    theta(state);
    rho_west(state);
    iota(state, ridx);
    chi(state);
    rho_east(state);
}

/// Xoodoo\[n_r\] permutation function s.t. n_r ( <= MAX_ROUNDS ) times round function
/// is applied on permutation state, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
pub fn permute<const ROUNDS: usize>(state: &mut [u32]) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );
    debug_assert!(
        ROUNDS <= MAX_ROUNDS,
        "Requested rounds must be < MAX_ROUNDS !"
    );

    let start = MAX_ROUNDS - ROUNDS;
    for ridx in start..MAX_ROUNDS {
        round(state, ridx);
    }
}
