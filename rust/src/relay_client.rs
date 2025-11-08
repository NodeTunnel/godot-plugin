use std::error::Error;
use std::net::SocketAddr;
use std::thread::sleep;
use std::time::{Duration, Instant};
use godot::classes::multiplayer_peer::TransferMode;
use godot::global::{godot_print, godot_warn};
use renet::DefaultChannel;
use reqwest::blocking::Client;
use crate::packet_type::PacketType;
use crate::renet_packet_peer::RenetPacketPeer;

#[derive(Debug, PartialEq)]
pub enum RelayMode {
    None,
    HostingRoom,
    JoiningRoom(String),
}

#[derive(Debug, PartialEq)]
enum RelayState {
    Disconnected,
    Connecting,
    Connected,
    AwaitingRoom,
    InRoom
}

#[derive(Debug)]
pub enum RelayEvent {
    RoomJoined { room_id: String, peer_id: i32 },
    PeerJoinedRoom { peer_id: i32 },
    GameDataReceived { transfer_mode: TransferMode, from_peer: i32, data: Vec<u8> }
}

pub struct RelayClient {
    packet_peer: RenetPacketPeer,
    pub mode: RelayMode,
    state: RelayState,
    room_id: Option<String>,
}

impl RelayClient {
    pub fn new() -> Self {
        Self {
            packet_peer: RenetPacketPeer::new(),
            mode: RelayMode::None,
            state: RelayState::Disconnected,
            room_id: None,
        }
    }

    pub fn wake_server(&mut self, endpoint: &str, timeout: u64) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let start = Instant::now();

        while start.elapsed() < Duration::from_secs(timeout) {
            if client.get(endpoint)
                .timeout(Duration::from_secs(2))
                .send()
                .is_ok()
            {
                return Ok(())
            }
            sleep(Duration::from_millis(200));
        }

        Err("Timeout".into())
    }

    pub fn connect(&mut self, relay_addr: SocketAddr) -> Result<(), Box<dyn Error>> {
        self.wake_server("https://relay-server.fly.dev/ready", 10)?;
        self.packet_peer.connect(relay_addr)?;
        self.state = RelayState::Connecting;
        Ok(())
    }

    pub fn create_room(&mut self) -> Result<(), Box<dyn Error>> {
        self.mode = RelayMode::HostingRoom;
        Ok(())
    }

    pub fn join_room(&mut self, room_id: String) -> Result<(), Box<dyn Error>> {
        self.mode = RelayMode::JoiningRoom(room_id);
        Ok(())
    }

    pub fn send_game_data(&mut self, target: i32, data: Vec<u8>, transfer_mode: TransferMode) -> Result<(), Box<dyn Error>> {
        if self.state != RelayState::InRoom {
            return Err("Not in room".into())
        }
        
        let packet = PacketType::GameData(target, data).to_bytes();

        match transfer_mode {
            TransferMode::RELIABLE => {
                self.packet_peer.send(&packet, DefaultChannel::ReliableOrdered)?;
            },
            TransferMode::UNRELIABLE_ORDERED | TransferMode::UNRELIABLE => {
                self.packet_peer.send(&packet, DefaultChannel::Unreliable)?;
            },
            _ => {
                godot_warn!("[NodeTunnel] Unrecognized transfer mode")
            }
        }

        Ok(())
    }

    pub fn update(&mut self) -> Result<Vec<RelayEvent>, Box<dyn Error>> {
        let received_packets = self.packet_peer.update()?;
        let mut events = Vec::new();

        if self.state == RelayState::Connecting && self.packet_peer.is_connected() {
            self.state = RelayState::Connected;
        }

        if self.state == RelayState::Connected {
            match &self.mode {
                RelayMode::HostingRoom => {
                    self.packet_peer.send(&PacketType::CreateRoom.to_bytes(), DefaultChannel::ReliableOrdered)?;
                    self.state = RelayState::AwaitingRoom;
                }
                RelayMode::JoiningRoom(room_id) => {
                    self.packet_peer.send(&PacketType::JoinRoom(room_id.clone()).to_bytes(), DefaultChannel::ReliableOrdered)?;
                    self.state = RelayState::AwaitingRoom;
                }
                RelayMode::None => {}
            }
        }

        for received_packet in received_packets {
            if let Ok(packet) = PacketType::from_bytes(&received_packet.data) {
                match packet {
                    PacketType::ConnectedToRoom(room_id, peer_id) => {
                        self.state = RelayState::InRoom;
                        events.push(RelayEvent::RoomJoined { room_id, peer_id })
                    }
                    PacketType::PeerJoinedRoom(peer_id) => {
                        events.push(RelayEvent::PeerJoinedRoom { peer_id })
                    }
                    PacketType::GameData(from_peer, data) => {
                        events.push(RelayEvent::GameDataReceived { 
                            transfer_mode: Self::channel_to_transfer_mode(received_packet.channel),
                            from_peer, data 
                        })
                    }
                    _ => {
                        godot_warn!("Received unexpected packet type: {:?}", packet)
                    }
                }
            } else {
                godot_warn!("Failed to parse packet, {} bytes", received_packet.data.len());
            }
        }

        Ok(events)
    }

    pub fn disconnect(&mut self) {
        self.packet_peer.disconnect();
        self.state = RelayState::Disconnected;
        self.mode = RelayMode::None;
        self.room_id = None;
    }
    
    fn channel_to_transfer_mode(channel: DefaultChannel) -> TransferMode {
        match channel {
            DefaultChannel::Unreliable => TransferMode::UNRELIABLE,
            DefaultChannel::ReliableOrdered => TransferMode::RELIABLE,
            DefaultChannel::ReliableUnordered => TransferMode::RELIABLE,
        }
    }
}