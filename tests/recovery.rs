use flaky_network::protocol::{PacketKind, WirePacket};
use flaky_network::recovery::RecoveryWindow;
use flaky_network::{PAYLOAD_BYTES, Payload, xor_payloads};

fn packet(kind: PacketKind, payload: &Payload) -> WirePacket<'_> {
    WirePacket {
        kind,
        sequence: 0,
        payload,
    }
}

#[test]
fn recovers_missing_current_frame_from_previous_data() {
    let first = [0x12; PAYLOAD_BYTES];
    let second = [0xa7; PAYLOAD_BYTES];
    let parity = xor_payloads(&first, &second);
    let mut delivered = Vec::new();
    let mut window = RecoveryWindow::new(8, 2);
    let mut collect = |sequence, payload: &Payload| {
        delivered.push((sequence, *payload));
        Ok::<_, ()>(())
    };

    window
        .ingest(0, packet(PacketKind::Data, &first), &mut collect)
        .unwrap();
    window
        .ingest(1, packet(PacketKind::Parity, &parity), &mut collect)
        .unwrap();

    assert_eq!(delivered, vec![(0, first), (1, second)]);
}

#[test]
fn recovers_missing_previous_frame_from_current_data() {
    let first = [0x35; PAYLOAD_BYTES];
    let second = [0xc1; PAYLOAD_BYTES];
    let parity = xor_payloads(&first, &second);
    let mut delivered = Vec::new();
    let mut window = RecoveryWindow::new(8, 2);
    let mut collect = |sequence, payload: &Payload| {
        delivered.push((sequence, *payload));
        Ok::<_, ()>(())
    };

    window
        .ingest(1, packet(PacketKind::Parity, &parity), &mut collect)
        .unwrap();
    window
        .ingest(1, packet(PacketKind::Data, &second), &mut collect)
        .unwrap();

    assert_eq!(delivered, vec![(1, second), (0, first)]);
}

#[test]
fn propagates_across_a_chain_and_suppresses_duplicates() {
    let frames = [
        [1; PAYLOAD_BYTES],
        [2; PAYLOAD_BYTES],
        [3; PAYLOAD_BYTES],
        [4; PAYLOAD_BYTES],
    ];
    let parities = [
        xor_payloads(&frames[0], &frames[1]),
        xor_payloads(&frames[1], &frames[2]),
        xor_payloads(&frames[2], &frames[3]),
    ];
    let mut delivered = Vec::new();
    let mut window = RecoveryWindow::new(8, 2);

    for (sequence, parity) in (1..=3).zip(&parities) {
        window
            .ingest(
                sequence,
                packet(PacketKind::Parity, parity),
                &mut |sequence, payload| {
                    delivered.push((sequence, *payload));
                    Ok::<_, ()>(())
                },
            )
            .unwrap();
    }
    for _ in 0..2 {
        window
            .ingest(
                3,
                packet(PacketKind::Data, &frames[3]),
                &mut |sequence, payload| {
                    delivered.push((sequence, *payload));
                    Ok::<_, ()>(())
                },
            )
            .unwrap();
    }

    delivered.sort_unstable_by_key(|(sequence, _)| *sequence);
    assert_eq!(
        delivered,
        frames
            .into_iter()
            .enumerate()
            .map(|(sequence, payload)| (sequence as u32, payload))
            .collect::<Vec<_>>()
    );
}

#[test]
fn late_packet_cannot_be_delivered_twice() {
    let payload = [7; PAYLOAD_BYTES];
    let mut window = RecoveryWindow::new(4, 2);
    let mut delivered = Vec::new();

    for sequence in 0..=4 {
        window
            .ingest(
                sequence,
                packet(PacketKind::Data, &payload),
                &mut |sequence, _| {
                    delivered.push(sequence);
                    Ok::<_, ()>(())
                },
            )
            .unwrap();
    }
    window
        .ingest(0, packet(PacketKind::Data, &payload), &mut |sequence, _| {
            delivered.push(sequence);
            Ok::<_, ()>(())
        })
        .unwrap();

    assert_eq!(delivered, vec![0, 1, 2, 3, 4]);
}

#[test]
fn recovery_chain_stops_at_stale_window_boundary() {
    let frame = [9; PAYLOAD_BYTES];
    let parity = [0; PAYLOAD_BYTES];
    let mut window = RecoveryWindow::new(4, 2);
    let mut delivered = Vec::new();

    for sequence in 1..=4 {
        window
            .ingest(
                sequence,
                packet(PacketKind::Parity, &parity),
                &mut |sequence, _| {
                    delivered.push(sequence);
                    Ok::<_, ()>(())
                },
            )
            .unwrap();
    }
    window
        .ingest(4, packet(PacketKind::Data, &frame), &mut |sequence, _| {
            delivered.push(sequence);
            Ok::<_, ()>(())
        })
        .unwrap();

    assert_eq!(delivered, vec![4, 3, 2, 1]);
}

#[test]
fn width_three_equations_recover_two_frame_burst() {
    let frames = [
        [1; PAYLOAD_BYTES],
        [2; PAYLOAD_BYTES],
        [3; PAYLOAD_BYTES],
        [4; PAYLOAD_BYTES],
        [5; PAYLOAD_BYTES],
        [6; PAYLOAD_BYTES],
    ];
    let parity_4 = xor_payloads(&xor_payloads(&frames[2], &frames[3]), &frames[4]);
    let parity_5 = xor_payloads(&xor_payloads(&frames[3], &frames[4]), &frames[5]);
    let mut window = RecoveryWindow::new(16, 3);
    let mut delivered = Vec::new();

    for sequence in [0_u32, 1, 4, 5] {
        window
            .ingest(
                sequence,
                packet(PacketKind::Data, &frames[sequence as usize]),
                &mut |sequence, payload| {
                    delivered.push((sequence, *payload));
                    Ok::<_, ()>(())
                },
            )
            .unwrap();
    }
    for (sequence, parity) in [(4, &parity_4), (5, &parity_5)] {
        window
            .ingest(
                sequence,
                packet(PacketKind::Parity, parity),
                &mut |sequence, payload| {
                    delivered.push((sequence, *payload));
                    Ok::<_, ()>(())
                },
            )
            .unwrap();
    }

    delivered.sort_unstable_by_key(|(sequence, _)| *sequence);
    assert_eq!(
        delivered,
        frames
            .into_iter()
            .enumerate()
            .map(|(sequence, payload)| (sequence as u32, payload))
            .collect::<Vec<_>>()
    );
}

#[test]
fn width_three_recovers_every_internal_two_frame_burst() {
    const FRAME_COUNT: usize = 16;
    let mut frames = [[0; PAYLOAD_BYTES]; FRAME_COUNT];
    for (sequence, payload) in frames.iter_mut().enumerate() {
        for (offset, byte) in payload.iter_mut().enumerate() {
            *byte = (sequence as u8).wrapping_mul(31).wrapping_add(offset as u8);
        }
    }

    for missing_start in 2..FRAME_COUNT - 3 {
        let mut window = RecoveryWindow::new(32, 3);
        let mut delivered = Vec::new();
        for sequence in 0..FRAME_COUNT {
            if sequence != missing_start && sequence != missing_start + 1 {
                window
                    .ingest(
                        sequence as u32,
                        packet(PacketKind::Data, &frames[sequence]),
                        &mut |sequence, payload| {
                            delivered.push((sequence, *payload));
                            Ok::<_, ()>(())
                        },
                    )
                    .unwrap();
            }
            if sequence >= 2 {
                let parity = xor_payloads(
                    &xor_payloads(&frames[sequence - 2], &frames[sequence - 1]),
                    &frames[sequence],
                );
                window
                    .ingest(
                        sequence as u32,
                        packet(PacketKind::Parity, &parity),
                        &mut |sequence, payload| {
                            delivered.push((sequence, *payload));
                            Ok::<_, ()>(())
                        },
                    )
                    .unwrap();
            }
        }

        delivered.sort_unstable_by_key(|(sequence, _)| *sequence);
        assert_eq!(
            delivered,
            frames
                .iter()
                .copied()
                .enumerate()
                .map(|(sequence, payload)| (sequence as u32, payload))
                .collect::<Vec<_>>(),
            "failed to recover frames {missing_start} and {}",
            missing_start + 1
        );
    }
}

#[test]
fn recovery_is_independent_of_data_and_parity_arrival_order() {
    let frames = [
        [0x19; PAYLOAD_BYTES],
        [0x73; PAYLOAD_BYTES],
        [0xc4; PAYLOAD_BYTES],
    ];
    let parity = xor_payloads(&xor_payloads(&frames[0], &frames[1]), &frames[2]);

    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let mut window = RecoveryWindow::new(8, 3);
        let mut delivered = Vec::new();
        for item in order {
            let (sequence, wire_packet) = match item {
                0 => (0, packet(PacketKind::Data, &frames[0])),
                1 => (2, packet(PacketKind::Data, &frames[2])),
                _ => (2, packet(PacketKind::Parity, &parity)),
            };
            window
                .ingest(sequence, wire_packet, &mut |sequence, payload| {
                    delivered.push((sequence, *payload));
                    Ok::<_, ()>(())
                })
                .unwrap();
        }

        delivered.sort_unstable_by_key(|(sequence, _)| *sequence);
        assert_eq!(
            delivered,
            frames
                .into_iter()
                .enumerate()
                .map(|(sequence, payload)| (sequence as u32, payload))
                .collect::<Vec<_>>()
        );
    }
}
