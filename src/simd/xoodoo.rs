use core::simd::{LaneCount, Simd, SupportedLaneCount};
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

#[inline(always)]
pub fn cyclic_shiftx<const N: usize, const T: usize, const V: u32>(
    plane: &[Simd<u32, N>],
) -> [Simd<u32, N>; 4]
where
    LaneCount<N>: SupportedLaneCount,
{
    debug_assert!(
        plane.len() == 4,
        "Each lane of Xoodoo permutation state must have four lanes !"
    );

    let shl = Simd::<u32, N>::splat(V);
    let shr = Simd::<u32, N>::splat(32 - V);

    let mut shifted = [Simd::<u32, N>::splat(0u32); 4];
    unroll! {
        for i in 0..4 {
            shifted[(T + i) & 3usize] = (plane[i] << shl) | (plane[i] >> shr);
        }
    }

    shifted
}

/// θ step mapping of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn thetax<const N: usize>(state: &mut [Simd<u32, N>])
where
    LaneCount<N>: SupportedLaneCount,
{
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    let mut p = [Simd::<u32, N>::splat(0u32); 4];
    unroll! {
        for i in (0..12).step_by(4) {
            p[0] ^= state[i + 0];
            p[1] ^= state[i + 1];
            p[2] ^= state[i + 2];
            p[3] ^= state[i + 3];
        }
    }

    let t0 = cyclic_shiftx::<N, 1, 5>(&p);
    let t1 = cyclic_shiftx::<N, 1, 14>(&p);

    let mut e = [Simd::<u32, N>::splat(0u32); 4];
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
fn rho_westx<const N: usize>(state: &mut [Simd<u32, N>])
where
    LaneCount<N>: SupportedLaneCount,
{
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    let t0 = cyclic_shiftx::<N, 1, 0>(&state[4..8]);
    let t1 = cyclic_shiftx::<N, 0, 11>(&state[8..12]);

    state[4..8].copy_from_slice(&t0);
    state[8..12].copy_from_slice(&t1);
}

/// ρ_east step mapping function of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn rho_eastx<const N: usize>(state: &mut [Simd<u32, N>])
where
    LaneCount<N>: SupportedLaneCount,
{
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    let t0 = cyclic_shiftx::<N, 0, 1>(&state[4..8]);
    let t1 = cyclic_shiftx::<N, 2, 8>(&state[8..12]);

    state[4..8].copy_from_slice(&t0);
    state[8..12].copy_from_slice(&t1);
}

/// ι step mapping function of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn iotax<const N: usize>(state: &mut [Simd<u32, N>], ridx: usize)
where
    LaneCount<N>: SupportedLaneCount,
{
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    state[0] ^= Simd::<u32, N>::splat(RC[ridx]);
}

/// χ step mapping function of Xoodoo permutation, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
fn chix<const N: usize>(state: &mut [Simd<u32, N>])
where
    LaneCount<N>: SupportedLaneCount,
{
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    let mut b0 = [Simd::<u32, N>::splat(0u32); 4];
    unroll! {
        for i in 0..4 {
            b0[i] = !state[4 + i] & state[8 + i];
        }
    }

    let mut b1 = [Simd::<u32, N>::splat(0u32); 4];
    unroll! {
        for i in 0..4 {
            b1[i] = !state[8 + i] & state[i];
        }
    }

    let mut b2 = [Simd::<u32, N>::splat(0u32); 4];
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
fn roundx<const N: usize>(state: &mut [Simd<u32, N>], ridx: usize)
where
    LaneCount<N>: SupportedLaneCount,
{
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );
    debug_assert!(ridx < MAX_ROUNDS, "Round index must ∈ [0, MAX_ROUNDS) !");

    thetax(state);
    rho_westx(state);
    iotax(state, ridx);
    chix(state);
    rho_eastx(state);
}

/// Xoodoo\[n_r\] permutation function s.t. n_r ( <= MAX_ROUNDS ) times round function
/// is applied on permutation state, as described in algorithm 1 of https://ia.cr/2018/767.
#[inline(always)]
pub fn permutex<const N: usize, const ROUNDS: usize>(state: &mut [Simd<u32, N>])
where
    LaneCount<N>: SupportedLaneCount,
{
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
        roundx(state, ridx);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_xoodoo_simd() {
        use crate::simd::xoodoo::permutex;
        use crate::xoodoo::permute;
        use core::simd::u32x2;
        use rand::{thread_rng, Rng};

        let mut rng = thread_rng();

        let mut state1 = [0u32; 12];
        let mut state2 = [0u32; 12];

        rng.fill(&mut state1);
        rng.fill(&mut state2);

        let mut statex2 = [u32x2::splat(0u32); 12];
        for i in 0..12 {
            statex2[i] = u32x2::from_slice(&[state1[i], state2[i]]);
        }

        permute::<12>(&mut state1);
        permute::<12>(&mut state2);
        permutex::<2, 12>(&mut statex2);

        let mut state12 = [0u32; 12];
        let mut state22 = [0u32; 12];
        for i in 0..12 {
            let [s1, s2] = statex2[i].to_array();
            state12[i] = s1;
            state22[i] = s2;
        }

        assert_eq!(state1, state12);
        assert_eq!(state2, state22);
    }
}
