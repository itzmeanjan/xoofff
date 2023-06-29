use super::xoodoo;
use crate::rolling;
use crate::xoodoo as serial_xoodoo;
use crate::xoofff::{bytes_to_le_words, pad10x, words_to_le_bytes};
use core::simd::{u32x4, SimdUint};
use crunchy::unroll;
use std::cmp;

/// Xoodoo\[n_r\] being a 384-bit permutation messages are consumed in 48-byte chunks.
const BLOCK_SIZE: usize = 48;

/// The byte width of the parallel permutation.
const PAR_BLOCK_SIZE: usize = BLOCK_SIZE * 4;

/// \# -of rounds for Xoodoo permutation, see definition 3 of https://ia.cr/2018/767
const ROUNDS: usize = 6;

/// \# -of lanes ( each of 32 -bit width ) in Xoodoo permutation state
const LANE_CNT: usize = BLOCK_SIZE / std::mem::size_of::<u32>();

/// Xoofff is a deck function, obtained by instantiating Farfalle construction with
/// Xoodoo\[6\] permutation and two rolling functions, having nice incremental input/
/// output processing capability, offering ability of restarting `absorb->finalize->squeeze`
/// cycle arbitrary number of times, so that arbitrary number of message sequences ( s.t.
/// each message itself is arbitrary bytes wide ) can be consumed in very flexible fashion.
///
/// One can absorb arbitrary bytes wide message into deck function state and finalize it for
/// squeezing any number of bytes. Once done with squeezing, when new message arrives,
/// `absorb->finalize->squeeze` cycle can be restarted by calling `restart` function, which
/// will prepare deck function state so that new message ( of arbitrary bytes ) can be consumed
/// into deck function state. This way one can absorb messages from a sequence of arbitrary length.
///
/// See https://ia.cr/2016/1188 for definition of Farfalle.
/// Also see https://ia.cr/2018/767 for definition of Xoofff.
///
#[derive(Clone)]
pub struct Xoofff {
    imask: [u32; LANE_CNT],     // input mask
    omask: [u32; LANE_CNT],     // output mask
    acc: [u32x4; LANE_CNT],     // accumulator
    iblk: [u8; PAR_BLOCK_SIZE], // input message block ( buffer )
    oblk: [u8; PAR_BLOCK_SIZE], // output message block ( buffer )
    ioff: usize,                // offset into input message block
    ooff: usize,                // offset into output message block
    finalized: usize,           // is deck function state finalized ?
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
        let mut masked_key = bytes_to_le_words(&padded_key);
        serial_xoodoo::permute::<ROUNDS>(&mut masked_key);

        Self {
            imask: masked_key,
            omask: [0u32; LANE_CNT],
            acc: [u32x4::splat(0u32); LANE_CNT],
            iblk: [0u8; PAR_BLOCK_SIZE],
            oblk: [0u8; PAR_BLOCK_SIZE],
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

        let par_blk_cnt = (self.ioff + msg.len()) / PAR_BLOCK_SIZE;
        let mut moff = 0;

        for _ in 0..par_blk_cnt {
            let byte_cnt = PAR_BLOCK_SIZE - self.ioff;
            self.iblk[self.ioff..].copy_from_slice(&msg[moff..(moff + byte_cnt)]);

            let mut imasks = [[0u32; 12]; 4];
            unroll! {
                for i in 0..4 {
                    imasks[i] = self.imask;
                    rolling::roll_xc(&mut self.imask);
                }
            }

            let imaskx = words_to_statex4(&imasks);

            let mut words = [[0u32; 12]; 4];
            unroll! {
                for i in 0..4 {
                    words[i] = bytes_to_le_words(
                        &self.iblk[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE]
                            .try_into()
                            .unwrap(),
                    );
                }
            }

            let mut states = words_to_statex4(&words);

            debug_assert_eq!(LANE_CNT, 12);

            unroll! {
                for i in 0..12 {
                    states[i] ^= imaskx[i];
                }
            }

            xoodoo::permutex::<4, ROUNDS>(&mut states);

            unroll! {
                for i in 0..12 {
                    self.acc[i] ^= states[i];
                }
            }

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
    /// can be consumed into deck function state, by restarting `absorb->finalize->squeeze` cycle.
    ///
    /// This routine implements portion of algorithm 1 of https://ia.cr/2016/1188.
    #[inline(always)]
    pub fn finalize(&mut self, domain_seperator: u8, ds_bit_width: usize, offset: usize) {
        debug_assert!(
            offset <= BLOCK_SIZE,
            "Byte offset, considered during squeezing, must be <= 44 -bytes"
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

        let blocks = self.ioff / BLOCK_SIZE + 1;

        self.iblk[self.ioff..].fill(0);
        self.iblk[self.ioff] = pad_byte;

        // Absorb the remainder in serial
        let mut acc_final = [[0u32; 12]; 4];
        for i in 0..blocks {
            let mut words = bytes_to_le_words(
                &self.iblk[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE]
                    .try_into()
                    .unwrap(),
            );

            debug_assert_eq!(LANE_CNT, 12);

            unroll! {
                for j in 0..12 {
                    words[j] ^= self.imask[j];
                }
            }

            serial_xoodoo::permute::<ROUNDS>(&mut words);

            unroll! {
                for j in 0..12 {
                    acc_final[0][j] ^= words[j];
                }
            }

            rolling::roll_xc(&mut self.imask);
        }

        let accx = words_to_statex4(&acc_final);

        unroll! {
            for i in 0..12 {
                self.acc[i] ^= accx[i];
            }
        }

        rolling::roll_xc(&mut self.imask);

        self.iblk.fill(0);
        self.ioff = 0;
        self.finalized = usize::MAX;

        unroll! {
            for i in 0..12 {
                self.omask[i] = self.acc[i].reduce_xor();
            }
        }

        serial_xoodoo::permute::<ROUNDS>(&mut self.omask);

        let mut omasks = [[0u32; 12]; 4];
        unroll! {
            for i in 0..4 {
                omasks[i] = self.omask;
                rolling::roll_xe(&mut self.omask);
            }
        }

        let mut states = words_to_statex4(&omasks);

        xoodoo::permutex::<4, ROUNDS>(&mut states);

        debug_assert_eq!(LANE_CNT, 12);
        unroll! {
            for i in 0..12 {
                states[i] ^= u32x4::splat(self.imask[i]);
            }
        }

        statex4_to_bytes(&states, &mut self.oblk);

        self.ooff = offset;
    }

    /// Given that N -many message bytes are already absorbed into deck function state and
    /// state is finalized, this routine can be used for squeezing arbitrary many bytes out
    /// of deck function state. One can call this function arbitrary many times, each time
    /// requesting arbitrary many bytes, if and only if state is already finalized and it's
    /// not yet restarted for processing another message using `absorb->finalize->squeeze` cycle.
    ///
    /// This routine implements last portion of algorithm 1 of https://ia.cr/2016/1188.
    #[inline(always)]
    pub fn squeeze(&mut self, out: &mut [u8]) {
        if self.finalized != usize::MAX {
            return;
        }

        let mut off = 0;

        while off < out.len() {
            let read = cmp::min(PAR_BLOCK_SIZE - self.ooff, out.len() - off);
            out[off..off + read].copy_from_slice(&self.oblk[self.ooff..self.ooff + read]);

            self.ooff += read;
            off += read;

            if self.ooff == PAR_BLOCK_SIZE {
                let mut omasks = [[0u32; 12]; 4];

                unroll! {
                    for i in 0..4 {
                        omasks[i] = self.omask;
                        rolling::roll_xe(&mut self.omask);
                    }
                }

                let mut states = words_to_statex4(&omasks);

                xoodoo::permutex::<4, ROUNDS>(&mut states);

                debug_assert_eq!(LANE_CNT, 12);
                unroll! {
                    for i in 0..12 {
                        states[i] ^= u32x4::splat(self.imask[i]);
                    }
                }

                statex4_to_bytes(&states, &mut self.oblk);

                self.ooff = 0;
            }
        }
    }

    /// Given that a message of arbitrary byte length is absorbed into deck function state and
    /// it's also finalized i.e. ready to be squeezed, this function can be invoked when you've
    /// new message waiting to be absorbed into deck function state and you need to restart the
    /// `absorb->finalize->squeeze` cycle.
    ///
    /// Note, if the deck function state is not yet finalized, calling this function should do nothing.
    /// Remember you're very much allowed to restart `absorb->finalize->squeeze` cycle any number of times
    /// you want.
    ///
    /// This routine implements portion of algorithm 1 of https://ia.cr/2016/1188.
    #[inline(always)]
    pub fn restart(&mut self) {
        if self.finalized != usize::MAX {
            return;
        }

        self.omask.fill(0);
        self.oblk.fill(0);
        self.ooff = 0;
        self.finalized = usize::MIN;
    }
}

#[inline(always)]
fn statex4_to_words(states: &[u32x4; LANE_CNT]) -> [[u32; LANE_CNT]; 4] {
    let mut words = [[0u32; LANE_CNT]; 4];

    debug_assert_eq!(LANE_CNT, 12);

    unroll! {
        for i in 0..12 {
            let arr = states[i].to_array();
            for j in 0..4 {
                words[j][i] = arr[j];
            }
        }
    }

    words
}

#[inline(always)]
pub fn statex4_to_bytes(states: &[u32x4; LANE_CNT], out: &mut [u8; PAR_BLOCK_SIZE]) {
    let words = statex4_to_words(&states);

    unroll! {
        for i in 0..4 {
            words_to_le_bytes(
                &words[i],
                (&mut out[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE])
                    .try_into()
                    .unwrap(),
            );
        }
    }
}

#[inline(always)]
fn words_to_statex4(words: &[[u32; LANE_CNT]; 4]) -> [u32x4; LANE_CNT] {
    let mut states = [u32x4::splat(0u32); LANE_CNT];

    debug_assert_eq!(LANE_CNT, 12);

    unroll! {
        for i in 0..12 {
            let mut arr = [0u32; 4];
            for j in 0..4 {
                arr[j] = words[j][i];
            }
            states[i] = u32x4::from_array(arr);
        }
    }

    states
}
