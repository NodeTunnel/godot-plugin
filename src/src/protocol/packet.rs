use crate::protocol::ids::*;
use crate::protocol::error::ProtocolError;
use crate::protocol::serialize::{push_i32, push_string, push_vec_i32, read_i32, read_string, read_vec_i32};

#[derive(Debug, Clone)]
pub enum PacketType {
    Authenticate { app_id: String, version: String },
    ClientAuthenticated,
    CreateRoom,
    JoinRoom { room_id: String },
    ConnectedToRoom { room_id: String, peer_id: i32, existing_peers: Vec<i32> },
    PeerReady,
    PeerJoinedRoom { peer_id: i32 },
    PeerLeftRoom { peer_id: i32 },
    GameData { from_peer: i32, data: Vec<u8> },
    ForceDisconnect,
    Error { error_code: i32, error_message: String }
}

impl PacketType {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.is_empty() {
            return Err(ProtocolError::EmptyPacket);
        }

        let packet_id = bytes[0];
        let rest = &bytes[1..];

        Ok(match packet_id {
            AUTHENTICATE => {
                let (app_id, r) = read_string(rest)?;
                let (version, _) = read_string(r)?;
                PacketType::Authenticate { app_id, version }
            }

            CLIENT_AUTHENTICATED => PacketType::ClientAuthenticated,

            CREATE_ROOM => PacketType::CreateRoom,

            JOIN_ROOM => {
                let (room_id, _) = read_string(rest)?;
                PacketType::JoinRoom { room_id }
            }

            CONNECTED_TO_ROOM => {
                let (room_id, r) = read_string(rest)?;
                let (peer_id, r) = read_i32(r)?;
                let (existing_peers, _) = read_vec_i32(r)?;
                PacketType::ConnectedToRoom { room_id, peer_id, existing_peers }
            }

            PEER_READY => PacketType::PeerReady,

            PEER_JOINED => {
                let (peer_id, _) = read_i32(rest)?;
                PacketType::PeerJoinedRoom { peer_id }
            }

            PEER_LEFT => {
                let (peer_id, _) = read_i32(rest)?;
                PacketType::PeerLeftRoom { peer_id }
            }

            GAME_DATA => {
                let (peer_id, r) = read_i32(rest)?;
                PacketType::GameData { from_peer: peer_id, data: r.to_vec() }
            }

            FORCE_DISCONNECT => PacketType::ForceDisconnect,

            ERROR_PACKET => {
                let (error_code, r) = read_i32(rest)?;
                let (error_message, _) = read_string(r)?;
                PacketType::Error { error_code, error_message }
            }

            _ => return Err(ProtocolError::UnknownPacketType(packet_id))
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            PacketType::Authenticate { app_id, version } => {
                buf.push(AUTHENTICATE);
                push_string(&mut buf, app_id);
                push_string(&mut buf, version);
            }

            PacketType::ClientAuthenticated => {
                buf.push(CLIENT_AUTHENTICATED);
            }

            PacketType::CreateRoom => {
                buf.push(CREATE_ROOM);
            }

            PacketType::JoinRoom { room_id } => {
                buf.push(JOIN_ROOM);
                push_string(&mut buf, room_id);
            }

            PacketType::ConnectedToRoom { room_id, peer_id, existing_peers } => {
                buf.push(CONNECTED_TO_ROOM);
                push_string(&mut buf, room_id);
                push_i32(&mut buf, *peer_id);
                push_vec_i32(&mut buf, existing_peers);
            }

            PacketType::PeerReady => {
                buf.push(PEER_READY);
            }

            PacketType::PeerJoinedRoom { peer_id } => {
                buf.push(PEER_JOINED);
                push_i32(&mut buf, *peer_id);
            }

            PacketType::PeerLeftRoom { peer_id } => {
                buf.push(PEER_LEFT);
                push_i32(&mut buf, *peer_id);
            }

            PacketType::GameData { from_peer: peer_id, data } => {
                buf.push(GAME_DATA);
                push_i32(&mut buf, *peer_id);
                buf.extend(data);
            }

            PacketType::ForceDisconnect => {
                buf.push(FORCE_DISCONNECT);
            }

            PacketType::Error { error_code, error_message } => {
                buf.push(ERROR_PACKET);
                push_i32(&mut buf, *error_code);
                push_string(&mut buf, error_message);
            }
        }

        buf
    }
}
