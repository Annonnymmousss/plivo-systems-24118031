use flaky_network::PAYLOAD_BYTES;
use flaky_network::protocol::{
    HARNESS_PACKET_BYTES, MAX_OVERHEAD_MULTIPLIER, PacketKind, RedundancyBudget, SequenceUnwrapper,
    WIRE_PACKET_BYTES, decode_harness_packet, decode_wire_packet, encode_wire_packet,
};

#[test]
fn wire_round_trip_preserves_kind_sequence_and_payload() {
    let payload = [0x5a; PAYLOAD_BYTES];
    for kind in [PacketKind::Data, PacketKind::Parity] {
        let encoded = encode_wire_packet(kind, 12_345, &payload);
        let decoded = decode_wire_packet(&encoded).unwrap();
        assert_eq!(decoded.kind, kind);
        assert_eq!(decoded.sequence, 12_345);
        assert_eq!(decoded.payload, &payload);
    }
}

#[test]
fn malformed_packet_lengths_are_rejected() {
    assert!(decode_wire_packet(&[0; WIRE_PACKET_BYTES - 1]).is_none());
    assert!(decode_harness_packet(&[0; HARNESS_PACKET_BYTES + 1]).is_none());
}

#[test]
fn sequence_unwrapper_handles_wrap_and_reordering() {
    let mut unwrapper = SequenceUnwrapper::default();
    assert_eq!(unwrapper.unwrap(32_766), 32_766);
    assert_eq!(unwrapper.unwrap(32_767), 32_767);
    assert_eq!(unwrapper.unwrap(0), 32_768);
    assert_eq!(unwrapper.unwrap(2), 32_770);
    assert_eq!(unwrapper.unwrap(32_760), 32_760);
}

#[test]
fn parity_schedule_never_exceeds_bandwidth_cap() {
    let mut budget = RedundancyBudget::default();
    let mut parity_packets = 0;
    for frame_count in 1_u32..=100_000 {
        budget.begin_frame();
        if budget.consume_parity() {
            parity_packets += 1;
        }
        let relay_bytes = (frame_count as usize + parity_packets) * WIRE_PACKET_BYTES;
        let byte_limit = frame_count as usize * PAYLOAD_BYTES * MAX_OVERHEAD_MULTIPLIER;
        assert!(relay_bytes <= byte_limit);
    }
    assert_eq!(parity_packets, 97_530);
}

#[test]
fn unused_redundancy_credit_survives_temporarily_unavailable_equations() {
    let mut budget = RedundancyBudget::default();
    for _ in 0..10 {
        budget.begin_frame();
    }

    let mut sent = 0;
    for _ in 0..10 {
        budget.begin_frame();
        sent += usize::from(budget.consume_parity());
        assert!(!budget.consume_parity());
    }

    assert_eq!(sent, 10);
}
