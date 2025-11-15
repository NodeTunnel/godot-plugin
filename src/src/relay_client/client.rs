use crate::protocol::packet::PacketType;
use crate::relay_client::events::RelayEvent;
use crate::transport::renet::RenetTransport;
use crate::transport::types::{Channel, Packet};
use godot::global::godot_print;
use std::cmp::PartialEq;
use std::time::Duration;
use crate::relay_client::error::RelayClientError;

#[derive(Debug, PartialEq)]
enum ClientState {
    Connecting,
    Connected,
    Authenticated,
}

pub struct RelayClient {
    transport: Option<RenetTransport>,
    client_state: ClientState,
    last_update: Duration,
}

impl RelayClient {
    pub fn new() -> Self {
        Self {
            transport: None,
            client_state: ClientState::Connecting,
            last_update: Duration::from_secs(0),
        }
    }

    pub fn connect(&mut self, transport: RenetTransport) {
        self.client_state = ClientState::Connecting;
        self.transport = Some(transport);
    }

    pub fn update(&mut self) -> Result<Vec<RelayEvent>, RelayClientError> {
        let delta = Duration::from_nanos(self.last_update.subsec_nanos() as u64);
        let transport = self.transport.as_mut().ok_or(
            RelayClientError::TransportNotInitialized
        )?;

        transport.update(delta)?;

        let mut events = vec![];
        let packets = transport.recv_packets();

        if let Some(event) = self.update_state() {
            events.push(event);
        }

        for packet in packets {
            let event = self.handle_packet(packet)?;
            events.extend(event);
        }

        Ok(events)
    }

    fn update_state(&mut self) -> Option<RelayEvent> {
        if self.client_state == ClientState::Connecting && self.is_connected() {
            self.client_state = ClientState::Connected;
            return Some(RelayEvent::ConnectedToServer);
        }

        None
    }

    fn handle_packet(&mut self, packet: Packet) -> Result<Vec<RelayEvent>, RelayClientError> {
        let mut events = vec![];

        if let Ok(packet_type) = PacketType::from_bytes(&packet.data) {
            match packet_type {
                PacketType::ClientAuthenticated => {
                    godot_print!("Client authenticated");
                    self.client_state = ClientState::Authenticated;
                    events.push(RelayEvent::Authenticated);
                }
                PacketType::ConnectedToRoom { room_id, peer_id } =>
                    events.push(RelayEvent::RoomJoined { room_id, peer_id }),
                PacketType::PeerJoinedRoom { peer_id } =>
                    events.push(RelayEvent::PeerJoinedRoom { peer_id }),
                PacketType::PeerLeftRoom { peer_id } =>
                    events.push(RelayEvent::PeerLeftRoom { peer_id }),
                PacketType::GameData { from_peer, data } =>
                    events.push(RelayEvent::GameDataReceived { data, from_peer, channel: packet.channel }),
                PacketType::ForceDisconnect =>
                    events.push(RelayEvent::ForceDisconnect),
                PacketType::Error { error_code, error_message } =>
                    events.push(RelayEvent::Error { error_code, error_message }),
                _ => {
                    return Err(RelayClientError::InvalidPacketType);
                }
            }
        } else {
            return Err(RelayClientError::PacketParsingError);
        }

        Ok(events)
    }

    pub fn req_auth(&mut self, app_id: String) -> Result<(), RelayClientError> {
        self.send_packet(
            PacketType::Authenticate {
                app_id,
            },
            Channel::Reliable
        )?;

        Ok(())
    }

    pub fn req_create_room(&mut self) -> Result<(), RelayClientError> {
        self.send_packet(
            PacketType::CreateRoom,
            Channel::Reliable
        )?;

        Ok(())
    }

    pub fn req_join_room(&mut self, room_id: String) -> Result<(), RelayClientError> {
        self.send_packet(
            PacketType::JoinRoom { room_id },
            Channel::Reliable
        )?;

        Ok(())
    }

    pub fn send_game_data(&mut self, peer_id: i32, data: Vec<u8>, channel: Channel) -> Result<(), RelayClientError> {
        self.send_packet(
            PacketType::GameData { from_peer: peer_id, data },
            channel
        )?;

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.transport.as_ref().map_or(false, |transport| transport.is_connected())
    }

    pub fn disconnect(&mut self) -> Result<(), RelayClientError> {
        let transport = self.transport.as_mut().ok_or(
            RelayClientError::TransportNotInitialized
        )?;

        transport.disconnect_from_server();

        Ok(())
    }

    fn send_packet(&mut self, packet_type: PacketType, channel: Channel) -> Result<(), RelayClientError> {
        let transport = self.transport.as_mut().ok_or(
            RelayClientError::TransportNotInitialized
        )?;

        transport.send_to_server(
            packet_type.to_bytes(),
            channel,
        )?;

        Ok(())
    }
}
