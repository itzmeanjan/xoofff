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
    mkey: [u32; LANE_CNT],  // masked key
    imask: [u32; LANE_CNT], // input mask
    omask: [u32; LANE_CNT], // output mask
    acc: [u32; LANE_CNT],   // accumulator
    iblk: [u8; BLOCK_SIZE], // input message block ( buffer )
    oblk: [u8; BLOCK_SIZE], // output message block ( buffer )
    ioff: usize,            // offset into input message block
    ooff: usize,            // offset into output message block
    finalized: usize,       // is deck function state finalized ?
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
        let mut masked_key = bytes_to_le_words(&padded_key);
        xoodoo::permute::<ROUNDS>(&mut masked_key);

        Self {
            mkey: masked_key,
            imask: masked_key,
            omask: [0u32; LANE_CNT],
            acc: [0u32; LANE_CNT],
            iblk: [0u8; BLOCK_SIZE],
            oblk: [0u8; BLOCK_SIZE],
            ioff: 0,
            ooff: 0,
            finalized: usize::MIN,
        }
    }

    /// Given a message M of byte length N (>=0), this routine can be used for absorbing
    /// message bytes into the state of the deck function Xoofff, following algorithm 1,
    /// defined in Farfalle specification https://ia.cr/2016/1188.
    ///
    /// Note, this function can be called multiple times until Xoofff state is finalized. Once
    /// finalized, bytes can be squeezed out of deck function state. Even after finalization
    /// new absorption->finalization->squeezing phase can be started by calling restart function.
    #[inline(always)]
    pub fn absorb(&mut self, msg: &[u8]) {
        if self.finalized == usize::MAX {
            return;
        }

        let blk_cnt = (self.ioff + msg.len()) / BLOCK_SIZE;
        let mut moff = 0;

        for _ in 0..blk_cnt {
            let byte_cnt = BLOCK_SIZE - self.ioff;

            self.iblk[self.ioff..].copy_from_slice(&msg[moff..(moff + byte_cnt)]);
            let mut words = bytes_to_le_words(&self.iblk);

            debug_assert_eq!(LANE_CNT, 12);
            unroll! {
                for i in 0..12 {
                    words[i] ^= self.imask[i];
                }
            }

            xoodoo::permute::<ROUNDS>(&mut words);

            debug_assert_eq!(LANE_CNT, 12);
            unroll! {
                for i in 0..12 {
                    self.acc[i] ^= words[i];
                }
            }

            rolling::roll_xc(&mut self.imask);
            moff += byte_cnt;
            self.ioff = 0;
        }

        let rm_bytes = msg.len() - moff;
        let dst_frm = self.ioff;
        let dst_to = dst_frm + rm_bytes;

        self.iblk[dst_frm..dst_to].copy_from_slice(&msg[moff..]);
        self.ioff += rm_bytes;
    }

    /// Given that arbitrary many message bytes are already absorbed into deck function
    /// state, this routine can be used for finalizing the state, so that arbitrary many
    /// bytes can be squeezed out of deck function state.
    ///
    /// - Once finalized, calling this routine again on same object does nothing.
    /// - Attempting to absorb new message bytes on already finalized state, does nothing.
    /// - After finalization, one might start squeezing arbitrary many output bytes.
    /// - After finishing squeezing, when new message arrives, arbitrary many bytes
    /// can be consumed into deck function state, by restarting `absorb -> finalize -> squeeze` cycle.
    #[inline(always)]
    pub fn finalize<const Q: usize>(&mut self, domain_seperator: u8, ds_bit_width: usize) {
        debug_assert!(
            Q % 8 == 0,
            "# -of bits to be dropped during squeezing must be multiple of 8"
        );
        debug_assert!(
            (Q >> 3) < BLOCK_SIZE,
            "Byte offset, considered during squeezing, must be < 48 -bytes"
        );
        debug_assert!(
            ds_bit_width <= 7,
            "Domain seperator bit width is not allowed to be > 7"
        );

        if self.finalized == usize::MAX {
            return;
        }

        let mask = (1u8 << ds_bit_width) - 1u8;
        let pad_byte = (1u8 << ds_bit_width) | (domain_seperator & mask);

        self.iblk[self.ioff..].fill(0);
        self.iblk[self.ioff] = pad_byte;

        let mut words = bytes_to_le_words(&self.iblk);

        debug_assert_eq!(LANE_CNT, 12);
        unroll! {
            for i in 0..12 {
                words[i] ^= self.imask[i];
            }
        }

        xoodoo::permute::<ROUNDS>(&mut words);

        debug_assert_eq!(LANE_CNT, 12);
        unroll! {
            for i in 0..12 {
                self.acc[i] ^= words[i];
            }
        }

        rolling::roll_xc(&mut self.imask);
        rolling::roll_xc(&mut self.imask);

        self.iblk.fill(0);
        self.ioff = 0;
        self.finalized = usize::MAX;

        self.omask.copy_from_slice(&self.acc);
        xoodoo::permute::<ROUNDS>(&mut self.omask);

        let mut words = self.omask;
        xoodoo::permute::<ROUNDS>(&mut words);

        debug_assert_eq!(LANE_CNT, 12);
        unroll! {
            for i in 0..12 {
                words[i] ^= self.imask[i];
            }
        }

        words_to_le_bytes(&words, &mut self.oblk);
        self.ooff = Q >> 3;

        rolling::roll_xe(&mut self.omask);
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

/// Given a byte array of length 48, this routine interprets those bytes as 12 unsigned
/// 32 -bit integers (= u32) s.t. four consecutive bytes are placed in little endian order
/// in a u32 word.
#[inline(always)]
fn bytes_to_le_words(bytes: &[u8; BLOCK_SIZE]) -> [u32; LANE_CNT] {
    let mut words = [0u32; LANE_CNT];

    debug_assert_eq!(LANE_CNT, 12);
    unroll! {
        for i in 0..12 {
            words[i] = u32::from_le_bytes(bytes[i * 4..(i + 1) * 4].try_into().unwrap());
        }
    }
    words
}

#[inline(always)]
fn words_to_le_bytes(words: &[u32; LANE_CNT], bytes: &mut [u8; BLOCK_SIZE]) {
    debug_assert_eq!(LANE_CNT, 12);
    unroll! {
        for i in 0..12 {
            bytes[i*4..(i+1)*4].copy_from_slice(&words[i].to_le_bytes());
        }
    }
}
