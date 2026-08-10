use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Empty packet")]
    EmptyPacket,

    #[error("Unknown packet type: {0}")]
    UnknownPacketType(u8),

    #[error("Not enough bytes: {0}")]
    NotEnoughBytes(String),

    #[error("Failed to parse i32: {0}")]
    InvalidI32(#[from] std::array::TryFromSliceError),

    #[error("Failed to parse UTF8 string: {0}")]
    InvalidUtf8String(#[from] std::string::FromUtf8Error),

    #[error("Negative length: {0}")]
    NegativeLength(i32),

    #[error("Value too long: {len} bytes (max {max})")]
    TooLong { len: usize, max: usize },

    #[error("Packet type {0} cannot be sent in this direction")]
    UnexpectedDirection(u8)
}