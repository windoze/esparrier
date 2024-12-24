use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("io error")]
    IoError,
    #[error("did not match format")]
    FormatError,
    #[error("not enough data")]
    InsufficientDataError,
    #[error("Packet too small")]
    PacketTooSmall,
}

#[derive(Error, Debug)]
pub enum BarrierError {
    #[error("Disconnected")]
    Disconnected,
    #[error("tcp connection failed")]
    TcpError,
    #[error("invalid data received")]
    ProtocolError(#[from] PacketError),
}
