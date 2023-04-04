use crate::Xoofff;
use rand::{thread_rng, RngCore};
use std::cmp;
use test_case::test_case;

#[test_case(32, 0, 32, 0b1, 1, 0; "key = 32bytes message = 0bytes digest = 32bytes offset = 0bits")]
#[test_case(16, 32, 64, 0b11, 2, 0; "key = 16bytes message = 32bytes digest = 64bytes offset = 0bits")]
#[test_case(32, 64, 128, 0b101, 3, 8; "key = 32bytes message = 64bytes digest = 128bytes offset = 8bits")]
#[test_case(32, 128, 256, 0b101, 3, 16; "key = 32bytes message = 128bytes digest = 256bytes offset = 16bits")]
#[test_case(32, 256, 512, 0b1101, 4, 32; "key = 32bytes message = 256bytes digest = 512bytes offset = 32bits")]
#[test_case(32, 512, 1024, 0b10101, 5, 64; "key = 32bytes message = 512bytes digest = 1024bytes offset = 64bits")]
#[test_case(32, 1024, 2048, 0, 0, 128; "key = 32bytes message = 1024bytes digest = 2048bytes offset = 128bits")]
#[test_case(47, 2048, 4096, 0b1, 2, 128; "key = 47bytes message = 1024bytes digest = 4096bytes offset = 128bits")]
#[test_case(48, 1024, 32, 0, 0, 128 => panics; "key = 48bytes message = 1024bytes digest = 32bytes offset = 128bits")]
#[test_case(32, 1024, 32, 0, 0, 127 => panics; "key = 32bytes message = 1024bytes digest = 32bytes offset = 127bits")]
#[test_case(49, 16, 32, 0b1, 2, 255 => panics; "key = 49bytes message = 1024bytes digest = 32bytes offset = 255bits")]
fn test_xoofff_incremental_io(
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

    let mut off = 0;
    let mut read = 0u8;
    while off < dlen {
        let elen = cmp::min(cmp::max(read as usize, 1), dlen - off);

        deck1.squeeze(&mut dig1[off..(off + elen)]);
        off += elen;
        read = dig1[off - 1];
    }

    assert_eq!(dig0, dig1);
}
