use std::io;
use std::net::UdpSocket;

use flaky_network::config::TransportConfig;
use flaky_network::protocol::{
    SequenceUnwrapper, WIRE_PACKET_BYTES, decode_wire_packet, encode_harness_packet,
};
use flaky_network::recovery::RecoveryWindow;

const RELAY_ADDRESS: &str = "127.0.0.1:47002";
const PLAYER_ADDRESS: &str = "127.0.0.1:47020";

fn main() -> io::Result<()> {
    let input = UdpSocket::bind(RELAY_ADDRESS)?;
    let output = UdpSocket::bind("127.0.0.1:0")?;
    output.connect(PLAYER_ADDRESS)?;

    let config = TransportConfig::from_env();
    let mut recovery = RecoveryWindow::new(config.recovery_slots, config.fec_width);
    let mut unwrapper = SequenceUnwrapper::default();
    let mut receive_buffer = [0; WIRE_PACKET_BYTES + 1];

    loop {
        let bytes_received = input.recv(&mut receive_buffer)?;
        let Some(packet) = decode_wire_packet(&receive_buffer[..bytes_received]) else {
            continue;
        };
        let sequence = unwrapper.unwrap(packet.sequence);
        recovery.ingest(sequence, packet, &mut |sequence, payload| {
            output
                .send(&encode_harness_packet(sequence, payload))
                .map(|_| ())
        })?;
    }
}
