use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Failed to bind socket. Error: {0}")]
    BindFailed(std::io::Error),

    #[error("Failed to set socket nonblocking. Error: {0}")]
    SetNonBlockingFailed(std::io::Error),

    #[error("System clock error (clock before Unix epoch?): {0}")]
    ClockError(#[from] std::time::SystemTimeError),

    #[error("Netcode error: {0:?}")]
    NetcodeError(#[from] renet_netcode::NetcodeError),

    #[error("Netcode Transport error: {0:?}")]
    NetcodeTransportError(#[from] renet_netcode::NetcodeTransportError),
}