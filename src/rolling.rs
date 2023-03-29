use super::xoodoo::cyclic_shift;

/// Input mask rolling function roll_Xc, updating the Xoodoo permutation state, as
/// described in section 3 of https://ia.cr/2018/767
pub fn roll_xc(state: &mut [u32]) {
    debug_assert!(
        state.len() == 12,
        "Xoodoo permutation state must have 12 lanes !"
    );

    state[0] ^= (state[0] << 13) ^ (state[4].rotate_left(3));
    let b = cyclic_shift::<3, 0>(&state[..4]);

    let (p0, tmp) = state.split_at_mut(4);
    let (p1, p2) = tmp.split_at_mut(4);

    p0.copy_from_slice(p1);
    p1.copy_from_slice(p2);
    p2.copy_from_slice(&b);
}