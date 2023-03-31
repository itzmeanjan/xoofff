use crate::xoodoo;

/// Xoodoo\[n_r\] being a 384 -bit permutation, messages are consumed in 48 -bytes chunks
const BLOCK_SIZE: usize = 48;

/// \# -of rounds for Xoodoo permutation, see definition 3 of https://ia.cr/2018/767
const ROUNDS: usize = 6;

/// \# -of lanes ( each of 32 -bit width ) in Xoodoo permutation state
const LANE_CNT: usize = BLOCK_SIZE / std::mem::size_of::<u32>();

/// Xoofff is a deck function, obtained by instantiating Farfalle construction with
/// Xoodoo\[6\] permutation and two rolling functions, having nice incremental input/
/// output processing capability.
///
/// See https://ia.cr/2016/1188 for definition of Farfalle.
/// Also see https://ia.cr/2018/767 for definition of Xoofff.
#[derive(Clone, Copy)]
pub struct Xoofff {
    masked_key: [u32; LANE_CNT],
    input_mask: [u32; LANE_CNT],
    accumulator: [u32; LANE_CNT],
    input_block_index: usize,
    output_block_index: usize,
}

impl Xoofff {
    /// Create a new instance of Xoofff, with a key of byte length < 48, which
    /// can be used for incrementally absorbing messages and squeezing output bytes.
    #[inline(always)]
    pub fn new(key: &[u8]) -> Self {
        debug_assert!(
            key.len() < BLOCK_SIZE,
            "Key byte length must be < {}",
            BLOCK_SIZE
        );

        let padded_key = pad10x(key);

        // masked key derivation phase
        let mut masked_key = [0u32; LANE_CNT];
        for i in 0..LANE_CNT {
            masked_key[i] = u32::from_le_bytes(padded_key[i * 4..(i + 1) * 4].try_into().unwrap());
        }
        xoodoo::permute::<ROUNDS>(&mut masked_key);

        Self {
            masked_key,
            input_mask: masked_key,
            accumulator: [0u32; LANE_CNT],
            input_block_index: 0,
            output_block_index: 0,
        }
    }
}

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
