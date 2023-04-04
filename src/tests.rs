use crate::Xoofff;
use rand::{thread_rng, RngCore};
use std::cmp;
use test_case::test_case;

#[test_case(32, 0, 32, 0b1, 1, 0; "key = 32B message = 0B digest = 32B offset = 0bits")]
#[test_case(16, 32, 32, 0b11, 2, 0; "key = 16B message = 32B digest = 32B offset = 0bits")]
#[test_case(32, 64, 32, 0b101, 3, 8; "key = 32B message = 64B digest = 32B offset = 8bits")]
#[test_case(32, 128, 32, 0b101, 3, 16; "key = 32B message = 128B digest = 32B offset = 16bits")]
#[test_case(32, 256, 32, 0b1101, 4, 32; "key = 32B message = 256B digest = 32B offset = 32bits")]
#[test_case(32, 512, 32, 0b10101, 5, 64; "key = 32B message = 512B digest = 32B offset = 64bits")]
#[test_case(32, 1024, 32, 0, 0, 128; "key = 32B message = 1024B digest = 32B offset = 128bits")]
#[test_case(47, 2048, 32, 0b1, 2, 128; "key = 47B message = 1024B digest = 32B offset = 128bits")]
#[test_case(48, 1024, 32, 0, 0, 128 => panics; "key = 48B message = 1024B digest = 32B offset = 128bits")]
#[test_case(32, 1024, 32, 0, 0, 127 => panics; "key = 32B message = 1024B digest = 32B offset = 127bits")]
#[test_case(49, 16, 32, 0b1, 2, 255 => panics; "key = 49B message = 1024B digest = 32B offset = 255bits")]
fn test_xoofff_incremental_absorption(
    klen: usize,
    mlen: usize,
    dlen: usize,
    domain_seperator: u8,
    ds_bit_width: usize,
    offset: usize,
) {
    let mut rng = thread_rng();

    let mut key = vec![0u8; klen];
    let mut msg = vec![0u8; mlen];
    let mut dig0 = vec![0u8; dlen]; // digest from oneshot absorption
    let mut dig1 = vec![0u8; dlen]; // digest from incremental absorption

    rng.fill_bytes(&mut key);
    rng.fill_bytes(&mut msg);

    // oneshot absorption
    let mut deck0 = Xoofff::new(&key);
    deck0.absorb(&msg);
    deck0.absorb(&[]); // empty message absorption should have no side effect !
    deck0.finalize(domain_seperator, ds_bit_width, offset);
    deck0.squeeze(&mut dig0);

    // incremental absorption
    let mut deck1 = Xoofff::new(&key);

    let mut off = 0;
    while off < mlen {
        // because we don't want to be stuck in an infinite loop if msg[off] = 0 !
        let elen = cmp::min(cmp::max(msg[off] as usize, 1), mlen - off);

        deck1.absorb(&msg[off..(off + elen)]);
        off += elen;
    }

    deck1.finalize(domain_seperator, ds_bit_width, offset);
    deck1.squeeze(&mut dig1);

    assert_eq!(dig0, dig1);
}
