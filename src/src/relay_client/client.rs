use crate::protocol::packet::PacketType;
use crate::relay_client::events::RelayEvent;
use godot::global::{godot_print, godot_warn};
use std::cmp::PartialEq;
use std::time::Duration;
use crate::protocol::version;
use crate::relay_client::error::RelayClientError;
use crate::transport::client::{ClientEvent, ClientTransport};
use crate::transport::common::{Channel, Packet};

#[derive(Debug, PartialEq)]
enum ClientState {
    Connecting,
    Connected,
    Authenticated,
}

pub struct RelayClient {
    transport: Option<ClientTransport>,
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

    pub fn connect(&mut self, transport: ClientTransport) {
        self.client_state = ClientState::Connecting;
        self.transport = Some(transport);
    }

    pub fn update(&mut self, delta: Duration) -> Result<Vec<RelayEvent>, RelayClientError> {
        let transport = self.transport.as_mut().ok_or(
            RelayClientError::TransportNotInitialized
        )?;

        self.last_update += delta;
        if self.last_update >= Duration::from_secs(5) {
            transport.send_keepalive().expect("TODO: panic message");
            self.last_update = Duration::ZERO;
        }

        let events = transport.recv_packets();

        let mut relay_events = vec![];

        if let Some(event) = self.update_state() {
            relay_events.push(event);
        }

        for event in events {
            if let ClientEvent::PacketReceived { data, channel } = event {
                let packet_events = self.handle_packet(data, channel)?;
                relay_events.extend(packet_events);
            }
        }

        Ok(relay_events)
    }

    fn update_state(&mut self) -> Option<RelayEvent> {
        if self.client_state == ClientState::Connecting && self.is_connected() {
            self.client_state = ClientState::Connected;
            return Some(RelayEvent::ConnectedToServer);
        }

        None
    }

    fn handle_packet(&mut self, data: Vec<u8>, channel: Channel) -> Result<Vec<RelayEvent>, RelayClientError> {
        let mut events = vec![];

        if let Ok(packet_type) = PacketType::from_bytes(&data) {
            match packet_type {
                PacketType::ClientAuthenticated => {
                    godot_print!("Client authenticated");
                    self.client_state = ClientState::Authenticated;
                    events.push(RelayEvent::Authenticated);
                }
                PacketType::ConnectedToRoom { room_id, peer_id, existing_peers } =>
                    events.push(RelayEvent::RoomJoined { room_id, peer_id, existing_peers }),
                PacketType::PeerJoinedRoom { peer_id } =>
                    events.push(RelayEvent::PeerJoinedRoom { peer_id }),
                PacketType::PeerLeftRoom { peer_id } =>
                    events.push(RelayEvent::PeerLeftRoom { peer_id }),
                PacketType::GameData { from_peer, data } => {
                    events.push(RelayEvent::GameDataReceived { data, from_peer, channel });
                }
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
                version: version::CLIENT_VERSION.to_string()
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

    pub fn send_ready(&mut self) -> Result<(), RelayClientError> {
        self.send_packet(
            PacketType::PeerReady,
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

        Ok(())
    }

    fn send_packet(&mut self, packet_type: PacketType, channel: Channel) -> Result<(), RelayClientError> {
        let transport = self.transport.as_mut().ok_or(
            RelayClientError::TransportNotInitialized
        )?;

        transport.send(
            packet_type.to_bytes(),
            channel,
        ).expect("TODO: panic message");

        Ok(())
    }
}
