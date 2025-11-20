use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use renet::ClientId;
use tokio::sync::Mutex;
use crate::transport::reliability::{ReliableReceiver, ReliableSender, SequenceNumber};
use super::common::{Channel, ServerEvent};

pub struct ClientSession {
    reliable_sender: Mutex<ReliableSender>,
    reliable_receiver: Mutex<ReliableReceiver>,
    last_heard_from: Instant,
}

pub struct TokioTransport {
    pub(crate) socket: Arc<UdpSocket>,
    clients: HashMap<SocketAddr, ClientId>,
    client_sessions: HashMap<ClientId, ClientSession>,
    client_addrs: HashMap<ClientId, SocketAddr>,
    next_client_id: u64,
    pending_events: Vec<ServerEvent>,
}

impl TokioTransport {
    pub async fn new(addr: SocketAddr) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(addr).await?;

        Ok(Self {
            socket: Arc::new(socket),
            clients: HashMap::new(),
            client_sessions: HashMap::new(),
            client_addrs: HashMap::new(),
            next_client_id: 1,
            pending_events: Vec::new(),
        })
    }

    pub async fn recv_events(&mut self) -> Vec<ServerEvent> {
        let mut buf = [0u8; 65535];

        loop {
            match self.socket.try_recv_from(&mut buf) {
                Ok((len, addr)) => {
                    if len == 0 { continue; }

                    let client_id = if let Some(&id) = self.clients.get(&addr) {
                        id
                    } else {
                        let id = ClientId::from(self.next_client_id);
                        self.next_client_id += 1;
                        self.clients.insert(addr, id);
                        self.client_addrs.insert(id, addr);

                        self.client_sessions.insert(id, ClientSession {
                            reliable_sender: Mutex::new(ReliableSender::new()),
                            reliable_receiver: Mutex::new(ReliableReceiver::new()),
                            last_heard_from: Instant::now(),
                        });

                        self.pending_events.push(ServerEvent::ClientConnected { client_id: id });
                        id
                    };

                    if let Some(session) = self.client_sessions.get_mut(&client_id) {
                        session.last_heard_from = Instant::now();
                    }

                    let packet_type = buf[0];

                    match packet_type {
                        0 => { // Reliable packet
                            if buf.len() < 5 { continue; }
                            let seq = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                            let data = buf[5..len].to_vec();

                            if let Some(session) = self.client_sessions.get_mut(&client_id) {
                                let mut receiver = session.reliable_receiver.lock().await;
                                let acks = receiver.receive(SequenceNumber::new(seq), data);

                                // Queue acks
                                let mut sender = session.reliable_sender.lock().await;
                                for ack in acks {
                                    sender.queue_ack(ack);
                                }

                                // Extract received packets
                                let packets = receiver.take_all_packets();
                                for packet in packets {
                                    self.pending_events.push(ServerEvent::PacketReceived {
                                        client_id,
                                        data: packet,
                                        channel: Channel::Reliable,
                                    });
                                }
                            }
                        }
                        1 => { // Unreliable packet
                            self.pending_events.push(ServerEvent::PacketReceived {
                                client_id,
                                data: buf[1..len].to_vec(),
                                channel: Channel::Unreliable,
                            });
                        }
                        2 => { // ACK packet
                            if buf.len() < 5 { continue; }
                            let seq = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);

                            if let Some(session) = self.client_sessions.get_mut(&client_id) {
                                let mut sender = session.reliable_sender.lock().await;
                                sender.ack_received(SequenceNumber::new(seq));
                            }
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

    pub async fn send(&self, target: ClientId, data: Vec<u8>, channel: Channel) -> Result<(), std::io::Error> {
        if let Some(&addr) = self.client_addrs.get(&target) {
            match channel {
                Channel::Reliable => {
                    if let Some(session) = self.client_sessions.get(&target) {
                        let mut sender = session.reliable_sender.lock().await;
                        let seq = sender.send(data.clone());

                        let mut packet = vec![0u8];
                        packet.extend(seq.0.to_be_bytes());
                        packet.extend(data);
                        self.socket.send_to(&packet, addr).await?;
                    }
                }
                Channel::Unreliable => {
                    let mut packet = vec![1u8];
                    packet.extend(data);
                    self.socket.send_to(&packet, addr).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn send_acks(&self) -> Result<(), std::io::Error> {
        for (client_id, session) in &self.client_sessions {
            let mut sender = session.reliable_sender.lock().await;
            let pending_acks = sender.get_pending_acks();

            for ack in pending_acks {
                let mut packet = vec![2u8]; // ACK packet type
                packet.extend(ack.0.to_be_bytes());

                if let Some(&addr) = self.client_addrs.get(client_id) {
                    self.socket.send_to(&packet, addr).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn resend_unacked(&self) -> Result<(), std::io::Error> {
        for (client_id, session) in &self.client_sessions {
            let mut sender = session.reliable_sender.lock().await;
            let resends = sender.get_resends();

            for (seq, data) in resends {
                let mut packet = vec![0u8];
                packet.extend(seq.0.to_be_bytes());
                packet.extend(data);

                if let Some(&addr) = self.client_addrs.get(client_id) {
                    self.socket.send_to(&packet, addr).await?;
                }
            }
        }
        Ok(())
    }

    pub fn disconnect_client(&mut self, target: ClientId) {
        if let Some(addr) = self.client_addrs.remove(&target) {
            self.clients.remove(&addr);
            self.client_sessions.remove(&target);
            self.pending_events.push(ServerEvent::ClientDisconnected { client_id: target });
        }
    }
}