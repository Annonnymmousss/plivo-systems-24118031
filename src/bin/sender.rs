use std::io;
use std::net::UdpSocket;

use flaky_network::Payload;
use flaky_network::config::{MAX_FEC_WIDTH, TransportConfig};
use flaky_network::protocol::{
    HARNESS_PACKET_BYTES, PacketKind, RedundancyBudget, decode_harness_packet, encode_wire_packet,
};
use flaky_network::xor_in_place;

const SOURCE_ADDRESS: &str = "127.0.0.1:47010";
const RELAY_ADDRESS: &str = "127.0.0.1:47001";

fn main() -> io::Result<()> {
    let input = UdpSocket::bind(SOURCE_ADDRESS)?;
    let output = UdpSocket::bind("127.0.0.1:0")?;
    output.connect(RELAY_ADDRESS)?;
    let config = TransportConfig::from_env();

    let mut receive_buffer = [0; HARNESS_PACKET_BYTES + 1];
    let mut history: [Option<(u32, Payload)>; MAX_FEC_WIDTH as usize] =
        [None; MAX_FEC_WIDTH as usize];
    let mut redundancy = RedundancyBudget::default();

    loop {
        let bytes_received = input.recv(&mut receive_buffer)?;
        let Some((sequence, payload)) = decode_harness_packet(&receive_buffer[..bytes_received])
        else {
            continue;
        };

        redundancy.begin_frame();
        output.send(&encode_wire_packet(PacketKind::Data, sequence, payload))?;

        let equation_start = sequence.saturating_sub(config.fec_width - 1);
        let mut parity = *payload;
        let complete = (equation_start..sequence).all(|member_sequence| {
            let member = &history[member_sequence as usize % history.len()];
            if let Some((stored_sequence, stored_payload)) = member
                && *stored_sequence == member_sequence
            {
                xor_in_place(&mut parity, stored_payload);
                true
            } else {
                false
            }
        });
        if complete && sequence > 0 && redundancy.consume_parity() {
            output.send(&encode_wire_packet(PacketKind::Parity, sequence, &parity))?;
        }
        let history_index = sequence as usize % history.len();
        history[history_index] = Some((sequence, *payload));
    }
}
