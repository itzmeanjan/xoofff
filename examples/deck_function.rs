extern crate hex;
use rand::{thread_rng, RngCore};
use xoofff::Xoofff;

fn main() {
    const KLEN: usize = 32; // bytes
    const MSG0_LEN: usize = 32; // bytes
    const MSG1_LEN: usize = 64; // bytes
    const MSG2_LEN: usize = 128; // bytes
    const MSG3_LEN: usize = 256; // bytes
    const DLEN: usize = 32; // bytes

    const DOMAIN_SEPERATOR: u8 = 0b11;
    const DOMAIN_SEPERATOR_BIT_WIDTH: usize = 2;
    const OFFSET: usize = 17; // bytes

    let mut rng = thread_rng();

    let mut key = vec![0u8; KLEN];
    let mut msg0 = vec![0u8; MSG0_LEN];
    let mut msg1 = vec![0u8; MSG1_LEN];
    let mut msg2 = vec![0u8; MSG2_LEN];
    let mut msg3 = vec![0u8; MSG3_LEN];
    let mut dig = vec![0u8; DLEN];

    // random sample (demo) key
    rng.fill_bytes(&mut key);

    // random sample four messages, creating a message sequence
    rng.fill_bytes(&mut msg0);
    rng.fill_bytes(&mut msg1);
    rng.fill_bytes(&mut msg2);
    rng.fill_bytes(&mut msg3);

    let mut deck = Xoofff::new(&key);

    // absorb first message
    deck.absorb(&msg0);
    deck.finalize(DOMAIN_SEPERATOR, DOMAIN_SEPERATOR_BIT_WIDTH, OFFSET);
    deck.squeeze(&mut dig);
    println!("Digest after consuming msg0 = {}", hex::encode(&dig));

    deck.restart(); // restart absorb->squeeze->finalize cycle

    // absorb second message
    deck.absorb(&msg1);
    deck.finalize(DOMAIN_SEPERATOR, DOMAIN_SEPERATOR_BIT_WIDTH, OFFSET);
    deck.squeeze(&mut dig);
    println!("Digest after consuming msg1 = {}", hex::encode(&dig));

    deck.restart(); // restart absorb->squeeze->finalize cycle

    // absorb third message
    deck.absorb(&msg2);
    deck.finalize(DOMAIN_SEPERATOR, DOMAIN_SEPERATOR_BIT_WIDTH, OFFSET);
    deck.squeeze(&mut dig);
    println!("Digest after consuming msg2 = {}", hex::encode(&dig));

    deck.restart(); // restart absorb->squeeze->finalize cycle

    // absorb last message
    deck.absorb(&msg3);
    deck.finalize(DOMAIN_SEPERATOR, DOMAIN_SEPERATOR_BIT_WIDTH, OFFSET);
    deck.squeeze(&mut dig);
    println!("Digest after consuming msg3 = {}", hex::encode(&dig));
}
