use godot::classes::multiplayer_peer::TransferMode;
use renet::{DefaultChannel};

/// A wrapper for data received from Renet
pub struct Packet {
    pub data: Vec<u8>,
    pub channel: Channel,
}

/// A simplified version of renet::DefaultChannel
#[derive(Debug, Clone, Copy)]
pub enum Channel {
    Reliable,
    Unreliable,
}

/// Conversion from renet::DefaultChannel
impl From<DefaultChannel> for Channel {
    fn from(ch: DefaultChannel) -> Self {
        match ch {
            DefaultChannel::ReliableOrdered => Channel::Reliable,
            DefaultChannel::Unreliable => Channel::Unreliable,
            _ => Channel::Reliable,
        }
    }
}

/// Conversion from Channel to renet::DefaultChannel
impl From<Channel> for DefaultChannel {
    fn from(ch: Channel) -> Self {
        match ch {
            Channel::Reliable => DefaultChannel::ReliableOrdered,
            Channel::Unreliable => DefaultChannel::Unreliable,
        }
    }
}

/// Conversion from Channel to godot::TransferMode
impl From<Channel> for TransferMode {
    fn from(ch: Channel) -> Self {
        match ch {
            Channel::Reliable => TransferMode::RELIABLE,
            Channel::Unreliable => TransferMode::UNRELIABLE,
        }
    }
}

/// Conversion from godot::TransferMode to Channel
impl From<TransferMode> for Channel {
    fn from(tm: TransferMode) -> Self {
        match tm {
            TransferMode::UNRELIABLE => Channel::Unreliable,
            _ => Channel::Reliable,
        }
    }
}
