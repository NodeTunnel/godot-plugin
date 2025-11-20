use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Failed to bind UDP socket: {0}")]
    BindError(std::io::Error),

    #[error("Clock may have gone backwards: {0}")]
    ClockError(#[from] std::time::SystemTimeError),

    #[error("Failed to create Netcode server transport: {0}")]
    NetcodeCreationFailed(std::io::Error),

    #[error("Failed to update Netcode server transport: {0}")]
    NetcodeUpdateFailed(renet_netcode::NetcodeTransportError),
}