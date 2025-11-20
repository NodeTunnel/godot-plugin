use std::net::{SocketAddr, UdpSocket};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use godot::global::{godot_print, godot_warn};
use crate::transport::common::Channel;
use crate::transport::reliability::{ReliableReceiver, ReliableSender, SequenceNumber};

pub struct ClientTransport {
    socket: UdpSocket,
    server_addr: SocketAddr,
    reliable_sender: Mutex<ReliableSender>,
    reliable_receiver: Mutex<ReliableReceiver>,
    pending_events: Vec<ClientEvent>,
    last_resend_check: Instant,
    last_ack_send: Instant,
    connected: bool,
}

#[derive(Debug, Clone)]
pub enum ClientEvent {
    PacketReceived { data: Vec<u8>, channel: Channel },
}

impl ClientTransport {
    pub fn new(server_addr: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            server_addr,
            reliable_sender: Mutex::new(ReliableSender::new()),
            reliable_receiver: Mutex::new(ReliableReceiver::new()),
            pending_events: Vec::new(),
            last_resend_check: Instant::now(),
            last_ack_send: Instant::now(),
            connected: true,
        })
    }

    pub fn recv_packets(&mut self) -> Vec<ClientEvent> {
        let mut buf = [0u8; 65535];
        let now = Instant::now();

        // Check resends
        if now.duration_since(self.last_resend_check) > Duration::from_millis(50) {
            self.do_resends();
            self.last_resend_check = now;
        }

        // Check ACKs
        if now.duration_since(self.last_ack_send) > Duration::from_millis(10) {
            self.send_acks().ok();
            self.last_ack_send = now;
        }

        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((len, _addr)) => {
                    if len == 0 { continue; }

                    let packet_type = buf[0];

                    match packet_type {
                        0 => { // Reliable packet
                            if buf.len() < 5 { continue; }
                            let seq = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                            let data = buf[5..len].to_vec();

                            let mut receiver = self.reliable_receiver.lock().unwrap();
                            let acks = receiver.receive(SequenceNumber::new(seq), data);

                            // Queue acks
                            let mut sender = self.reliable_sender.lock().unwrap();
                            for ack in acks {
                                sender.queue_ack(ack);
                            }

                            // Extract received packets
                            let packets = receiver.take_all_packets();
                            for packet in packets {
                                self.pending_events.push(ClientEvent::PacketReceived {
                                    data: packet,
                                    channel: Channel::Reliable,
                                });
                            }
                        }
                        1 => { // Unreliable packet
                            self.pending_events.push(ClientEvent::PacketReceived {
                                data: buf[1..len].to_vec(),
                                channel: Channel::Unreliable,
                            });
                        }
                        2 => { // ACK packet
                            if buf.len() < 5 { continue; }
                            let seq = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                            let mut sender = self.reliable_sender.lock().unwrap();
                            sender.ack_received(SequenceNumber::new(seq));
                        }
                        _ => {}
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        std::mem::take(&mut self.pending_events)
    }

    pub fn send(&self, data: Vec<u8>, channel: Channel) -> Result<(), std::io::Error> {
        match channel {
            Channel::Reliable => {
                let mut sender = self.reliable_sender.lock().unwrap();
                let seq = sender.send(data.clone());

                let mut packet = vec![0u8];
                packet.extend(seq.0.to_be_bytes());
                packet.extend(data);
                self.socket.send_to(&packet, self.server_addr)?;
            }
            Channel::Unreliable => {
                let mut packet = vec![1u8];
                packet.extend(data);
                self.socket.send_to(&packet, self.server_addr)?;
            }
        }
        Ok(())
    }

    fn send_acks(&self) -> Result<(), std::io::Error> {
        let mut sender = self.reliable_sender.lock().unwrap();
        let pending_acks = sender.get_pending_acks();

        for ack in pending_acks {
            let mut packet = vec![2u8];
            packet.extend(ack.0.to_be_bytes());
            self.socket.send_to(&packet, self.server_addr)?;
        }
        Ok(())
    }

    fn do_resends(&self) {
        let mut sender = self.reliable_sender.lock().unwrap();
        let resends = sender.get_resends();

        for (seq, data) in resends {
            let mut packet = vec![0u8];
            packet.extend(seq.0.to_be_bytes());
            packet.extend(data);

            let _ = self.socket.send_to(&packet, self.server_addr);
        }
    }

    pub(crate) fn is_connected(&self) -> bool {
        self.connected
    }
}