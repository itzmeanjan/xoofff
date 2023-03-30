/// Xoodoo\[n_r\] being a 384 -bit permutation, messages are consumed in 48 -bytes chunks
const BLOCK_SIZE: usize = 48;

/// Given a message of length N -bytes ( s.t. N < 48 ), this routine pads the
/// message following pad10* rule such that padded message length becomes 48 -bytes.
#[inline(always)]
fn pad10x(msg: &[u8]) -> [u8; BLOCK_SIZE] {
    debug_assert!(
        msg.len() < BLOCK_SIZE,
        "Paddable message length must be < {}",
        BLOCK_SIZE
    );

    let mlen = msg.len();
    let mut res = [0u8; BLOCK_SIZE];

    res[..mlen].copy_from_slice(msg);
    res[mlen] = 0x01;

    res
}
