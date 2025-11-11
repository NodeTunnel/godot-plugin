use std::error::Error;

#[derive(Debug, Clone)]
pub enum PacketType {
    CreateRoom,
    JoinRoom(String),
    ConnectedToRoom(String, i32),
    GameData(i32, Vec<u8>),
    PeerJoinedRoom(i32),
    PeerLeftRoom(i32),
    ForceDisconnect(),
    Authenticate(String),
    ClientAuthenticated(),
}

impl PacketType {
    pub(crate) fn from_bytes(bytes: &Vec<u8>) -> Result<Self, Box<dyn Error>> {
        if bytes.is_empty() {
            return Err("Empty packet".into());
        }

        match bytes[0] {
            0 => Ok(PacketType::CreateRoom),
            1 => {
                let room_id = String::from_utf8(bytes[1..].to_vec())?;
                Ok(PacketType::JoinRoom(room_id))
            },
            2 => {
                if bytes.len() < 5 {
                    return Err("Packet too short for ConnectedToRoom".into());
                }

                let peer_id_bytes: [u8; 4] = bytes[1..5]
                    .try_into()
                    .map_err(|_| "Failed to read peer ID")?;

                let peer_id = i32::from_be_bytes(peer_id_bytes);
                let room_id = String::from_utf8(bytes[5..].to_vec())?;

                Ok(PacketType::ConnectedToRoom(room_id, peer_id))
            },
            3 => {
                if bytes.len() < 5 {
                    return Err("Packet too short for GameData".into());
                }

                let from_pid_bytes: [u8; 4] = bytes[1..5]
                    .try_into()
                    .map_err(|_| "Failed to read sender ID")?;

                let from_pid = i32::from_be_bytes(from_pid_bytes);
                let game_data = bytes[5..].to_vec();

                Ok(PacketType::GameData(from_pid, game_data))
            },
            4 => {
                if bytes.len() < 4 {
                    return Err("Packet too short for PeerJoinedRoom".into());
                }

                let peer_id_bytes: [u8; 4] = bytes[1..5]
                    .try_into()
                    .map_err(|_| "Failed to read peer ID")?;

                let peer_id = i32::from_be_bytes(peer_id_bytes);

                Ok(PacketType::PeerJoinedRoom(peer_id))
            },
            5 => {
                if bytes.len() < 4 {
                    return Err("Packet too short for PeerLeftRoom".into());
                }

                let peer_id_bytes: [u8; 4] = bytes[1..5]
                    .try_into()
                    .map_err(|_| "Failed to read peer ID")?;

                let peer_id = i32::from_be_bytes(peer_id_bytes);

                Ok(PacketType::PeerLeftRoom(peer_id))
            },
            6 => {
                Ok(PacketType::ForceDisconnect())
            }
            7 => {
                let app_id = String::from_utf8(bytes[1..].to_vec())?;
                Ok(PacketType::Authenticate(app_id))
            }
            8 => {
                Ok(PacketType::ClientAuthenticated())
            }
            _ => Err(format!("Unknown packet type: {}", bytes[0]).into()),
        }
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        match self {
            PacketType::CreateRoom => vec![0],
            PacketType::JoinRoom(room_id) => {
                let mut result = vec![1];
                result.extend(room_id.as_bytes());
                result
            },
            PacketType::ConnectedToRoom(room_id, godot_pid) => {
                let mut result = vec![2];
                result.extend(godot_pid.to_be_bytes());
                result.extend(room_id.as_bytes());
                result
            },
            PacketType::GameData(target, data) => {
                let mut result = vec![3];
                result.extend(target.to_be_bytes());
                result.extend(data);
                result
            },
            PacketType::PeerJoinedRoom(godot_pid) => {
                let mut result = vec![4];
                result.extend(godot_pid.to_be_bytes());
                result
            },
            PacketType::PeerLeftRoom(godot_pid) => {
                let mut result = vec![5];
                result.extend(godot_pid.to_be_bytes());
                result
            },
            PacketType::ForceDisconnect() => vec![6],
            PacketType::Authenticate(app_id) => {
                let mut result = vec![7];
                result.extend(app_id.as_bytes());
                result
            }
            PacketType::ClientAuthenticated() => vec![8],
        }
    }
}