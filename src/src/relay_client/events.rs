use crate::transport::common::Channel;

#[derive(Debug)]
pub enum RelayEvent {
    ConnectedToServer,
    Authenticated,
    RoomJoined { room_id: String, peer_id: i32, existing_peers: Vec<i32> },
    PeerJoinedRoom { peer_id: i32 },
    GameDataReceived { channel: Channel, from_peer: i32, data: Vec<u8> },
    PeerLeftRoom { peer_id: i32 },
    ForceDisconnect,
    Error { error_code: i32, error_message: String },
}