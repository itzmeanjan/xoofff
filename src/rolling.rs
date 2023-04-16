use super::xoodoo::cyclic_shift;

/// Input mask rolling function roll_Xc, updating the Xoodoo permutation state, as
/// described in section 3 of https://ia.cr/2018/767
pub fn roll_xc(state: &mut [u32]) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    state[0] ^= (state[0] << 13) ^ state[4].rotate_left(3);
    let b = cyclic_shift::<3, 0>(&state[..4]);

    state.copy_within(4..12, 0);
    state[8..12].copy_from_slice(&b);
}

/// State rolling function roll_Xe, updating Xoodoo permutation state, as described
/// in section 3 of https://ia.cr/2018/767
pub fn roll_xe(state: &mut [u32]) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    let tmp = state[4] & state[8];
    state[0] = tmp ^ state[0].rotate_left(5) ^ state[4].rotate_left(13) ^ 0x00000007u32;
    let b = cyclic_shift::<3, 0>(&state[..4]);

    state.copy_within(4..12, 0);
    state[8..12].copy_from_slice(&b);
}
