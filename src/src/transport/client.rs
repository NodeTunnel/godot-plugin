use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use paperudp::channel::DecodeResult;
use paperudp::packet::PacketType;
use crate::transport::common::Channel;

pub struct ClientTransport {
    socket: UdpSocket,
    channel: paperudp::channel::Channel,
    server_addr: SocketAddr,
    pending_events: Vec<ClientEvent>,
    pending_sends: Vec<Vec<u8>>,
    last_resend_check: Instant,
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
            channel: paperudp::channel::Channel::new(),
            pending_events: Vec::new(),
            pending_sends: Vec::new(),
            last_resend_check: Instant::now(),
            connected: true,
        })
    }

    pub fn recv_packets(&mut self) -> Vec<ClientEvent> {
        let mut buf = [0u8; 65535];
        let now = Instant::now();

        self.flush_pending_packets();

        if now.duration_since(self.last_resend_check) > Duration::from_millis(50) {
            self.do_resends();
            self.last_resend_check = now;
        }

        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((len, _addr)) => {
                    if len == 0 { continue; }
                    let res = self.channel.decode(&buf[..len]);

                    match res {
                        DecodeResult::Data { payload, ack_packet } => {
                            for p in payload {
                                self.pending_events.push(ClientEvent::PacketReceived {
                                    data: p,
                                    channel: Channel::Reliable,
                                });
                            }

                            if let Some(ack) = ack_packet {
                                self.socket.send_to(&ack, self.server_addr).unwrap();
                            }
                        }
                        DecodeResult::Ack { .. } => {}
                        DecodeResult::None => {}
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        std::mem::take(&mut self.pending_events)
    }

    pub fn send(&mut self, data: Vec<u8>, channel: Channel) -> Result<(), std::io::Error> {
        let packet = match channel {
            Channel::Reliable => {
                let pkt = self.channel.encode(
                    &data,
                    PacketType::ReliableOrdered,
                );
                pkt
            }
            Channel::Unreliable => {
                let pkt = self.channel.encode(
                    &data,
                    PacketType::Unreliable,
                );
                pkt
            }
        };

        self.try_send_packet(packet)?;

        Ok(())
    }

    fn try_send_packet(&mut self, packet: Vec<u8>) -> Result<(), std::io::Error> {
        match self.socket.send_to(&packet, self.server_addr) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                self.pending_sends.push(packet);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn flush_pending_packets(&mut self) {
        let mut still_pending = Vec::new();

        for packet in self.pending_sends.drain(..) {
            match self.socket.send_to(&packet, self.server_addr) {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    still_pending.push(packet);
                }
                Err(_) => {}
            }
        }

        self.pending_sends = still_pending;
    }

    fn do_resends(&mut self) {
        for packet in self.channel.collect_resends(Duration::from_millis(100)) {
            self.try_send_packet(packet).unwrap();
        }
    }

    pub fn send_keepalive(&mut self) -> Result<(), std::io::Error> {
        let payload = vec![3u8];
        let pkt = self.channel.encode(
            &payload,
            PacketType::Unreliable,
        );
        self.socket.send_to(&pkt, self.server_addr)?;
        Ok(())
    }

    pub(crate) fn is_connected(&self) -> bool {
        self.connected
    }
}