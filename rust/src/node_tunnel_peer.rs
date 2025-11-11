use godot::builtin::PackedByteArray;
use godot::prelude::{godot_api, GodotClass};
use godot::classes::{IMultiplayerPeerExtension, MultiplayerPeerExtension};
use godot::classes::multiplayer_peer::{ConnectionStatus, TransferMode};
use godot::global::{godot_error, godot_print, godot_warn, Error};
use godot::obj::{Base, WithUserSignals};
use crate::relay_client::{RelayClient, RelayEvent};

struct GamePacket {
    from_peer: i32,
    data: Vec<u8>,
    transfer_mode: TransferMode,
}

#[derive(GodotClass)]
#[class(tool, base=MultiplayerPeerExtension)]
struct NodeTunnelPeer {
    unique_id: i32,
    connection_status: ConnectionStatus,
    target_peer: i32,
    transfer_mode: TransferMode,
    incoming_packets: Vec<GamePacket>,
    relay_client: RelayClient,
    base: Base<MultiplayerPeerExtension>
}

#[godot_api]
impl NodeTunnelPeer {
    #[signal]
    fn room_connected(room_id: String);

    #[signal]
    fn forced_disconnect();

    #[func]
    fn connect_to_relay(&mut self, relay_address: String, app_id: String) {
        if let Err(e) = self.relay_client.connect(relay_address, app_id) {
            godot_error!("[NodeTunnel] Failed to join relay: {}", e);
            return;
        }
        self.connection_status = ConnectionStatus::CONNECTING;
    }

    #[func]
    fn host_room(&mut self) {
        if let Err(e) = self.relay_client.create_room() {
            godot_error!("[NodeTunnel] Failed to create room: {}", e);
            return;
        }
        self.connection_status = ConnectionStatus::CONNECTING;
    }

    #[func]
    fn join_room(&mut self, host_id: String) {
        if let Err(e) = self.relay_client.join_room(host_id) {
            godot_error!("[NodeTunnel] Failed to join room: {}", e);
            return;
        }
        self.connection_status = ConnectionStatus::CONNECTED;
        self.signals().peer_connected().emit(1);
    }

    fn handle_relay_event(&mut self, event: RelayEvent) {
        match event {
            RelayEvent::RoomJoined { room_id, peer_id } => {
                godot_print!("Joined room {}", peer_id);

                self.unique_id = peer_id;

                if self.is_server() {
                    godot_print!("'Server' connected to room");
                } else {
                    self.signals().peer_connected().emit(1);
                    godot_print!("'Client' connected to room")
                }

                self.connection_status = ConnectionStatus::CONNECTED;
                self.signals().room_connected().emit(room_id);
            }
            RelayEvent::PeerJoinedRoom { peer_id } => {
                if self.is_server() {
                    self.signals().peer_connected().emit(peer_id as i64);
                }
            }
            RelayEvent::PeerLeftRoom { peer_id } => {
                if self.is_server() {
                    self.signals().peer_disconnected().emit(peer_id as i64);
                }
            }
            RelayEvent::GameDataReceived { transfer_mode, from_peer, data } => {
                self.incoming_packets.push(GamePacket {
                    transfer_mode,
                    from_peer,
                    data
                })
            }
            RelayEvent::ForceDisconnect => {
                self.close();
            }
        }
    }
}

#[godot_api]
impl IMultiplayerPeerExtension for NodeTunnelPeer {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            unique_id: 0,
            connection_status: ConnectionStatus::DISCONNECTED,
            target_peer: 0,
            transfer_mode: TransferMode::UNRELIABLE,
            incoming_packets: vec![],
            relay_client: RelayClient::new(),
            base,
        }
    }

    fn get_available_packet_count(&self) -> i32 {
        self.incoming_packets.len() as i32
    }

    fn get_max_packet_size(&self) -> i32 {
        1 << 24
    }

    fn get_packet_script(&mut self) -> PackedByteArray {
        if !self.incoming_packets.is_empty() {
            let packet = self.incoming_packets.remove(0);
            PackedByteArray::from(packet.data.as_slice())
        } else {
            PackedByteArray::new()
        }
    }

    fn put_packet_script(&mut self, p_buffer: PackedByteArray) -> Error {
        let data: Vec<u8> = p_buffer.to_vec();

        if let Err(e) = self.relay_client.send_game_data(self.target_peer, data, self.transfer_mode) {
            godot_error!("[NodeTunnel] Failed to send packet: {}", e);
            return Error::ERR_CANT_CONNECT;
        }

        Error::OK
    }

    fn get_packet_channel(&self) -> i32 {
        0
    }

    fn get_packet_mode(&self) -> TransferMode {
        self.incoming_packets.first()
            .map(|p| p.transfer_mode)
            .unwrap_or(TransferMode::UNRELIABLE)
    }

    fn set_transfer_channel(&mut self, p_channel: i32) {
        if p_channel != 0 {
            godot_warn!("[NodeTunnel] Set to invalid channel: {}", p_channel);
        }
    }

    fn get_transfer_channel(&self) -> i32 {
        0
    }

    fn set_transfer_mode(&mut self, p_mode: TransferMode) {
        self.transfer_mode = p_mode;
    }

    fn get_transfer_mode(&self) -> TransferMode {
        self.transfer_mode
    }

    fn set_target_peer(&mut self, p_peer: i32) {
        self.target_peer = p_peer;
    }

    fn get_packet_peer(&self) -> i32 {
        self.incoming_packets.first()
            .map(|p| p.from_peer)
            .unwrap_or(0)
    }

    fn is_server(&self) -> bool {
        self.unique_id == 1
    }

    fn poll(&mut self) {
        match self.relay_client.update() {
            Ok(events) => {
                for event in events {
                    self.handle_relay_event(event);
                }
            }
            Err(e) => {
                godot_error!("[NodeTunnel] Relay error: {}", e);
            }
        }
    }

    fn close(&mut self) {
        self.unique_id = 0;
        self.connection_status = ConnectionStatus::DISCONNECTED;
        self.relay_client.disconnect();
        self.signals().forced_disconnect().emit();
    }

    // TODO: Implement
    fn disconnect_peer(&mut self, _p_peer: i32, _p_force: bool) {}

    fn get_unique_id(&self) -> i32 {
        self.unique_id
    }

    fn is_server_relay_supported(&self) -> bool {
        true
    }

    fn get_connection_status(&self) -> ConnectionStatus {
        self.connection_status
    }
}
