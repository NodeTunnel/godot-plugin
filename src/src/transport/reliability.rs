use std::collections::HashMap;
use std::time::{Instant, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SequenceNumber(pub u32);

impl SequenceNumber {
    pub fn new(num: u32) -> Self {
        SequenceNumber(num)
    }

    pub fn next(self) -> Self {
        SequenceNumber(self.0.wrapping_add(1))
    }

    pub fn is_newer_than(self, other: SequenceNumber) -> bool {
        let diff = self.0.wrapping_sub(other.0);
        diff < 2_147_483_648
    }
}

pub struct ReliableSender {
    next_sequence_num: SequenceNumber,
    sent_packets: HashMap<SequenceNumber, SentPacket>,
    ack_timeout: Duration,
    last_resend_check: Instant,
    pending_acks: Vec<SequenceNumber>,
}

struct SentPacket {
    data: Vec<u8>,
    sent_at: Instant,
    resend_count: u32,
}

impl ReliableSender {
    pub fn new() -> Self {
        Self {
            next_sequence_num: SequenceNumber::new(0),
            sent_packets: HashMap::new(),
            ack_timeout: Duration::from_millis(200),
            last_resend_check: Instant::now(),
            pending_acks: Vec::new(),
        }
    }

    pub fn send(&mut self, data: Vec<u8>) -> SequenceNumber {
        let seq = self.next_sequence_num;
        self.next_sequence_num = seq.next();

        self.sent_packets.insert(seq, SentPacket {
            data,
            sent_at: Instant::now(),
            resend_count: 0,
        });

        seq
    }

    pub fn get_resends(&mut self) -> Vec<(SequenceNumber, Vec<u8>)> {
        let now = Instant::now();

        // Only check periodically to avoid constant iteration
        if now.duration_since(self.last_resend_check) < Duration::from_millis(50) {
            return Vec::new();
        }
        self.last_resend_check = now;

        let mut resends = Vec::new();

        for (seq, packet) in self.sent_packets.iter_mut() {
            if now.duration_since(packet.sent_at) > self.ack_timeout {
                resends.push((*seq, packet.data.clone()));
                packet.sent_at = now;
                packet.resend_count += 1;

                if packet.resend_count > 10 {
                    // Too many retries - connection is probably dead
                    // Signal this somehow (or just keep retrying)
                }
            }
        }

        resends
    }

    pub fn ack_received(&mut self, seq: SequenceNumber) {
        self.sent_packets.remove(&seq);
    }

    pub fn has_unacked(&self) -> bool {
        !self.sent_packets.is_empty()
    }

    pub fn queue_ack(&mut self, seq: SequenceNumber) {
        self.pending_acks.push(seq);
    }

    pub fn get_pending_acks(&mut self) -> Vec<SequenceNumber> {
        std::mem::take(&mut self.pending_acks)
    }
}

pub struct ReliableReceiver {
    highest_seq_received: SequenceNumber,
    received_packets: HashMap<SequenceNumber, Vec<u8>>,
    ordered_buffer: Vec<Vec<u8>>,
    expected_next: SequenceNumber,
}

impl ReliableReceiver {
    pub fn new() -> Self {
        Self {
            highest_seq_received: SequenceNumber::new(0),
            received_packets: HashMap::new(),
            ordered_buffer: Vec::new(),
            expected_next: SequenceNumber::new(0),
        }
    }

    /// Process incoming packet, return sequence numbers to ACK
    pub fn receive(&mut self, seq: SequenceNumber, data: Vec<u8>) -> Vec<SequenceNumber> {
        let mut acks = Vec::new();

        // Update highest seen
        if seq.is_newer_than(self.highest_seq_received) || self.highest_seq_received.0 == 0 {
            self.highest_seq_received = seq;
        }

        // If this is the expected packet
        if seq.0 == self.expected_next.0 {
            self.ordered_buffer.push(data);
            self.expected_next = seq.next();
            acks.push(seq);

            // Check if we have buffered packets that come next
            while let Some(buffered) = self.received_packets.remove(&self.expected_next) {
                self.ordered_buffer.push(buffered);
                acks.push(self.expected_next);
                self.expected_next = self.expected_next.next();
            }
        } else if seq.0 > self.expected_next.0 {
            // Out of order - buffer it
            self.received_packets.insert(seq, data);
            acks.push(seq);
        }
        // If seq < expected_next, it's a duplicate - still ack it
        else {
            acks.push(seq);
        }

        acks
    }

    pub fn pop_packet(&mut self) -> Option<Vec<u8>> {
        if self.ordered_buffer.is_empty() {
            None
        } else {
            Some(self.ordered_buffer.remove(0))
        }
    }

    pub fn take_all_packets(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.ordered_buffer)
    }
}