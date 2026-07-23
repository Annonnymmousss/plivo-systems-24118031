use crate::protocol::{PacketKind, WirePacket};
use crate::{PAYLOAD_BYTES, Payload, xor_in_place};

struct Slot {
    sequence: u32,
    occupied: bool,
    delivered: bool,
    has_data: bool,
    has_parity: bool,
    data: Payload,
    parity: Payload,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            sequence: 0,
            occupied: false,
            delivered: false,
            has_data: false,
            has_parity: false,
            data: [0; PAYLOAD_BYTES],
            parity: [0; PAYLOAD_BYTES],
        }
    }
}

pub struct RecoveryWindow {
    slots: Vec<Slot>,
    mask: usize,
    fec_width: u32,
    newest: Option<u32>,
    work: Vec<u32>,
}

impl RecoveryWindow {
    pub fn new(capacity: usize, fec_width: u32) -> Self {
        let capacity = capacity.max(2).next_power_of_two();
        let fec_width = fec_width.clamp(2, capacity as u32);
        let slots = std::iter::repeat_with(Slot::default)
            .take(capacity)
            .collect();
        Self {
            slots,
            mask: capacity - 1,
            fec_width,
            newest: None,
            work: Vec::with_capacity((capacity + 1) * fec_width as usize),
        }
    }

    pub fn ingest<E>(
        &mut self,
        sequence: u32,
        packet: WirePacket<'_>,
        deliver: &mut impl FnMut(u32, &Payload) -> Result<(), E>,
    ) -> Result<(), E> {
        if self.is_stale(sequence) {
            return Ok(());
        }
        self.newest = Some(self.newest.map_or(sequence, |newest| newest.max(sequence)));

        match packet.kind {
            PacketKind::Data => self.store_data(sequence, packet.payload),
            PacketKind::Parity => self.store_parity(sequence, packet.payload),
        }

        self.work.clear();
        if packet.kind == PacketKind::Data {
            self.deliver_if_new(sequence, deliver)?;
            self.enqueue_equations_containing(sequence);
        } else {
            self.work.push(sequence);
        }

        while let Some(endpoint) = self.work.pop() {
            if let Some(recovered_sequence) = self.try_recover(endpoint) {
                self.deliver_if_new(recovered_sequence, deliver)?;
                self.enqueue_equations_containing(recovered_sequence);
            }
        }
        Ok(())
    }

    fn is_stale(&self, sequence: u32) -> bool {
        self.newest
            .is_some_and(|newest| sequence.saturating_add(self.slots.len() as u32) <= newest)
    }

    fn slot(&self, sequence: u32) -> Option<&Slot> {
        let slot = &self.slots[sequence as usize & self.mask];
        (slot.occupied && slot.sequence == sequence).then_some(slot)
    }

    fn slot_mut(&mut self, sequence: u32) -> &mut Slot {
        let slot = &mut self.slots[sequence as usize & self.mask];
        if !slot.occupied || slot.sequence != sequence {
            *slot = Slot {
                sequence,
                occupied: true,
                ..Slot::default()
            };
        }
        slot
    }

    fn data(&self, sequence: u32) -> Option<&Payload> {
        self.slot(sequence)
            .filter(|slot| slot.has_data)
            .map(|slot| &slot.data)
    }

    fn parity(&self, sequence: u32) -> Option<Payload> {
        self.slot(sequence)
            .filter(|slot| slot.has_parity)
            .map(|slot| slot.parity)
    }

    fn store_data(&mut self, sequence: u32, payload: &Payload) {
        let slot = self.slot_mut(sequence);
        if !slot.has_data {
            slot.data = *payload;
            slot.has_data = true;
        }
    }

    fn store_parity(&mut self, sequence: u32, payload: &Payload) {
        let slot = self.slot_mut(sequence);
        if !slot.has_parity {
            slot.parity = *payload;
            slot.has_parity = true;
        }
    }

    fn deliver_if_new<E>(
        &mut self,
        sequence: u32,
        deliver: &mut impl FnMut(u32, &Payload) -> Result<(), E>,
    ) -> Result<(), E> {
        let slot = self.slot_mut(sequence);
        if slot.has_data && !slot.delivered {
            slot.delivered = true;
            deliver(sequence, &slot.data)?;
        }
        Ok(())
    }

    fn enqueue_equations_containing(&mut self, sequence: u32) {
        for offset in 0..self.fec_width {
            if let Some(endpoint) = sequence.checked_add(offset) {
                self.work.push(endpoint);
            }
        }
    }

    fn try_recover(&mut self, endpoint: u32) -> Option<u32> {
        let mut recovered = self.parity(endpoint)?;
        let start = endpoint.saturating_sub(self.fec_width - 1);
        let mut missing = None;

        for sequence in start..=endpoint {
            if let Some(data) = self.data(sequence) {
                xor_in_place(&mut recovered, data);
            } else if missing.replace(sequence).is_some() {
                return None;
            }
        }

        let sequence = missing?;
        if self.is_stale(sequence) {
            return None;
        }
        self.store_data(sequence, &recovered);
        Some(sequence)
    }
}
