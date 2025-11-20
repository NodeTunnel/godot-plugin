use renet::ClientId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Reliable,
    Unreliable,
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub client_id: ClientId,
    pub data: Vec<u8>,
    pub channel: Channel,
}

#[derive(Debug, Clone)]
pub enum ServerEvent {
    ClientConnected { client_id: ClientId },
    ClientDisconnected { client_id: ClientId },
    PacketReceived { client_id: ClientId, data: Vec<u8>, channel: Channel },
}