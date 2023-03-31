use crate::rolling;
use crate::xoodoo;
use crunchy::unroll;
use std::cmp;

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

        // masked key derivation phase
        let padded_key = pad10x(key);
        let mut masked_key = bytes_to_le_words(padded_key);
        xoodoo::permute::<ROUNDS>(&mut masked_key);

        Self {
            masked_key,
            input_mask: masked_key,
            accumulator: [0u32; LANE_CNT],
            input_block_index: 0,
            output_block_index: 0,
        }
    }

    /// Given a message M of byte length N (>=0), this routine can be used for absorbing
    /// message bytes into the state of the deck function Xoofff, following algorithm 1,
    /// defined in Farfalle specification https://ia.cr/2016/1188
    #[inline(always)]
    pub fn absorb(&mut self, msg: &[u8]) {
        let blk_cnt = (msg.len() + BLOCK_SIZE) / BLOCK_SIZE;

        for i in 0..blk_cnt {
            let block = get_ith_block(msg, i);
            let mut words = bytes_to_le_words(block);

            debug_assert_eq!(LANE_CNT, 12);
            unroll! {
                for j in 0..12 {
                    words[j] ^= self.input_mask[j];
                }
            }

            xoodoo::permute::<ROUNDS>(&mut words);

            debug_assert_eq!(LANE_CNT, 12);
            unroll! {
                for j in 0..12 {
                    self.accumulator[j] ^= words[j];
                }
            }

            self.input_block_index += 1;
            rolling::roll_xc(&mut self.input_mask);
        }

        rolling::roll_xc(&mut self.input_mask);
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

/// Given a message of byte length N ( >=0 ), this routine can be used for extracting out
/// i-th message block s.t. `msg` is first padded using `pad10*` rule so that padded message
/// length becomes a multiple of BLOCK_SIZE.
///
/// Block index `i` can take values from interval `[0..((msg.len() + BLOCK_SIZE) / BLOCK_SIZE))`
#[inline(always)]
fn get_ith_block(msg: &[u8], i: usize) -> [u8; BLOCK_SIZE] {
    debug_assert!(
        i >= ((msg.len() + BLOCK_SIZE) / BLOCK_SIZE),
        "Maximum valid message block index can be {}",
        (((msg.len() + BLOCK_SIZE) / BLOCK_SIZE) - 1)
    );

    let start = i * BLOCK_SIZE;
    let end = (i + 1) * BLOCK_SIZE;

    if end <= msg.len() {
        let mut block = [0u8; BLOCK_SIZE];
        block.copy_from_slice(&msg[start..end]);

        return block;
    }

    pad10x(&msg[cmp::min(start, msg.len())..])
}

/// Given a byte array of length 48, this routine interprets those bytes as 12 unsigned
/// 32 -bit integers (= u32) s.t. four consecutive bytes are placed in little endian order
/// in a u32 word.
#[inline(always)]
fn bytes_to_le_words(bytes: [u8; BLOCK_SIZE]) -> [u32; LANE_CNT] {
    let mut words = [0u32; LANE_CNT];

    debug_assert_eq!(LANE_CNT, 12);
    unroll! {
        for i in 0..12 {
            words[i] = u32::from_le_bytes(bytes[i * 4..(i + 1) * 4].try_into().unwrap());
        }
    }
    words
}
