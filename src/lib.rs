pub mod config;
pub mod protocol;
pub mod recovery;

pub const PAYLOAD_BYTES: usize = 160;
pub type Payload = [u8; PAYLOAD_BYTES];

pub fn xor_in_place(target: &mut Payload, other: &Payload) {
    for (target_byte, other_byte) in target.iter_mut().zip(other) {
        *target_byte ^= other_byte;
    }
}

pub fn xor_payloads(left: &Payload, right: &Payload) -> Payload {
    let mut result = *left;
    xor_in_place(&mut result, right);
    result
}
