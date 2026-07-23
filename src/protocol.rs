use crate::{PAYLOAD_BYTES, Payload};

pub const HARNESS_PACKET_BYTES: usize = 4 + PAYLOAD_BYTES;
pub const WIRE_HEADER_BYTES: usize = 2;
pub const WIRE_PACKET_BYTES: usize = WIRE_HEADER_BYTES + PAYLOAD_BYTES;
pub const SEQUENCE_MODULUS: u32 = 1 << 15;
pub const MAX_OVERHEAD_MULTIPLIER: usize = 2;

const PARITY_BIT: u16 = 1 << 15;
const SEQUENCE_MASK: u16 = PARITY_BIT - 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketKind {
    Data,
    Parity,
}

#[derive(Clone, Copy, Debug)]
pub struct WirePacket<'a> {
    pub kind: PacketKind,
    pub sequence: u16,
    pub payload: &'a Payload,
}

pub fn decode_harness_packet(packet: &[u8]) -> Option<(u32, &Payload)> {
    if packet.len() != HARNESS_PACKET_BYTES {
        return None;
    }
    let sequence = u32::from_be_bytes(packet[..4].try_into().ok()?);
    let payload = packet[4..].try_into().ok()?;
    Some((sequence, payload))
}

pub fn encode_harness_packet(sequence: u32, payload: &Payload) -> [u8; HARNESS_PACKET_BYTES] {
    let mut packet = [0; HARNESS_PACKET_BYTES];
    packet[..4].copy_from_slice(&sequence.to_be_bytes());
    packet[4..].copy_from_slice(payload);
    packet
}

pub fn decode_wire_packet(packet: &[u8]) -> Option<WirePacket<'_>> {
    if packet.len() != WIRE_PACKET_BYTES {
        return None;
    }
    let header = u16::from_be_bytes(packet[..2].try_into().ok()?);
    let kind = if header & PARITY_BIT == 0 {
        PacketKind::Data
    } else {
        PacketKind::Parity
    };
    Some(WirePacket {
        kind,
        sequence: header & SEQUENCE_MASK,
        payload: packet[2..].try_into().ok()?,
    })
}

pub fn encode_wire_packet(
    kind: PacketKind,
    sequence: u32,
    payload: &Payload,
) -> [u8; WIRE_PACKET_BYTES] {
    let kind_bit = match kind {
        PacketKind::Data => 0,
        PacketKind::Parity => PARITY_BIT,
    };
    let header = kind_bit | (sequence as u16 & SEQUENCE_MASK);
    let mut packet = [0; WIRE_PACKET_BYTES];
    packet[..2].copy_from_slice(&header.to_be_bytes());
    packet[2..].copy_from_slice(payload);
    packet
}

#[derive(Default)]
pub struct RedundancyBudget {
    available_bytes: usize,
    parity_sent_this_frame: bool,
}

impl RedundancyBudget {
    pub fn begin_frame(&mut self) {
        let frame_budget = PAYLOAD_BYTES * MAX_OVERHEAD_MULTIPLIER;
        self.available_bytes = self
            .available_bytes
            .saturating_add(frame_budget - WIRE_PACKET_BYTES);
        self.parity_sent_this_frame = false;
    }

    pub fn consume_parity(&mut self) -> bool {
        if self.parity_sent_this_frame || self.available_bytes < WIRE_PACKET_BYTES {
            return false;
        }
        self.available_bytes -= WIRE_PACKET_BYTES;
        self.parity_sent_this_frame = true;
        true
    }
}

#[derive(Default)]
pub struct SequenceUnwrapper {
    newest: Option<u32>,
}

impl SequenceUnwrapper {
    pub fn unwrap(&mut self, wire_sequence: u16) -> u32 {
        let low = u32::from(wire_sequence & SEQUENCE_MASK);
        let Some(newest) = self.newest else {
            self.newest = Some(low);
            return low;
        };

        let modulus = i64::from(SEQUENCE_MODULUS);
        let base = i64::from(newest & !(SEQUENCE_MODULUS - 1));
        let low = i64::from(low);
        let newest_i64 = i64::from(newest);
        let candidates = [base + low - modulus, base + low, base + low + modulus];
        let sequence = candidates
            .into_iter()
            .filter(|candidate| *candidate >= 0)
            .min_by_key(|candidate| (candidate - newest_i64).abs())
            .unwrap_or(low) as u32;

        if sequence > newest {
            self.newest = Some(sequence);
        }
        sequence
    }
}
